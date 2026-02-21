use tokio::task::JoinSet;

use super::SharedState;

pub fn spawn_clickhouse_writer(
    state: &SharedState,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    let ch_client = crate::storage::clickhouse::create_client(&state.config.clickhouse_url);
    let ch_rx = state.tick_tx.subscribe();
    tasks.spawn(async move {
        crate::storage::clickhouse::run_writer(ch_client, ch_rx).await
    });
}

pub fn spawn_kafka_publisher(
    state: &SharedState,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) -> anyhow::Result<()> {
    let kf_producer = crate::kafka::producer::create_producer(&state.config.kafka_brokers)?;
    let kf_rx = state.tick_tx.subscribe();
    tasks.spawn(async move {
        crate::kafka::producer::run_publisher(kf_producer, kf_rx).await;
        Ok(())
    });
    Ok(())
}
