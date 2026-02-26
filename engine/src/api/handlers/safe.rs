use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::api::error::ApiError;
use crate::api::state::ApiState;

#[derive(Deserialize)]
pub struct DeploySafeRequest {
    pub wallet_id: u64,
    pub signer_address: String,
    pub private_key_enc: String,
}

#[derive(Serialize)]
pub struct DeploySafeResponse {
    pub safe_address: String,
    pub transaction_hash: String,
}

pub async fn deploy_safe(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<DeploySafeRequest>,
) -> Result<Json<DeploySafeResponse>, ApiError> {
    if req.private_key_enc.is_empty() {
        return Err(ApiError::Validation("private_key_enc is required".into()));
    }
    if req.signer_address.is_empty() {
        return Err(ApiError::Validation("signer_address is required".into()));
    }

    // Store the encrypted signer key so the relayer can decrypt and sign
    state
        .wallet_keys
        .store_key(req.wallet_id, &req.private_key_enc)
        .map_err(|e| ApiError::Internal(format!("failed to store wallet key: {e}")))?;

    // Deploy Safe via Builder Relayer (includes USDC approvals)
    let result = state
        .relayer
        .deploy_safe(req.wallet_id)
        .await
        .map_err(|e| ApiError::Internal(format!("safe deployment failed: {e}")))?;

    Ok(Json(DeploySafeResponse {
        safe_address: result.safe_address,
        transaction_hash: result.transaction_hash,
    }))
}
