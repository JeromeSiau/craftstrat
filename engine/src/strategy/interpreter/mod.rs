mod form_mode;
mod node_mode;
mod risk;

use serde_json::Value;

use super::bandit;
use super::eval::get_field;
use super::indicators;
use super::state::{Position, StrategyState};
use super::{OrderType, Outcome, Signal};
use crate::fetcher::models::Tick;
use crate::tasks::api_fetch_task::ApiFetchCache;
use crate::tasks::model_score_task::ModelScoreCache;

use form_mode::evaluate_form_conditions;
use node_mode::evaluate_node;
use risk::{check_cooldown, check_daily_loss, check_duplicate, check_risk};

/// Main entry point — dispatches to form or node mode.
/// Risk management and trade counting apply uniformly to both modes.
pub fn evaluate(graph: &Value, tick: &Tick, state: &mut StrategyState) -> Signal {
    evaluate_with_caches(graph, tick, state, None, None)
}

/// Evaluate with an optional API fetch cache (used by the live engine).
pub fn evaluate_with_cache(
    graph: &Value,
    tick: &Tick,
    state: &mut StrategyState,
    api_cache: Option<&ApiFetchCache>,
) -> Signal {
    evaluate_with_caches(graph, tick, state, api_cache, None)
}

pub fn evaluate_with_caches(
    graph: &Value,
    tick: &Tick,
    state: &mut StrategyState,
    api_cache: Option<&ApiFetchCache>,
    model_score_cache: Option<&ModelScoreCache>,
) -> Signal {
    bandit::update_pending_rewards(graph, tick, state);
    state.push_tick(tick.clone());

    // Daily loss limit — blocks ALL trading (entries and exits) when breached
    if check_daily_loss(graph, state, tick) {
        return Signal::Hold;
    }

    // Reset trades counter on new slot
    if tick.slot_ts != state.current_slot_ts {
        state.trades_this_slot = 0;
        state.current_slot_ts = tick.slot_ts;
    }

    // Universal risk management on open position
    if let Some(pos) = state.position.clone() {
        if let Some(signal) = check_risk(graph, tick, &pos) {
            state.position = None; // clear position after exit signal
            return signal;
        }

        let signal = evaluate_graph_signal(graph, tick, state, api_cache, model_score_cache);
        if let Signal::Sell {
            outcome,
            ref order_type,
            ..
        } = signal
        {
            if outcome == pos.outcome {
                state.position = None;
                return Signal::Sell {
                    outcome: pos.outcome,
                    size_usdc: pos.size_usdc,
                    order_type: order_type.clone(),
                };
            }
        }

        return match signal {
            Signal::Notify { channel, message } => Signal::Notify { channel, message },
            _ => Signal::Hold,
        };
    }

    if state.pending_entry_symbol.is_some() {
        return Signal::Hold;
    }

    // Cooldown — block entries if too soon after last trade
    if check_cooldown(graph, state, tick) {
        return Signal::Hold;
    }

    // Universal max trades per slot guard
    let max_trades = graph["risk"]["max_trades_per_slot"]
        .as_u64()
        .unwrap_or(u64::MAX) as u32;
    if state.trades_this_slot >= max_trades {
        return Signal::Hold;
    }

    if let Some(decision) = bandit::evaluate_entry_signal(graph, tick, state, model_score_cache) {
        state.pending_entry_symbol = Some(tick.symbol.clone());
        state.trades_this_slot += 1;
        bandit::stage_pending_choice(state, &tick.symbol, &decision);
        return decision.signal;
    }

    let signal = evaluate_graph_signal(graph, tick, state, api_cache, model_score_cache);

    // Duplicate prevention — block if same position already open
    if check_duplicate(graph, state, &signal) {
        return Signal::Hold;
    }

    if matches!(signal, Signal::Buy { .. }) {
        state.pending_entry_symbol = Some(tick.symbol.clone());
    }

    if matches!(signal, Signal::Buy { .. } | Signal::Sell { .. }) {
        state.trades_this_slot += 1;
    }
    signal
}

fn evaluate_graph_signal(
    graph: &Value,
    tick: &Tick,
    state: &mut StrategyState,
    api_cache: Option<&ApiFetchCache>,
    model_score_cache: Option<&ModelScoreCache>,
) -> Signal {
    match graph["mode"].as_str().unwrap_or("form") {
        "form" => evaluate_form_conditions(graph, tick, state),
        "node" => evaluate_node(graph, tick, state, api_cache, model_score_cache),
        _ => Signal::Hold,
    }
}

// ── Shared utilities (used by both form_mode and node_mode) ──────────

fn position_mark_price(position: &Position, tick: &Tick) -> f64 {
    match position.outcome {
        Outcome::Up => tick.bid_up as f64,
        Outcome::Down => tick.bid_down as f64,
    }
}

pub(super) fn resolve_field(name: &str, tick: &Tick, state: &StrategyState) -> Option<f64> {
    if let Some(value) = get_field(tick, name) {
        return Some(value);
    }

    let position = state.position.as_ref();
    let current_price = position
        .map(|pos| position_mark_price(pos, tick))
        .unwrap_or(0.0);
    let unrealized_pct = position
        .filter(|pos| pos.entry_price > 0.0 && current_price > 0.0)
        .map(|pos| (current_price - pos.entry_price) / pos.entry_price * 100.0)
        .unwrap_or(0.0);
    let unrealized_usdc = position
        .map(|pos| unrealized_pct / 100.0 * pos.size_usdc)
        .unwrap_or(0.0);

    match name {
        "position_is_open" => Some(if position.is_some() { 1.0 } else { 0.0 }),
        "position_is_up" => Some(
            if matches!(position.map(|pos| pos.outcome), Some(Outcome::Up)) {
                1.0
            } else {
                0.0
            },
        ),
        "position_is_down" => Some(
            if matches!(position.map(|pos| pos.outcome), Some(Outcome::Down)) {
                1.0
            } else {
                0.0
            },
        ),
        "position_entry_price" => Some(position.map(|pos| pos.entry_price).unwrap_or(0.0)),
        "position_size_usdc" => Some(position.map(|pos| pos.size_usdc).unwrap_or(0.0)),
        "position_age_sec" => Some(
            position
                .map(|pos| (tick.captured_at.unix_timestamp() - pos.entry_at).max(0) as f64)
                .unwrap_or(0.0),
        ),
        "position_current_price" => Some(current_price),
        "position_unrealized_pnl_pct" => Some(unrealized_pct),
        "position_unrealized_pnl_usdc" => Some(unrealized_usdc),
        _ => None,
    }
}

pub(super) fn resolve_indicator(
    indicator: &Value,
    tick: &Tick,
    state: &StrategyState,
) -> Option<f64> {
    // String → direct tick field (stateless)
    if let Some(name) = indicator.as_str() {
        return resolve_field(name, tick, state);
    }

    // Object → stateful indicator function
    let obj = indicator.as_object()?;
    let func = obj.get("fn")?.as_str()?;
    let field = obj
        .get("field")
        .and_then(|v| v.as_str())
        .unwrap_or("mid_up");

    let values: Vec<f64> = state
        .window
        .iter()
        .filter_map(|t| get_field(t, field))
        .collect();

    match func {
        "EMA" => {
            let period = obj.get("period").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
            Some(indicators::ema(&values, period))
        }
        "SMA" => {
            let period = obj.get("period").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
            Some(indicators::sma(&values, period))
        }
        "RSI" => {
            let period = obj.get("period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
            Some(indicators::rsi(&values, period))
        }
        "VWAP" => {
            let ticks: Vec<Tick> = state.window.iter().cloned().collect();
            Some(indicators::vwap(&ticks, field))
        }
        "cross_above" | "cross_below" => resolve_cross(func, indicator, state),
        _ => None,
    }
}

/// Compute a scalar indicator value from a sub-indicator spec over a given window slice.
fn compute_scalar(spec: &Value, window: &[Tick]) -> Option<f64> {
    if let Some(name) = spec.as_str() {
        // Stateless field — use last tick in window
        return window.last().and_then(|t| get_field(t, name));
    }
    let obj = spec.as_object()?;
    let func = obj.get("fn")?.as_str()?;
    let field = obj
        .get("field")
        .and_then(|v| v.as_str())
        .unwrap_or("mid_up");
    let values: Vec<f64> = window.iter().filter_map(|t| get_field(t, field)).collect();
    match func {
        "EMA" => {
            let period = obj.get("period").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
            Some(indicators::ema(&values, period))
        }
        "SMA" => {
            let period = obj.get("period").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
            Some(indicators::sma(&values, period))
        }
        "RSI" => {
            let period = obj.get("period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
            Some(indicators::rsi(&values, period))
        }
        _ => None,
    }
}

/// Resolve cross_above / cross_below by comparing two sub-indicators
/// at the previous tick vs the current tick.
fn resolve_cross(func: &str, indicator: &Value, state: &StrategyState) -> Option<f64> {
    if state.window.len() < 2 {
        return Some(0.0);
    }
    let spec_a = &indicator["a"];
    let spec_b = &indicator["b"];

    let all_ticks: Vec<Tick> = state.window.iter().cloned().collect();
    let prev_ticks = &all_ticks[..all_ticks.len() - 1];

    let curr_a = compute_scalar(spec_a, &all_ticks)?;
    let curr_b = compute_scalar(spec_b, &all_ticks)?;
    let prev_a = compute_scalar(spec_a, prev_ticks)?;
    let prev_b = compute_scalar(spec_b, prev_ticks)?;

    let result = match func {
        "cross_above" => indicators::cross_above(prev_a, curr_a, prev_b, curr_b),
        "cross_below" => indicators::cross_below(prev_a, curr_a, prev_b, curr_b),
        _ => false,
    };
    // Return 1.0 for true, 0.0 for false — used with operator "==" 1.0 or "> 0"
    Some(if result { 1.0 } else { 0.0 })
}

pub(super) fn build_action_signal(action: &Value) -> Signal {
    let outcome = match action["outcome"].as_str().unwrap_or("UP") {
        "DOWN" => Outcome::Down,
        _ => Outcome::Up,
    };
    let size_usdc = action["size_usdc"].as_f64().unwrap_or(10.0);
    let order_type = match action["order_type"].as_str().unwrap_or("market") {
        "limit" => OrderType::Limit {
            price: action["limit_price"].as_f64().unwrap_or(0.0),
        },
        _ => OrderType::Market,
    };
    let signal_type = action["signal"].as_str().unwrap_or("buy");
    match signal_type {
        "sell" => Signal::Sell {
            outcome,
            size_usdc,
            order_type,
        },
        _ => Signal::Buy {
            outcome,
            size_usdc,
            order_type,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::test_utils::test_tick;
    use crate::tasks::model_score_task::ModelScoreCache;

    #[test]
    fn test_daily_loss_limit_blocks_evaluation() {
        let graph = serde_json::json!({
            "mode": "form",
            "conditions": [{
                "type": "AND",
                "rules": [{ "indicator": "abs_move_pct", "operator": ">", "value": 0.5 }]
            }],
            "action": { "signal": "buy", "outcome": "UP", "size_usdc": 50, "order_type": "market" },
            "risk": {
                "daily_loss_limit_usdc": 100.0,
                "max_trades_per_slot": 10
            }
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        state.daily_pnl = -100.0;
        state.daily_pnl_date = 20231114; // same date as test_tick

        let signal = evaluate(&graph, &tick, &mut state);
        assert!(
            matches!(signal, Signal::Hold),
            "expected Hold when daily loss limit reached, got {:?}",
            signal
        );
    }

    #[test]
    fn test_daily_loss_limit_allows_when_ok() {
        let graph = serde_json::json!({
            "mode": "form",
            "conditions": [{
                "type": "AND",
                "rules": [{ "indicator": "abs_move_pct", "operator": ">", "value": 0.5 }]
            }],
            "action": { "signal": "buy", "outcome": "UP", "size_usdc": 50, "order_type": "market" },
            "risk": {
                "daily_loss_limit_usdc": 100.0,
                "max_trades_per_slot": 10
            }
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        state.daily_pnl = -50.0;
        state.daily_pnl_date = 20231114;

        let signal = evaluate(&graph, &tick, &mut state);
        assert!(
            matches!(signal, Signal::Buy { .. }),
            "expected Buy when under daily loss limit, got {:?}",
            signal
        );
    }

    #[test]
    fn test_pending_entry_blocks_duplicate_buys_until_execution_finishes() {
        let graph = serde_json::json!({
            "mode": "form",
            "conditions": [{
                "type": "AND",
                "rules": [{ "indicator": "abs_move_pct", "operator": ">", "value": 0.5 }]
            }],
            "action": { "signal": "buy", "outcome": "UP", "size_usdc": 50, "order_type": "market" },
            "risk": {
                "max_trades_per_slot": 10
            }
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);

        let first_signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(first_signal, Signal::Buy { .. }));
        assert_eq!(
            state.pending_entry_symbol.as_deref(),
            Some(tick.symbol.as_str())
        );

        let second_signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(second_signal, Signal::Hold));
        assert_eq!(state.trades_this_slot, 1);
    }

    #[test]
    fn test_open_position_allows_form_exit_signal() {
        let graph = serde_json::json!({
            "mode": "form",
            "conditions": [{
                "type": "AND",
                "rules": [{ "indicator": "pct_into_slot", "operator": ">", "value": 0.1 }]
            }],
            "action": { "signal": "sell", "outcome": "UP", "size_usdc": 1, "order_type": "market" },
            "risk": {}
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        state.position = Some(crate::strategy::state::Position {
            outcome: Outcome::Up,
            entry_price: 0.50,
            size_usdc: 42.0,
            entry_at: 1700000000,
            symbol: tick.symbol.clone(),
        });

        let signal = evaluate(&graph, &tick, &mut state);

        match signal {
            Signal::Sell {
                outcome,
                size_usdc,
                order_type,
            } => {
                assert_eq!(outcome, Outcome::Up);
                assert!((size_usdc - 42.0).abs() < f64::EPSILON);
                assert!(matches!(order_type, OrderType::Market));
            }
            _ => panic!("expected Sell, got {:?}", signal),
        }
        assert!(
            state.position.is_none(),
            "position should be cleared on exit signal"
        );
    }

    #[test]
    fn test_open_position_ignores_entry_only_graph_signal() {
        let graph = serde_json::json!({
            "mode": "form",
            "conditions": [{
                "type": "AND",
                "rules": [{ "indicator": "pct_into_slot", "operator": ">", "value": 0.1 }]
            }],
            "action": { "signal": "buy", "outcome": "UP", "size_usdc": 50, "order_type": "market" },
            "risk": {}
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        state.position = Some(crate::strategy::state::Position {
            outcome: Outcome::Up,
            entry_price: 0.50,
            size_usdc: 42.0,
            entry_at: 1700000000,
            symbol: tick.symbol.clone(),
        });

        let signal = evaluate(&graph, &tick, &mut state);

        assert!(matches!(signal, Signal::Hold));
        assert!(
            state.position.is_some(),
            "position should stay open without exit signal"
        );
    }

    #[test]
    fn test_position_fields_resolve_for_open_up_position() {
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        state.position = Some(crate::strategy::state::Position {
            outcome: Outcome::Up,
            entry_price: 0.50,
            size_usdc: 42.0,
            entry_at: 1_700_000_000,
            symbol: tick.symbol.clone(),
        });

        assert_eq!(resolve_field("position_is_open", &tick, &state), Some(1.0));
        assert_eq!(resolve_field("position_is_up", &tick, &state), Some(1.0));
        assert_eq!(resolve_field("position_is_down", &tick, &state), Some(0.0));
        assert_eq!(
            resolve_field("position_entry_price", &tick, &state),
            Some(0.50)
        );
        assert_eq!(
            resolve_field("position_size_usdc", &tick, &state),
            Some(42.0)
        );
        assert_eq!(
            resolve_field("position_current_price", &tick, &state),
            Some(tick.bid_up as f64)
        );
        assert!(resolve_field("position_unrealized_pnl_pct", &tick, &state)
            .is_some_and(|value| value > 0.0));
        assert_eq!(
            resolve_field("position_age_sec", &tick, &state),
            Some((tick.captured_at.unix_timestamp() - 1_700_000_000) as f64)
        );
    }

    #[test]
    fn test_position_fields_default_to_zero_without_position() {
        let tick = test_tick();
        let state = StrategyState::new(100);

        assert_eq!(resolve_field("position_is_open", &tick, &state), Some(0.0));
        assert_eq!(resolve_field("position_is_up", &tick, &state), Some(0.0));
        assert_eq!(
            resolve_field("position_current_price", &tick, &state),
            Some(0.0)
        );
        assert_eq!(
            resolve_field("position_unrealized_pnl_usdc", &tick, &state),
            Some(0.0)
        );
    }

    #[test]
    fn test_entry_bandit_can_emit_buy_without_entry_nodes() {
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [],
            "edges": [],
            "bandit": {
                "entry": {
                    "enabled": true,
                    "url": "https://ml.example.com/predict",
                    "interval_ms": 2_000,
                    "size_usdc": 1.25,
                    "profiles": [
                        {
                            "id": "balanced",
                            "min_value": 0.02,
                            "max_spread_rel": 0.05,
                            "max_pct_into_slot": 0.75
                        }
                    ]
                }
            }
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        let cache = ModelScoreCache::new();
        cache.set(
            "https://ml.example.com/predict#btc-updown-15m-1700000000".into(),
            serde_json::json!({
                "entry_value_up": 0.03,
                "entry_value_down": 0.01
            }),
        );

        let signal = evaluate_with_caches(&graph, &tick, &mut state, None, Some(&cache));

        assert!(matches!(
            signal,
            Signal::Buy {
                outcome: Outcome::Up,
                size_usdc,
                ..
            } if (size_usdc - 1.25).abs() < f64::EPSILON
        ));
        assert_eq!(
            state.pending_entry_symbol.as_deref(),
            Some(tick.symbol.as_str())
        );
        assert_eq!(
            state
                .pending_bandit_choice
                .as_ref()
                .map(|choice| choice.profile_id.as_str()),
            Some("balanced")
        );
    }
}
