use anyhow::Result;
use sqlx::PgPool;

use crate::execution::{ExecutionOrder, OrderResult, OrderStatus, Side};
use crate::strategy::OrderType;

// ---------------------------------------------------------------------------
// Connection pool
// ---------------------------------------------------------------------------

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    tracing::info!("postgres_pool_connecting");
    let pool = PgPool::connect(database_url).await?;
    tracing::info!("postgres_pool_connected");
    Ok(pool)
}

// ---------------------------------------------------------------------------
// Write trade
// ---------------------------------------------------------------------------

pub async fn write_trade(
    pool: &PgPool,
    order: &ExecutionOrder,
    result: &OrderResult,
) -> Result<i64> {
    let side_str = match order.side {
        Side::Buy => "buy",
        Side::Sell => "sell",
    };

    let outcome_str = match order.outcome {
        crate::strategy::Outcome::Up => "UP",
        crate::strategy::Outcome::Down => "DOWN",
    };

    let order_type_str = match &order.order_type {
        OrderType::Market => "market",
        OrderType::Limit { .. } => "limit",
        OrderType::StopLoss { .. } => "stoploss",
        OrderType::TakeProfit { .. } => "take_profit",
    };

    let status_str = match result.status {
        OrderStatus::Filled => "filled",
        OrderStatus::Cancelled => "cancelled",
        OrderStatus::Failed => "failed",
        OrderStatus::Timeout => "timeout",
    };

    let trade_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO trades (
            wallet_id, strategy_id, copy_relationship_id,
            symbol, token_id, side, outcome,
            order_type, price, size_usdc,
            polymarket_order_id, status, filled_price, fee_bps,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, to_timestamp($15))
        RETURNING id
        "#,
    )
    .bind(order.wallet_id as i64)
    .bind(order.strategy_id.map(|id| id as i64))
    .bind(order.copy_relationship_id.map(|id| id as i64))
    .bind(&order.symbol)
    .bind(&order.token_id)
    .bind(side_str)
    .bind(outcome_str)
    .bind(order_type_str)
    .bind(order.price)
    .bind(order.size_usdc)
    .bind(&result.polymarket_order_id)
    .bind(status_str)
    .bind(result.filled_price)
    .bind(result.fee_bps.map(|b| b as i16))
    .bind(order.created_at)
    .fetch_one(pool)
    .await?;

    tracing::info!(trade_id, symbol = %order.symbol, status = status_str, "trade_written");
    Ok(trade_id)
}

// ---------------------------------------------------------------------------
// Write copy trade
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub async fn write_copy_trade(
    pool: &PgPool,
    copy_relationship_id: i64,
    follower_trade_id: Option<i64>,
    leader_address: &str,
    leader_market_id: &str,
    leader_outcome: &str,
    leader_price: f64,
    leader_size_usdc: f64,
    leader_tx_hash: &str,
    follower_price: Option<f64>,
    status: &str,
    skip_reason: Option<&str>,
) -> Result<i64> {
    let slippage_pct = follower_price.map(|fp| (fp - leader_price) / leader_price);

    let copy_trade_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO copy_trades (
            copy_relationship_id, follower_trade_id,
            leader_address, leader_market_id, leader_outcome,
            leader_price, leader_size_usdc, leader_tx_hash,
            follower_price, slippage_pct,
            status, skip_reason
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id
        "#,
    )
    .bind(copy_relationship_id)
    .bind(follower_trade_id)
    .bind(leader_address)
    .bind(leader_market_id)
    .bind(leader_outcome)
    .bind(leader_price)
    .bind(leader_size_usdc)
    .bind(leader_tx_hash)
    .bind(follower_price)
    .bind(slippage_pct)
    .bind(status)
    .bind(skip_reason)
    .fetch_one(pool)
    .await?;

    tracing::info!(copy_trade_id, status, "copy_trade_written");
    Ok(copy_trade_id)
}

// ---------------------------------------------------------------------------
// CopyRelationship
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CopyRelationship {
    pub id: i64,
    pub follower_wallet_id: i64,
    pub size_mode: String,
    pub size_value: f64,
    pub max_position_usdc: f64,
    pub markets_filter: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Get active followers
// ---------------------------------------------------------------------------

pub async fn get_active_followers(
    pool: &PgPool,
    watched_address: &str,
) -> Result<Vec<CopyRelationship>> {
    let rows = sqlx::query_as::<_, (i64, i64, String, f64, f64, Option<serde_json::Value>)>(
        r#"
        SELECT cr.id, cr.follower_wallet_id, cr.size_mode, cr.size_value,
               cr.max_position_usdc, cr.markets_filter
        FROM copy_relationships cr
        JOIN watched_wallets ww ON ww.id = cr.watched_wallet_id
        WHERE ww.address = $1
          AND cr.is_active = true
        "#,
    )
    .bind(watched_address)
    .fetch_all(pool)
    .await?;

    let followers = rows
        .into_iter()
        .map(|(id, follower_wallet_id, size_mode, size_value, max_position_usdc, markets_filter)| {
            CopyRelationship {
                id,
                follower_wallet_id,
                size_mode,
                size_value,
                max_position_usdc,
                markets_filter,
            }
        })
        .collect();

    Ok(followers)
}

// ---------------------------------------------------------------------------
// Load watched addresses
// ---------------------------------------------------------------------------

pub async fn load_watched_addresses(pool: &PgPool) -> Result<Vec<String>> {
    let addresses = sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT ww.address
        FROM watched_wallets ww
        JOIN copy_relationships cr ON cr.watched_wallet_id = ww.id
        WHERE cr.is_active = true
        "#,
    )
    .fetch_all(pool)
    .await?;

    tracing::info!(count = addresses.len(), "watched_addresses_loaded");
    Ok(addresses)
}
