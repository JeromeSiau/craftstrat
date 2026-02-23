use tokio::task::JoinSet;

use super::SharedState;

pub fn spawn_clickhouse_writer(
    state: &SharedState,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    let ch_url = state.config.clickhouse_url.clone();
    let tick_tx = state.tick_tx.clone();

    tasks.spawn(crate::supervisor::supervised("ch_writer", move || {
        let client = crate::storage::clickhouse::create_client(&ch_url);
        let rx = tick_tx.subscribe();
        async move { crate::storage::clickhouse::run_writer(client, rx).await }
    }));
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
