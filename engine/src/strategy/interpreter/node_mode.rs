use std::collections::{HashMap, VecDeque};

use serde_json::Value;

use super::{build_action_signal, resolve_indicator};
use crate::fetcher::models::Tick;
use crate::strategy::eval::{evaluate_op, get_field};
use crate::strategy::state::StrategyState;
use crate::strategy::{Outcome, Signal};
use crate::tasks::api_fetch_task::ApiFetchCache;

#[derive(Debug, Clone)]
enum NodeValue {
    Number(f64),
    Bool(bool),
}

/// An edge with optional named handles.
#[derive(Debug, Clone)]
struct HandleEdge<'a> {
    source: &'a str,
    source_handle: Option<&'a str>,
    target_handle: Option<&'a str>,
}

/// Resolve a numeric input arriving at a specific named target handle.
fn resolve_handle_input(
    values: &HashMap<&str, NodeValue>,
    handle_inputs: &[HandleEdge<'_>],
    target_handle: &str,
) -> f64 {
    handle_inputs
        .iter()
        .find(|e| e.target_handle == Some(target_handle))
        .and_then(|e| values.get(e.source))
        .and_then(|v| match v {
            NodeValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(0.0)
}

/// Check whether an edge from an if_else node should be active.
/// Returns false (skip) if the source is an if_else and the branch doesn't match.
fn is_edge_active(
    edge: &HandleEdge<'_>,
    values: &HashMap<&str, NodeValue>,
    node_map: &HashMap<&str, &Value>,
) -> bool {
    let Some(source_node) = node_map.get(edge.source) else {
        return true;
    };
    if source_node["type"].as_str() != Some("if_else") {
        return true;
    }
    let condition = matches!(values.get(edge.source), Some(NodeValue::Bool(true)));
    match edge.source_handle {
        Some("true") => condition,
        Some("false") => !condition,
        _ => true,
    }
}

pub(super) fn evaluate_node(
    graph: &Value,
    tick: &Tick,
    state: &mut StrategyState,
    api_cache: Option<&ApiFetchCache>,
) -> Signal {
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
    // Handle-aware input tracking: target_id → Vec<HandleEdge>
    let mut handle_inputs_for: HashMap<&str, Vec<HandleEdge<'_>>> =
        node_ids.iter().map(|&id| (id, vec![])).collect();

    for edge in edges {
        let Some(src) = edge["source"].as_str() else {
            continue;
        };
        let Some(tgt) = edge["target"].as_str() else {
            continue;
        };
        *in_degree.entry(tgt).or_insert(0) += 1;
        adj.entry(src).or_default().push(tgt);
        handle_inputs_for.entry(tgt).or_default().push(HandleEdge {
            source: src,
            source_handle: edge["sourceHandle"].as_str(),
            target_handle: edge["targetHandle"].as_str(),
        });
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
    let empty_handle_inputs: Vec<HandleEdge<'_>> = vec![];

    for &id in &order {
        let Some(node) = node_map.get(id) else {
            continue;
        };
        let node_type = node["type"].as_str().unwrap_or("");
        let data = &node["data"];
        let hinputs = handle_inputs_for.get(id).unwrap_or(&empty_handle_inputs);

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
                let input_val = hinputs
                    .first()
                    .and_then(|e| values.get(e.source))
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
                let bools: Vec<bool> = hinputs
                    .iter()
                    .filter(|e| is_edge_active(e, &values, &node_map))
                    .filter_map(|e| values.get(e.source))
                    .map(|v| match v {
                        NodeValue::Bool(b) => *b,
                        NodeValue::Number(n) => *n != 0.0,
                    })
                    .collect();
                let result = match op {
                    "OR" => bools.iter().any(|&b| b),
                    _ => !bools.is_empty() && bools.iter().all(|&b| b),
                };
                NodeValue::Bool(result)
            }
            "action" => {
                let triggered = hinputs
                    .iter()
                    .filter(|e| is_edge_active(e, &values, &node_map))
                    .filter_map(|e| values.get(e.source))
                    .all(|v| match v {
                        NodeValue::Bool(b) => *b,
                        NodeValue::Number(n) => *n != 0.0,
                    });
                if triggered
                    && hinputs
                        .iter()
                        .any(|e| is_edge_active(e, &values, &node_map))
                {
                    return build_action_signal(data);
                }
                NodeValue::Bool(false)
            }
            "cancel" => {
                let triggered = hinputs
                    .iter()
                    .filter(|e| is_edge_active(e, &values, &node_map))
                    .filter_map(|e| values.get(e.source))
                    .all(|v| match v {
                        NodeValue::Bool(b) => *b,
                        NodeValue::Number(n) => *n != 0.0,
                    });
                if triggered
                    && hinputs
                        .iter()
                        .any(|e| is_edge_active(e, &values, &node_map))
                {
                    let outcome = match data["outcome"].as_str().unwrap_or("UP") {
                        "DOWN" => Outcome::Down,
                        _ => Outcome::Up,
                    };
                    return Signal::Cancel { outcome };
                }
                NodeValue::Bool(false)
            }
            "notify" => {
                let triggered = hinputs
                    .iter()
                    .filter(|e| is_edge_active(e, &values, &node_map))
                    .filter_map(|e| values.get(e.source))
                    .all(|v| match v {
                        NodeValue::Bool(b) => *b,
                        NodeValue::Number(n) => *n != 0.0,
                    });
                if triggered
                    && hinputs
                        .iter()
                        .any(|e| is_edge_active(e, &values, &node_map))
                {
                    let channel = data["channel"]
                        .as_str()
                        .unwrap_or("database")
                        .to_string();
                    let message = data["message"]
                        .as_str()
                        .unwrap_or("Strategy alert")
                        .to_string();
                    return Signal::Notify { channel, message };
                }
                NodeValue::Bool(false)
            }
            "not" => {
                let input_bool = hinputs
                    .first()
                    .and_then(|e| values.get(e.source))
                    .map(|v| match v {
                        NodeValue::Bool(b) => *b,
                        NodeValue::Number(n) => *n != 0.0,
                    })
                    .unwrap_or(false);
                NodeValue::Bool(!input_bool)
            }
            "if_else" => {
                let condition = hinputs
                    .first()
                    .and_then(|e| values.get(e.source))
                    .map(|v| match v {
                        NodeValue::Bool(b) => *b,
                        NodeValue::Number(n) => *n != 0.0,
                    })
                    .unwrap_or(false);
                NodeValue::Bool(condition)
            }
            "math" => {
                let operation = data["operation"].as_str().unwrap_or("+");
                let a = resolve_handle_input(&values, hinputs, "a");
                let b = resolve_handle_input(&values, hinputs, "b");
                let result = match operation {
                    "+" => a + b,
                    "-" => a - b,
                    "*" => a * b,
                    "/" => {
                        if b.abs() < f64::EPSILON {
                            0.0
                        } else {
                            a / b
                        }
                    }
                    "%" => {
                        if b.abs() < f64::EPSILON {
                            0.0
                        } else {
                            a % b
                        }
                    }
                    "min" => a.min(b),
                    "max" => a.max(b),
                    "abs" => a.abs(),
                    _ => 0.0,
                };
                NodeValue::Number(if result.is_finite() { result } else { 0.0 })
            }
            "ev_calculator" => {
                let mode = data["mode"].as_str().unwrap_or("simple");
                let price = resolve_handle_input(&values, hinputs, "price");
                let prob = resolve_handle_input(&values, hinputs, "prob");
                let ev = match mode {
                    "simple" => (prob * (1.0 - price)) - ((1.0 - prob) * price),
                    "custom" => prob * price,
                    _ => 0.0,
                };
                NodeValue::Number(if ev.is_finite() { ev } else { 0.0 })
            }
            "kelly" => {
                let fraction = data["fraction"].as_f64().unwrap_or(0.5);
                let prob = resolve_handle_input(&values, hinputs, "prob");
                let price = resolve_handle_input(&values, hinputs, "price");
                let result = if price > f64::EPSILON && price < 1.0 {
                    let b = (1.0 - price) / price;
                    let kelly_f = ((prob * b) - (1.0 - prob)) / b;
                    (kelly_f * fraction).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                NodeValue::Number(result)
            }
            "api_fetch" => {
                let url = data["url"].as_str().unwrap_or("");
                let json_path = data["json_path"].as_str().unwrap_or("");
                let interval_secs = data["interval_secs"].as_u64().unwrap_or(60).max(30);
                let max_age = interval_secs * 3;
                let cache_key = format!("{}#{}", url, json_path);
                let value = api_cache
                    .map(|c| c.get(&cache_key, max_age))
                    .unwrap_or(0.0);
                NodeValue::Number(value)
            }
            _ => NodeValue::Number(0.0),
        };

        values.insert(id, result);
    }

    Signal::Hold
}

#[cfg(test)]
mod tests {
    use crate::strategy::interpreter::evaluate;
    use crate::strategy::state::StrategyState;
    use crate::strategy::test_utils::test_tick;
    use crate::strategy::{Outcome, Signal};

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

    #[test]
    fn test_not_gate_inverts_true() {
        // Input > 0.5 is true (test_tick abs_move_pct = 1.5), NOT makes it false → Hold
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 0.5 } },
                { "id": "n3", "type": "not",         "data": {} },
                { "id": "n4", "type": "action",     "data": { "signal": "buy", "outcome": "UP", "size_usdc": 10 } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" },
                { "source": "n3", "target": "n4" }
            ]
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        assert!(matches!(evaluate(&graph, &tick, &mut state), Signal::Hold));
    }

    #[test]
    fn test_not_gate_inverts_false() {
        // Input > 10.0 is false (test_tick abs_move_pct = 1.5), NOT makes it true → Buy
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 10.0 } },
                { "id": "n3", "type": "not",         "data": {} },
                { "id": "n4", "type": "action",     "data": { "signal": "buy", "outcome": "UP", "size_usdc": 10 } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" },
                { "source": "n3", "target": "n4" }
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
    fn test_if_else_true_branch() {
        // Condition true → true branch fires action, false branch does not
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 0.5 } },
                { "id": "n3", "type": "if_else",    "data": {} },
                { "id": "n4", "type": "action",     "data": { "signal": "buy", "outcome": "UP", "size_usdc": 10 } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" },
                { "source": "n3", "target": "n4", "sourceHandle": "true" }
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
    fn test_if_else_false_branch() {
        // Condition true → false branch should NOT fire
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 0.5 } },
                { "id": "n3", "type": "if_else",    "data": {} },
                { "id": "n4", "type": "action",     "data": { "signal": "buy", "outcome": "UP", "size_usdc": 10 } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" },
                { "source": "n3", "target": "n4", "sourceHandle": "false" }
            ]
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        assert!(matches!(evaluate(&graph, &tick, &mut state), Signal::Hold));
    }

    #[test]
    fn test_math_addition() {
        // Two inputs added together → compare against threshold
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input", "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "input", "data": { "field": "pct_into_slot" } },
                { "id": "n3", "type": "math",  "data": { "operation": "+" } },
                { "id": "n4", "type": "comparator", "data": { "operator": ">", "value": 1.0 } },
                { "id": "n5", "type": "action", "data": { "signal": "buy", "outcome": "UP", "size_usdc": 10 } }
            ],
            "edges": [
                { "source": "n1", "target": "n3", "targetHandle": "a" },
                { "source": "n2", "target": "n3", "targetHandle": "b" },
                { "source": "n3", "target": "n4" },
                { "source": "n4", "target": "n5" }
            ]
        });
        let tick = test_tick(); // abs_move_pct=1.5, pct_into_slot=0.5 → sum=2.0 > 1.0
        let mut state = StrategyState::new(100);
        assert!(matches!(
            evaluate(&graph, &tick, &mut state),
            Signal::Buy { .. }
        ));
    }

    #[test]
    fn test_math_division_by_zero() {
        // Division by zero → 0.0, should not trigger > 0
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input", "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "input", "data": { "field": "spread_up" } },
                { "id": "n3", "type": "math",  "data": { "operation": "/" } },
                { "id": "n4", "type": "comparator", "data": { "operator": ">", "value": 0.0 } },
                { "id": "n5", "type": "action", "data": { "signal": "buy", "outcome": "UP", "size_usdc": 10 } }
            ],
            "edges": [
                { "source": "n1", "target": "n3", "targetHandle": "a" },
                { "source": "n2", "target": "n3", "targetHandle": "b" },
                { "source": "n3", "target": "n4" },
                { "source": "n4", "target": "n5" }
            ]
        });
        let mut tick = test_tick();
        tick.spread_up = 0.0; // divisor = 0
        let mut state = StrategyState::new(100);
        assert!(matches!(evaluate(&graph, &tick, &mut state), Signal::Hold));
    }

    #[test]
    fn test_ev_calculator_positive() {
        // EV = prob*(1-price) - (1-prob)*price
        // prob=0.7, price=0.4 → EV = 0.7*0.6 - 0.3*0.4 = 0.42 - 0.12 = 0.30
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",         "data": { "field": "mid_up" } },
                { "id": "n2", "type": "input",         "data": { "field": "pct_into_slot" } },
                { "id": "n3", "type": "ev_calculator", "data": { "mode": "simple" } },
                { "id": "n4", "type": "comparator",    "data": { "operator": ">", "value": 0.05 } },
                { "id": "n5", "type": "action",        "data": { "signal": "buy", "outcome": "UP", "size_usdc": 10 } }
            ],
            "edges": [
                { "source": "n1", "target": "n3", "targetHandle": "price" },
                { "source": "n2", "target": "n3", "targetHandle": "prob" },
                { "source": "n3", "target": "n4" },
                { "source": "n4", "target": "n5" }
            ]
        });
        let mut tick = test_tick();
        tick.mid_up = 0.4;         // price
        tick.pct_into_slot = 0.7;  // prob (repurposed for test)
        let mut state = StrategyState::new(100);
        assert!(matches!(
            evaluate(&graph, &tick, &mut state),
            Signal::Buy { .. }
        ));
    }

    #[test]
    fn test_ev_calculator_negative() {
        // prob=0.3, price=0.6 → EV = 0.3*0.4 - 0.7*0.6 = 0.12 - 0.42 = -0.30 (< 0.05)
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",         "data": { "field": "mid_up" } },
                { "id": "n2", "type": "input",         "data": { "field": "pct_into_slot" } },
                { "id": "n3", "type": "ev_calculator", "data": { "mode": "simple" } },
                { "id": "n4", "type": "comparator",    "data": { "operator": ">", "value": 0.05 } },
                { "id": "n5", "type": "action",        "data": { "signal": "buy", "outcome": "UP", "size_usdc": 10 } }
            ],
            "edges": [
                { "source": "n1", "target": "n3", "targetHandle": "price" },
                { "source": "n2", "target": "n3", "targetHandle": "prob" },
                { "source": "n3", "target": "n4" },
                { "source": "n4", "target": "n5" }
            ]
        });
        let mut tick = test_tick();
        tick.mid_up = 0.6;         // price
        tick.pct_into_slot = 0.3;  // prob
        let mut state = StrategyState::new(100);
        assert!(matches!(evaluate(&graph, &tick, &mut state), Signal::Hold));
    }

    #[test]
    fn test_kelly_favorable_prob() {
        // prob=0.7, price=0.4 → b=1.5, kelly_f=(0.7*1.5-0.3)/1.5=0.75/1.5=0.5
        // half-kelly (fraction=0.5) → 0.25, which is > 0.1
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input", "data": { "field": "pct_into_slot" } },
                { "id": "n2", "type": "input", "data": { "field": "mid_up" } },
                { "id": "n3", "type": "kelly", "data": { "fraction": 0.5 } },
                { "id": "n4", "type": "comparator", "data": { "operator": ">", "value": 0.1 } },
                { "id": "n5", "type": "action", "data": { "signal": "buy", "outcome": "UP", "size_usdc": 10 } }
            ],
            "edges": [
                { "source": "n1", "target": "n3", "targetHandle": "prob" },
                { "source": "n2", "target": "n3", "targetHandle": "price" },
                { "source": "n3", "target": "n4" },
                { "source": "n4", "target": "n5" }
            ]
        });
        let mut tick = test_tick();
        tick.pct_into_slot = 0.7;  // prob
        tick.mid_up = 0.4;         // price
        let mut state = StrategyState::new(100);
        assert!(matches!(
            evaluate(&graph, &tick, &mut state),
            Signal::Buy { .. }
        ));
    }

    #[test]
    fn test_kelly_unfavorable_prob() {
        // prob=0.3, price=0.6 → b=0.667, kelly_f=(0.3*0.667-0.7)/0.667 < 0
        // clamped to 0 → 0 is not > 0.1
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input", "data": { "field": "pct_into_slot" } },
                { "id": "n2", "type": "input", "data": { "field": "mid_up" } },
                { "id": "n3", "type": "kelly", "data": { "fraction": 0.5 } },
                { "id": "n4", "type": "comparator", "data": { "operator": ">", "value": 0.1 } },
                { "id": "n5", "type": "action", "data": { "signal": "buy", "outcome": "UP", "size_usdc": 10 } }
            ],
            "edges": [
                { "source": "n1", "target": "n3", "targetHandle": "prob" },
                { "source": "n2", "target": "n3", "targetHandle": "price" },
                { "source": "n3", "target": "n4" },
                { "source": "n4", "target": "n5" }
            ]
        });
        let mut tick = test_tick();
        tick.pct_into_slot = 0.3;  // prob
        tick.mid_up = 0.6;         // price
        let mut state = StrategyState::new(100);
        assert!(matches!(evaluate(&graph, &tick, &mut state), Signal::Hold));
    }

    #[test]
    fn test_cancel_node_triggers() {
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 0.5 } },
                { "id": "n3", "type": "cancel",     "data": { "outcome": "UP" } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" }
            ]
        });
        let tick = test_tick(); // abs_move_pct = 1.0 > 0.5
        let mut state = StrategyState::new(100);
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(
            matches!(signal, Signal::Cancel { outcome: Outcome::Up }),
            "expected Cancel(Up), got {:?}",
            signal
        );
    }

    #[test]
    fn test_cancel_node_does_not_trigger() {
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 10.0 } },
                { "id": "n3", "type": "cancel",     "data": { "outcome": "DOWN" } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" }
            ]
        });
        let tick = test_tick(); // abs_move_pct = 1.0 < 10.0
        let mut state = StrategyState::new(100);
        assert!(matches!(evaluate(&graph, &tick, &mut state), Signal::Hold));
    }

    #[test]
    fn test_notify_node_triggers() {
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 0.5 } },
                { "id": "n3", "type": "notify",     "data": { "channel": "mail", "message": "Big move!" } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" }
            ]
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        let signal = evaluate(&graph, &tick, &mut state);
        match signal {
            Signal::Notify { channel, message } => {
                assert_eq!(channel, "mail");
                assert_eq!(message, "Big move!");
            }
            other => panic!("expected Notify, got {:?}", other),
        }
    }

    #[test]
    fn test_notify_node_does_not_trigger() {
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 10.0 } },
                { "id": "n3", "type": "notify",     "data": { "channel": "database", "message": "Alert" } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" }
            ]
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        assert!(matches!(evaluate(&graph, &tick, &mut state), Signal::Hold));
    }

    #[test]
    fn test_api_fetch_node_with_cache_hit() {
        use crate::strategy::interpreter::evaluate_with_cache;
        use crate::tasks::api_fetch_task::ApiFetchCache;

        let cache = ApiFetchCache::new();
        cache.set("https://api.example.com/weather#main.temp".into(), 25.0);

        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "api_fetch", "data": {
                    "url": "https://api.example.com/weather",
                    "json_path": "main.temp",
                    "interval_secs": 60
                }},
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 20.0 } },
                { "id": "n3", "type": "action", "data": { "signal": "buy", "outcome": "UP", "size_usdc": 10 } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" }
            ]
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        let signal = evaluate_with_cache(&graph, &tick, &mut state, Some(&cache));
        assert!(
            matches!(signal, Signal::Buy { .. }),
            "expected Buy when API value 25.0 > 20.0, got {:?}",
            signal
        );
    }

    #[test]
    fn test_api_fetch_node_cache_miss_returns_zero() {
        use crate::strategy::interpreter::evaluate_with_cache;
        use crate::tasks::api_fetch_task::ApiFetchCache;

        let cache = ApiFetchCache::new(); // empty cache

        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "api_fetch", "data": {
                    "url": "https://api.example.com/weather",
                    "json_path": "main.temp",
                    "interval_secs": 60
                }},
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 0.0 } },
                { "id": "n3", "type": "action", "data": { "signal": "buy", "outcome": "UP", "size_usdc": 10 } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" }
            ]
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        let signal = evaluate_with_cache(&graph, &tick, &mut state, Some(&cache));
        // Cache miss returns 0.0, which is NOT > 0.0
        assert!(
            matches!(signal, Signal::Hold),
            "expected Hold when cache miss (0.0 is not > 0.0), got {:?}",
            signal
        );
    }

    #[test]
    fn test_api_fetch_node_without_cache() {
        // When no cache is provided (e.g. backtest), api_fetch returns 0.0
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "api_fetch", "data": {
                    "url": "https://api.example.com/weather",
                    "json_path": "main.temp",
                    "interval_secs": 60
                }},
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 0.0 } },
                { "id": "n3", "type": "action", "data": { "signal": "buy", "outcome": "UP", "size_usdc": 10 } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" }
            ]
        });
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        // evaluate() passes None for cache
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(
            matches!(signal, Signal::Hold),
            "expected Hold when no cache (0.0 is not > 0.0), got {:?}",
            signal
        );
    }
}
