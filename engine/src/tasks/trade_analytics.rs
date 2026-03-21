use std::time::Duration;

use anyhow::Result;
use clickhouse::Client;
use serde::Deserialize;
use sqlx::PgPool;

use crate::execution::analytics::markout_bps_60s;
use crate::execution::Side;

const MARKOUT_DELAY_SECS: i64 = 60;
const PRICE_EPSILON: f64 = 1e-6;

#[derive(Debug, clickhouse::Row, Deserialize)]
struct MarkoutTick {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    captured_at: time::OffsetDateTime,
    mid_up: f32,
    mid_down: f32,
}

#[derive(Debug, sqlx::FromRow)]
struct TradeAnalyticsCandidate {
    id: i64,
    symbol: String,
    side: String,
    outcome: String,
    filled_price: Option<f64>,
    reference_price: Option<f64>,
    resolved_price: Option<f64>,
    executed_ts: i64,
}

pub async fn run_trade_analytics(ch: Client, db: PgPool) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        let pending: Vec<TradeAnalyticsCandidate> = match sqlx::query_as(
            r#"
            SELECT
                id,
                symbol,
                side,
                outcome,
                filled_price::float8,
                reference_price::float8,
                resolved_price::float8,
                EXTRACT(EPOCH FROM COALESCE(executed_at, created_at))::bigint AS executed_ts
            FROM trades
            WHERE symbol IS NOT NULL
                AND COALESCE(filled_price, reference_price) IS NOT NULL
                AND COALESCE(executed_at, created_at) <= NOW() - INTERVAL '60 seconds'
                AND status IN ('filled', 'won', 'lost')
                AND (
                    markout_bps_60s IS NULL
                    OR (
                        filled_price IS NOT NULL
                        AND reference_price IS NOT NULL
                        AND resolved_price IS NOT NULL
                        AND ABS(filled_price::float8 - resolved_price::float8) < 0.000001
                        AND ABS(reference_price::float8 - filled_price::float8) > 0.000001
                    )
                )
            ORDER BY
                CASE WHEN markout_bps_60s IS NULL THEN 0 ELSE 1 END,
                COALESCE(executed_at, created_at) ASC
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

        for candidate in pending {
            let Some(filled_price) = effective_fill_price(&candidate) else {
                continue;
            };

            let markout_from_ms = (candidate.executed_ts + MARKOUT_DELAY_SECS) * 1_000;
            let mut cursor = match ch
                .query(
                    "SELECT captured_at, mid_up, mid_down
                     FROM slot_snapshots
                     WHERE symbol = ? AND captured_at >= fromUnixTimestamp64Milli(?)
                     ORDER BY captured_at ASC
                     LIMIT 1",
                )
                .bind(candidate.symbol.as_str())
                .bind(markout_from_ms)
                .fetch::<MarkoutTick>()
            {
                Ok(cursor) => cursor,
                Err(e) => {
                    tracing::warn!(
                        trade_id = candidate.id,
                        error = %e,
                        "trade_analytics_markout_query_failed"
                    );
                    continue;
                }
            };

            let Some(markout_tick) = (match cursor.next().await {
                Ok(row) => row,
                Err(e) => {
                    tracing::warn!(
                        trade_id = candidate.id,
                        error = %e,
                        "trade_analytics_markout_fetch_failed"
                    );
                    continue;
                }
            }) else {
                continue;
            };

            let Some(side) = parse_side(&candidate.side) else {
                continue;
            };

            let markout_price = match candidate.outcome.as_str() {
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
            .bind(candidate.id)
            .execute(&db)
            .await
            {
                tracing::warn!(
                    trade_id = candidate.id,
                    error = %e,
                    "trade_analytics_update_failed"
                );
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

fn effective_fill_price(candidate: &TradeAnalyticsCandidate) -> Option<f64> {
    if is_resolution_overwrite(
        candidate.filled_price,
        candidate.reference_price,
        candidate.resolved_price,
    ) {
        candidate.reference_price.or(candidate.filled_price)
    } else {
        candidate.filled_price.or(candidate.reference_price)
    }
}

fn is_resolution_overwrite(
    filled_price: Option<f64>,
    reference_price: Option<f64>,
    resolved_price: Option<f64>,
) -> bool {
    let (Some(filled_price), Some(reference_price), Some(resolved_price)) =
        (filled_price, reference_price, resolved_price)
    else {
        return false;
    };

    (filled_price - resolved_price).abs() < PRICE_EPSILON
        && (reference_price - filled_price).abs() > PRICE_EPSILON
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

    #[test]
    fn detects_resolution_overwrite_when_fill_matches_resolution() {
        assert!(is_resolution_overwrite(Some(1.0), Some(0.39), Some(1.0)));
        assert!(!is_resolution_overwrite(Some(0.39), Some(0.39), Some(1.0)));
    }

    #[test]
    fn uses_reference_price_for_backfill_when_fill_was_overwritten() {
        let candidate = TradeAnalyticsCandidate {
            id: 1,
            symbol: "btc-updown-15m".into(),
            side: "buy".into(),
            outcome: "UP".into(),
            filled_price: Some(1.0),
            reference_price: Some(0.39),
            resolved_price: Some(1.0),
            executed_ts: 0,
        };

        assert_eq!(effective_fill_price(&candidate), Some(0.39));
    }

    #[test]
    fn keeps_true_filled_price_when_trade_is_not_suspicious() {
        let candidate = TradeAnalyticsCandidate {
            id: 1,
            symbol: "btc-updown-15m".into(),
            side: "buy".into(),
            outcome: "UP".into(),
            filled_price: Some(0.41),
            reference_price: Some(0.40),
            resolved_price: Some(1.0),
            executed_ts: 0,
        };

        assert_eq!(effective_fill_price(&candidate), Some(0.41));
    }
}
