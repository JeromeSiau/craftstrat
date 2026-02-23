use std::future::Future;
use std::time::{Duration, Instant};

const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(60);
const STABLE_THRESHOLD: Duration = Duration::from_secs(60);

/// Runs a task in a retry loop with exponential backoff.
///
/// Used for "Important" tasks that should survive transient failures
/// without killing the engine. The factory closure is called on each
/// restart to create fresh resources (connections, receivers, etc.).
///
/// Returns `Ok(())` only when the factory returns `Ok(())` (clean exit).
/// Never returns `Err` — errors are logged and retried.
pub async fn supervised<F, Fut>(name: &'static str, mut factory: F) -> anyhow::Result<()>
where
    F: FnMut() -> Fut + Send,
    Fut: Future<Output = anyhow::Result<()>> + Send,
{
    let mut backoff = INITIAL_BACKOFF;
    let mut total_restarts: u64 = 0;

    loop {
        let started_at = Instant::now();

        match factory().await {
            Ok(()) => {
                tracing::info!(task = name, "supervised_task_exited_cleanly");
                return Ok(());
            }
            Err(e) => {
                total_restarts += 1;
                let ran_for = started_at.elapsed();

                tracing::error!(
                    task = name,
                    error = %e,
                    ran_for_secs = ran_for.as_secs(),
                    backoff_ms = backoff.as_millis() as u64,
                    total_restarts,
                    "supervised_task_restarting"
                );

                if ran_for > STABLE_THRESHOLD {
                    backoff = INITIAL_BACKOFF;
                }

                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(MAX_BACKOFF);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_returns_ok_on_clean_exit() {
        let result = supervised("test", || async { Ok(()) }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_retries_then_succeeds() {
        tokio::time::pause();

        let attempts = Arc::new(AtomicU32::new(0));
        let c = attempts.clone();

        let result = supervised("test", move || {
            let a = c.clone();
            async move {
                let n = a.fetch_add(1, Ordering::SeqCst);
                if n < 3 {
                    anyhow::bail!("fail #{n}");
                }
                Ok(())
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 4); // 3 failures + 1 success
    }

    #[tokio::test]
    async fn test_resets_backoff_after_stable_run() {
        tokio::time::pause();

        let attempts = Arc::new(AtomicU32::new(0));
        let c = attempts.clone();

        let result = supervised("test", move || {
            let a = c.clone();
            async move {
                let n = a.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    // Simulate stable run then crash
                    tokio::time::sleep(Duration::from_secs(61)).await;
                    anyhow::bail!("crash after stable run");
                }
                Ok(())
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }
}
