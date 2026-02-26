use anyhow::Result;
use clickhouse::query::RowCursor;
use clickhouse::Client;
use std::time::Duration;

use crate::fetcher::models::Tick;

pub fn create_client(url: &str) -> Client {
    // The clickhouse crate doesn't extract user:password from the URL,
    // so we parse them out and set them separately.
    let mut client = Client::default();
    if let Some(at_pos) = url.find('@') {
        let scheme_end = url.find("://").map(|p| p + 3).unwrap_or(0);
        let userinfo = &url[scheme_end..at_pos];
        let base_url = format!("{}{}", &url[..scheme_end], &url[at_pos + 1..]);
        client = client.with_url(base_url);
        if let Some(colon) = userinfo.find(':') {
            client = client
                .with_user(&userinfo[..colon])
                .with_password(&userinfo[colon + 1..]);
        } else {
            client = client.with_user(userinfo);
        }
    } else {
        client = client.with_url(url);
    }
    client
        .with_option("connect_timeout", "5")
        .with_option("receive_timeout", "15")
        .with_option("send_timeout", "10")
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
    let sql = if symbols.is_empty() {
        "SELECT ?fields FROM slot_snapshots WHERE captured_at >= ? AND captured_at <= ? ORDER BY captured_at ASC".to_string()
    } else {
        let placeholders: Vec<&str> = symbols.iter().map(|_| "?").collect();
        format!(
            "SELECT ?fields FROM slot_snapshots WHERE symbol IN ({}) AND captured_at >= ? AND captured_at <= ? ORDER BY captured_at ASC",
            placeholders.join(", ")
        )
    };
    let mut query = client.query(&sql);
    for s in symbols {
        query = query.bind(s.as_str());
    }
    query = query.bind(date_from).bind(date_to);
    Ok(query.fetch::<Tick>()?)
}
