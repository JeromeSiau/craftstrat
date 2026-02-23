use serde_json::Value;

use crate::fetcher::models::Tick;
use crate::strategy::eval::get_field;
use crate::strategy::state::{Position, StrategyState};
use crate::strategy::{OrderType, Outcome, Signal};

pub(super) fn check_risk(graph: &Value, tick: &Tick, pos: &Position) -> Option<Signal> {
    let risk = &graph["risk"];
    let current_price = match pos.outcome {
        Outcome::Up => get_field(tick, "mid_up").unwrap_or(0.0),
        Outcome::Down => get_field(tick, "mid_down").unwrap_or(0.0),
    };

    if pos.entry_price <= 0.0 || current_price <= 0.0 {
        return None;
    }

    let pnl_pct = (current_price - pos.entry_price) / pos.entry_price * 100.0;

    // Stoploss: price dropped below threshold
    if let Some(sl) = risk["stoploss_pct"].as_f64() {
        if pnl_pct <= -sl {
            return Some(Signal::Sell {
                outcome: pos.outcome,
                size_usdc: pos.size_usdc,
                order_type: OrderType::StopLoss {
                    trigger_price: current_price,
                },
            });
        }
    }

    // Take profit: price rose above threshold
    if let Some(tp) = risk["take_profit_pct"].as_f64() {
        if pnl_pct >= tp {
            return Some(Signal::Sell {
                outcome: pos.outcome,
                size_usdc: pos.size_usdc,
                order_type: OrderType::TakeProfit {
                    trigger_price: current_price,
                },
            });
        }
    }

    None
}

/// Returns `true` if trading should be blocked because the daily PnL loss limit
/// has been reached. Resets `daily_pnl` when the date changes.
pub(super) fn check_daily_loss(graph: &Value, state: &mut StrategyState, tick: &Tick) -> bool {
    let today = tick_date(tick);
    if state.daily_pnl_date != today {
        state.daily_pnl = 0.0;
        state.daily_pnl_date = today;
    }
    if let Some(limit) = graph["risk"]["daily_loss_limit_usdc"].as_f64() {
        if limit > 0.0 {
            return state.daily_pnl <= -limit;
        }
    }
    false
}

/// Returns `true` if trading should be blocked because the cooldown period
/// has not yet elapsed since the last trade.
pub(super) fn check_cooldown(graph: &Value, state: &StrategyState, tick: &Tick) -> bool {
    if let Some(cooldown_secs) = graph["risk"]["cooldown_seconds"].as_u64() {
        if let Some(last) = state.last_trade_at {
            let elapsed = tick.captured_at.unix_timestamp() - last;
            return elapsed < cooldown_secs as i64;
        }
    }
    false
}

/// Returns `true` if the signal would create a duplicate of the current open position.
pub(super) fn check_duplicate(graph: &Value, state: &StrategyState, signal: &Signal) -> bool {
    if !graph["risk"]["prevent_duplicates"].as_bool().unwrap_or(false) {
        return false;
    }
    if let Some(ref pos) = state.position {
        if let Signal::Buy { outcome, .. } = signal {
            return pos.outcome == *outcome;
        }
    }
    false
}

/// Extract YYYYMMDD date from a tick's `captured_at` timestamp.
fn tick_date(tick: &Tick) -> u32 {
    let (year, month, day) = tick.captured_at.to_calendar_date();
    year as u32 * 10000 + month as u32 * 100 + day as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::test_utils::test_tick;

    fn graph_with_daily_limit(limit: f64) -> Value {
        serde_json::json!({
            "risk": { "daily_loss_limit_usdc": limit }
        })
    }

    #[test]
    fn test_daily_loss_blocks_when_limit_reached() {
        let graph = graph_with_daily_limit(50.0);
        let tick = test_tick();
        let mut state = StrategyState::new(10);
        state.daily_pnl = -50.0; // exactly at limit
        state.daily_pnl_date = 20231114; // same date as test_tick

        assert!(
            check_daily_loss(&graph, &mut state, &tick),
            "should block when daily_pnl equals -limit"
        );
    }

    #[test]
    fn test_daily_loss_allows_when_under_limit() {
        let graph = graph_with_daily_limit(50.0);
        let tick = test_tick();
        let mut state = StrategyState::new(10);
        state.daily_pnl = -30.0;
        state.daily_pnl_date = 20231114; // same date as test_tick

        assert!(
            !check_daily_loss(&graph, &mut state, &tick),
            "should allow when daily_pnl is above -limit"
        );
    }

    #[test]
    fn test_daily_loss_resets_on_date_change() {
        let graph = graph_with_daily_limit(50.0);
        let tick = test_tick(); // 2023-11-14
        let mut state = StrategyState::new(10);
        state.daily_pnl = -100.0;
        state.daily_pnl_date = 20230101; // old date

        // Should reset daily_pnl to 0 and allow trading
        assert!(
            !check_daily_loss(&graph, &mut state, &tick),
            "should reset and allow after date change"
        );
        assert!(
            (state.daily_pnl).abs() < f64::EPSILON,
            "daily_pnl should be reset to 0"
        );
    }

    #[test]
    fn test_daily_loss_passes_when_no_limit_set() {
        let graph = serde_json::json!({ "risk": {} });
        let tick = test_tick();
        let mut state = StrategyState::new(10);
        state.daily_pnl = -1000.0;

        assert!(
            !check_daily_loss(&graph, &mut state, &tick),
            "should pass when no limit configured"
        );
    }

    #[test]
    fn test_tick_date_extraction() {
        let tick = test_tick(); // captured_at = 1700000450 = 2023-11-14
        assert_eq!(tick_date(&tick), 20231114);
    }

    // -- Cooldown tests --

    #[test]
    fn test_cooldown_blocks_during_period() {
        let graph = serde_json::json!({ "risk": { "cooldown_seconds": 60 } });
        let tick = test_tick(); // captured_at unix ts
        let mut state = StrategyState::new(10);
        state.last_trade_at = Some(tick.captured_at.unix_timestamp() - 30); // 30s ago

        assert!(
            check_cooldown(&graph, &state, &tick),
            "should block when cooldown not elapsed"
        );
    }

    #[test]
    fn test_cooldown_allows_after_period() {
        let graph = serde_json::json!({ "risk": { "cooldown_seconds": 60 } });
        let tick = test_tick();
        let mut state = StrategyState::new(10);
        state.last_trade_at = Some(tick.captured_at.unix_timestamp() - 120); // 120s ago

        assert!(
            !check_cooldown(&graph, &state, &tick),
            "should allow when cooldown elapsed"
        );
    }

    #[test]
    fn test_cooldown_allows_no_previous_trade() {
        let graph = serde_json::json!({ "risk": { "cooldown_seconds": 60 } });
        let tick = test_tick();
        let state = StrategyState::new(10);

        assert!(
            !check_cooldown(&graph, &state, &tick),
            "should allow when no previous trade"
        );
    }

    #[test]
    fn test_cooldown_passes_when_not_configured() {
        let graph = serde_json::json!({ "risk": {} });
        let tick = test_tick();
        let mut state = StrategyState::new(10);
        state.last_trade_at = Some(tick.captured_at.unix_timestamp() - 1);

        assert!(
            !check_cooldown(&graph, &state, &tick),
            "should pass when no cooldown configured"
        );
    }

    // -- Duplicate prevention tests --

    #[test]
    fn test_duplicate_blocks_same_outcome() {
        let graph = serde_json::json!({ "risk": { "prevent_duplicates": true } });
        let mut state = StrategyState::new(10);
        state.position = Some(Position {
            outcome: Outcome::Up,
            entry_price: 0.5,
            size_usdc: 50.0,
            entry_at: 0,
        });
        let signal = Signal::Buy {
            outcome: Outcome::Up,
            size_usdc: 50.0,
            order_type: OrderType::Market,
        };

        assert!(
            check_duplicate(&graph, &state, &signal),
            "should block duplicate buy with same outcome"
        );
    }

    #[test]
    fn test_duplicate_allows_different_outcome() {
        let graph = serde_json::json!({ "risk": { "prevent_duplicates": true } });
        let mut state = StrategyState::new(10);
        state.position = Some(Position {
            outcome: Outcome::Up,
            entry_price: 0.5,
            size_usdc: 50.0,
            entry_at: 0,
        });
        let signal = Signal::Buy {
            outcome: Outcome::Down,
            size_usdc: 50.0,
            order_type: OrderType::Market,
        };

        assert!(
            !check_duplicate(&graph, &state, &signal),
            "should allow buy with different outcome"
        );
    }

    #[test]
    fn test_duplicate_allows_when_no_position() {
        let graph = serde_json::json!({ "risk": { "prevent_duplicates": true } });
        let state = StrategyState::new(10);
        let signal = Signal::Buy {
            outcome: Outcome::Up,
            size_usdc: 50.0,
            order_type: OrderType::Market,
        };

        assert!(
            !check_duplicate(&graph, &state, &signal),
            "should allow when no position open"
        );
    }

    #[test]
    fn test_duplicate_passes_when_disabled() {
        let graph = serde_json::json!({ "risk": { "prevent_duplicates": false } });
        let mut state = StrategyState::new(10);
        state.position = Some(Position {
            outcome: Outcome::Up,
            entry_price: 0.5,
            size_usdc: 50.0,
            entry_at: 0,
        });
        let signal = Signal::Buy {
            outcome: Outcome::Up,
            size_usdc: 50.0,
            order_type: OrderType::Market,
        };

        assert!(
            !check_duplicate(&graph, &state, &signal),
            "should pass when prevent_duplicates is false"
        );
    }
}
