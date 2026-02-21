mod config;
mod fetcher;
mod kafka;
mod storage;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::task::JoinSet;

use config::Config;
use fetcher::models::ActiveMarket;
use fetcher::tick_builder::PriceCache;
use fetcher::websocket::{OrderBookCache, WsCommand};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = Config::from_env()?;
    tracing::info!(sources = cfg.sources.len(), "oddex_engine_starting");

    let books: OrderBookCache = Arc::new(RwLock::new(HashMap::new()));
    let markets: Arc<RwLock<HashMap<String, ActiveMarket>>> = Arc::new(RwLock::new(HashMap::new()));
    let prices: PriceCache = Arc::new(RwLock::new(HashMap::new()));
    let (ws_cmd_tx, ws_cmd_rx) = mpsc::channel::<WsCommand>(64);
    let (tick_tx, _) = broadcast::channel::<fetcher::models::Tick>(1024);

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let mut tasks = JoinSet::new();

    // 1. WebSocket feed
    let ws_books = books.clone();
    let ws_url = cfg.clob_ws_url.clone();
    tasks.spawn(async move {
        fetcher::websocket::run_ws_feed(ws_url, ws_books, ws_cmd_rx).await;
        anyhow::bail!("ws_feed exited unexpectedly")
    });

    // 2. Price poller — all Binance symbols concurrently every 2s
    let price_url = cfg.binance_api_url.clone();
    let price_http = http.clone();
    let price_cache = prices.clone();
    let binance_symbols = cfg.binance_symbols();
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

    // 3. Market discovery (every 60s)
    let disc_markets = markets.clone();
    let disc_http = http.clone();
    let disc_cfg = cfg.clone();
    let disc_prices = prices.clone();
    tasks.spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let mut interval = tokio::time::interval(Duration::from_secs(disc_cfg.discovery_interval_secs));
        loop {
            interval.tick().await;
            let current_prices = disc_prices.read().await.clone();
            match fetcher::gamma::discover_markets(
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

    // 4. Tick builder → broadcast
    let tb_books = books.clone();
    let tb_markets = markets.clone();
    let tb_prices = prices.clone();
    let tb_tick_tx = tick_tx.clone();
    tasks.spawn(async move {
        fetcher::tick_builder::run_tick_builder(
            tb_books,
            tb_markets,
            tb_prices,
            tb_tick_tx,
            Duration::from_millis(cfg.tick_interval_ms),
        )
        .await;
        anyhow::bail!("tick_builder exited unexpectedly")
    });

    // 5. ClickHouse writer
    let ch_client = storage::clickhouse::create_client(&cfg.clickhouse_url);
    let ch_rx = tick_tx.subscribe();
    tasks.spawn(async move {
        storage::clickhouse::run_writer(ch_client, ch_rx).await
    });

    // 6. Kafka publisher
    let kf_producer = kafka::producer::create_producer(&cfg.kafka_brokers)?;
    let kf_rx = tick_tx.subscribe();
    tasks.spawn(async move {
        kafka::producer::run_publisher(kf_producer, kf_rx).await;
        Ok(())
    });

    tracing::info!("oddex_engine_running");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("oddex_engine_shutdown");
        }
        result = async { tasks.join_next().await.unwrap() } => {
            match result {
                Ok(Ok(())) => tracing::error!("task_exited_unexpectedly"),
                Ok(Err(e)) => tracing::error!(error = %e, "task_fatal"),
                Err(e) => tracing::error!(error = %e, "task_panicked"),
            }
        }
    }

    // Graceful shutdown: drop the broadcast sender so receivers get None
    drop(tick_tx);
    // Give writers time to flush
    tokio::time::sleep(Duration::from_millis(500)).await;
    tasks.shutdown().await;
    tracing::info!("oddex_engine_stopped");
    Ok(())
}
