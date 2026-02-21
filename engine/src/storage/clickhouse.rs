use anyhow::Result;
use clickhouse::query::RowCursor;
use clickhouse::Client;
use std::time::Duration;

use crate::fetcher::models::Tick;

pub fn create_client(url: &str) -> Client {
    Client::default().with_url(url)
}

pub async fn run_writer(client: Client, mut tick_rx: tokio::sync::broadcast::Receiver<Tick>) -> Result<()> {
    let mut inserter = client
        .inserter("slot_snapshots")?
        .with_max_rows(100)
        .with_period(Some(Duration::from_secs(10)));

    loop {
        match tick_rx.recv().await {
            Ok(tick) => {
                inserter.write(&tick)?;
                let stats = inserter.commit().await?;
                if stats.rows > 0 {
                    tracing::info!(rows = stats.rows, "clickhouse_flushed");
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!(skipped = n, "clickhouse_lagged");
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                let stats = inserter.end().await?;
                if stats.rows > 0 {
                    tracing::info!(rows = stats.rows, "clickhouse_final_flush");
                }
                return Ok(());
            }
        }
    }
}

pub fn fetch_ticks(
    client: &Client,
    symbols: &[String],
    date_from: time::OffsetDateTime,
    date_to: time::OffsetDateTime,
) -> Result<RowCursor<Tick>> {
    let placeholders: Vec<&str> = symbols.iter().map(|_| "?").collect();
    let sql = format!(
        "SELECT ?fields FROM slot_snapshots WHERE symbol IN ({}) AND captured_at >= ? AND captured_at <= ? ORDER BY captured_at ASC",
        placeholders.join(", ")
    );
    let mut query = client.query(&sql);
    for s in symbols {
        query = query.bind(s.as_str());
    }
    query = query.bind(date_from).bind(date_to);
    Ok(query.fetch::<Tick>()?)
}
