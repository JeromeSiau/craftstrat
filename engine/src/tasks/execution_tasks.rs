use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinSet;

use super::SharedState;
use crate::execution::executor;
use crate::execution::fees::FeeCache;
use crate::execution::orders::{BuilderCredentials, OrderSubmitter};
use crate::execution::queue::ExecutionQueue;
use crate::execution::wallet::WalletKeyStore;
use crate::execution::{ExecutionOrder, OrderPriority, Side};
use crate::strategy::registry::AssignmentRegistry;
use crate::strategy::{EngineOutput, OrderType, Outcome, Signal};

// ---------------------------------------------------------------------------
// spawn_execution — executor loop + signal-to-queue bridge
// ---------------------------------------------------------------------------

pub fn spawn_execution(
    state: &SharedState,
    registry: AssignmentRegistry,
    signal_rx: mpsc::Receiver<EngineOutput>,
    queue: Arc<Mutex<ExecutionQueue>>,
    db: PgPool,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    let cfg = &state.config;

    // Wallet key store
    let wallet_keys = Arc::new(
        WalletKeyStore::new(&cfg.encryption_key).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "wallet_key_store_init_failed, using dummy key");
            WalletKeyStore::new(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap()
        }),
    );

    // Fee cache
    let fee_cache = Arc::new(FeeCache::new(state.http.clone(), &cfg.clob_api_url));

    // Order submitter
    let credentials = BuilderCredentials {
        api_key: cfg.builder_api_key.clone(),
        secret: cfg.builder_secret.clone(),
        passphrase: cfg.builder_passphrase.clone(),
    };
    let submitter = Arc::new(OrderSubmitter::new(
        state.http.clone(),
        &cfg.clob_api_url,
        credentials,
        wallet_keys,
        fee_cache,
        cfg.neg_risk,
    ));

    // Signal → queue bridge
    let bridge_queue = queue.clone();
    tasks.spawn(async move { signal_to_queue(signal_rx, bridge_queue).await });

    // Executor loop
    let exec_queue = queue;
    tasks.spawn(async move { executor::run(exec_queue, submitter, registry, db).await });
}

// ---------------------------------------------------------------------------
// spawn_watcher — copy trading watcher
// ---------------------------------------------------------------------------

pub fn spawn_watcher(
    state: &SharedState,
    queue: Arc<Mutex<ExecutionQueue>>,
    db: PgPool,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    let data_api_url = state.config.data_api_url.clone();
    let http = state.http.clone();
    let redis_url = state.config.redis_url.clone();

    tasks.spawn(crate::supervisor::supervised("copy_watcher", move || {
        let url = data_api_url.clone();
        let h = http.clone();
        let q = queue.clone();
        let d = db.clone();
        let r = redis_url.clone();
        async move {
            let client = redis::Client::open(r.as_str())?;
            let conn = client.get_multiplexed_tokio_connection().await?;
            crate::watcher::polymarket::run(&url, h, q, d, conn).await
        }
    }));
}

// ---------------------------------------------------------------------------
// signal_to_queue — bridge strategy engine signals to execution queue
// ---------------------------------------------------------------------------

async fn signal_to_queue(
    mut signal_rx: mpsc::Receiver<EngineOutput>,
    queue: Arc<Mutex<ExecutionQueue>>,
) -> anyhow::Result<()> {
    tracing::info!("signal_to_queue_bridge_started");

    while let Some(output) = signal_rx.recv().await {
        let order = match &output.signal {
            Signal::Buy {
                outcome,
                size_usdc,
                order_type,
            } => build_order_from_signal(
                output.wallet_id,
                output.strategy_id,
                &output.symbol,
                Side::Buy,
                *outcome,
                *size_usdc,
                order_type,
                output.is_paper,
            ),
            Signal::Sell {
                outcome,
                size_usdc,
                order_type,
            } => build_order_from_signal(
                output.wallet_id,
                output.strategy_id,
                &output.symbol,
                Side::Sell,
                *outcome,
                *size_usdc,
                order_type,
                output.is_paper,
            ),
            Signal::Cancel { outcome } => {
                tracing::info!(
                    wallet_id = output.wallet_id,
                    strategy_id = output.strategy_id,
                    symbol = %output.symbol,
                    outcome = ?outcome,
                    "cancel_signal_received"
                );
                continue;
            }
            Signal::Notify { channel, message } => {
                tracing::info!(
                    wallet_id = output.wallet_id,
                    strategy_id = output.strategy_id,
                    channel = %channel,
                    message = %message,
                    "notify_signal_received"
                );
                continue;
            }
            Signal::Hold => continue,
        };

        tracing::info!(
            wallet_id = order.wallet_id,
            strategy_id = output.strategy_id,
            symbol = %order.symbol,
            side = ?order.side,
            size = order.size_usdc,
            "signal_queued_for_execution"
        );

        let mut q = queue.lock().await;
        q.push(order);
    }

    Ok(())
}

fn build_order_from_signal(
    wallet_id: u64,
    strategy_id: u64,
    symbol: &str,
    side: Side,
    outcome: Outcome,
    size_usdc: f64,
    order_type: &OrderType,
    is_paper: bool,
) -> ExecutionOrder {
    let (priority, price) = match order_type {
        OrderType::Market => (OrderPriority::StrategyMarket, None),
        OrderType::Limit { price } => (OrderPriority::Limit, Some(*price)),
        OrderType::StopLoss { trigger_price } => (OrderPriority::StopLoss, Some(*trigger_price)),
        OrderType::TakeProfit { trigger_price } => {
            (OrderPriority::TakeProfit, Some(*trigger_price))
        }
    };

    ExecutionOrder {
        id: uuid::Uuid::new_v4(),
        wallet_id,
        strategy_id: Some(strategy_id),
        copy_relationship_id: None,
        symbol: symbol.to_string(),
        token_id: String::new(), // resolved by the executor from symbol/outcome
        side,
        outcome,
        price,
        size_usdc,
        order_type: order_type.clone(),
        priority,
        created_at: chrono::Utc::now().timestamp(),
        leader_address: String::new(),
        leader_tx_hash: String::new(),
        is_paper,
    }
}
