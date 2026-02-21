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
