use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::api::error::ApiError;
use crate::api::state::ApiState;

#[derive(Deserialize)]
pub struct ActivateRequest {
    pub wallet_id: u64,
    pub strategy_id: u64,
    pub graph: serde_json::Value,
    pub markets: Vec<String>,
    #[serde(default = "default_max_position")]
    pub max_position_usdc: f64,
}

fn default_max_position() -> f64 {
    1000.0
}

#[derive(Deserialize)]
pub struct DeactivateRequest {
    pub wallet_id: u64,
    pub strategy_id: u64,
}

pub async fn activate(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<ActivateRequest>,
) -> Result<StatusCode, ApiError> {
    if req.markets.is_empty() {
        return Err(ApiError::Validation("markets must not be empty".into()));
    }
    if req.max_position_usdc <= 0.0 {
        return Err(ApiError::Validation(
            "max_position_usdc must be positive".into(),
        ));
    }

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
    Ok(StatusCode::OK)
}

pub async fn deactivate(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<DeactivateRequest>,
) -> StatusCode {
    crate::strategy::registry::deactivate(&state.registry, req.wallet_id, req.strategy_id).await;
    StatusCode::OK
}
