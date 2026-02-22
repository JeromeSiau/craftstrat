mod data_feed;
mod engine_tasks;
mod execution_tasks;
mod persistence;
mod writers;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tokio::task::JoinSet;

use crate::config::Config;
use crate::fetcher::models::{ActiveMarket, Tick};
use crate::fetcher::tick_builder::PriceCache;
use crate::fetcher::websocket::{OrderBookCache, WsCommand};
use crate::proxy::HttpPool;

pub struct SpawnedHandles {
    pub registry: crate::strategy::registry::AssignmentRegistry,
    pub exec_queue: Arc<Mutex<crate::execution::queue::ExecutionQueue>>,
    pub db: sqlx::PgPool,
}

pub struct SharedState {
    pub config: Config,
    pub books: OrderBookCache,
    pub markets: Arc<RwLock<HashMap<String, ActiveMarket>>>,
    pub prices: PriceCache,
    pub tick_tx: broadcast::Sender<Tick>,
    pub ws_cmd_tx: mpsc::Sender<WsCommand>,
    pub http: HttpPool,
}

pub async fn spawn_all(
    state: &SharedState,
    ws_cmd_rx: mpsc::Receiver<WsCommand>,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) -> anyhow::Result<SpawnedHandles> {
    // Data feed tasks
    data_feed::spawn_ws_feed(state, ws_cmd_rx, tasks);
    data_feed::spawn_price_poller(state, tasks);
    data_feed::spawn_market_discovery(state, tasks);
    data_feed::spawn_tick_builder(state, tasks);

    // Writer tasks
    writers::spawn_clickhouse_writer(state, tasks);
    writers::spawn_kafka_publisher(state, tasks)?;

    // Strategy engine
    let engine_registry = crate::strategy::registry::AssignmentRegistry::new();
    let (signal_tx, signal_rx) = mpsc::channel::<crate::strategy::EngineOutput>(256);

    engine_tasks::spawn_strategy_engine(state, engine_registry.clone(), signal_tx, tasks);

    // PostgreSQL connection pool
    let db = crate::storage::postgres::create_pool(&state.config.database_url).await?;

    // Shared execution queue
    let exec_queue = Arc::new(Mutex::new(
        crate::execution::queue::ExecutionQueue::new(state.config.max_orders_per_day),
    ));

    // Execution pipeline (replaces signal logger)
    execution_tasks::spawn_execution(
        state,
        engine_registry.clone(),
        signal_rx,
        exec_queue.clone(),
        db.clone(),
        tasks,
    );

    // Copy trading watcher
    execution_tasks::spawn_watcher(state, exec_queue.clone(), db.clone(), tasks);

    // Redis state persistence
    persistence::spawn_redis_state_persister(state, engine_registry.clone(), tasks);

    Ok(SpawnedHandles {
        registry: engine_registry,
        exec_queue,
        db,
    })
}
