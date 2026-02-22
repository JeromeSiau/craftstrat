# Phase 6 — Internal API Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an Axum HTTP server to the Rust engine exposing all `/internal/*` endpoints, and create the Laravel `EngineService` that calls them.

**Architecture:** A new `api/` module in the engine with an Axum router. Shared state (registry, execution queue, DB pool, ClickHouse client) passed via Axum's `State` extractor. The server spawns as an additional task alongside existing ones. On the Laravel side, a single `EngineService` class wraps all HTTP calls with typed responses.

**Tech Stack:** Rust (Axum 0.8, tokio, serde), PHP (Laravel 12 HTTP client, service class).

---

## Key Design Decisions

- **Axum `State` extractor** — All handlers receive a shared `ApiState` struct via `State<Arc<ApiState>>`. No global singletons.
- **7 endpoints** — 5 from spec + 2 copy trading watch/unwatch endpoints (cleaner than Redis side-channel).
- **Backtest is async** — `POST /internal/backtest/run` blocks until ClickHouse replay completes. Fine for internal use; Laravel can wrap in a queued job if needed.
- **Wallet state from registry** — `GET /internal/wallet/{id}/state` reads from the in-memory `AssignmentRegistry`, not from the DB. Real-time data.
- **Engine status from live counters** — `GET /internal/engine/status` reads from an `AtomicU64`-based metrics struct.
- **No auth on internal API** — Network-level isolation via Docker (not exposed publicly). Matches spec constraint.

## Endpoints Summary

```
POST   /internal/strategy/activate       { wallet_id, strategy_id, graph, markets, max_position_usdc }
POST   /internal/strategy/deactivate     { wallet_id, strategy_id }
GET    /internal/wallet/{id}/state        → { position, pnl, last_signal, last_tick_at }
POST   /internal/backtest/run             { strategy_graph, market_filter, date_from, date_to }
GET    /internal/engine/status            → { active_wallets, ticks_per_sec, uptime_secs }
POST   /internal/copy/watch              { leader_address }
POST   /internal/copy/unwatch            { leader_address }
```

## Important Context

**SharedState** (see `engine/src/tasks/mod.rs:18-26`):
Current `SharedState` holds config, books, markets, prices, tick_tx, ws_cmd_tx, http. The API server needs access to additional state created in `spawn_all()`: `AssignmentRegistry`, `ExecutionQueue`, `PgPool`, and a new ClickHouse client.

**AssignmentRegistry** (see `engine/src/strategy/registry.rs`):
Already has `activate()` and `deactivate()` functions. The API handlers call these directly.

**BacktestRequest** (see `engine/src/backtest/mod.rs:12-19`):
Already has `Deserialize` — can be used as Axum JSON body directly.

---

## Task 1: API Types & Router Skeleton

**Files:**
- Create: `engine/src/api/mod.rs`
- Create: `engine/src/api/routes.rs`
- Create: `engine/src/api/state.rs`
- Modify: `engine/src/main.rs:1-9` (add `mod api;`)

### Step 1: Create `engine/src/api/state.rs` — shared API state

```rust
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use clickhouse::Client as ChClient;
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::execution::queue::ExecutionQueue;
use crate::strategy::registry::AssignmentRegistry;

pub struct ApiState {
    pub registry: AssignmentRegistry,
    pub exec_queue: Arc<Mutex<ExecutionQueue>>,
    pub db: PgPool,
    pub ch: ChClient,
    pub start_time: std::time::Instant,
    pub tick_count: Arc<AtomicU64>,
}
```

### Step 2: Create `engine/src/api/routes.rs` — empty router

```rust
use std::sync::Arc;

use axum::{Router, routing::{get, post}};

use super::state::ApiState;

pub fn router(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/internal/strategy/activate", post(strategy_activate))
        .route("/internal/strategy/deactivate", post(strategy_deactivate))
        .route("/internal/wallet/{id}/state", get(wallet_state))
        .route("/internal/backtest/run", post(backtest_run))
        .route("/internal/engine/status", get(engine_status))
        .route("/internal/copy/watch", post(copy_watch))
        .route("/internal/copy/unwatch", post(copy_unwatch))
        .with_state(state)
}

// Handlers — implemented in subsequent tasks
async fn strategy_activate() -> &'static str { "TODO" }
async fn strategy_deactivate() -> &'static str { "TODO" }
async fn wallet_state() -> &'static str { "TODO" }
async fn backtest_run() -> &'static str { "TODO" }
async fn engine_status() -> &'static str { "TODO" }
async fn copy_watch() -> &'static str { "TODO" }
async fn copy_unwatch() -> &'static str { "TODO" }
```

### Step 3: Create `engine/src/api/mod.rs` — module root + server launcher

```rust
pub mod routes;
pub mod state;

use std::sync::Arc;

use state::ApiState;

pub async fn serve(state: Arc<ApiState>, port: u16) -> anyhow::Result<()> {
    let app = routes::router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!(port, "internal_api_listening");
    axum::serve(listener, app).await?;
    Ok(())
}
```

### Step 4: Add `mod api;` to `engine/src/main.rs`

Add `mod api;` after line 9 (`mod backtest;`).

### Step 5: Verify it compiles

Run: `cd engine && cargo check 2>&1 | head -20`
Expected: Compiles with warnings (unused imports), no errors.

### Step 6: Commit

```bash
git add engine/src/api/ engine/src/main.rs
git commit -m "feat(api): add Axum router skeleton with 7 endpoint stubs"
```

---

## Task 2: Wire API Server into Main Loop

**Files:**
- Modify: `engine/src/tasks/mod.rs` (return shared handles)
- Modify: `engine/src/main.rs` (spawn API server)
- Modify: `engine/src/config.rs` (add `api_port`)

### Step 1: Add `api_port` to `Config`

In `engine/src/config.rs`, add field to `Config`:

```rust
pub api_port: u16,
```

In `Config::from_env()`, add:

```rust
api_port: std::env::var("INTERNAL_API_PORT")
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(8080),
```

### Step 2: Modify `spawn_all()` to return shared handles

Change `spawn_all()` signature to return the objects the API needs:

```rust
pub struct SpawnedHandles {
    pub registry: AssignmentRegistry,
    pub exec_queue: Arc<Mutex<ExecutionQueue>>,
    pub db: PgPool,
}
```

Return `Ok(SpawnedHandles { registry: engine_registry, exec_queue, db })` at the end of `spawn_all()`.

### Step 3: Wire API server in `main.rs`

After `spawn_all()`, create `ApiState` and spawn the server:

```rust
let ch_client = clickhouse::Client::default().with_url(&state.config.clickhouse_url);
let api_state = Arc::new(api::state::ApiState {
    registry: handles.registry,
    exec_queue: handles.exec_queue,
    db: handles.db,
    ch: ch_client,
    start_time: std::time::Instant::now(),
    tick_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
});

let api_port = state.config.api_port;
tasks.spawn(async move {
    api::serve(api_state, api_port).await
});
```

### Step 4: Verify it compiles

Run: `cd engine && cargo check 2>&1 | head -20`
Expected: Compiles. The API server is now spawned in the JoinSet.

### Step 5: Commit

```bash
git add engine/src/
git commit -m "feat(api): wire Axum server into main loop with shared state"
```

---

## Task 3: Strategy Activate & Deactivate Handlers

**Files:**
- Modify: `engine/src/api/routes.rs` (implement 2 handlers)

### Step 1: Implement `strategy_activate` handler

```rust
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
struct ActivateRequest {
    wallet_id: u64,
    strategy_id: u64,
    graph: serde_json::Value,
    markets: Vec<String>,
    #[serde(default = "default_max_position")]
    max_position_usdc: f64,
}

fn default_max_position() -> f64 { 1000.0 }

async fn strategy_activate(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<ActivateRequest>,
) -> StatusCode {
    crate::strategy::registry::activate(
        &state.registry,
        req.wallet_id,
        req.strategy_id,
        req.graph,
        req.markets,
        req.max_position_usdc,
        None,
    ).await;
    StatusCode::OK
}
```

### Step 2: Implement `strategy_deactivate` handler

```rust
#[derive(Deserialize)]
struct DeactivateRequest {
    wallet_id: u64,
    strategy_id: u64,
}

async fn strategy_deactivate(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<DeactivateRequest>,
) -> StatusCode {
    crate::strategy::registry::deactivate(
        &state.registry,
        req.wallet_id,
        req.strategy_id,
    ).await;
    StatusCode::OK
}
```

### Step 3: Verify it compiles

Run: `cd engine && cargo check 2>&1 | head -20`

### Step 4: Commit

```bash
git add engine/src/api/routes.rs
git commit -m "feat(api): implement strategy activate/deactivate endpoints"
```

---

## Task 4: Backtest Run Handler

**Files:**
- Modify: `engine/src/api/routes.rs`

### Step 1: Implement `backtest_run` handler

```rust
use crate::backtest::{self, BacktestRequest, BacktestResult};

async fn backtest_run(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<BacktestRequest>,
) -> Result<Json<BacktestResult>, (StatusCode, String)> {
    backtest::runner::run(&req, &state.ch)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))
}
```

### Step 2: Verify it compiles

Run: `cd engine && cargo check 2>&1 | head -20`

### Step 3: Commit

```bash
git add engine/src/api/routes.rs
git commit -m "feat(api): implement backtest/run endpoint"
```

---

## Task 5: Wallet State Handler

**Files:**
- Modify: `engine/src/api/routes.rs`

### Step 1: Implement `wallet_state` handler

```rust
use axum::extract::Path;
use serde::Serialize;

#[derive(Serialize)]
struct WalletStateResponse {
    wallet_id: u64,
    assignments: Vec<AssignmentState>,
}

#[derive(Serialize)]
struct AssignmentState {
    strategy_id: u64,
    markets: Vec<String>,
    position: Option<PositionSnapshot>,
    pnl: f64,
}

#[derive(Serialize)]
struct PositionSnapshot {
    outcome: String,
    entry_price: f64,
    size_usdc: f64,
    entry_at: i64,
}

async fn wallet_state(
    State(state): State<Arc<ApiState>>,
    Path(wallet_id): Path<u64>,
) -> Result<Json<WalletStateResponse>, StatusCode> {
    let reg = state.registry.read().await;
    let mut assignments = Vec::new();

    for (_, market_assignments) in reg.iter() {
        for a in market_assignments {
            if a.wallet_id == wallet_id {
                // Avoid duplicates (same assignment across multiple markets)
                if assignments.iter().any(|existing: &AssignmentState| existing.strategy_id == a.strategy_id) {
                    continue;
                }
                let state_lock = a.state.lock().unwrap();
                let position = state_lock.position.as_ref().map(|p| PositionSnapshot {
                    outcome: format!("{:?}", p.outcome),
                    entry_price: p.entry_price,
                    size_usdc: p.size_usdc,
                    entry_at: p.entry_at,
                });
                assignments.push(AssignmentState {
                    strategy_id: a.strategy_id,
                    markets: a.markets.clone(),
                    position,
                    pnl: state_lock.pnl,
                });
            }
        }
    }

    Ok(Json(WalletStateResponse { wallet_id, assignments }))
}
```

### Step 2: Verify it compiles

Run: `cd engine && cargo check 2>&1 | head -20`

### Step 3: Commit

```bash
git add engine/src/api/routes.rs
git commit -m "feat(api): implement wallet state endpoint"
```

---

## Task 6: Engine Status Handler

**Files:**
- Modify: `engine/src/api/routes.rs`

### Step 1: Implement `engine_status` handler

```rust
use std::sync::atomic::Ordering;

#[derive(Serialize)]
struct EngineStatusResponse {
    active_wallets: usize,
    active_assignments: usize,
    ticks_processed: u64,
    uptime_secs: u64,
}

async fn engine_status(
    State(state): State<Arc<ApiState>>,
) -> Json<EngineStatusResponse> {
    let reg = state.registry.read().await;
    let mut wallet_ids = std::collections::HashSet::new();
    let mut assignment_count = 0usize;

    for assignments in reg.values() {
        for a in assignments {
            wallet_ids.insert(a.wallet_id);
            assignment_count += 1;
        }
    }

    Json(EngineStatusResponse {
        active_wallets: wallet_ids.len(),
        active_assignments: assignment_count,
        ticks_processed: state.tick_count.load(Ordering::Relaxed),
        uptime_secs: state.start_time.elapsed().as_secs(),
    })
}
```

### Step 2: Verify it compiles

Run: `cd engine && cargo check 2>&1 | head -20`

### Step 3: Commit

```bash
git add engine/src/api/routes.rs
git commit -m "feat(api): implement engine status endpoint"
```

---

## Task 7: Copy Watch/Unwatch Handlers

**Files:**
- Modify: `engine/src/api/routes.rs`
- Modify: `engine/src/api/state.rs` (add Redis connection)

### Step 1: Add Redis connection to `ApiState`

In `engine/src/api/state.rs`, add:

```rust
pub redis: redis::aio::MultiplexedConnection,
```

Update construction in `main.rs` accordingly (create Redis connection before building `ApiState`).

### Step 2: Implement `copy_watch` handler

```rust
#[derive(Deserialize)]
struct CopyWatchRequest {
    leader_address: String,
}

async fn copy_watch(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<CopyWatchRequest>,
) -> StatusCode {
    let key = format!("craftstrat:watcher:watched:{}", req.leader_address);
    let result: Result<(), _> = redis::cmd("SET")
        .arg(&key)
        .arg("1")
        .query_async(&mut state.redis.clone())
        .await;
    match result {
        Ok(()) => StatusCode::OK,
        Err(e) => {
            tracing::error!(error = %e, address = %req.leader_address, "copy_watch_failed");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
```

### Step 3: Implement `copy_unwatch` handler

```rust
async fn copy_unwatch(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<CopyWatchRequest>,
) -> StatusCode {
    let key = format!("craftstrat:watcher:watched:{}", req.leader_address);
    let result: Result<(), _> = redis::cmd("DEL")
        .arg(&key)
        .query_async(&mut state.redis.clone())
        .await;
    match result {
        Ok(()) => StatusCode::OK,
        Err(e) => {
            tracing::error!(error = %e, address = %req.leader_address, "copy_unwatch_failed");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
```

### Step 4: Verify it compiles

Run: `cd engine && cargo check 2>&1 | head -20`

### Step 5: Commit

```bash
git add engine/src/api/
git commit -m "feat(api): implement copy watch/unwatch endpoints"
```

---

## Task 8: Integration Tests for API

**Files:**
- Create: `engine/tests/api_integration.rs`

### Step 1: Write integration test for strategy activate/deactivate

Uses `axum::test` helpers with a test router. No real DB or ClickHouse needed for strategy endpoints.

```rust
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

// Helper to build a test ApiState with in-memory dependencies
fn test_state() -> Arc<ApiState> { /* ... */ }

#[tokio::test]
async fn test_activate_then_deactivate() {
    let state = test_state();
    let app = craftstrat_engine::api::routes::router(state.clone());

    // Activate
    let body = serde_json::json!({
        "wallet_id": 1,
        "strategy_id": 100,
        "graph": {"mode": "form"},
        "markets": ["btc-15m"]
    });
    let req = Request::builder()
        .method("POST")
        .uri("/internal/strategy/activate")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify state
    let reg = state.registry.read().await;
    assert!(reg.contains_key("btc-15m"));
    drop(reg);

    // Deactivate
    let body = serde_json::json!({"wallet_id": 1, "strategy_id": 100});
    let req = Request::builder()
        .method("POST")
        .uri("/internal/strategy/deactivate")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let reg = state.registry.read().await;
    assert!(!reg.contains_key("btc-15m"));
}
```

### Step 2: Write test for wallet state endpoint

```rust
#[tokio::test]
async fn test_wallet_state_empty() {
    let state = test_state();
    let app = craftstrat_engine::api::routes::router(state);

    let req = Request::builder()
        .uri("/internal/wallet/999/state")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    // Body should contain empty assignments
}
```

### Step 3: Write test for engine status endpoint

```rust
#[tokio::test]
async fn test_engine_status() {
    let state = test_state();
    let app = craftstrat_engine::api::routes::router(state);

    let req = Request::builder()
        .uri("/internal/engine/status")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
```

### Step 4: Run tests

Run: `cd engine && cargo test api_integration -- --nocapture 2>&1 | tail -20`
Expected: All pass.

### Step 5: Commit

```bash
git add engine/tests/api_integration.rs
git commit -m "test(api): add integration tests for internal API endpoints"
```

---

## Task 9: Laravel Engine Config

**Files:**
- Modify: `web/config/services.php` (add engine config)
- Modify: `web/.env.example` (if exists, verify ENGINE_INTERNAL_URL)

### Step 1: Add engine config to `web/config/services.php`

```php
'engine' => [
    'url' => env('ENGINE_INTERNAL_URL', 'http://engine:8080'),
    'timeout' => env('ENGINE_TIMEOUT', 30),
],
```

### Step 2: Commit

```bash
git add web/config/services.php
git commit -m "feat(laravel): add engine service config"
```

---

## Task 10: Laravel EngineService

**Files:**
- Create: `web/app/Services/EngineService.php`
- Modify: `web/app/Providers/AppServiceProvider.php` (register singleton)

### Step 1: Create `EngineService`

```php
<?php

namespace App\Services;

use Illuminate\Http\Client\PendingRequest;
use Illuminate\Support\Facades\Http;

class EngineService
{
    public function __construct(
        private readonly string $baseUrl,
        private readonly int $timeout,
    ) {}

    public function activateStrategy(
        int $walletId,
        int $strategyId,
        array $graph,
        array $markets,
        float $maxPositionUsdc = 1000.0,
    ): void {
        $this->client()->post('/internal/strategy/activate', [
            'wallet_id' => $walletId,
            'strategy_id' => $strategyId,
            'graph' => $graph,
            'markets' => $markets,
            'max_position_usdc' => $maxPositionUsdc,
        ])->throw();
    }

    public function deactivateStrategy(int $walletId, int $strategyId): void
    {
        $this->client()->post('/internal/strategy/deactivate', [
            'wallet_id' => $walletId,
            'strategy_id' => $strategyId,
        ])->throw();
    }

    public function walletState(int $walletId): array
    {
        return $this->client()
            ->get("/internal/wallet/{$walletId}/state")
            ->throw()
            ->json();
    }

    public function runBacktest(array $strategyGraph, array $marketFilter, string $dateFrom, string $dateTo): array
    {
        return $this->client()
            ->timeout($this->timeout * 3) // backtests can be slow
            ->post('/internal/backtest/run', [
                'strategy_graph' => $strategyGraph,
                'market_filter' => $marketFilter,
                'date_from' => $dateFrom,
                'date_to' => $dateTo,
            ])
            ->throw()
            ->json();
    }

    public function engineStatus(): array
    {
        return $this->client()
            ->get('/internal/engine/status')
            ->throw()
            ->json();
    }

    public function watchLeader(string $leaderAddress): void
    {
        $this->client()->post('/internal/copy/watch', [
            'leader_address' => $leaderAddress,
        ])->throw();
    }

    public function unwatchLeader(string $leaderAddress): void
    {
        $this->client()->post('/internal/copy/unwatch', [
            'leader_address' => $leaderAddress,
        ])->throw();
    }

    private function client(): PendingRequest
    {
        return Http::baseUrl($this->baseUrl)
            ->timeout($this->timeout)
            ->acceptJson();
    }
}
```

### Step 2: Register singleton in `AppServiceProvider`

In `web/app/Providers/AppServiceProvider.php`, add to `register()`:

```php
$this->app->singleton(\App\Services\EngineService::class, function ($app) {
    return new \App\Services\EngineService(
        baseUrl: config('services.engine.url'),
        timeout: (int) config('services.engine.timeout'),
    );
});
```

### Step 3: Run Pint

Run: `cd web && vendor/bin/pint --dirty --format agent`

### Step 4: Commit

```bash
git add web/app/Services/EngineService.php web/app/Providers/AppServiceProvider.php
git commit -m "feat(laravel): add EngineService with all internal API methods"
```

---

## Task 11: Laravel EngineService Tests

**Files:**
- Create: `web/tests/Unit/Services/EngineServiceTest.php`

### Step 1: Create test

```bash
cd web && php artisan make:test --pest --unit Services/EngineServiceTest
```

### Step 2: Write tests using Http::fake()

```php
<?php

use App\Services\EngineService;
use Illuminate\Support\Facades\Http;

beforeEach(function () {
    $this->service = new EngineService(
        baseUrl: 'http://engine:8080',
        timeout: 10,
    );
});

it('sends activate strategy request', function () {
    Http::fake(['engine:8080/internal/strategy/activate' => Http::response(null, 200)]);

    $this->service->activateStrategy(1, 100, ['mode' => 'form'], ['btc-15m']);

    Http::assertSent(fn ($request) =>
        $request->url() === 'http://engine:8080/internal/strategy/activate'
        && $request['wallet_id'] === 1
        && $request['strategy_id'] === 100
    );
});

it('sends deactivate strategy request', function () {
    Http::fake(['engine:8080/internal/strategy/deactivate' => Http::response(null, 200)]);

    $this->service->deactivateStrategy(1, 100);

    Http::assertSent(fn ($request) =>
        $request->url() === 'http://engine:8080/internal/strategy/deactivate'
        && $request['wallet_id'] === 1
    );
});

it('fetches wallet state', function () {
    Http::fake(['engine:8080/internal/wallet/42/state' => Http::response([
        'wallet_id' => 42,
        'assignments' => [],
    ])]);

    $result = $this->service->walletState(42);

    expect($result)->toHaveKey('wallet_id', 42);
});

it('runs backtest', function () {
    Http::fake(['engine:8080/internal/backtest/run' => Http::response([
        'total_trades' => 5,
        'win_rate' => 0.6,
        'total_pnl_usdc' => 42.5,
        'max_drawdown' => 0.15,
        'sharpe_ratio' => 1.2,
        'trades' => [],
    ])]);

    $result = $this->service->runBacktest(['mode' => 'form'], ['btc-15m'], '2025-01-01T00:00:00Z', '2025-02-01T00:00:00Z');

    expect($result)
        ->toHaveKey('total_trades', 5)
        ->toHaveKey('win_rate', 0.6);
});

it('fetches engine status', function () {
    Http::fake(['engine:8080/internal/engine/status' => Http::response([
        'active_wallets' => 3,
        'active_assignments' => 7,
        'ticks_processed' => 150000,
        'uptime_secs' => 3600,
    ])]);

    $result = $this->service->engineStatus();

    expect($result)->toHaveKey('active_wallets', 3);
});

it('sends watch leader request', function () {
    Http::fake(['engine:8080/internal/copy/watch' => Http::response(null, 200)]);

    $this->service->watchLeader('0xabc123');

    Http::assertSent(fn ($request) =>
        $request['leader_address'] === '0xabc123'
    );
});

it('sends unwatch leader request', function () {
    Http::fake(['engine:8080/internal/copy/unwatch' => Http::response(null, 200)]);

    $this->service->unwatchLeader('0xabc123');

    Http::assertSent(fn ($request) =>
        $request['leader_address'] === '0xabc123'
    );
});

it('throws on engine error', function () {
    Http::fake(['engine:8080/internal/engine/status' => Http::response(null, 500)]);

    $this->service->engineStatus();
})->throws(\Illuminate\Http\Client\RequestException::class);
```

### Step 3: Run tests

Run: `cd web && php artisan test --compact --filter=EngineServiceTest`
Expected: All pass.

### Step 4: Commit

```bash
git add web/tests/Unit/Services/EngineServiceTest.php
git commit -m "test(laravel): add EngineService unit tests"
```

---

## Task 12: Update SPEC.md

**Files:**
- Modify: `docs/SPEC.md` (add copy endpoints to section 7, update phase 6)

### Step 1: Add copy watch/unwatch endpoints to section 7

After the existing `GET /internal/engine/status` block, add:

```
POST   /internal/copy/watch
       Body: { leader_address }

POST   /internal/copy/unwatch
       Body: { leader_address }
```

### Step 2: Update Phase 6 description

Update lines 967-969 to reflect 7 endpoints.

### Step 3: Commit

```bash
git add docs/SPEC.md
git commit -m "docs: add copy watch/unwatch endpoints to spec"
```
