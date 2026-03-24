use serde_json::Value;

use super::state::{PendingBanditChoice, PendingBanditRewardObservation, StrategyState};
use super::{OrderType, Outcome, Signal};
use crate::fetcher::models::Tick;
use crate::tasks::model_score_task::ModelScoreCache;

const DEFAULT_INTERVAL_MS: u64 = 2_000;
const DEFAULT_REWARD_HORIZON_SEC: i64 = 60;
const DEFAULT_EXPLORATION_BPS: f64 = 25.0;
const DEFAULT_PRIOR_MEAN_BPS: f64 = 0.0;
const DEFAULT_PRIOR_COUNT: f64 = 2.0;
const DEFAULT_REWARD_CLIP_BPS: f64 = 750.0;
const DEFAULT_SIZE_USDC: f64 = 1.0;

#[derive(Debug, Clone)]
pub struct EntryBanditDecision {
    pub signal: Signal,
    pub profile_id: String,
    pub profile_index: usize,
    pub reward_horizon_sec: i64,
}

#[derive(Debug, Clone)]
struct EntryBanditConfig {
    url: String,
    interval_ms: u64,
    reward_horizon_sec: i64,
    exploration_bps: f64,
    prior_mean_bps: f64,
    prior_count: f64,
    reward_clip_bps: f64,
    size_usdc: f64,
    profiles: Vec<EntryBanditProfile>,
}

#[derive(Debug, Clone)]
struct EntryBanditProfile {
    id: String,
    min_value: f64,
    min_pct_into_slot: f64,
    max_pct_into_slot: f64,
    max_spread_rel: f64,
}

#[derive(Debug, Clone)]
struct CandidateDecision {
    profile_id: String,
    profile_index: usize,
    outcome: Outcome,
    selection_score: f64,
}

pub fn update_pending_rewards(graph: &Value, tick: &Tick, state: &mut StrategyState) {
    if entry_bandit_config(graph).is_none() {
        state.pending_bandit_reward_observations.clear();
        return;
    }

    if state.pending_bandit_reward_observations.is_empty() {
        return;
    }

    let now = tick.captured_at.unix_timestamp();
    let mut matured = Vec::new();
    state
        .pending_bandit_reward_observations
        .retain(|observation| {
            if observation.symbol == tick.symbol && observation.due_at <= now {
                matured.push(observation.clone());
                false
            } else {
                true
            }
        });

    for observation in matured {
        let Some(mark_price) = mark_price_for_outcome(observation.outcome, tick) else {
            continue;
        };

        if observation.entry_price <= 0.0 || mark_price <= 0.0 {
            continue;
        }

        let raw_reward_bps =
            (mark_price - observation.entry_price) / observation.entry_price * 10_000.0;
        let reward_bps =
            raw_reward_bps.clamp(-observation.reward_clip_bps, observation.reward_clip_bps);
        let arm = state
            .bandit_entry_stats
            .entry(observation.profile_id.clone())
            .or_default();
        arm.pulls = arm.pulls.saturating_add(1);
        arm.total_reward_bps += reward_bps;
        arm.last_reward_bps = Some(reward_bps);
    }
}

pub fn evaluate_entry_signal(
    graph: &Value,
    tick: &Tick,
    state: &StrategyState,
    model_score_cache: Option<&ModelScoreCache>,
) -> Option<EntryBanditDecision> {
    let config = entry_bandit_config(graph)?;
    let cache = model_score_cache?;
    let cache_key = format!("{}#{}", config.url, tick.symbol);
    let max_age_ms = config.interval_ms.saturating_mul(3);
    let entry_value_up = cache.get_number(&cache_key, max_age_ms, "entry_value_up");
    let entry_value_down = cache.get_number(&cache_key, max_age_ms, "entry_value_down");
    let pct_into_slot = tick.pct_into_slot as f64;
    let spread_up_rel = if tick.mid_up > 0.0 {
        tick.spread_up as f64 / tick.mid_up as f64
    } else {
        f64::INFINITY
    };
    let spread_down_rel = if tick.mid_down > 0.0 {
        tick.spread_down as f64 / tick.mid_down as f64
    } else {
        f64::INFINITY
    };

    let total_observations = state
        .bandit_entry_stats
        .values()
        .map(|arm| arm.pulls as f64)
        .sum::<f64>()
        + (config.prior_count * config.profiles.len() as f64);

    let mut best: Option<CandidateDecision> = None;

    for (profile_index, profile) in config.profiles.iter().enumerate() {
        if pct_into_slot < profile.min_pct_into_slot || pct_into_slot > profile.max_pct_into_slot {
            continue;
        }

        let take_up = entry_value_up >= profile.min_value
            && entry_value_up >= entry_value_down
            && spread_up_rel <= profile.max_spread_rel;
        let take_down = entry_value_down >= profile.min_value
            && entry_value_down > entry_value_up
            && spread_down_rel <= profile.max_spread_rel;

        let outcome = match (take_up, take_down) {
            (true, false) => Outcome::Up,
            (false, true) => Outcome::Down,
            _ => continue,
        };

        let arm = state
            .bandit_entry_stats
            .get(&profile.id)
            .cloned()
            .unwrap_or_default();
        let effective_pulls = config.prior_count + arm.pulls as f64;
        let effective_total_reward =
            (config.prior_mean_bps * config.prior_count) + arm.total_reward_bps;
        let mean_reward_bps = if effective_pulls > 0.0 {
            effective_total_reward / effective_pulls
        } else {
            config.prior_mean_bps
        };
        let exploration_bonus = if effective_pulls > 0.0 && config.exploration_bps > 0.0 {
            config.exploration_bps * ((1.0 + total_observations).ln() / effective_pulls).sqrt()
        } else {
            0.0
        };
        let predicted_reward_bps = match outcome {
            Outcome::Up => entry_value_up * 10_000.0,
            Outcome::Down => entry_value_down * 10_000.0,
        };
        let selection_score = predicted_reward_bps + mean_reward_bps + exploration_bonus;

        let candidate = CandidateDecision {
            profile_id: profile.id.clone(),
            profile_index,
            outcome,
            selection_score,
        };

        let is_better = best
            .as_ref()
            .map(|current| {
                candidate.selection_score > current.selection_score
                    || ((candidate.selection_score - current.selection_score).abs() < f64::EPSILON
                        && candidate.profile_index < current.profile_index)
            })
            .unwrap_or(true);

        if is_better {
            best = Some(candidate);
        }
    }

    let candidate = best?;
    Some(EntryBanditDecision {
        signal: Signal::Buy {
            outcome: candidate.outcome,
            size_usdc: config.size_usdc,
            order_type: OrderType::Market,
        },
        profile_id: candidate.profile_id,
        profile_index: candidate.profile_index,
        reward_horizon_sec: config.reward_horizon_sec,
    })
}

pub fn stage_pending_choice(
    state: &mut StrategyState,
    symbol: &str,
    decision: &EntryBanditDecision,
) {
    let Signal::Buy {
        outcome,
        size_usdc: _,
        order_type: _,
    } = &decision.signal
    else {
        return;
    };

    state.pending_bandit_choice = Some(PendingBanditChoice {
        profile_id: decision.profile_id.clone(),
        profile_index: decision.profile_index as u32,
        outcome: *outcome,
        symbol: symbol.to_string(),
        reward_horizon_sec: decision.reward_horizon_sec,
    });
}

pub fn clear_pending_choice(state: &mut StrategyState) {
    state.pending_bandit_choice = None;
}

pub fn record_entry_fill(
    graph: &Value,
    state: &mut StrategyState,
    symbol: &str,
    filled_price: f64,
    filled_at: i64,
) {
    let Some(choice) = state.pending_bandit_choice.take() else {
        return;
    };

    if choice.symbol != symbol || filled_price <= 0.0 || entry_bandit_config(graph).is_none() {
        return;
    }

    let config = entry_bandit_config(graph).expect("config checked above");
    state
        .pending_bandit_reward_observations
        .push(PendingBanditRewardObservation {
            profile_id: choice.profile_id,
            profile_index: choice.profile_index,
            outcome: choice.outcome,
            symbol: symbol.to_string(),
            entry_price: filled_price,
            due_at: filled_at + choice.reward_horizon_sec.max(1),
            reward_clip_bps: config.reward_clip_bps,
        });
}

pub fn collect_model_targets(graph: &Value) -> Vec<(String, u64)> {
    let Some(config) = entry_bandit_config(graph) else {
        return Vec::new();
    };

    vec![(config.url, config.interval_ms)]
}

fn entry_bandit_config(graph: &Value) -> Option<EntryBanditConfig> {
    let entry = graph.get("bandit")?.get("entry")?;
    if !entry
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return None;
    }

    let url = entry.get("url")?.as_str()?.trim().to_string();
    if url.is_empty() {
        return None;
    }

    let profiles = entry
        .get("profiles")
        .and_then(Value::as_array)
        .map(|profiles| {
            profiles
                .iter()
                .enumerate()
                .filter_map(|(index, profile)| parse_profile(profile, index))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if profiles.is_empty() {
        return None;
    }

    Some(EntryBanditConfig {
        url,
        interval_ms: entry
            .get("interval_ms")
            .and_then(Value::as_u64)
            .unwrap_or(DEFAULT_INTERVAL_MS)
            .max(1_000),
        reward_horizon_sec: entry
            .get("reward_horizon_sec")
            .and_then(Value::as_i64)
            .unwrap_or(DEFAULT_REWARD_HORIZON_SEC)
            .max(1),
        exploration_bps: entry
            .get("exploration_bps")
            .and_then(Value::as_f64)
            .unwrap_or(DEFAULT_EXPLORATION_BPS)
            .max(0.0),
        prior_mean_bps: entry
            .get("prior_mean_bps")
            .and_then(Value::as_f64)
            .unwrap_or(DEFAULT_PRIOR_MEAN_BPS),
        prior_count: entry
            .get("prior_count")
            .and_then(Value::as_f64)
            .unwrap_or(DEFAULT_PRIOR_COUNT)
            .max(0.0),
        reward_clip_bps: entry
            .get("reward_clip_bps")
            .and_then(Value::as_f64)
            .unwrap_or(DEFAULT_REWARD_CLIP_BPS)
            .max(1.0),
        size_usdc: entry
            .get("size_usdc")
            .and_then(Value::as_f64)
            .unwrap_or(DEFAULT_SIZE_USDC)
            .max(0.01),
        profiles,
    })
}

fn parse_profile(profile: &Value, index: usize) -> Option<EntryBanditProfile> {
    let id = profile
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("profile-{}", index + 1));
    let min_pct_into_slot = profile
        .get("min_pct_into_slot")
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let max_pct_into_slot = profile
        .get("max_pct_into_slot")
        .and_then(Value::as_f64)
        .unwrap_or(1.0)
        .clamp(0.0, 1.0);
    if min_pct_into_slot > max_pct_into_slot {
        return None;
    }

    Some(EntryBanditProfile {
        id,
        min_value: profile
            .get("min_value")
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
        min_pct_into_slot,
        max_pct_into_slot,
        max_spread_rel: profile
            .get("max_spread_rel")
            .and_then(Value::as_f64)
            .unwrap_or(0.05)
            .max(0.0),
    })
}

fn mark_price_for_outcome(outcome: Outcome, tick: &Tick) -> Option<f64> {
    let price = match outcome {
        Outcome::Up => tick.bid_up as f64,
        Outcome::Down => tick.bid_down as f64,
    };

    if price > 0.0 {
        Some(price)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::test_utils::test_tick;

    fn bandit_graph() -> Value {
        serde_json::json!({
            "mode": "node",
            "nodes": [],
            "edges": [],
            "bandit": {
                "entry": {
                    "enabled": true,
                    "url": "https://ml.example.com/predict",
                    "interval_ms": 2_000,
                    "reward_horizon_sec": 60,
                    "exploration_bps": 10.0,
                    "prior_mean_bps": 0.0,
                    "prior_count": 1.0,
                    "reward_clip_bps": 500.0,
                    "size_usdc": 1.5,
                    "profiles": [
                        {
                            "id": "safe",
                            "min_value": 0.04,
                            "max_spread_rel": 0.03,
                            "max_pct_into_slot": 0.30
                        },
                        {
                            "id": "balanced",
                            "min_value": 0.02,
                            "max_spread_rel": 0.04,
                            "max_pct_into_slot": 0.60
                        }
                    ]
                }
            }
        })
    }

    #[test]
    fn evaluates_entry_signal_with_best_available_profile() {
        let graph = bandit_graph();
        let tick = test_tick();
        let state = StrategyState::new(32);
        let cache = ModelScoreCache::new();
        cache.set(
            "https://ml.example.com/predict#btc-updown-15m-1700000000".into(),
            serde_json::json!({
                "entry_value_up": 0.05,
                "entry_value_down": 0.01
            }),
        );

        let decision =
            evaluate_entry_signal(&graph, &tick, &state, Some(&cache)).expect("bandit decision");

        assert_eq!(decision.profile_id, "balanced");
        assert_eq!(decision.profile_index, 1);
        assert!(matches!(
            decision.signal,
            Signal::Buy {
                outcome: Outcome::Up,
                size_usdc,
                ..
            } if (size_usdc - 1.5).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn updates_profile_stats_from_matured_markout_reward() {
        let graph = bandit_graph();
        let mut tick = test_tick();
        tick.bid_up = 0.66;
        tick.captured_at = time::OffsetDateTime::from_unix_timestamp(1_700_000_600).unwrap();

        let mut state = StrategyState::new(32);
        state
            .pending_bandit_reward_observations
            .push(PendingBanditRewardObservation {
                profile_id: "safe".into(),
                profile_index: 0,
                outcome: Outcome::Up,
                symbol: tick.symbol.clone(),
                entry_price: 0.60,
                due_at: 1_700_000_550,
                reward_clip_bps: 500.0,
            });

        update_pending_rewards(&graph, &tick, &mut state);

        let arm = state.bandit_entry_stats.get("safe").expect("safe arm");
        assert_eq!(arm.pulls, 1);
        assert!(arm.total_reward_bps > 0.0);
        assert!(state.pending_bandit_reward_observations.is_empty());
    }
}
