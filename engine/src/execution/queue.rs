use std::collections::{BinaryHeap, HashMap};
use std::time::Instant;

use super::ExecutionOrder;

// ---------------------------------------------------------------------------
// TokenBucket — per-wallet rate limiter
// ---------------------------------------------------------------------------

pub struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(max_per_day: u32) -> Self {
        let max = f64::from(max_per_day);
        Self {
            tokens: max,
            max_tokens: max,
            refill_rate: max / 86_400.0,
            last_refill: Instant::now(),
        }
    }

    pub fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    #[cfg(test)]
    pub fn with_tokens(max_per_day: u32, tokens: u32) -> Self {
        let max = f64::from(max_per_day);
        Self {
            tokens: f64::from(tokens),
            max_tokens: max,
            refill_rate: max / 86_400.0,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

// ---------------------------------------------------------------------------
// PriorityOrder — BinaryHeap wrapper with correct ordering
// ---------------------------------------------------------------------------

struct PriorityOrder(ExecutionOrder);

impl PartialEq for PriorityOrder {
    fn eq(&self, other: &Self) -> bool {
        self.0.priority == other.0.priority && self.0.created_at == other.0.created_at
    }
}

impl Eq for PriorityOrder {}

impl Ord for PriorityOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .priority
            .cmp(&other.0.priority)
            .then_with(|| other.0.created_at.cmp(&self.0.created_at))
    }
}

impl PartialOrd for PriorityOrder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ---------------------------------------------------------------------------
// ExecutionQueue
// ---------------------------------------------------------------------------

pub struct ExecutionQueue {
    heap: BinaryHeap<PriorityOrder>,
    rate_limiters: HashMap<u64, TokenBucket>,
    max_orders_per_day: u32,
}

impl ExecutionQueue {
    pub fn new(max_orders_per_day: u32) -> Self {
        Self {
            heap: BinaryHeap::new(),
            rate_limiters: HashMap::new(),
            max_orders_per_day,
        }
    }

    pub fn push(&mut self, order: ExecutionOrder) {
        self.heap.push(PriorityOrder(order));
    }

    #[allow(dead_code)]
    pub fn pop(&mut self) -> Option<ExecutionOrder> {
        self.heap.pop().map(|po| po.0)
    }

    /// Atomically peek + rate-limit check + pop.
    /// Returns `None` if the queue is empty.
    /// Returns `Some(None)` if the next order is rate-limited (stays in queue).
    /// Returns `Some(Some(order))` if allowed — order is removed and token consumed.
    pub fn pop_if_allowed(&mut self) -> Option<Option<ExecutionOrder>> {
        let next = self.heap.peek()?;
        let wallet_id = next.0.wallet_id;
        let max = self.max_orders_per_day;
        let bucket = self
            .rate_limiters
            .entry(wallet_id)
            .or_insert_with(|| TokenBucket::new(max));

        if bucket.try_consume() {
            Some(self.heap.pop().map(|po| po.0))
        } else {
            Some(None) // rate-limited, order stays in queue
        }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::execution::{OrderPriority, Side};
    use crate::strategy::{OrderType, Outcome};

    fn make_order(priority: OrderPriority, created_at: i64) -> ExecutionOrder {
        make_order_for_wallet(1, priority, created_at)
    }

    fn make_order_for_wallet(wallet_id: u64, priority: OrderPriority, created_at: i64) -> ExecutionOrder {
        ExecutionOrder {
            id: Uuid::new_v4(),
            wallet_id,
            strategy_id: None,
            copy_relationship_id: None,
            symbol: "TEST".to_string(),
            token_id: "tok".to_string(),
            side: Side::Buy,
            outcome: Outcome::Up,
            price: None,
            size_usdc: 10.0,
            order_type: OrderType::Market,
            priority,
            created_at,
            leader_address: String::new(),
            leader_tx_hash: String::new(),
            is_paper: false,
        }
    }

    #[test]
    fn test_priority_ordering_stoploss_first() {
        let mut q = ExecutionQueue::new(1000);
        q.push(make_order(OrderPriority::Limit, 1));
        q.push(make_order(OrderPriority::StopLoss, 2));
        q.push(make_order(OrderPriority::StrategyMarket, 3));

        let first = q.pop().unwrap();
        assert_eq!(first.priority, OrderPriority::StopLoss);

        let second = q.pop().unwrap();
        assert_eq!(second.priority, OrderPriority::StrategyMarket);

        let third = q.pop().unwrap();
        assert_eq!(third.priority, OrderPriority::Limit);
    }

    #[test]
    fn test_fifo_within_same_priority() {
        let mut q = ExecutionQueue::new(1000);
        q.push(make_order(OrderPriority::StrategyMarket, 100));
        q.push(make_order(OrderPriority::StrategyMarket, 50));

        let first = q.pop().unwrap();
        assert_eq!(first.created_at, 50, "older order (created_at=50) should come first");

        let second = q.pop().unwrap();
        assert_eq!(second.created_at, 100);
    }

    #[test]
    fn test_token_bucket_allows_burst() {
        let mut bucket = TokenBucket::new(3000);
        for i in 0..100 {
            assert!(bucket.try_consume(), "consume #{i} should succeed");
        }
    }

    #[test]
    fn test_token_bucket_exhaustion() {
        let mut bucket = TokenBucket::with_tokens(100, 2);

        assert!(bucket.try_consume(), "first consume should succeed");
        assert!(bucket.try_consume(), "second consume should succeed");
        assert!(!bucket.try_consume(), "third consume should fail — bucket exhausted");
    }

    #[test]
    fn test_rate_limit_per_wallet() {
        let mut q = ExecutionQueue::new(2);

        // Push 3 orders for wallet 1 and 1 for wallet 2
        q.push(make_order_for_wallet(1, OrderPriority::StrategyMarket, 1));
        q.push(make_order_for_wallet(1, OrderPriority::StrategyMarket, 2));
        q.push(make_order_for_wallet(1, OrderPriority::StrategyMarket, 3));
        q.push(make_order_for_wallet(2, OrderPriority::StopLoss, 4));

        // Wallet 2's StopLoss has highest priority — should pop first
        let first = q.pop_if_allowed().unwrap().unwrap();
        assert_eq!(first.wallet_id, 2);

        // Wallet 1: first two should pop (2 tokens)
        let second = q.pop_if_allowed().unwrap().unwrap();
        assert_eq!(second.wallet_id, 1);
        let third = q.pop_if_allowed().unwrap().unwrap();
        assert_eq!(third.wallet_id, 1);

        // Wallet 1: third order should be rate-limited (Some(None))
        assert!(
            matches!(q.pop_if_allowed(), Some(None)),
            "wallet 1 should be rate-limited"
        );
    }
}
