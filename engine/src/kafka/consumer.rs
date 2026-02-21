use anyhow::Result;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};

pub fn create_consumer(brokers: &str, group_id: &str) -> Result<StreamConsumer> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("auto.offset.reset", "latest")
        .set("enable.auto.commit", "true")
        .set("enable.auto.offset.store", "true")
        .create()?;
    consumer.subscribe(&["ticks"])?;
    tracing::info!(group_id, "kafka_consumer_created");
    Ok(consumer)
}
