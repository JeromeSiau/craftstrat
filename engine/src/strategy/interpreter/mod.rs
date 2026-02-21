mod form_mode;
mod node_mode;
mod risk;

use serde_json::Value;

use super::eval::get_field;
use super::indicators;
use super::state::StrategyState;
use super::{OrderType, Outcome, Signal};
use crate::fetcher::models::Tick;

use form_mode::evaluate_form_conditions;
use node_mode::evaluate_node;
use risk::check_risk;

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

// ── Shared utilities (used by both form_mode and node_mode) ──────────

pub(super) fn resolve_indicator(
    indicator: &Value,
    tick: &Tick,
    state: &StrategyState,
) -> Option<f64> {
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
