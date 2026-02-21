mod config;
mod fetcher;
mod kafka;
mod storage;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

use config::Config;
use fetcher::models::ActiveMarket;
use fetcher::tick_builder::PriceCache;
use fetcher::websocket::{OrderBookCache, WsCommand};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = Config::from_env()?;
    tracing::info!(symbols = cfg.symbols.len(), "oddex_engine_starting");

    let books: OrderBookCache = Arc::new(RwLock::new(HashMap::new()));
    let markets: Arc<RwLock<HashMap<String, ActiveMarket>>> = Arc::new(RwLock::new(HashMap::new()));
    let prices: PriceCache = Arc::new(RwLock::new(HashMap::new()));
    let (ws_cmd_tx, ws_cmd_rx) = mpsc::channel::<WsCommand>(64);
    let (ch_tx, ch_rx) = mpsc::channel(1000);
    let (kafka_tx, kafka_rx) = mpsc::channel(1000);

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    // 1. WebSocket feed
    let ws_books = books.clone();
    let ws_url = cfg.clob_ws_url.clone();
    tokio::spawn(async move {
        fetcher::websocket::run_ws_feed(ws_url, ws_books, ws_cmd_rx).await;
    });

    // 2. Price poller — all Binance symbols every 2s
    let price_url = cfg.binance_api_url.clone();
    let price_http = http.clone();
    let price_cache = prices.clone();
    let binance_symbols = cfg.binance_symbols();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            for sym in &binance_symbols {
                if let Ok(resp) = price_http.get(format!("{price_url}?symbol={sym}")).send().await {
                    if let Ok(body) = resp.json::<serde_json::Value>().await {
                        if let Some(p) = body["price"].as_str().and_then(|s| s.parse::<f64>().ok()) {
                            price_cache.write().await.insert(sym.clone(), p);
                        }
                    }
                }
            }
        }
    });

    // 3. Market discovery (every 60s)
    let disc_markets = markets.clone();
    let disc_http = http.clone();
    let disc_cfg = cfg.clone();
    let disc_prices = prices.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let mut interval = tokio::time::interval(Duration::from_secs(disc_cfg.discovery_interval_secs));
        loop {
            interval.tick().await;
            let current_prices = disc_prices.read().await.clone();
            match fetcher::gamma::discover_markets(
                &disc_http,
                &disc_cfg.gamma_api_url,
                &disc_cfg.symbols,
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

    // 4. Tick builder -> fan out to ClickHouse + Kafka
    let tb_books = books.clone();
    let tb_markets = markets.clone();
    let tb_prices = prices.clone();
    tokio::spawn(async move {
        let (internal_tx, mut internal_rx) = mpsc::channel::<fetcher::models::Tick>(1000);
        let ch = ch_tx;
        let kf = kafka_tx;
        tokio::spawn(async move {
            while let Some(tick) = internal_rx.recv().await {
                let _ = ch.send(tick.clone()).await;
                let _ = kf.send(tick).await;
            }
        });
        fetcher::tick_builder::run_tick_builder(
            tb_books,
            tb_markets,
            tb_prices,
            internal_tx,
            Duration::from_millis(cfg.tick_interval_ms),
        )
        .await;
    });

    // 5. ClickHouse writer
    let ch_client = storage::clickhouse::create_client(&cfg.clickhouse_url);
    tokio::spawn(async move {
        if let Err(e) = storage::clickhouse::run_writer(ch_client, ch_rx).await {
            tracing::error!(error = %e, "clickhouse_fatal");
        }
    });

    // 6. Kafka publisher
    let kf_producer = kafka::producer::create_producer(&cfg.kafka_brokers)?;
    tokio::spawn(async move {
        kafka::producer::run_publisher(kf_producer, kafka_rx).await;
    });

    tracing::info!("oddex_engine_running");
    tokio::signal::ctrl_c().await?;
    tracing::info!("oddex_engine_shutdown");
    Ok(())
}
