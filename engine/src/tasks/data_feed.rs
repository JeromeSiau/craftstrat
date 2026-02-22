use std::time::Duration;

use tokio::sync::mpsc;
use tokio::task::JoinSet;

use super::SharedState;
use crate::fetcher::websocket::WsCommand;

pub fn spawn_ws_feed(
    state: &SharedState,
    ws_cmd_rx: mpsc::Receiver<WsCommand>,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    let ws_books = state.books.clone();
    let ws_url = state.config.clob_ws_url.clone();
    tasks.spawn(async move {
        crate::fetcher::websocket::run_ws_feed(ws_url, ws_books, ws_cmd_rx).await;
        anyhow::bail!("ws_feed exited unexpectedly")
    });
}

pub fn spawn_price_poller(
    state: &SharedState,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    let price_url = state.config.binance_api_url.clone();
    let price_http = state.http.direct().clone();
    let price_cache = state.prices.clone();
    let binance_symbols = state.config.binance_symbols();
    tasks.spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            let fetches: Vec<_> = binance_symbols
                .iter()
                .map(|sym| {
                    let http = &price_http;
                    let url = format!("{price_url}?symbol={sym}");
                    let sym = sym.clone();
                    async move {
                        match http.get(&url).send().await {
                            Ok(resp) if resp.status().is_success() => {
                                match resp.json::<serde_json::Value>().await {
                                    Ok(body) => {
                                        if let Some(p) = body["price"]
                                            .as_str()
                                            .and_then(|s| s.parse::<f64>().ok())
                                        {
                                            return Some((sym, p));
                                        }
                                        tracing::warn!(symbol = %sym, "binance_parse_failed");
                                    }
                                    Err(e) => tracing::warn!(symbol = %sym, error = %e, "binance_json_error"),
                                }
                            }
                            Ok(resp) => tracing::warn!(symbol = %sym, status = %resp.status(), "binance_http_error"),
                            Err(e) => tracing::warn!(symbol = %sym, error = %e, "binance_request_failed"),
                        }
                        None
                    }
                })
                .collect();
            let results = futures_util::future::join_all(fetches).await;
            let mut cache = price_cache.write().await;
            for result in results.into_iter().flatten() {
                cache.insert(result.0, result.1);
            }
        }
    });
}

pub fn spawn_market_discovery(
    state: &SharedState,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    let disc_markets = state.markets.clone();
    let disc_http = state.http.clone();
    let disc_cfg = state.config.clone();
    let disc_prices = state.prices.clone();
    let ws_cmd_tx = state.ws_cmd_tx.clone();
    tasks.spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let mut interval = tokio::time::interval(Duration::from_secs(disc_cfg.discovery_interval_secs));
        loop {
            interval.tick().await;
            let current_prices = disc_prices.read().await.clone();
            match crate::fetcher::gamma::discover_markets(
                &disc_http,
                &disc_cfg.gamma_api_url,
                &disc_cfg.sources,
                &current_prices,
            )
            .await
            {
                Ok(found) => {
                    let mut active = disc_markets.write().await;
                    let mut new_tokens = Vec::new();
                    for m in found {
                        if !active.contains_key(&m.condition_id) {
                            tracing::info!(slug = %m.slug, "market_discovered");
                            new_tokens.push(m.token_up.clone());
                            new_tokens.push(m.token_down.clone());
                        }
                        active.insert(m.condition_id.clone(), m);
                    }
                    if !new_tokens.is_empty() {
                        let _ = ws_cmd_tx.send(WsCommand::Subscribe { token_ids: new_tokens }).await;
                    }
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64();
                    let expired: Vec<_> = active
                        .iter()
                        .filter(|(_, m)| m.end_time + 300.0 < now)
                        .map(|(k, _)| k.clone())
                        .collect();
                    for cid in &expired {
                        if let Some(m) = active.remove(cid) {
                            let _ = ws_cmd_tx
                                .send(WsCommand::Unsubscribe {
                                    token_ids: vec![m.token_up, m.token_down],
                                })
                                .await;
                        }
                    }
                }
                Err(e) => tracing::warn!(error = %e, "discovery_failed"),
            }
        }
    });
}

pub fn spawn_tick_builder(
    state: &SharedState,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    let tb_books = state.books.clone();
    let tb_markets = state.markets.clone();
    let tb_prices = state.prices.clone();
    let tb_tick_tx = state.tick_tx.clone();
    let tick_interval_ms = state.config.tick_interval_ms;
    tasks.spawn(async move {
        crate::fetcher::tick_builder::run_tick_builder(
            tb_books,
            tb_markets,
            tb_prices,
            tb_tick_tx,
            Duration::from_millis(tick_interval_ms),
        )
        .await;
        anyhow::bail!("tick_builder exited unexpectedly")
    });
}
