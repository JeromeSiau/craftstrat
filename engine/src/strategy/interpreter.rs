use std::collections::{HashMap, VecDeque};

use serde_json::Value;

use super::eval::{evaluate_op, get_field};
use super::indicators;
use super::state::{Position, StrategyState};
use super::{OrderType, Outcome, Signal};
use crate::fetcher::models::Tick;

/// Main entry point — dispatches to form or node mode.
/// Risk management and trade counting apply uniformly to both modes.
pub fn evaluate(graph: &Value, tick: &Tick, state: &mut StrategyState) -> Signal {
    state.push_tick(tick.clone());

    // Reset trades counter on new slot
    if tick.slot_ts != state.current_slot_ts {
        state.trades_this_slot = 0;
        state.current_slot_ts = tick.slot_ts;
    }

    // Universal risk management on open position
    if let Some(ref pos) = state.position {
        if let Some(signal) = check_risk(graph, tick, pos) {
            state.position = None; // clear position after exit signal
            return signal;
        }
        return Signal::Hold; // in position, no risk trigger → hold
    }

    // Universal max trades per slot guard
    let max_trades = graph["risk"]["max_trades_per_slot"].as_u64().unwrap_or(u64::MAX) as u32;
    if state.trades_this_slot >= max_trades {
        return Signal::Hold;
    }

    let mode = graph["mode"].as_str().unwrap_or("form");
    let signal = match mode {
        "form" => evaluate_form_conditions(graph, tick, state),
        "node" => evaluate_node(graph, tick, state),
        _ => Signal::Hold,
    };

    if matches!(signal, Signal::Buy { .. } | Signal::Sell { .. }) {
        state.trades_this_slot += 1;
    }
    signal
}

// ── Form Mode ────────────────────────────────────────────────────────

fn evaluate_form_conditions(graph: &Value, tick: &Tick, state: &mut StrategyState) -> Signal {
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

fn resolve_indicator(indicator: &Value, tick: &Tick, state: &StrategyState) -> Option<f64> {
    // String → direct tick field (stateless)
    if let Some(name) = indicator.as_str() {
        return get_field(tick, name);
    }

    // Object → stateful indicator function
    let obj = indicator.as_object()?;
    let func = obj.get("fn")?.as_str()?;
    let field = obj.get("field").and_then(|v| v.as_str()).unwrap_or("mid_up");

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
        "cross_above" | "cross_below" => {
            resolve_cross(func, indicator, state)
        }
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
    let field = obj.get("field").and_then(|v| v.as_str()).unwrap_or("mid_up");
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
/// JSON: { "fn": "cross_above", "a": { "fn": "EMA", "period": 10, "field": "mid_up" }, "b": { "fn": "EMA", "period": 20, "field": "mid_up" } }
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

fn check_risk(graph: &Value, tick: &Tick, pos: &Position) -> Option<Signal> {
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

fn build_action_signal(action: &Value) -> Signal {
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

// ── Node Mode ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum NodeValue {
    Number(f64),
    Bool(bool),
}

fn evaluate_node(graph: &Value, tick: &Tick, state: &mut StrategyState) -> Signal {
    let Some(nodes) = graph["nodes"].as_array() else {
        return Signal::Hold;
    };
    let Some(edges) = graph["edges"].as_array() else {
        return Signal::Hold;
    };

    // Build adjacency and in-degree for topological sort
    let node_ids: Vec<&str> = nodes.iter().filter_map(|n| n["id"].as_str()).collect();
    let mut in_degree: HashMap<&str, usize> = node_ids.iter().map(|&id| (id, 0)).collect();
    let mut adj: HashMap<&str, Vec<&str>> = node_ids.iter().map(|&id| (id, vec![])).collect();
    let mut inputs_for: HashMap<&str, Vec<&str>> = node_ids.iter().map(|&id| (id, vec![])).collect();

    for edge in edges {
        let Some(src) = edge["source"].as_str() else {
            continue;
        };
        let Some(tgt) = edge["target"].as_str() else {
            continue;
        };
        *in_degree.entry(tgt).or_insert(0) += 1;
        adj.entry(src).or_default().push(tgt);
        inputs_for.entry(tgt).or_default().push(src);
    }

    // Kahn's topological sort
    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();
    let mut order: Vec<&str> = Vec::new();
    while let Some(id) = queue.pop_front() {
        order.push(id);
        for &next in adj.get(id).unwrap_or(&vec![]) {
            let deg = in_degree.get_mut(next).unwrap();
            *deg -= 1;
            if *deg == 0 {
                queue.push_back(next);
            }
        }
    }

    // Cycle detection: if not all nodes were sorted, the graph has a cycle
    if order.len() != node_ids.len() {
        tracing::warn!("strategy graph contains a cycle, skipping evaluation");
        return Signal::Hold;
    }

    // Index nodes by ID
    let node_map: HashMap<&str, &Value> = nodes
        .iter()
        .filter_map(|n| n["id"].as_str().map(|id| (id, n)))
        .collect();

    // Evaluate in topological order
    let mut values: HashMap<&str, NodeValue> = HashMap::new();
    let empty_inputs: Vec<&str> = vec![];

    for &id in &order {
        let Some(node) = node_map.get(id) else {
            continue;
        };
        let node_type = node["type"].as_str().unwrap_or("");
        let data = &node["data"];

        let result = match node_type {
            "input" => {
                let field = data["field"].as_str().unwrap_or("");
                get_field(tick, field)
                    .map(NodeValue::Number)
                    .unwrap_or(NodeValue::Number(0.0))
            }
            "indicator" => {
                let indicator_val = resolve_indicator(data, tick, state);
                NodeValue::Number(indicator_val.unwrap_or(0.0))
            }
            "comparator" => {
                let input_ids = inputs_for.get(id).unwrap_or(&empty_inputs);
                let input_val = input_ids
                    .first()
                    .and_then(|&src_id| values.get(src_id))
                    .and_then(|v| match v {
                        NodeValue::Number(n) => Some(*n),
                        _ => None,
                    })
                    .unwrap_or(0.0);
                let op = data["operator"].as_str().unwrap_or("==");
                NodeValue::Bool(evaluate_op(input_val, op, &data["value"]))
            }
            "logic" => {
                let op = data["operator"].as_str().unwrap_or("AND");
                let input_ids = inputs_for.get(id).unwrap_or(&empty_inputs);
                let bools: Vec<bool> = input_ids
                    .iter()
                    .filter_map(|&src_id| values.get(src_id))
                    .map(|v| match v {
                        NodeValue::Bool(b) => *b,
                        NodeValue::Number(n) => *n != 0.0,
                    })
                    .collect();
                let result = match op {
                    "OR" => bools.iter().any(|&b| b),
                    _ => bools.iter().all(|&b| b),
                };
                NodeValue::Bool(result)
            }
            "action" => {
                let input_ids = inputs_for.get(id).unwrap_or(&empty_inputs);
                let triggered = input_ids
                    .iter()
                    .filter_map(|&src_id| values.get(src_id))
                    .all(|v| match v {
                        NodeValue::Bool(b) => *b,
                        NodeValue::Number(n) => *n != 0.0,
                    });
                if triggered {
                    return build_action_signal(data);
                }
                NodeValue::Bool(false)
            }
            _ => NodeValue::Number(0.0),
        };

        values.insert(id, result);
    }

    Signal::Hold
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::test_utils::test_tick;

    // ── Form Mode Tests ──

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

    // ── Node Mode Tests ──

    #[test]
    fn test_node_simple_graph() {
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "input",      "data": { "field": "pct_into_slot" } },
                { "id": "n3", "type": "comparator", "data": { "operator": ">", "value": 0.5 } },
                { "id": "n4", "type": "comparator", "data": { "operator": "between", "value": [0.1, 0.6] } },
                { "id": "n5", "type": "logic",      "data": { "operator": "AND" } },
                { "id": "n6", "type": "action",     "data": { "signal": "buy", "outcome": "UP", "size_usdc": 50 } }
            ],
            "edges": [
                { "source": "n1", "target": "n3" },
                { "source": "n2", "target": "n4" },
                { "source": "n3", "target": "n5" },
                { "source": "n4", "target": "n5" },
                { "source": "n5", "target": "n6" }
            ]
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        assert!(matches!(
            evaluate(&graph, &tick, &mut state),
            Signal::Buy { .. }
        ));
    }

    #[test]
    fn test_node_condition_fails() {
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 10.0 } },
                { "id": "n3", "type": "action",     "data": { "signal": "buy", "outcome": "UP", "size_usdc": 25 } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" }
            ]
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        assert!(matches!(
            evaluate(&graph, &tick, &mut state),
            Signal::Hold
        ));
    }

    #[test]
    fn test_node_with_indicator() {
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "indicator",  "data": { "fn": "EMA", "period": 3, "field": "mid_up" } },
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 0.5 } },
                { "id": "n3", "type": "action",     "data": { "signal": "buy", "outcome": "DOWN", "size_usdc": 30 } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" }
            ]
        });
        let mut state = StrategyState::new(100);
        for mid in [0.55, 0.58, 0.60] {
            let mut t = test_tick();
            t.mid_up = mid;
            state.push_tick(t);
        }
        let tick = test_tick();
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(
            signal,
            Signal::Buy {
                outcome: Outcome::Down,
                ..
            }
        ));
    }

    #[test]
    fn test_node_or_logic() {
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "input",      "data": { "field": "pct_into_slot" } },
                { "id": "n3", "type": "comparator", "data": { "operator": ">", "value": 10.0 } },
                { "id": "n4", "type": "comparator", "data": { "operator": ">", "value": 0.3 } },
                { "id": "n5", "type": "logic",      "data": { "operator": "OR" } },
                { "id": "n6", "type": "action",     "data": { "signal": "buy", "outcome": "UP", "size_usdc": 20 } }
            ],
            "edges": [
                { "source": "n1", "target": "n3" },
                { "source": "n2", "target": "n4" },
                { "source": "n3", "target": "n5" },
                { "source": "n4", "target": "n5" },
                { "source": "n5", "target": "n6" }
            ]
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        assert!(matches!(
            evaluate(&graph, &tick, &mut state),
            Signal::Buy { .. }
        ));
    }

    // ── Cross Indicator Tests ──

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
