pub mod engine;
pub mod eval;
pub mod indicators;
pub mod interpreter;
pub mod registry;
pub mod state;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Signal {
    Buy {
        outcome: Outcome,
        size_usdc: f64,
        order_type: OrderType,
    },
    Sell {
        outcome: Outcome,
        size_usdc: f64,
        order_type: OrderType,
    },
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit { price: f64 },
    StopLoss { trigger_price: f64 },
    TakeProfit { trigger_price: f64 },
}

pub struct EngineOutput {
    pub wallet_id: u64,
    pub strategy_id: u64,
    pub symbol: String,
    pub signal: Signal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_serialization_roundtrip() {
        let signal = Signal::Buy {
            outcome: Outcome::Up,
            size_usdc: 50.0,
            order_type: OrderType::Market,
        };
        let json = serde_json::to_string(&signal).unwrap();
        let deserialized: Signal = serde_json::from_str(&json).unwrap();
        match deserialized {
            Signal::Buy {
                outcome,
                size_usdc,
                ..
            } => {
                assert_eq!(outcome, Outcome::Up);
                assert!((size_usdc - 50.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected Buy signal"),
        }
    }

    #[test]
    fn test_outcome_equality() {
        assert_eq!(Outcome::Up, Outcome::Up);
        assert_ne!(Outcome::Up, Outcome::Down);
    }
}
