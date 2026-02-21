use std::sync::atomic::Ordering;
use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::Serialize;

use crate::api::state::ApiState;

#[derive(Serialize)]
pub struct EngineStatusResponse {
    pub active_wallets: usize,
    pub active_assignments: usize,
    pub ticks_processed: u64,
    pub uptime_secs: u64,
}

pub async fn status(State(state): State<Arc<ApiState>>) -> Json<EngineStatusResponse> {
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
