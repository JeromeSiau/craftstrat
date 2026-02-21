pub mod routes;
pub mod state;

use std::sync::Arc;

use state::ApiState;

pub async fn serve(state: Arc<ApiState>, port: u16) -> anyhow::Result<()> {
    let app = routes::router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!(port, "internal_api_listening");
    axum::serve(listener, app).await?;
    Ok(())
}
