use tokio::sync::mpsc;
use tokio::task::JoinSet;

use super::SharedState;
use crate::strategy::registry::AssignmentRegistry;
use crate::strategy::EngineOutput;

pub fn spawn_strategy_engine(
    state: &SharedState,
    engine_registry: AssignmentRegistry,
    signal_tx: mpsc::Sender<EngineOutput>,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    let eng_brokers = state.config.kafka_brokers.clone();
    tasks.spawn(async move {
        crate::strategy::engine::run(&eng_brokers, engine_registry, signal_tx).await
    });
}

