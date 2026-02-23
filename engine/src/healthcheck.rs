use std::time::Duration;

use anyhow::{Context, Result};

const CHECK_INTERVAL: Duration = Duration::from_secs(2);
const TIMEOUT: Duration = Duration::from_secs(60);

/// Blocks until ClickHouse, Redis, and PostgreSQL are all reachable.
///
/// Checks run in parallel. Each service is polled every 2 seconds.
/// Fails after 60 seconds if any service is still unreachable.
pub async fn wait_for_services(
    clickhouse_url: &str,
    redis_url: &str,
    database_url: &str,
) -> Result<()> {
    tracing::info!("healthcheck_starting");

    tokio::try_join!(
        wait_for_clickhouse(clickhouse_url),
        wait_for_redis(redis_url),
        wait_for_postgres(database_url),
    )?;

    tracing::info!("healthcheck_passed");
    Ok(())
}

async fn wait_for_clickhouse(url: &str) -> Result<()> {
    let client = crate::storage::clickhouse::create_client(url);
    let deadline = tokio::time::Instant::now() + TIMEOUT;

    loop {
        match client.query("SELECT 1").execute().await {
            Ok(_) => {
                tracing::info!("clickhouse_ready");
                return Ok(());
            }
            Err(e) => {
                if tokio::time::Instant::now() >= deadline {
                    return Err(e).context("ClickHouse not ready within 60s");
                }
                tracing::warn!(error = %e, "waiting_for_clickhouse");
                tokio::time::sleep(CHECK_INTERVAL).await;
            }
        }
    }
}

async fn wait_for_redis(url: &str) -> Result<()> {
    let deadline = tokio::time::Instant::now() + TIMEOUT;

    loop {
        let check = async {
            let client = redis::Client::open(url)?;
            let mut conn = client.get_multiplexed_tokio_connection().await?;
            redis::cmd("PING")
                .query_async::<String>(&mut conn)
                .await?;
            Ok::<(), anyhow::Error>(())
        };

        match check.await {
            Ok(()) => {
                tracing::info!("redis_ready");
                return Ok(());
            }
            Err(e) => {
                if tokio::time::Instant::now() >= deadline {
                    return Err(e).context("Redis not ready within 60s");
                }
                tracing::warn!(error = %e, "waiting_for_redis");
                tokio::time::sleep(CHECK_INTERVAL).await;
            }
        }
    }
}

async fn wait_for_postgres(url: &str) -> Result<()> {
    use sqlx::Connection;

    let deadline = tokio::time::Instant::now() + TIMEOUT;

    loop {
        match sqlx::PgConnection::connect(url).await {
            Ok(_) => {
                tracing::info!("postgres_ready");
                return Ok(());
            }
            Err(e) => {
                if tokio::time::Instant::now() >= deadline {
                    return Err(e).context("PostgreSQL not ready within 60s");
                }
                tracing::warn!(error = %e, "waiting_for_postgres");
                tokio::time::sleep(CHECK_INTERVAL).await;
            }
        }
    }
}
