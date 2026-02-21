use anyhow::Result;
use clickhouse::Client;
use std::time::Duration;

use crate::fetcher::models::Tick;

pub fn create_client(url: &str) -> Client {
    Client::default().with_url(url)
}

pub async fn run_writer(client: Client, mut tick_rx: tokio::sync::mpsc::Receiver<Tick>) -> Result<()> {
    let mut inserter = client
        .inserter("slot_snapshots")?
        .with_max_rows(100)
        .with_period(Some(Duration::from_secs(10)));

    loop {
        match tick_rx.recv().await {
            Some(tick) => {
                inserter.write(&tick)?;
                let stats = inserter.commit().await?;
                if stats.rows > 0 {
                    tracing::info!(rows = stats.rows, "clickhouse_flushed");
                }
            }
            None => {
                let stats = inserter.end().await?;
                if stats.rows > 0 {
                    tracing::info!(rows = stats.rows, "clickhouse_final_flush");
                }
                return Ok(());
            }
        }
    }
}
