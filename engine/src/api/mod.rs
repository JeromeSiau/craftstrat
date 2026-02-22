pub mod error;
pub mod handlers;
pub mod state;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use state::ApiState;

pub fn router(state: Arc<ApiState>) -> Router {
    Router::new()
        .route(
            "/internal/strategy/activate",
            post(handlers::strategy::activate),
        )
        .route(
            "/internal/strategy/deactivate",
            post(handlers::strategy::deactivate),
        )
        .route("/internal/wallet/{id}/state", get(handlers::wallet::state))
        .route("/internal/backtest/run", post(handlers::backtest::run))
        .route("/internal/engine/status", get(handlers::status::status))
        .route("/internal/copy/watch", post(handlers::copy::watch))
        .route("/internal/copy/unwatch", post(handlers::copy::unwatch))
        .route("/metrics", get(handlers::metrics::render))
        .route("/internal/stats/slots", get(handlers::stats::slots))
        .with_state(state)
}

pub async fn serve(state: Arc<ApiState>, port: u16) -> anyhow::Result<()> {
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!(port, "internal_api_listening");
    axum::serve(listener, app).await?;
    Ok(())
}
