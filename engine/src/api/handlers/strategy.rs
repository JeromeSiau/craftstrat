use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::api::error::ApiError;
use crate::api::state::ApiState;

use alloy::primitives::Address;

#[derive(Deserialize)]
pub struct ActivateRequest {
    pub wallet_id: u64,
    pub strategy_id: u64,
    pub graph: serde_json::Value,
    pub markets: Vec<String>,
    #[serde(default = "default_max_position")]
    pub max_position_usdc: f64,
    #[serde(default)]
    pub is_paper: bool,
    /// Encrypted signer private key (base64). Loaded into WalletKeyStore on activation.
    #[serde(default)]
    pub private_key_enc: String,
    /// Gnosis Safe address for this wallet (used as maker in orders).
    #[serde(default)]
    pub safe_address: String,
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

    // Load wallet signer key and Safe address into the shared WalletKeyStore
    if !req.private_key_enc.is_empty() {
        state
            .wallet_keys
            .store_key(req.wallet_id, &req.private_key_enc)
            .map_err(|e| ApiError::Internal(format!("failed to load wallet key: {e}")))?;
    }
    if !req.safe_address.is_empty() {
        let addr: Address = req
            .safe_address
            .parse()
            .map_err(|_| ApiError::Validation("invalid safe_address".into()))?;
        state
            .wallet_keys
            .store_safe_address(req.wallet_id, addr)
            .map_err(|e| ApiError::Internal(format!("failed to store safe address: {e}")))?;
    }

    crate::strategy::registry::activate(
        &state.registry,
        req.wallet_id,
        req.strategy_id,
        req.graph,
        req.markets,
        req.max_position_usdc,
        req.is_paper,
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

#[derive(Deserialize)]
pub struct KillRequest {
    pub wallet_id: u64,
    pub strategy_id: u64,
}

pub async fn kill(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<KillRequest>,
) -> Result<StatusCode, ApiError> {
    let found =
        crate::strategy::registry::kill(&state.registry, req.wallet_id, req.strategy_id).await;
    if found {
        Ok(StatusCode::OK)
    } else {
        Err(ApiError::NotFound("assignment not found".into()))
    }
}

pub async fn unkill(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<KillRequest>,
) -> Result<StatusCode, ApiError> {
    let found =
        crate::strategy::registry::unkill(&state.registry, req.wallet_id, req.strategy_id).await;
    if found {
        Ok(StatusCode::OK)
    } else {
        Err(ApiError::NotFound("assignment not found".into()))
    }
}
