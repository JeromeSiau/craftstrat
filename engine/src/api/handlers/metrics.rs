use std::sync::Arc;

use axum::extract::State;

use crate::api::state::ApiState;

pub async fn render(State(state): State<Arc<ApiState>>) -> String {
    state.prometheus.render()
}
