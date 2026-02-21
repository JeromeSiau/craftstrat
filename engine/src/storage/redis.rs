use anyhow::Result;
use std::collections::HashSet;
use std::time::Duration;

use crate::strategy::registry::AssignmentRegistry;
use crate::strategy::state::StrategyState;

pub async fn save_states(
    conn: &mut redis::aio::MultiplexedConnection,
    registry: &AssignmentRegistry,
) -> Result<()> {
    let reg = registry.read().await;
    let mut pipe = redis::pipe();
    let mut count = 0u32;
    let mut seen = HashSet::new();
    for assignments in reg.values() {
        for a in assignments {
            if !seen.insert((a.wallet_id, a.strategy_id)) {
                continue;
            }
            let state = match a.state.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let key = format!("oddex:strategy_state:{}:{}", a.wallet_id, a.strategy_id);
            let json = serde_json::to_string(&*state)?;
            pipe.set_ex(&key, json, 3600);
            count += 1;
        }
    }
    drop(reg);
    if count > 0 {
        pipe.query_async::<()>(conn).await?;
        tracing::debug!(count, "redis_states_saved");
    }
    Ok(())
}

pub async fn load_state(
    conn: &mut redis::aio::MultiplexedConnection,
    wallet_id: u64,
    strategy_id: u64,
) -> Result<Option<StrategyState>> {
    let key = format!("oddex:strategy_state:{}:{}", wallet_id, strategy_id);
    let json: Option<String> = redis::cmd("GET").arg(&key).query_async(conn).await?;
    match json {
        Some(s) => Ok(Some(serde_json::from_str(&s)?)),
        None => Ok(None),
    }
}

pub async fn run_state_persister(redis_url: &str, registry: AssignmentRegistry) -> Result<()> {
    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_tokio_connection().await?;
    tracing::info!("redis_state_persister_started");

    let mut interval = tokio::time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        if let Err(e) = save_states(&mut conn, &registry).await {
            tracing::warn!(error = %e, "redis_state_save_failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::strategy::state::StrategyState;

    #[test]
    fn test_state_key_format() {
        let key = format!("oddex:strategy_state:{}:{}", 42u64, 100u64);
        assert_eq!(key, "oddex:strategy_state:42:100");
    }

    #[test]
    fn test_state_json_roundtrip() {
        let mut state = StrategyState::new(50);
        state.pnl = 123.45;
        state.trades_this_slot = 3;
        state.indicator_cache.insert("ema_20".into(), 0.65);

        let json = serde_json::to_string(&state).unwrap();
        let restored: StrategyState = serde_json::from_str(&json).unwrap();
        assert!((restored.pnl - 123.45).abs() < f64::EPSILON);
        assert_eq!(restored.trades_this_slot, 3);
        assert!((restored.indicator_cache["ema_20"] - 0.65).abs() < f64::EPSILON);
    }
}
