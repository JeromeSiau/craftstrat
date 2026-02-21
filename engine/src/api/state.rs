use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use clickhouse::Client as ChClient;
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::execution::queue::ExecutionQueue;
use crate::strategy::registry::AssignmentRegistry;

pub struct ApiState {
    pub registry: AssignmentRegistry,
    pub exec_queue: Arc<Mutex<ExecutionQueue>>,
    pub db: PgPool,
    pub ch: ChClient,
    pub start_time: std::time::Instant,
    pub tick_count: Arc<AtomicU64>,
}
