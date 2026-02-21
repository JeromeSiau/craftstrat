use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;

use super::models::ActiveMarket;
use crate::config::SymbolConfig;

#[derive(Debug, Deserialize)]
struct GammaEvent {
    markets: Option<Vec<GammaMarket>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GammaMarket {
    condition_id: Option<String>,
    outcomes: Option<String>,
    clob_token_ids: Option<String>,
    end_date: Option<String>,
}

fn duration_suffix(secs: u32) -> &'static str {
    match secs {
        300 => "5m",
        900 => "15m",
        3600 => "1h",
        14400 => "4h",
        86400 => "24h",
        _ => "15m",
    }
}

pub async fn discover_markets(
    client: &reqwest::Client,
    gamma_url: &str,
    symbols: &[SymbolConfig],
    prices: &HashMap<String, f64>,
) -> Result<Vec<ActiveMarket>> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let mut markets = Vec::new();

    for sym in symbols {
        let ref_price = prices.get(&sym.binance_symbol).copied().unwrap_or(0.0) as f32;

        for &slot_duration in &sym.slot_durations {
            let current_slot = (now / slot_duration as u64) * slot_duration as u64;
            let suffix = duration_suffix(slot_duration);

            for offset in 0..2u64 {
                let slot_ts = current_slot + offset * slot_duration as u64;
                let slug = format!("{}-updown-{suffix}-{slot_ts}", sym.slug_prefix);
                let url = format!("{gamma_url}/events?slug={slug}");

                let resp = match client.get(&url).send().await {
                    Ok(r) if r.status().is_success() => r,
                    _ => continue,
                };

                let events: Vec<GammaEvent> = match resp.json().await {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                for event in &events {
                    let Some(mkts) = &event.markets else { continue };
                    for mkt in mkts {
                        let Some(ref cid) = mkt.condition_id else { continue };
                        let tokens = parse_json_str_array(mkt.clob_token_ids.as_deref());
                        let outcomes = parse_json_str_array(mkt.outcomes.as_deref());
                        if tokens.len() < 2 || outcomes.len() < 2 {
                            continue;
                        }

                        let end_time = mkt
                            .end_date
                            .as_deref()
                            .and_then(parse_iso_ts)
                            .unwrap_or((slot_ts + slot_duration as u64) as f64);

                        markets.push(ActiveMarket {
                            condition_id: cid.clone(),
                            slug: slug.clone(),
                            binance_symbol: sym.binance_symbol.clone(),
                            slot_ts: slot_ts as u32,
                            slot_duration,
                            end_time,
                            token_up: tokens[0].clone(),
                            token_down: tokens[1].clone(),
                            ref_price_start: if ref_price > 0.0 { Some(ref_price) } else { None },
                        });
                        break;
                    }
                }
            }
        }
    }

    Ok(markets)
}

fn parse_json_str_array(s: Option<&str>) -> Vec<String> {
    s.and_then(|v| serde_json::from_str::<Vec<String>>(v).ok())
        .unwrap_or_default()
}

fn parse_iso_ts(s: &str) -> Option<f64> {
    chrono::DateTime::parse_from_rfc3339(s)
        .or_else(|_| chrono::DateTime::parse_from_rfc3339(&format!("{s}+00:00")))
        .ok()
        .map(|dt| dt.timestamp() as f64)
}
