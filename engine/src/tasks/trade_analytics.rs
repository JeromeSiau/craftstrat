use std::time::Duration;

use anyhow::Result;
use clickhouse::Client;
use serde::Deserialize;
use sqlx::PgPool;

use crate::execution::analytics::markout_bps_60s;
use crate::execution::Side;

#[derive(Debug, clickhouse::Row, Deserialize)]
struct MarkoutTick {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    captured_at: time::OffsetDateTime,
    mid_up: f32,
    mid_down: f32,
}

pub async fn run_trade_analytics(ch: Client, db: PgPool) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        let pending: Vec<(i64, String, String, String, f64, i64)> = match sqlx::query_as(
            r#"
            SELECT
                id,
                symbol,
                side,
                outcome,
                filled_price::float8,
                EXTRACT(EPOCH FROM COALESCE(executed_at, created_at))::bigint AS executed_ts
            FROM trades
            WHERE symbol IS NOT NULL
                AND filled_price IS NOT NULL
                AND markout_bps_60s IS NULL
                AND COALESCE(executed_at, created_at) <= NOW() - INTERVAL '60 seconds'
                AND status IN ('filled', 'won', 'lost')
            ORDER BY COALESCE(executed_at, created_at) ASC
            LIMIT 200
            "#,
        )
        .fetch_all(&db)
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                tracing::warn!(error = %e, "trade_analytics_query_failed");
                continue;
            }
        };

        for (trade_id, symbol, side, outcome, filled_price, executed_ts) in pending {
            let markout_from_ms = (executed_ts + 60) * 1_000;
            let mut cursor = match ch
                .query(
                    "SELECT captured_at, mid_up, mid_down
                     FROM slot_snapshots
                     WHERE symbol = ? AND captured_at >= fromUnixTimestamp64Milli(?)
                     ORDER BY captured_at ASC
                     LIMIT 1",
                )
                .bind(symbol.as_str())
                .bind(markout_from_ms)
                .fetch::<MarkoutTick>()
            {
                Ok(cursor) => cursor,
                Err(e) => {
                    tracing::warn!(trade_id, error = %e, "trade_analytics_markout_query_failed");
                    continue;
                }
            };

            let Some(markout_tick) = (match cursor.next().await {
                Ok(row) => row,
                Err(e) => {
                    tracing::warn!(trade_id, error = %e, "trade_analytics_markout_fetch_failed");
                    continue;
                }
            }) else {
                continue;
            };

            let Some(side) = parse_side(&side) else {
                continue;
            };

            let markout_price = match outcome.as_str() {
                "UP" => markout_tick.mid_up as f64,
                "DOWN" => markout_tick.mid_down as f64,
                _ => continue,
            };

            let Some(markout_bps) = markout_bps_60s(side, Some(filled_price), Some(markout_price))
            else {
                continue;
            };

            if let Err(e) = sqlx::query(
                "UPDATE trades
                 SET markout_price_60s = $1,
                     markout_at_60s = to_timestamp($2),
                     markout_bps_60s = $3
                 WHERE id = $4",
            )
            .bind(markout_price)
            .bind(markout_tick.captured_at.unix_timestamp())
            .bind(markout_bps)
            .bind(trade_id)
            .execute(&db)
            .await
            {
                tracing::warn!(trade_id, error = %e, "trade_analytics_update_failed");
            }
        }
    }
}

fn parse_side(value: &str) -> Option<Side> {
    match value {
        "buy" => Some(Side::Buy),
        "sell" => Some(Side::Sell),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_side_labels() {
        assert_eq!(parse_side("buy"), Some(Side::Buy));
        assert_eq!(parse_side("sell"), Some(Side::Sell));
        assert_eq!(parse_side("other"), None);
    }
}
