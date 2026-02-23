# Engine Robustness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the Rust trading engine resilient to transient infrastructure failures (ClickHouse timeouts, Redis disconnects) without losing trading capability.

**Architecture:** Introduce a 3-tier task supervision model (Critical / Important / Best-effort), add service health checks at boot, and improve observability on WebSocket reconnections. Critical tasks (strategy engine, executor) still kill the engine on failure. Important tasks (CH writer, copy watcher) auto-restart with exponential backoff. Best-effort tasks (slot resolver) handle errors internally.

**Tech Stack:** Rust, Tokio, clickhouse-rs, redis-rs, sqlx, metrics crate

---

### Task 1: Create `supervisor.rs` — task supervision with exponential backoff

**Files:**
- Create: `engine/src/supervisor.rs`

**Step 1: Create the supervisor module**

Create `engine/src/supervisor.rs`:

```rust
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
```

**Step 2: Verify the tests compile and pass**

Run: `cd engine && cargo test supervisor --lib`
Expected: 3 tests pass.

**Step 3: Commit**

```bash
git add engine/src/supervisor.rs
git commit -m "feat(engine): add task supervisor with exponential backoff"
```

---

### Task 2: Create `healthcheck.rs` — service readiness checks at boot

**Files:**
- Create: `engine/src/healthcheck.rs`

**Step 1: Create the healthcheck module**

Create `engine/src/healthcheck.rs`:

```rust
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
```

**Step 2: Commit**

No unit tests for healthcheck — it's integration-tested by running the engine against real services.

```bash
git add engine/src/healthcheck.rs
git commit -m "feat(engine): add service health checks at boot"
```

---

### Task 3: Add ClickHouse client timeouts

**Files:**
- Modify: `engine/src/storage/clickhouse.rs:8-28`

**Step 1: Add timeouts to `create_client`**

Replace the `create_client` function body. After the existing URL parsing and `client` construction (the `if let Some(at_pos)` / `else` block), add timeout options before returning:

```rust
pub fn create_client(url: &str) -> Client {
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
```

**Step 2: Commit**

```bash
git add engine/src/storage/clickhouse.rs
git commit -m "feat(engine): add ClickHouse client-side timeouts"
```

---

### Task 4: Make slot resolver best-effort

**Files:**
- Modify: `engine/src/tasks/slot_resolver.rs:27-94`

**Step 1: Replace error propagation with logged continues**

Replace the entire `run_slot_resolver` function:

```rust
pub async fn run_slot_resolver(
    ch: Client,
    http: HttpPool,
    gamma_url: String,
) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        let unresolved: Vec<UnresolvedSlot> = match ch
            .query(
                "SELECT DISTINCT symbol FROM slot_snapshots \
                 WHERE winner IS NULL AND pct_into_slot >= 1.0 \
                 ORDER BY symbol",
            )
            .fetch_all()
            .await
        {
            Ok(rows) => rows,
            Err(e) => {
                tracing::warn!(error = %e, "slot_resolver_query_failed");
                continue;
            }
        };

        if unresolved.is_empty() {
            continue;
        }

        tracing::info!(count = unresolved.len(), "slot_resolver_checking");

        for slot in &unresolved {
            let url = format!("{gamma_url}/events?slug={}", slot.symbol);
            let resp = match http.proxied().get(&url).send().await {
                Ok(r) if r.status().is_success() => r,
                Ok(r) => {
                    tracing::warn!(slug = %slot.symbol, status = %r.status(), "slot_resolver_http_error");
                    continue;
                }
                Err(e) => {
                    tracing::warn!(slug = %slot.symbol, error = %e, "slot_resolver_request_failed");
                    continue;
                }
            };

            let events: Vec<GammaEvent> = match resp.json().await {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!(slug = %slot.symbol, error = %e, "slot_resolver_parse_failed");
                    continue;
                }
            };

            let Some(winner) = extract_winner(&events) else {
                continue;
            };

            if let Err(e) = ch
                .query("ALTER TABLE slot_snapshots UPDATE winner = ? WHERE symbol = ?")
                .bind(winner)
                .bind(slot.symbol.as_str())
                .execute()
                .await
            {
                tracing::warn!(slug = %slot.symbol, error = %e, "slot_resolver_update_failed");
                continue;
            }

            tracing::info!(
                slug = %slot.symbol,
                winner = winner,
                label = if winner == 1 { "UP" } else { "DOWN" },
                "slot_resolved",
            );
        }
    }
}
```

Note: the function signature still returns `Result<()>` for compatibility with the JoinSet, but it will never actually return `Err` now. It only exits if the interval stream ends (never).

**Step 2: Commit**

```bash
git add engine/src/tasks/slot_resolver.rs
git commit -m "fix(engine): make slot resolver error-tolerant"
```

---

### Task 5: Add WebSocket reconnection observability

**Files:**
- Modify: `engine/src/metrics.rs` — add 2 metric constants
- Modify: `engine/src/fetcher/websocket.rs:32-56` — add counters and structured logs

**Step 1: Add metric constants to `metrics.rs`**

Add after the existing constants (after line 16):

```rust
pub const WS_RECONNECTIONS_TOTAL: &str = "craftstrat_ws_reconnections_total";
pub const WS_ERRORS_TOTAL: &str = "craftstrat_ws_errors_total";
```

Add to `describe_metrics()` (after line 54):

```rust
metrics::describe_counter!(WS_RECONNECTIONS_TOTAL, "Total WebSocket reconnection attempts");
metrics::describe_counter!(WS_ERRORS_TOTAL, "Total WebSocket errors by type");
```

**Step 2: Update `run_ws_feed` in `websocket.rs`**

Add import at the top of `websocket.rs` (after the existing imports):

```rust
use metrics::counter;
```

Replace the `run_ws_feed` function (lines 32-56):

```rust
pub async fn run_ws_feed(
    ws_url: String,
    books: OrderBookCache,
    mut cmd_rx: tokio::sync::mpsc::Receiver<WsCommand>,
) {
    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);
    let mut subscribed: HashSet<String> = HashSet::new();
    let mut consecutive_reconnects: u32 = 0;

    loop {
        tracing::info!("clob_ws_connecting");
        let connected_at = Instant::now();

        match connect_and_stream(&ws_url, &books, &mut cmd_rx, &mut subscribed).await {
            Ok(_) => {
                tracing::warn!("clob_ws_disconnected");
                counter!(crate::metrics::WS_RECONNECTIONS_TOTAL, "reason" => "disconnected")
                    .increment(1);
            }
            Err(e) => {
                let error_type = classify_ws_error(&e);
                tracing::warn!(error = %e, error_type, "clob_ws_error");
                counter!(crate::metrics::WS_ERRORS_TOTAL, "error_type" => error_type)
                    .increment(1);
                counter!(crate::metrics::WS_RECONNECTIONS_TOTAL, "reason" => "error")
                    .increment(1);
            }
        }

        books.write().await.clear();

        if connected_at.elapsed() > Duration::from_secs(60) {
            backoff = Duration::from_secs(1);
            consecutive_reconnects = 0;
        } else {
            consecutive_reconnects += 1;
        }

        tracing::warn!(
            consecutive = consecutive_reconnects,
            backoff_ms = backoff.as_millis() as u64,
            "clob_ws_reconnecting"
        );

        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(max_backoff);
    }
}

fn classify_ws_error(error: &anyhow::Error) -> &'static str {
    let msg = error.to_string();
    if msg.contains("Connection reset") {
        "connection_reset"
    } else if msg.contains("imeout") {
        "timeout"
    } else {
        "other"
    }
}
```

**Step 3: Commit**

```bash
git add engine/src/metrics.rs engine/src/fetcher/websocket.rs
git commit -m "feat(engine): add WebSocket reconnection metrics and logs"
```

---

### Task 6: Wire everything — health checks, supervision, module declarations

**Files:**
- Modify: `engine/src/main.rs` — declare modules, add healthcheck call
- Modify: `engine/src/tasks/writers.rs` — wrap CH writer with supervisor
- Modify: `engine/src/tasks/execution_tasks.rs` — wrap copy watcher with supervisor

**Step 1: Register new modules in `main.rs`**

Add after line 11 (`mod api;`), before `mod metrics;`:

```rust
mod healthcheck;
mod supervisor;
```

**Step 2: Add healthcheck call in `main.rs`**

Insert after `tracing::info!(sources = cfg.sources.len(), "craftstrat_engine_starting");` (line 33) and before the channel creation (line 35):

```rust
    healthcheck::wait_for_services(
        &cfg.clickhouse_url,
        &cfg.redis_url,
        &cfg.database_url,
    )
    .await?;
```

**Step 3: Wrap CH writer with supervisor in `writers.rs`**

Replace the entire `spawn_clickhouse_writer` function:

```rust
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
```

**Step 4: Wrap copy watcher with supervisor in `execution_tasks.rs`**

Replace the entire `spawn_watcher` function:

```rust
pub fn spawn_watcher(
    state: &SharedState,
    queue: Arc<Mutex<ExecutionQueue>>,
    db: PgPool,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    let data_api_url = state.config.data_api_url.clone();
    let http = state.http.clone();
    let redis_url = state.config.redis_url.clone();

    tasks.spawn(crate::supervisor::supervised("copy_watcher", move || {
        let url = data_api_url.clone();
        let h = http.clone();
        let q = queue.clone();
        let d = db.clone();
        let r = redis_url.clone();
        async move {
            let client = redis::Client::open(r.as_str())?;
            let conn = client.get_multiplexed_tokio_connection().await?;
            crate::watcher::polymarket::run(&url, h, q, d, conn).await
        }
    }));
}
```

**Step 5: Commit**

```bash
git add engine/src/main.rs engine/src/tasks/writers.rs engine/src/tasks/execution_tasks.rs
git commit -m "feat(engine): wire health checks and task supervision"
```

---

### Task 7: Build and run tests

**Step 1: Build**

Run: `cd engine && cargo build 2>&1`
Expected: Clean compilation with no errors.

**Step 2: Run all tests**

Run: `cd engine && cargo test 2>&1`
Expected: All existing tests pass + 3 new supervisor tests pass.

**Step 3: Fix any compilation errors**

If there are compile errors, fix them. Common issues:
- Missing `use` imports (e.g., `sqlx::Connection` in healthcheck)
- Send/Sync bounds on closures in supervised wrappers
- Lifetime issues with captured references in closures

**Step 4: Final commit if fixes were needed**

```bash
git add -A engine/
git commit -m "fix(engine): resolve compilation issues from robustness changes"
```

---

## Summary of changes

| File | Change |
|------|--------|
| `engine/src/supervisor.rs` | **New** — `supervised()` function + 3 tests |
| `engine/src/healthcheck.rs` | **New** — `wait_for_services()` with CH/Redis/PG checks |
| `engine/src/main.rs` | Module declarations + healthcheck at boot |
| `engine/src/storage/clickhouse.rs` | Client timeouts (connect 5s, receive 15s, send 10s) |
| `engine/src/tasks/slot_resolver.rs` | `?` → `match`/`continue` (never crashes) |
| `engine/src/tasks/writers.rs` | CH writer wrapped with `supervised()` |
| `engine/src/tasks/execution_tasks.rs` | Copy watcher wrapped with `supervised()` |
| `engine/src/fetcher/websocket.rs` | Reconnection counter, `classify_ws_error`, metrics |
| `engine/src/metrics.rs` | 2 new constants + descriptions |

## Task supervision model after changes

| Task | Level | On failure |
|------|-------|------------|
| Strategy engine | Critical | Engine stops |
| Signal-to-queue bridge | Critical | Engine stops |
| Executor | Critical | Engine stops |
| API server | Critical | Engine stops |
| **ClickHouse writer** | **Important** | **Auto-restart with backoff** |
| **Copy watcher** | **Important** | **Auto-restart with backoff** |
| Slot resolver | Best-effort | Logs error, continues |
| Redis state persister | Best-effort | Logs error, continues |
| WS feed | Best-effort | Internal reconnect loop |
| Price poller | Best-effort | Logs error, continues |
| Market discovery | Best-effort | Logs error, continues |
| Kafka publisher | Best-effort | Logs error, continues |
| Tick builder | Critical | Engine stops |
