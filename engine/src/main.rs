mod config;
mod execution;
mod fetcher;
mod kafka;
mod storage;
mod strategy;
mod tasks;
mod watcher;
mod backtest;
mod api;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::task::JoinSet;

use config::Config;
use fetcher::tick_builder::PriceCache;
use fetcher::websocket::{OrderBookCache, WsCommand};
use tasks::SharedState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = Config::from_env()?;
    tracing::info!(sources = cfg.sources.len(), "oddex_engine_starting");

    let (ws_cmd_tx, ws_cmd_rx) = mpsc::channel::<WsCommand>(64);
    let (tick_tx, _) = broadcast::channel::<fetcher::models::Tick>(1024);

    let state = SharedState {
        config: cfg,
        books: OrderBookCache::new(),
        markets: Arc::new(RwLock::new(HashMap::new())),
        prices: PriceCache::new(),
        tick_tx: tick_tx.clone(),
        ws_cmd_tx,
        http: reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?,
    };

    let mut tasks = JoinSet::new();
    tasks::spawn_all(&state, ws_cmd_rx, &mut tasks).await?;

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

    drop(tick_tx);
    tokio::time::sleep(Duration::from_millis(500)).await;
    tasks.shutdown().await;
    tracing::info!("oddex_engine_stopped");
    Ok(())
}
