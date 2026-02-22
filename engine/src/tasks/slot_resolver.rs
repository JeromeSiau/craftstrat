use std::time::Duration;

use anyhow::Result;
use clickhouse::Client;
use serde::Deserialize;

use crate::proxy::HttpPool;

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
) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        let unresolved: Vec<UnresolvedSlot> = ch
            .query(
                "SELECT DISTINCT symbol FROM slot_snapshots \
                 WHERE winner IS NULL AND pct_into_slot >= 1.0 \
                 ORDER BY symbol",
            )
            .fetch_all()
            .await?;

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

            ch.query(
                "ALTER TABLE slot_snapshots UPDATE winner = ? WHERE symbol = ?",
            )
            .bind(winner)
            .bind(slot.symbol.as_str())
            .execute()
            .await?;

            tracing::info!(
                slug = %slot.symbol,
                winner = winner,
                label = if winner == 1 { "UP" } else { "DOWN" },
                "slot_resolved",
            );
        }
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
