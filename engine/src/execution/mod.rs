pub mod executor;
pub mod fees;
pub mod orders;
pub mod queue;
pub mod wallet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::strategy::{OrderType, Outcome};

// ---------------------------------------------------------------------------
// OrderPriority
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OrderPriority {
    Limit = 0,
    StrategyMarket = 1,
    CopyMarket = 2,
    TakeProfit = 3,
    StopLoss = 4,
}

impl OrderPriority {
    fn rank(self) -> u8 {
        self as u8
    }
}

impl Ord for OrderPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for OrderPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ---------------------------------------------------------------------------
// Side
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

// ---------------------------------------------------------------------------
// ExecutionOrder
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOrder {
    pub id: Uuid,
    pub wallet_id: u64,
    pub strategy_id: Option<u64>,
    pub copy_relationship_id: Option<u64>,
    pub symbol: String,
    pub token_id: String,
    pub side: Side,
    pub outcome: Outcome,
    pub price: Option<f64>,
    pub size_usdc: f64,
    pub order_type: OrderType,
    pub priority: OrderPriority,
    pub created_at: i64,
}

impl ExecutionOrder {
    pub fn from_signal(
        wallet_id: u64,
        strategy_id: u64,
        symbol: String,
        token_id: String,
        outcome: Outcome,
        size_usdc: f64,
        order_type: OrderType,
    ) -> Self {
        let (side, priority, price) = match &order_type {
            OrderType::Market => (Side::Buy, OrderPriority::StrategyMarket, None),
            OrderType::Limit { price } => (Side::Buy, OrderPriority::Limit, Some(*price)),
            OrderType::StopLoss { trigger_price } => {
                (Side::Sell, OrderPriority::StopLoss, Some(*trigger_price))
            }
            OrderType::TakeProfit { trigger_price } => {
                (Side::Sell, OrderPriority::TakeProfit, Some(*trigger_price))
            }
        };

        Self {
            id: Uuid::new_v4(),
            wallet_id,
            strategy_id: Some(strategy_id),
            copy_relationship_id: None,
            symbol,
            token_id,
            side,
            outcome,
            price,
            size_usdc,
            order_type,
            priority,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

// ---------------------------------------------------------------------------
// OrderResult / OrderStatus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResult {
    pub polymarket_order_id: String,
    pub status: OrderStatus,
    pub filled_price: Option<f64>,
    pub fee_bps: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Filled,
    Cancelled,
    Failed,
    Timeout,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(OrderPriority::StopLoss > OrderPriority::TakeProfit);
        assert!(OrderPriority::TakeProfit > OrderPriority::CopyMarket);
        assert!(OrderPriority::CopyMarket > OrderPriority::StrategyMarket);
        assert!(OrderPriority::StrategyMarket > OrderPriority::Limit);
    }

    #[test]
    fn test_from_signal_market_buy() {
        let order = ExecutionOrder::from_signal(
            1,
            10,
            "BTC-USD".to_string(),
            "token_abc".to_string(),
            Outcome::Up,
            100.0,
            OrderType::Market,
        );

        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.priority, OrderPriority::StrategyMarket);
        assert_eq!(order.wallet_id, 1);
        assert_eq!(order.strategy_id, Some(10));
        assert!(order.copy_relationship_id.is_none());
        assert!(order.price.is_none());
        assert!((order.size_usdc - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_from_signal_stoploss() {
        let order = ExecutionOrder::from_signal(
            2,
            20,
            "ETH-USD".to_string(),
            "token_xyz".to_string(),
            Outcome::Down,
            50.0,
            OrderType::StopLoss {
                trigger_price: 0.45,
            },
        );

        assert_eq!(order.side, Side::Sell);
        assert_eq!(order.priority, OrderPriority::StopLoss);
        assert_eq!(order.price, Some(0.45));
        assert_eq!(order.outcome, Outcome::Down);
    }
}
