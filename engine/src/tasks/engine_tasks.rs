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

pub fn spawn_signal_logger(
    mut signal_rx: mpsc::Receiver<EngineOutput>,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    tasks.spawn(async move {
        while let Some(output) = signal_rx.recv().await {
            tracing::info!(
                wallet_id = output.wallet_id,
                strategy_id = output.strategy_id,
                symbol = %output.symbol,
                signal = ?output.signal,
                "engine_signal_output"
            );
        }
        Ok(())
    });
}
