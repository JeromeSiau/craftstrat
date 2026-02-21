use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::RwLock;

use super::state::StrategyState;

#[derive(Clone)]
pub struct Assignment {
    pub wallet_id: u64,
    pub strategy_id: u64,
    pub graph: serde_json::Value,
    pub markets: Vec<String>,
    pub max_position_usdc: f64,
    pub state: Arc<Mutex<StrategyState>>,
}

pub type AssignmentRegistry = Arc<RwLock<HashMap<String, Vec<Assignment>>>>;

pub fn new_registry() -> AssignmentRegistry {
    Arc::new(RwLock::new(HashMap::new()))
}

pub async fn activate(
    registry: &AssignmentRegistry,
    wallet_id: u64,
    strategy_id: u64,
    graph: serde_json::Value,
    markets: Vec<String>,
    max_position_usdc: f64,
    initial_state: Option<StrategyState>,
) {
    let state = initial_state.unwrap_or_else(|| StrategyState::new(200));
    let assignment = Assignment {
        wallet_id,
        strategy_id,
        graph,
        markets: markets.clone(),
        max_position_usdc,
        state: Arc::new(Mutex::new(state)),
    };
    let mut reg = registry.write().await;
    for market in &markets {
        reg.entry(market.clone())
            .or_default()
            .push(assignment.clone());
    }
    tracing::info!(wallet_id, strategy_id, ?markets, "assignment_activated");
}

pub async fn deactivate(registry: &AssignmentRegistry, wallet_id: u64, strategy_id: u64) {
    let mut reg = registry.write().await;
    for assignments in reg.values_mut() {
        assignments.retain(|a| !(a.wallet_id == wallet_id && a.strategy_id == strategy_id));
    }
    reg.retain(|_, v| !v.is_empty());
    tracing::info!(wallet_id, strategy_id, "assignment_deactivated");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_activate_and_lookup() {
        let reg = new_registry();
        activate(
            &reg,
            1,
            100,
            serde_json::json!({"mode": "form"}),
            vec!["btc-updown-15m".into()],
            200.0,
            None,
        )
        .await;

        let r = reg.read().await;
        let assignments = r.get("btc-updown-15m").unwrap();
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].wallet_id, 1);
        assert_eq!(assignments[0].strategy_id, 100);
    }

    #[tokio::test]
    async fn test_activate_multi_market() {
        let reg = new_registry();
        activate(
            &reg,
            1,
            100,
            serde_json::json!({}),
            vec!["btc-15m".into(), "eth-15m".into()],
            100.0,
            None,
        )
        .await;

        let r = reg.read().await;
        assert!(r.contains_key("btc-15m"));
        assert!(r.contains_key("eth-15m"));
    }

    #[tokio::test]
    async fn test_deactivate() {
        let reg = new_registry();
        activate(
            &reg,
            1,
            100,
            serde_json::json!({}),
            vec!["btc".into()],
            100.0,
            None,
        )
        .await;
        activate(
            &reg,
            2,
            200,
            serde_json::json!({}),
            vec!["btc".into()],
            100.0,
            None,
        )
        .await;

        deactivate(&reg, 1, 100).await;

        let r = reg.read().await;
        let assignments = r.get("btc").unwrap();
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].wallet_id, 2);
    }

    #[tokio::test]
    async fn test_deactivate_removes_empty_entries() {
        let reg = new_registry();
        activate(
            &reg,
            1,
            100,
            serde_json::json!({}),
            vec!["btc".into()],
            100.0,
            None,
        )
        .await;
        deactivate(&reg, 1, 100).await;

        let r = reg.read().await;
        assert!(!r.contains_key("btc"));
    }
}
