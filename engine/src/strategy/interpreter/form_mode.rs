use serde_json::Value;

use super::{build_action_signal, resolve_indicator};
use crate::fetcher::models::Tick;
use crate::strategy::eval::evaluate_op;
use crate::strategy::state::StrategyState;
use crate::strategy::Signal;

pub(super) fn evaluate_form_conditions(
    graph: &Value,
    tick: &Tick,
    state: &mut StrategyState,
) -> Signal {
    // Evaluate entry conditions (OR across groups, AND/OR within group)
    if evaluate_conditions(&graph["conditions"], tick, state) {
        build_action_signal(&graph["action"])
    } else {
        Signal::Hold
    }
}

fn evaluate_conditions(conditions: &Value, tick: &Tick, state: &mut StrategyState) -> bool {
    let Some(groups) = conditions.as_array() else {
        return false;
    };
    // Any group matching → true (implicit OR across groups)
    groups.iter().any(|group| {
        let group_type = group["type"].as_str().unwrap_or("AND");
        let Some(rules) = group["rules"].as_array() else {
            return false;
        };
        match group_type {
            "OR" => rules.iter().any(|rule| evaluate_rule(rule, tick, state)),
            _ => rules.iter().all(|rule| evaluate_rule(rule, tick, state)),
        }
    })
}

fn evaluate_rule(rule: &Value, tick: &Tick, state: &mut StrategyState) -> bool {
    let indicator = &rule["indicator"];

    let value = match resolve_indicator(indicator, tick, state) {
        Some(v) => v,
        None => return false,
    };

    let operator = rule["operator"].as_str().unwrap_or("==");
    evaluate_op(value, operator, &rule["value"])
}

#[cfg(test)]
mod tests {
    use crate::strategy::interpreter::evaluate;
    use crate::strategy::state::{Position, StrategyState};
    use crate::strategy::test_utils::test_tick;
    use crate::strategy::{OrderType, Outcome, Signal};
    use serde_json::Value;

    #[test]
    fn test_form_conditions_met_produces_buy() {
        let graph = simple_form_graph();
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        let signal = evaluate(&graph, &tick, &mut state);
        match signal {
            Signal::Buy {
                outcome,
                size_usdc,
                order_type,
            } => {
                assert_eq!(outcome, Outcome::Up);
                assert!((size_usdc - 50.0).abs() < f64::EPSILON);
                assert!(matches!(order_type, OrderType::Market));
            }
            _ => panic!("expected Buy, got {:?}", signal),
        }
    }

    #[test]
    fn test_form_conditions_not_met_holds() {
        let graph = simple_form_graph();
        let mut tick = test_tick();
        tick.abs_move_pct = 0.2;
        let mut state = StrategyState::new(100);
        assert!(matches!(evaluate(&graph, &tick, &mut state), Signal::Hold));
    }

    #[test]
    fn test_form_max_trades_per_slot() {
        let graph = simple_form_graph();
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        assert!(matches!(
            evaluate(&graph, &tick, &mut state),
            Signal::Buy { .. }
        ));
        // Simulate filled position
        state.position = Some(Position {
            outcome: Outcome::Up,
            entry_price: 0.62,
            size_usdc: 50.0,
            entry_at: 1700000450,
        });
        assert!(matches!(evaluate(&graph, &tick, &mut state), Signal::Hold));
    }

    #[test]
    fn test_form_stoploss_triggers_sell() {
        let graph = simple_form_graph();
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        state.position = Some(Position {
            outcome: Outcome::Up,
            entry_price: 0.90,
            size_usdc: 50.0,
            entry_at: 1700000000,
        });
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(
            signal,
            Signal::Sell {
                order_type: OrderType::StopLoss { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_form_take_profit_triggers_sell() {
        let graph = simple_form_graph();
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        state.position = Some(Position {
            outcome: Outcome::Up,
            entry_price: 0.30,
            size_usdc: 50.0,
            entry_at: 1700000000,
        });
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(
            signal,
            Signal::Sell {
                order_type: OrderType::TakeProfit { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_form_slot_change_resets_trade_count() {
        let graph = simple_form_graph();
        let mut state = StrategyState::new(100);
        state.trades_this_slot = 1;
        state.current_slot_ts = 1700000000;
        let mut tick = test_tick();
        tick.slot_ts = 1700000900;
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(signal, Signal::Buy { .. }));
        assert_eq!(state.current_slot_ts, 1700000900);
    }

    #[test]
    fn test_form_stateful_indicator_ema() {
        let graph = serde_json::json!({
            "mode": "form",
            "conditions": [{
                "type": "AND",
                "rules": [{
                    "indicator": { "fn": "EMA", "period": 3, "field": "mid_up" },
                    "operator": ">",
                    "value": 0.5
                }]
            }],
            "action": { "signal": "buy", "outcome": "UP", "size_usdc": 25, "order_type": "market" },
            "risk": {}
        });
        let mut state = StrategyState::new(100);
        for mid in [0.55, 0.58, 0.60] {
            let mut t = test_tick();
            t.mid_up = mid;
            state.push_tick(t);
        }
        let tick = test_tick();
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(signal, Signal::Buy { .. }));
    }

    #[test]
    fn test_form_or_group() {
        let graph = serde_json::json!({
            "mode": "form",
            "conditions": [{
                "type": "OR",
                "rules": [
                    { "indicator": "abs_move_pct", "operator": ">", "value": 10.0 },
                    { "indicator": "pct_into_slot", "operator": ">", "value": 0.3 }
                ]
            }],
            "action": { "signal": "buy", "outcome": "DOWN", "size_usdc": 30, "order_type": "market" },
            "risk": {}
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        let signal = evaluate(&graph, &tick, &mut state);
        match signal {
            Signal::Buy { outcome, .. } => assert_eq!(outcome, Outcome::Down),
            _ => panic!("expected Buy Down"),
        }
    }

    #[test]
    fn test_form_cross_above_triggers() {
        let graph = cross_graph("cross_above");
        let mut state = StrategyState::new(100);
        // Declining → EMA(2) drops below SMA(4)
        for mid in [0.80, 0.70, 0.60, 0.50, 0.40] {
            let mut t = test_tick();
            t.mid_up = mid;
            state.push_tick(t);
        }
        // Sharp jump → EMA(2) crosses above SMA(4)
        let mut tick = test_tick();
        tick.mid_up = 0.90;
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(signal, Signal::Buy { .. }));
    }

    #[test]
    fn test_form_cross_above_no_cross() {
        let graph = cross_graph("cross_above");
        let mut state = StrategyState::new(100);
        // Steadily rising → EMA(2) stays above SMA(4), no crossover
        for mid in [0.50, 0.52, 0.54, 0.56, 0.58] {
            let mut t = test_tick();
            t.mid_up = mid;
            state.push_tick(t);
        }
        let mut tick = test_tick();
        tick.mid_up = 0.60;
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(signal, Signal::Hold));
    }

    #[test]
    fn test_form_cross_below_triggers() {
        let graph = cross_graph("cross_below");
        let mut state = StrategyState::new(100);
        // Rising → EMA(2) above SMA(4)
        for mid in [0.40, 0.50, 0.60, 0.70, 0.80] {
            let mut t = test_tick();
            t.mid_up = mid;
            state.push_tick(t);
        }
        // Sharp drop → EMA(2) crosses below SMA(4)
        let mut tick = test_tick();
        tick.mid_up = 0.30;
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(signal, Signal::Buy { .. }));
    }

    // ── Helpers ──

    fn cross_graph(func: &str) -> Value {
        serde_json::json!({
            "mode": "form",
            "conditions": [{
                "type": "AND",
                "rules": [{
                    "indicator": {
                        "fn": func,
                        "a": { "fn": "EMA", "period": 2, "field": "mid_up" },
                        "b": { "fn": "SMA", "period": 4, "field": "mid_up" }
                    },
                    "operator": ">",
                    "value": 0
                }]
            }],
            "action": { "signal": "buy", "outcome": "UP", "size_usdc": 25, "order_type": "market" },
            "risk": {}
        })
    }

    fn simple_form_graph() -> Value {
        serde_json::json!({
            "mode": "form",
            "conditions": [{
                "type": "AND",
                "rules": [
                    { "indicator": "abs_move_pct", "operator": ">", "value": 0.5 },
                    { "indicator": "pct_into_slot", "operator": "between", "value": [0.1, 0.6] }
                ]
            }],
            "action": {
                "signal": "buy",
                "outcome": "UP",
                "size_mode": "fixed",
                "size_usdc": 50,
                "order_type": "market"
            },
            "risk": {
                "stoploss_pct": 30,
                "take_profit_pct": 80,
                "max_position_usdc": 200,
                "max_trades_per_slot": 1
            }
        })
    }
}
