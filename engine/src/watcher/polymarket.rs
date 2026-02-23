use std::sync::Arc;

use anyhow::Result;
use metrics::counter;
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::execution::queue::ExecutionQueue;
use crate::execution::{ExecutionOrder, OrderPriority, Side};
use crate::metrics as m;
use crate::storage::postgres::{
    self, CopyRelationship,
};
use crate::proxy::HttpPool;
use crate::strategy::{OrderType, Outcome};

// ---------------------------------------------------------------------------
// LeaderTrade
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Deserialize)]
pub struct LeaderTrade {
    pub side: String,
    pub asset: String,
    #[serde(alias = "conditionId")]
    pub condition_id: String,
    pub size: f64,
    pub price: f64,
    pub timestamp: i64,
    #[serde(alias = "transactionHash")]
    pub transaction_hash: String,
    pub outcome: Option<String>,
}

// ---------------------------------------------------------------------------
// run() — main watcher loop
// ---------------------------------------------------------------------------

pub async fn run(
    data_api_url: &str,
    http: HttpPool,
    queue: Arc<Mutex<ExecutionQueue>>,
    db: PgPool,
    mut redis_conn: redis::aio::MultiplexedConnection,
) -> Result<()> {
    loop {
        let addresses = postgres::load_watched_addresses(&db).await?;

        let mut handles = Vec::new();
        for address in &addresses {
            let url = data_api_url.to_string();
            let client = http.clone();
            let addr = address.clone();
            let mut redis = redis_conn.clone();

            handles.push(tokio::spawn(async move {
                let last_seen = get_last_seen(&mut redis, &addr).await.unwrap_or(0);
                let trades = check_new_trades(&url, &client, &addr, last_seen).await;
                (addr, trades)
            }));
        }

        for handle in handles {
            let (address, trades_result) = handle.await?;
            let trades = match trades_result {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(address = %address, error = %e, "check_new_trades_failed");
                    continue;
                }
            };

            if trades.is_empty() {
                continue;
            }

            let followers = postgres::get_active_followers(&db, &address).await?;

            for trade in &trades {
                for follower in &followers {
                    match build_copy_order(trade, follower, &address) {
                        Some(order) => {
                            let mut q = queue.lock().await;
                            q.push(order);
                            counter!(m::COPY_TRADES_TOTAL, "status" => "queued").increment(1);
                        }
                        None => {
                            let outcome_str = trade.outcome.as_deref().unwrap_or("UP");
                            let _ = postgres::write_copy_trade(
                                &db,
                                follower.id,
                                None,
                                &address,
                                &trade.condition_id,
                                outcome_str,
                                trade.price,
                                trade.size,
                                &trade.transaction_hash,
                                None,
                                "skipped",
                                Some("exceeds_max_position_or_filtered"),
                            )
                            .await;
                            counter!(m::COPY_TRADES_TOTAL, "status" => "skipped").increment(1);
                        }
                    }
                }

                update_last_seen(&mut redis_conn, &address, trade.timestamp).await?;
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

// ---------------------------------------------------------------------------
// check_new_trades() — fetch recent trades from the data API
// ---------------------------------------------------------------------------

async fn check_new_trades(
    data_api_url: &str,
    http: &HttpPool,
    address: &str,
    last_seen: i64,
) -> Result<Vec<LeaderTrade>> {
    let url = format!(
        "{}/trades?user={}&limit=5&sortBy=TIMESTAMP&sortDirection=DESC",
        data_api_url, address
    );

    let trades: Vec<LeaderTrade> = http.proxied().get(&url).send().await?.json().await?;

    let new_trades: Vec<LeaderTrade> = trades
        .into_iter()
        .filter(|t| t.timestamp > last_seen)
        .collect();

    Ok(new_trades)
}

// ---------------------------------------------------------------------------
// Redis helpers — get/update last_seen
// ---------------------------------------------------------------------------

async fn get_last_seen(
    conn: &mut redis::aio::MultiplexedConnection,
    address: &str,
) -> Result<i64> {
    let key = format!("craftstrat:watcher:last_seen:{}", address);
    let val: Option<String> = redis::cmd("GET")
        .arg(&key)
        .query_async(conn)
        .await?;
    Ok(val.and_then(|v| v.parse().ok()).unwrap_or(0))
}

async fn update_last_seen(
    conn: &mut redis::aio::MultiplexedConnection,
    address: &str,
    timestamp: i64,
) -> Result<()> {
    let key = format!("craftstrat:watcher:last_seen:{}", address);
    redis::cmd("SET")
        .arg(&key)
        .arg(timestamp.to_string())
        .query_async::<()>(conn)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// build_copy_order() — pure function that maps a leader trade to an order
// ---------------------------------------------------------------------------

fn build_copy_order(
    trade: &LeaderTrade,
    follower: &CopyRelationship,
    leader_address: &str,
) -> Option<ExecutionOrder> {
    // 1. Check markets_filter
    if let Some(ref filter_value) = follower.markets_filter {
        if let Some(arr) = filter_value.as_array() {
            let allowed: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            if !allowed.contains(&trade.condition_id) {
                return None;
            }
        }
    }

    // 2. Calculate size
    let size = match follower.size_mode.as_str() {
        "fixed" => follower.size_value,
        "proportional" => trade.size * follower.size_value,
        _ => follower.size_value,
    };

    // 3. Check max_position
    if size > follower.max_position_usdc {
        return None;
    }

    // 4. Map outcome
    let outcome = match trade.outcome.as_deref() {
        Some("Yes") | Some("UP") => Outcome::Up,
        Some("No") | Some("DOWN") => Outcome::Down,
        _ => Outcome::Up,
    };

    // 5. Map side
    let side = match trade.side.as_str() {
        "SELL" => Side::Sell,
        _ => Side::Buy,
    };

    Some(ExecutionOrder {
        id: Uuid::new_v4(),
        wallet_id: follower.follower_wallet_id as u64,
        strategy_id: None,
        copy_relationship_id: Some(follower.id as u64),
        symbol: trade.condition_id.clone(),
        token_id: trade.asset.clone(),
        side,
        outcome,
        price: Some(trade.price),
        size_usdc: size,
        order_type: OrderType::Market,
        priority: OrderPriority::CopyMarket,
        created_at: chrono::Utc::now().timestamp(),
        leader_address: leader_address.to_string(),
        leader_tx_hash: trade.transaction_hash.clone(),
        is_paper: false,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_trade() -> LeaderTrade {
        LeaderTrade {
            side: "BUY".to_string(),
            asset: "token_abc".to_string(),
            condition_id: "condition_456".to_string(),
            size: 100.0,
            price: 0.65,
            timestamp: 1_700_000_000,
            transaction_hash: "0xdeadbeef".to_string(),
            outcome: Some("Yes".to_string()),
        }
    }

    fn test_follower() -> CopyRelationship {
        CopyRelationship {
            id: 1,
            follower_wallet_id: 42,
            size_mode: "fixed".to_string(),
            size_value: 50.0,
            max_position_usdc: 200.0,
            markets_filter: None,
        }
    }

    #[test]
    fn test_build_copy_order_fixed_size() {
        let trade = test_trade();
        let follower = test_follower();

        let order = build_copy_order(&trade, &follower, "0xleader").unwrap();

        assert!((order.size_usdc - 50.0).abs() < f64::EPSILON);
        assert_eq!(order.priority, OrderPriority::CopyMarket);
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.outcome, Outcome::Up);
        assert!(order.strategy_id.is_none());
        assert_eq!(order.copy_relationship_id, Some(1));
        assert_eq!(order.leader_address, "0xleader");
        assert_eq!(order.leader_tx_hash, "0xdeadbeef");
    }

    #[test]
    fn test_build_copy_order_proportional_size() {
        let trade = test_trade();
        let mut follower = test_follower();
        follower.size_mode = "proportional".to_string();
        follower.size_value = 0.5;

        let order = build_copy_order(&trade, &follower, "0xleader").unwrap();

        assert!((order.size_usdc - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_build_copy_order_exceeds_max_position() {
        let trade = test_trade();
        let mut follower = test_follower();
        follower.max_position_usdc = 10.0;

        let result = build_copy_order(&trade, &follower, "0xleader");

        assert!(result.is_none());
    }

    #[test]
    fn test_build_copy_order_markets_filter_pass() {
        let trade = test_trade();
        let mut follower = test_follower();
        follower.markets_filter =
            Some(serde_json::json!(["condition_456"]));

        let result = build_copy_order(&trade, &follower, "0xleader");

        assert!(result.is_some());
    }

    #[test]
    fn test_build_copy_order_markets_filter_reject() {
        let trade = test_trade();
        let mut follower = test_follower();
        follower.markets_filter =
            Some(serde_json::json!(["other"]));

        let result = build_copy_order(&trade, &follower, "0xleader");

        assert!(result.is_none());
    }

    #[test]
    fn test_build_copy_order_null_filter_passes_all() {
        let trade = test_trade();
        let follower = test_follower(); // markets_filter is None

        let result = build_copy_order(&trade, &follower, "0xleader");

        assert!(result.is_some());
    }

    #[test]
    fn test_build_copy_order_sell_side() {
        let mut trade = test_trade();
        trade.side = "SELL".to_string();
        let follower = test_follower();

        let order = build_copy_order(&trade, &follower, "0xleader").unwrap();

        assert_eq!(order.side, Side::Sell);
    }
}
