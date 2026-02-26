mod config;
mod execution;
mod fetcher;
mod kafka;
mod proxy;
mod storage;
mod strategy;
mod tasks;
mod watcher;
mod backtest;
mod stats;
mod api;
mod healthcheck;
mod supervisor;
mod metrics;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::task::JoinSet;

use config::Config;
use fetcher::tick_builder::PriceCache;
use fetcher::websocket::{OrderBookCache, WsCommand};
use proxy::HttpPool;
use tasks::SharedState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let prometheus_handle = metrics::init();
    let cfg = Config::from_env()?;
    tracing::info!(sources = cfg.sources.len(), "craftstrat_engine_starting");

    healthcheck::wait_for_services(
        &cfg.clickhouse_url,
        &cfg.redis_url,
        &cfg.database_url,
    )
    .await?;

    let (ws_cmd_tx, ws_cmd_rx) = mpsc::channel::<WsCommand>(64);
    let (tick_tx, _) = broadcast::channel::<fetcher::models::Tick>(1024);

    let http = HttpPool::new(&cfg.proxy_urls, Duration::from_secs(10))?;

    let state = SharedState {
        config: cfg,
        books: OrderBookCache::new(),
        markets: Arc::new(RwLock::new(HashMap::new())),
        prices: PriceCache::new(),
        tick_tx: tick_tx.clone(),
        ws_cmd_tx,
        http,
    };

    let mut tasks = JoinSet::new();
    let handles = tasks::spawn_all(&state, ws_cmd_rx, &mut tasks).await?;

    // Internal API server
    let ch_client = storage::clickhouse::create_client(&state.config.clickhouse_url);
    let redis_client = redis::Client::open(state.config.redis_url.as_str())?;
    let redis_conn = redis_client.get_multiplexed_tokio_connection().await?;
    // Builder Relayer client (for Safe deployment via API handlers)
    let relayer_credentials = execution::orders::BuilderCredentials {
        api_key: state.config.builder_api_key.clone(),
        secret: state.config.builder_secret.clone(),
        passphrase: state.config.builder_passphrase.clone(),
    };
    let relayer_client = std::sync::Arc::new(execution::relayer::RelayerClient::new(
        state.http.clone(),
        &state.config.relayer_url,
        relayer_credentials,
        handles.wallet_keys.clone(),
    ));

    let api_state = std::sync::Arc::new(api::state::ApiState {
        registry: handles.registry,
        exec_queue: handles.exec_queue,
        db: handles.db,
        ch: ch_client,
        redis: Some(redis_conn),
        start_time: std::time::Instant::now(),
        tick_count: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        prometheus: prometheus_handle,
        wallet_keys: handles.wallet_keys,
        relayer: relayer_client,
    });
    let api_port = state.config.api_port;
    tasks.spawn(async move {
        api::serve(api_state, api_port).await
    });

    tracing::info!("craftstrat_engine_running");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("craftstrat_engine_shutdown");
        }
        result = async { tasks.join_next().await.unwrap() } => {
            match result {
                Ok(Ok(())) => tracing::error!("task_exited_unexpectedly"),
                Ok(Err(e)) => tracing::error!(error = %e, "task_fatal"),
                Err(e) => tracing::error!(error = %e, "task_panicked"),
            }
        }
    }

    drop(tick_tx);
    tokio::time::sleep(Duration::from_millis(500)).await;
    tasks.shutdown().await;
    tracing::info!("craftstrat_engine_stopped");
    Ok(())
}
