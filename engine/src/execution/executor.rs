use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use sqlx::PgPool;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use super::orders::OrderSubmitter;
use super::queue::ExecutionQueue;
use super::{ExecutionOrder, OrderResult, OrderStatus, Side};
use crate::strategy::registry::AssignmentRegistry;
use crate::strategy::state::Position;
use crate::strategy::Outcome;

// ---------------------------------------------------------------------------
// run — main executor loop
// ---------------------------------------------------------------------------

pub async fn run(
    queue: Arc<Mutex<ExecutionQueue>>,
    submitter: Arc<OrderSubmitter>,
    registry: AssignmentRegistry,
    db: PgPool,
) -> Result<()> {
    info!("executor_started");

    loop {
        // 1. Atomic peek + rate-limit + pop (no order loss on rate-limit)
        let order = {
            let mut q = queue.lock().await;
            q.pop_if_allowed()
        };

        let order = match order {
            None => {
                tokio::time::sleep(Duration::from_millis(50)).await;
                continue;
            }
            Some(None) => {
                // Next order is rate-limited — back off without removing it
                warn!("rate_limited, backing off");
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }
            Some(Some(o)) => o,
        };

        // 2. Submit order
        let result = match submitter.submit(&order).await {
            Ok(r) => r,
            Err(e) => {
                error!(
                    order_id = %order.id,
                    error = %e,
                    "order_submission_failed"
                );
                OrderResult {
                    polymarket_order_id: String::new(),
                    status: OrderStatus::Failed,
                    filled_price: None,
                    fee_bps: None,
                }
            }
        };

        // 4. If filled, update position state
        if result.status == OrderStatus::Filled {
            update_position(&registry, &order, &result).await;
        }

        // 5. Write trade to PostgreSQL
        if let Err(e) = crate::storage::postgres::write_trade(&db, &order, &result).await {
            error!(order_id = %order.id, error = %e, "write_trade_failed");
        }

        // 6. If copy trade, write copy_trade record
        if let Some(copy_rel_id) = order.copy_relationship_id {
            let outcome_str = match order.outcome {
                Outcome::Up => "UP",
                Outcome::Down => "DOWN",
            };
            let status_str = match result.status {
                OrderStatus::Filled => "filled",
                OrderStatus::Cancelled => "cancelled",
                OrderStatus::Failed => "failed",
                OrderStatus::Timeout => "timeout",
            };

            if let Err(e) = crate::storage::postgres::write_copy_trade(
                &db,
                copy_rel_id as i64,
                None,
                &order.leader_address,
                &order.symbol,
                outcome_str,
                order.price.unwrap_or(0.0),
                order.size_usdc,
                &order.leader_tx_hash,
                result.filled_price,
                status_str,
                None,
            )
            .await
            {
                error!(order_id = %order.id, error = %e, "write_copy_trade_failed");
            }
        }

        // 7. Log completion
        info!(
            order_id = %order.id,
            wallet_id = order.wallet_id,
            symbol = %order.symbol,
            side = ?order.side,
            status = ?result.status,
            "order_executed"
        );
    }
}

// ---------------------------------------------------------------------------
// update_position — adjust strategy state after a fill
// ---------------------------------------------------------------------------

async fn update_position(
    registry: &AssignmentRegistry,
    order: &ExecutionOrder,
    result: &OrderResult,
) {
    let strategy_id = match order.strategy_id {
        Some(id) => id,
        None => return, // copy trades don't update strategy positions
    };

    let filled_price = result.filled_price.unwrap_or(order.price.unwrap_or(0.0));

    let reg = registry.read().await;

    // Find the assignment matching (wallet_id, strategy_id)
    let assignment = reg
        .values()
        .flatten()
        .find(|a| a.wallet_id == order.wallet_id && a.strategy_id == strategy_id);

    let assignment = match assignment {
        Some(a) => a,
        None => {
            warn!(
                wallet_id = order.wallet_id,
                strategy_id,
                "no_assignment_found_for_position_update"
            );
            return;
        }
    };

    let mut state = match assignment.state.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    match order.side {
        Side::Buy => {
            state.position = Some(Position {
                outcome: order.outcome,
                entry_price: filled_price,
                size_usdc: order.size_usdc,
                entry_at: chrono::Utc::now().timestamp(),
            });
        }
        Side::Sell => {
            if let Some(ref pos) = state.position {
                let pnl = (filled_price - pos.entry_price) * pos.size_usdc;
                state.pnl += pnl;
            }
            state.position = None;
        }
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
    use crate::strategy::registry::activate;
    use crate::strategy::state::StrategyState;
    use crate::strategy::{OrderType, Outcome};

    fn make_order(wallet_id: u64, strategy_id: u64, side: Side, size_usdc: f64) -> ExecutionOrder {
        ExecutionOrder {
            id: Uuid::new_v4(),
            wallet_id,
            strategy_id: Some(strategy_id),
            copy_relationship_id: None,
            symbol: "btc".to_string(),
            token_id: "tok".to_string(),
            side,
            outcome: Outcome::Up,
            price: None,
            size_usdc,
            order_type: OrderType::Market,
            priority: OrderPriority::StrategyMarket,
            created_at: 0,
            leader_address: String::new(),
            leader_tx_hash: String::new(),
        }
    }

    fn make_filled_result(price: f64) -> OrderResult {
        OrderResult {
            polymarket_order_id: "test-order-id".to_string(),
            status: OrderStatus::Filled,
            filled_price: Some(price),
            fee_bps: Some(100),
        }
    }

    #[tokio::test]
    async fn test_update_position_buy_sets_position() {
        let registry = AssignmentRegistry::new();
        activate(
            &registry,
            1,
            100,
            serde_json::json!({}),
            vec!["btc".into()],
            200.0,
            None,
        )
        .await;

        let order = make_order(1, 100, Side::Buy, 50.0);
        let result = make_filled_result(0.60);

        update_position(&registry, &order, &result).await;

        let reg = registry.read().await;
        let assignment = reg.get("btc").unwrap().first().unwrap();
        let state = assignment.state.lock().unwrap();

        assert!(state.position.is_some(), "position should be set after buy");
        let pos = state.position.as_ref().unwrap();
        assert!(
            (pos.entry_price - 0.60).abs() < f64::EPSILON,
            "entry_price should be 0.60"
        );
        assert!(
            (pos.size_usdc - 50.0).abs() < f64::EPSILON,
            "size_usdc should be 50.0"
        );
        assert_eq!(pos.outcome, Outcome::Up);
    }

    #[tokio::test]
    async fn test_update_position_sell_clears_and_updates_pnl() {
        let registry = AssignmentRegistry::new();

        // Create state with an existing position at entry_price=0.50
        let mut initial_state = StrategyState::new(200);
        initial_state.position = Some(Position {
            outcome: Outcome::Up,
            entry_price: 0.50,
            size_usdc: 50.0,
            entry_at: 0,
        });

        activate(
            &registry,
            1,
            100,
            serde_json::json!({}),
            vec!["btc".into()],
            200.0,
            Some(initial_state),
        )
        .await;

        let order = make_order(1, 100, Side::Sell, 50.0);
        let result = make_filled_result(0.70);

        update_position(&registry, &order, &result).await;

        let reg = registry.read().await;
        let assignment = reg.get("btc").unwrap().first().unwrap();
        let state = assignment.state.lock().unwrap();

        assert!(
            state.position.is_none(),
            "position should be cleared after sell"
        );

        let expected_pnl = (0.70 - 0.50) * 50.0; // 10.0
        assert!(
            (state.pnl - expected_pnl).abs() < f64::EPSILON,
            "pnl should be {expected_pnl}, got {}",
            state.pnl
        );
    }
}
