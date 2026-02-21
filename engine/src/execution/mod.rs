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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// Copy trading context — leader wallet address (empty for strategy orders).
    #[serde(default)]
    pub leader_address: String,
    /// Copy trading context — leader transaction hash (empty for strategy orders).
    #[serde(default)]
    pub leader_tx_hash: String,
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
}
