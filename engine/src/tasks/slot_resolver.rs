use std::time::Duration;

use anyhow::Result;
use clickhouse::Client;
use metrics::gauge;
use serde::Deserialize;
use sqlx::PgPool;

use crate::metrics as m;
use crate::proxy::HttpPool;
use crate::strategy::registry::AssignmentRegistry;
use crate::strategy::Outcome;

#[derive(Debug, clickhouse::Row, Deserialize)]
struct UnresolvedSlot {
    symbol: String,
}

#[derive(Debug, Deserialize)]
struct GammaEvent {
    markets: Option<Vec<GammaMarket>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GammaMarket {
    outcome_prices: Option<String>,
    #[serde(default)]
    closed: bool,
}

pub async fn run_slot_resolver(
    ch: Client,
    http: HttpPool,
    gamma_url: String,
    db: PgPool,
    registry: AssignmentRegistry,
) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        let unresolved: Vec<UnresolvedSlot> = match ch
            .query(
                "SELECT DISTINCT symbol FROM slot_snapshots \
                 WHERE winner IS NULL AND pct_into_slot >= 1.0 \
                 ORDER BY symbol",
            )
            .fetch_all()
            .await
        {
            Ok(rows) => rows,
            Err(e) => {
                tracing::warn!(error = %e, "slot_resolver_query_failed");
                continue;
            }
        };

        if unresolved.is_empty() {
            continue;
        }

        tracing::info!(count = unresolved.len(), "slot_resolver_checking");

        for slot in &unresolved {
            let url = format!("{gamma_url}/events?slug={}", slot.symbol);
            let resp = match http.proxied().get(&url).send().await {
                Ok(r) if r.status().is_success() => r,
                Ok(r) => {
                    tracing::warn!(slug = %slot.symbol, status = %r.status(), "slot_resolver_http_error");
                    continue;
                }
                Err(e) => {
                    tracing::warn!(slug = %slot.symbol, error = %e, "slot_resolver_request_failed");
                    continue;
                }
            };

            let events: Vec<GammaEvent> = match resp.json().await {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!(slug = %slot.symbol, error = %e, "slot_resolver_parse_failed");
                    continue;
                }
            };

            let Some(winner) = extract_winner(&events) else {
                continue;
            };

            // 1. Update ClickHouse analytics
            if let Err(e) = ch
                .query("ALTER TABLE slot_snapshots UPDATE winner = ? WHERE symbol = ?")
                .bind(winner)
                .bind(slot.symbol.as_str())
                .execute()
                .await
            {
                tracing::warn!(slug = %slot.symbol, error = %e, "slot_resolver_update_failed");
                continue;
            }

            let winning_outcome = if winner == 1 { "UP" } else { "DOWN" };

            tracing::info!(
                slug = %slot.symbol,
                winner = winner,
                label = winning_outcome,
                "slot_resolved",
            );

            // 2. Resolve open trades in PostgreSQL
            resolve_trades(&db, &registry, &slot.symbol, winning_outcome).await;
        }
    }
}

// ---------------------------------------------------------------------------
// resolve_trades — close open trades and clear strategy positions
// ---------------------------------------------------------------------------

async fn resolve_trades(
    db: &PgPool,
    registry: &AssignmentRegistry,
    symbol: &str,
    winning_outcome: &str,
) {
    // Find all open trades for this symbol
    let rows: Vec<(i64, i64, Option<i64>, String, Option<f64>, f64)> = match sqlx::query_as(
        "SELECT id, wallet_id, strategy_id, outcome, price, size_usdc \
         FROM trades WHERE symbol = $1 AND status = 'filled'",
    )
    .bind(symbol)
    .fetch_all(db)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(symbol, error = %e, "resolve_trades_query_failed");
            return;
        }
    };

    if rows.is_empty() {
        return;
    }

    for (trade_id, wallet_id, strategy_id, outcome, entry_price, size_usdc) in &rows {
        let is_winner = outcome == winning_outcome;
        let resolved_price: f64 = if is_winner { 1.0 } else { 0.0 };
        let new_status = if is_winner { "won" } else { "lost" };
        let entry = entry_price.unwrap_or(0.5);
        let pnl = (resolved_price - entry) * size_usdc;

        // Update trade record
        if let Err(e) = sqlx::query(
            "UPDATE trades SET status = $1, filled_price = $2 WHERE id = $3",
        )
        .bind(new_status)
        .bind(resolved_price)
        .bind(trade_id)
        .execute(db)
        .await
        {
            tracing::warn!(trade_id, error = %e, "resolve_trade_update_failed");
            continue;
        }

        tracing::info!(
            trade_id,
            wallet_id,
            symbol,
            outcome = outcome.as_str(),
            result = new_status,
            pnl = format!("{pnl:.2}"),
            "trade_resolved",
        );

        // Clear position in strategy state
        if let Some(sid) = strategy_id {
            let winning = if winning_outcome == "UP" {
                Outcome::Up
            } else {
                Outcome::Down
            };
            clear_position(registry, *wallet_id as u64, *sid as u64, pnl, symbol, winning).await;
        }
    }
}

// ---------------------------------------------------------------------------
// clear_position — reset the assignment's in-memory position and update PnL
// ---------------------------------------------------------------------------

async fn clear_position(
    registry: &AssignmentRegistry,
    wallet_id: u64,
    strategy_id: u64,
    pnl: f64,
    symbol: &str,
    _winning_outcome: Outcome,
) {
    let reg = registry.read().await;

    // The registry is keyed by market prefix (e.g. "btc-updown-15m").
    // Find the assignment matching (wallet_id, strategy_id).
    let assignment = reg.values().flatten().find(|a| {
        a.wallet_id == wallet_id && a.strategy_id == strategy_id
    });

    let assignment = match assignment {
        Some(a) => a,
        None => return, // strategy may have been deactivated
    };

    let mut state = match assignment.state.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    // Only clear if the position matches this symbol
    let should_clear = state
        .position
        .as_ref()
        .map(|p| p.symbol == symbol)
        .unwrap_or(false);

    if should_clear {
        state.position = None;
        state.pnl += pnl;
        state.daily_pnl += pnl;
        gauge!(m::PNL_USDC).increment(pnl);

        tracing::info!(
            wallet_id,
            strategy_id,
            pnl = format!("{pnl:.2}"),
            "position_cleared_by_resolution",
        );
    }
}

fn extract_winner(events: &[GammaEvent]) -> Option<i8> {
    for event in events {
        let markets = event.markets.as_ref()?;
        for market in markets {
            if !market.closed {
                return None;
            }
            let prices_str = market.outcome_prices.as_deref()?;
            let prices: Vec<String> = serde_json::from_str(prices_str).ok()?;
            if prices.len() < 2 {
                return None;
            }
            // outcomePrices: ["1", "0"] = UP won, ["0", "1"] = DOWN won
            let up_price: f64 = prices[0].parse().unwrap_or(0.0);
            let down_price: f64 = prices[1].parse().unwrap_or(0.0);
            if up_price > 0.5 {
                return Some(1); // UP
            } else if down_price > 0.5 {
                return Some(2); // DOWN
            }
        }
    }
    None
}
