use std::sync::Arc;

use axum::extract::State;
use axum::Json;

use crate::api::error::ApiError;
use crate::api::state::ApiState;
use crate::backtest::{BacktestRequest, BacktestResult};

pub async fn run(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<BacktestRequest>,
) -> Result<Json<BacktestResult>, ApiError> {
    crate::backtest::runner::run(&req, &state.ch)
        .await
        .map(Json)
        .map_err(|e| ApiError::Validation(e.to_string()))
}
