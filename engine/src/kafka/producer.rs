use anyhow::Result;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

use crate::fetcher::models::Tick;

pub fn create_producer(brokers: &str) -> Result<FutureProducer> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()?;
    Ok(producer)
}

pub async fn run_publisher(
    producer: FutureProducer,
    mut tick_rx: tokio::sync::broadcast::Receiver<Tick>,
) {
    loop {
        match tick_rx.recv().await {
            Ok(tick) => {
                let Ok(payload) = serde_json::to_string(&tick) else {
                    continue;
                };
                let key = tick.symbol.clone();
                let record = FutureRecord::to("ticks").key(&key).payload(&payload);

                if let Err((err, _)) = producer.send(record, Duration::from_secs(5)).await {
                    tracing::warn!(error = %err, "kafka_send_failed");
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!(skipped = n, "kafka_lagged");
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                tracing::info!("kafka_publisher_shutdown");
                return;
            }
        }
    }
}
