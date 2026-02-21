use std::collections::{HashMap, VecDeque};

use serde_json::Value;

use super::{build_action_signal, resolve_indicator};
use crate::fetcher::models::Tick;
use crate::strategy::eval::{evaluate_op, get_field};
use crate::strategy::state::StrategyState;
use crate::strategy::Signal;

#[derive(Debug, Clone)]
enum NodeValue {
    Number(f64),
    Bool(bool),
}

pub(super) fn evaluate_node(graph: &Value, tick: &Tick, state: &mut StrategyState) -> Signal {
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
}
