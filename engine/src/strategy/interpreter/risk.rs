use serde_json::Value;

use crate::fetcher::models::Tick;
use crate::strategy::eval::get_field;
use crate::strategy::state::Position;
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
