use std::sync::atomic::Ordering;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use super::state::ApiState;
use crate::backtest::{BacktestRequest, BacktestResult};

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

// ---------- Strategy Activate / Deactivate ----------

#[derive(Deserialize)]
struct ActivateRequest {
    wallet_id: u64,
    strategy_id: u64,
    graph: serde_json::Value,
    markets: Vec<String>,
    #[serde(default = "default_max_position")]
    max_position_usdc: f64,
}

fn default_max_position() -> f64 {
    1000.0
}

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
    )
    .await;
    StatusCode::OK
}

#[derive(Deserialize)]
struct DeactivateRequest {
    wallet_id: u64,
    strategy_id: u64,
}

async fn strategy_deactivate(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<DeactivateRequest>,
) -> StatusCode {
    crate::strategy::registry::deactivate(&state.registry, req.wallet_id, req.strategy_id).await;
    StatusCode::OK
}

// ---------- Backtest ----------

async fn backtest_run(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<BacktestRequest>,
) -> Result<Json<BacktestResult>, (StatusCode, String)> {
    crate::backtest::runner::run(&req, &state.ch)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))
}

// ---------- Wallet State ----------

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
) -> Json<WalletStateResponse> {
    let reg = state.registry.read().await;
    let mut assignments = Vec::new();

    for (_, market_assignments) in reg.iter() {
        for a in market_assignments {
            if a.wallet_id == wallet_id {
                // Avoid duplicates (same assignment across multiple markets)
                if assignments
                    .iter()
                    .any(|existing: &AssignmentState| existing.strategy_id == a.strategy_id)
                {
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

    Json(WalletStateResponse {
        wallet_id,
        assignments,
    })
}

// ---------- Engine Status ----------

#[derive(Serialize)]
struct EngineStatusResponse {
    active_wallets: usize,
    active_assignments: usize,
    ticks_processed: u64,
    uptime_secs: u64,
}

async fn engine_status(State(state): State<Arc<ApiState>>) -> Json<EngineStatusResponse> {
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

// ---------- Copy Trading ----------

#[derive(Deserialize)]
struct CopyWatchRequest {
    leader_address: String,
}

async fn copy_watch(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<CopyWatchRequest>,
) -> StatusCode {
    let key = format!("oddex:watcher:watched:{}", req.leader_address);
    let result: Result<(), redis::RedisError> = redis::cmd("SET")
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

async fn copy_unwatch(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<CopyWatchRequest>,
) -> StatusCode {
    let key = format!("oddex:watcher:watched:{}", req.leader_address);
    let result: Result<(), redis::RedisError> = redis::cmd("DEL")
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
