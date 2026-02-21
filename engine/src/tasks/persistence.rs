use tokio::task::JoinSet;

use super::SharedState;
use crate::strategy::registry::AssignmentRegistry;

pub fn spawn_redis_state_persister(
    state: &SharedState,
    registry: AssignmentRegistry,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    let redis_url = state.config.redis_url.clone();
    tasks.spawn(async move {
        crate::storage::redis::run_state_persister(&redis_url, registry).await
    });
}
