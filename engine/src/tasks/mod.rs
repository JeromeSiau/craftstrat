pub mod api_fetch_task;
mod data_feed;
mod engine_tasks;
mod execution_tasks;
mod json_path;
pub mod model_score_task;
mod persistence;
mod slot_resolver;
mod trade_analytics;
mod writers;

use std::collections::HashMap;
use std::sync::Arc;

use alloy::primitives::Address;
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tokio::task::JoinSet;

use crate::config::Config;
use crate::fetcher::models::{ActiveMarket, Tick};
use crate::fetcher::tick_builder::PriceCache;
use crate::fetcher::websocket::{OrderBookCache, WsCommand};
use crate::proxy::HttpPool;

pub struct SpawnedHandles {
    pub registry: crate::strategy::registry::AssignmentRegistry,
    pub wallet_keys: Arc<crate::execution::wallet::WalletKeyStore>,
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

    // API fetch cache (shared between background poller and strategy evaluation)
    let api_cache = api_fetch_task::ApiFetchCache::new();
    let model_score_cache = model_score_task::ModelScoreCache::new();

    // Background API fetcher
    {
        let registry = engine_registry.clone();
        let cache = api_cache.clone();
        let http = state.http.clone();
        tasks.spawn(async move { api_fetch_task::run(registry, cache, http).await });
    }

    // Background model scorer
    {
        let registry = engine_registry.clone();
        let cache = model_score_cache.clone();
        let http = state.http.clone();
        let tick_rx = state.tick_tx.subscribe();
        tasks.spawn(async move { model_score_task::run(registry, cache, http, tick_rx).await });
    }

    engine_tasks::spawn_strategy_engine(
        state,
        engine_registry.clone(),
        api_cache,
        model_score_cache,
        signal_tx,
        tasks,
    );

    // PostgreSQL connection pool
    let db = crate::storage::postgres::create_pool(&state.config.database_url).await?;

    // Shared execution queue
    let exec_queue = Arc::new(Mutex::new(crate::execution::queue::ExecutionQueue::new(
        state.config.max_orders_per_day,
    )));

    // Wallet key store (shared between execution and API)
    let wallet_keys = Arc::new(
        crate::execution::wallet::WalletKeyStore::new(&state.config.encryption_key).unwrap_or_else(
            |e| {
                tracing::warn!(error = %e, "wallet_key_store_init_failed, using dummy key");
                crate::execution::wallet::WalletKeyStore::new(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .unwrap()
            },
        ),
    );

    rehydrate_running_assignments(
        &state.config.redis_url,
        &db,
        &engine_registry,
        wallet_keys.as_ref(),
    )
    .await;

    // Execution pipeline (replaces signal logger)
    execution_tasks::spawn_execution(
        state,
        engine_registry.clone(),
        signal_rx,
        exec_queue.clone(),
        db.clone(),
        wallet_keys.clone(),
        tasks,
    );

    // Copy trading watcher
    execution_tasks::spawn_watcher(state, exec_queue.clone(), db.clone(), tasks);

    // Slot resolution (backfill winner from Gamma API + resolve trades)
    {
        let ch = crate::storage::clickhouse::create_client(&state.config.clickhouse_url);
        let http = state.http.clone();
        let gamma_url = state.config.gamma_api_url.clone();
        let resolver_db = db.clone();
        let resolver_registry = engine_registry.clone();
        tasks.spawn(async move {
            slot_resolver::run_slot_resolver(ch, http, gamma_url, resolver_db, resolver_registry)
                .await
        });
    }

    // Post-fill execution analytics (60s markout from ClickHouse mid prices)
    {
        let ch = crate::storage::clickhouse::create_client(&state.config.clickhouse_url);
        let analytics_db = db.clone();
        tasks.spawn(async move { trade_analytics::run_trade_analytics(ch, analytics_db).await });
    }

    // Redis state persistence
    persistence::spawn_redis_state_persister(state, engine_registry.clone(), tasks);

    Ok(SpawnedHandles {
        registry: engine_registry,
        wallet_keys,
    })
}

async fn rehydrate_running_assignments(
    redis_url: &str,
    db: &sqlx::PgPool,
    registry: &crate::strategy::registry::AssignmentRegistry,
    wallet_keys: &crate::execution::wallet::WalletKeyStore,
) {
    let assignments = match crate::storage::postgres::load_running_strategy_assignments(db).await {
        Ok(assignments) => assignments,
        Err(e) => {
            tracing::warn!(error = %e, "running_assignments_load_failed");
            return;
        }
    };

    let mut redis_conn = match redis::Client::open(redis_url) {
        Ok(client) => match client.get_multiplexed_tokio_connection().await {
            Ok(conn) => Some(conn),
            Err(e) => {
                tracing::warn!(error = %e, "rehydration_redis_connect_failed");
                None
            }
        },
        Err(e) => {
            tracing::warn!(error = %e, "rehydration_redis_client_failed");
            None
        }
    };

    let mut count = 0usize;

    for assignment in assignments {
        if !assignment.private_key_enc.is_empty() {
            if let Err(e) =
                wallet_keys.store_key(assignment.wallet_id as u64, &assignment.private_key_enc)
            {
                tracing::warn!(
                    wallet_id = assignment.wallet_id,
                    strategy_id = assignment.strategy_id,
                    error = %e,
                    "running_assignment_key_load_failed"
                );
                continue;
            }
        }

        if let Some(safe_address) = assignment.safe_address.as_deref().filter(|s| !s.is_empty()) {
            match safe_address.parse::<Address>() {
                Ok(address) => {
                    if let Err(e) =
                        wallet_keys.store_safe_address(assignment.wallet_id as u64, address)
                    {
                        tracing::warn!(
                            wallet_id = assignment.wallet_id,
                            strategy_id = assignment.strategy_id,
                            error = %e,
                            "running_assignment_safe_store_failed"
                        );
                        continue;
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        wallet_id = assignment.wallet_id,
                        strategy_id = assignment.strategy_id,
                        error = %e,
                        "running_assignment_safe_parse_failed"
                    );
                    continue;
                }
            }
        }

        let initial_state = match redis_conn.as_mut() {
            Some(conn) => match crate::storage::redis::load_state(
                conn,
                assignment.wallet_id as u64,
                assignment.strategy_id as u64,
            )
            .await
            {
                Ok(state) => state,
                Err(e) => {
                    tracing::warn!(
                        wallet_id = assignment.wallet_id,
                        strategy_id = assignment.strategy_id,
                        error = %e,
                        "running_assignment_state_load_failed"
                    );
                    None
                }
            },
            None => None,
        };

        crate::strategy::registry::activate(
            registry,
            assignment.wallet_id as u64,
            assignment.strategy_id as u64,
            assignment.graph,
            assignment.markets,
            assignment.max_position_usdc,
            assignment.is_paper,
            initial_state,
        )
        .await;
        count += 1;
    }

    tracing::info!(count, "running_assignments_rehydrated");
}
