use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use clickhouse::Client as ChClient;
use metrics_exporter_prometheus::PrometheusHandle;
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::execution::queue::ExecutionQueue;
use crate::execution::relayer::RelayerClient;
use crate::execution::wallet::WalletKeyStore;
use crate::strategy::registry::AssignmentRegistry;

pub struct ApiState {
    pub registry: AssignmentRegistry,
    pub exec_queue: Arc<Mutex<ExecutionQueue>>,
    pub db: PgPool,
    pub ch: ChClient,
    pub redis: Option<redis::aio::MultiplexedConnection>,
    pub start_time: std::time::Instant,
    pub tick_count: Arc<AtomicU64>,
    pub prometheus: PrometheusHandle,
    pub wallet_keys: Arc<WalletKeyStore>,
    pub relayer: Arc<RelayerClient>,
}
