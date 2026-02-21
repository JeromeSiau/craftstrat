use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;

use crate::api::state::ApiState;

#[derive(Serialize)]
pub struct WalletStateResponse {
    pub wallet_id: u64,
    pub assignments: Vec<AssignmentState>,
}

#[derive(Serialize)]
pub struct AssignmentState {
    pub strategy_id: u64,
    pub markets: Vec<String>,
    pub position: Option<PositionSnapshot>,
    pub pnl: f64,
}

#[derive(Serialize)]
pub struct PositionSnapshot {
    pub outcome: String,
    pub entry_price: f64,
    pub size_usdc: f64,
    pub entry_at: i64,
}

pub async fn state(
    State(app): State<Arc<ApiState>>,
    Path(wallet_id): Path<u64>,
) -> Json<WalletStateResponse> {
    let reg = app.registry.read().await;
    let mut assignments = Vec::new();

    for (_, market_assignments) in reg.iter() {
        for a in market_assignments {
            if a.wallet_id == wallet_id {
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
