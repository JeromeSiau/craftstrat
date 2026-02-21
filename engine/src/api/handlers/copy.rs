use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::api::error::ApiError;
use crate::api::state::ApiState;

#[derive(Deserialize)]
pub struct CopyWatchRequest {
    pub leader_address: String,
}

pub async fn watch(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<CopyWatchRequest>,
) -> Result<StatusCode, ApiError> {
    redis_key_op(&state, &req.leader_address, RedisOp::Set).await
}

pub async fn unwatch(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<CopyWatchRequest>,
) -> Result<StatusCode, ApiError> {
    redis_key_op(&state, &req.leader_address, RedisOp::Del).await
}

enum RedisOp {
    Set,
    Del,
}

async fn redis_key_op(
    state: &ApiState,
    leader_address: &str,
    op: RedisOp,
) -> Result<StatusCode, ApiError> {
    let Some(ref redis) = state.redis else {
        return Err(ApiError::ServiceUnavailable);
    };
    let key = format!("oddex:watcher:watched:{}", leader_address);
    let cmd = match op {
        RedisOp::Set => "SET",
        RedisOp::Del => "DEL",
    };
    let mut args = redis::cmd(cmd);
    args.arg(&key);
    if matches!(op, RedisOp::Set) {
        args.arg("1");
    }
    args.query_async::<()>(&mut redis.clone())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, address = %leader_address, "copy_redis_op_failed");
            ApiError::Internal(e.to_string())
        })?;
    Ok(StatusCode::OK)
}
