use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use metrics::gauge;
use tokio::sync::RwLock;

use super::state::StrategyState;
use crate::metrics as m;

#[derive(Clone)]
pub struct Assignment {
    pub wallet_id: u64,
    pub strategy_id: u64,
    pub graph: serde_json::Value,
    pub markets: Vec<String>,
    pub max_position_usdc: f64,
    pub is_paper: bool,
    pub is_killed: bool,
    pub state: Arc<Mutex<StrategyState>>,
}

#[derive(Clone)]
pub struct AssignmentRegistry(Arc<RwLock<HashMap<String, Vec<Assignment>>>>);

impl AssignmentRegistry {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }
}

impl std::ops::Deref for AssignmentRegistry {
    type Target = RwLock<HashMap<String, Vec<Assignment>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn activate(
    registry: &AssignmentRegistry,
    wallet_id: u64,
    strategy_id: u64,
    graph: serde_json::Value,
    markets: Vec<String>,
    max_position_usdc: f64,
    is_paper: bool,
    initial_state: Option<StrategyState>,
) {
    let state = initial_state.unwrap_or_else(|| StrategyState::new(200));
    let assignment = Assignment {
        wallet_id,
        strategy_id,
        graph,
        markets: markets.clone(),
        max_position_usdc,
        is_paper,
        is_killed: false,
        state: Arc::new(Mutex::new(state)),
    };
    let (wallets, assignments) = {
        let mut reg = registry.write().await;
        for market in &markets {
            reg.entry(market.clone())
                .or_default()
                .push(assignment.clone());
        }
        count_from_reg(&reg)
    };
    tracing::info!(wallet_id, strategy_id, ?markets, "assignment_activated");
    gauge!(m::ACTIVE_WALLETS).set(wallets as f64);
    gauge!(m::ACTIVE_ASSIGNMENTS).set(assignments as f64);
}

pub async fn deactivate(registry: &AssignmentRegistry, wallet_id: u64, strategy_id: u64) {
    let (wallets, assignments) = {
        let mut reg = registry.write().await;
        for assignments in reg.values_mut() {
            assignments.retain(|a| !(a.wallet_id == wallet_id && a.strategy_id == strategy_id));
        }
        reg.retain(|_, v| !v.is_empty());
        count_from_reg(&reg)
    };
    tracing::info!(wallet_id, strategy_id, "assignment_deactivated");
    gauge!(m::ACTIVE_WALLETS).set(wallets as f64);
    gauge!(m::ACTIVE_ASSIGNMENTS).set(assignments as f64);
}

pub async fn kill(registry: &AssignmentRegistry, wallet_id: u64, strategy_id: u64) -> bool {
    set_killed(registry, wallet_id, strategy_id, true).await
}

pub async fn unkill(registry: &AssignmentRegistry, wallet_id: u64, strategy_id: u64) -> bool {
    set_killed(registry, wallet_id, strategy_id, false).await
}

async fn set_killed(
    registry: &AssignmentRegistry,
    wallet_id: u64,
    strategy_id: u64,
    killed: bool,
) -> bool {
    let mut reg = registry.write().await;
    let mut found = false;
    for assignments in reg.values_mut() {
        for a in assignments.iter_mut() {
            if a.wallet_id == wallet_id && a.strategy_id == strategy_id {
                a.is_killed = killed;
                found = true;
            }
        }
    }
    if found {
        tracing::info!(wallet_id, strategy_id, killed, "assignment_kill_switch_toggled");
    }
    found
}

fn count_from_reg(reg: &HashMap<String, Vec<Assignment>>) -> (usize, usize) {
    let mut wallet_ids = std::collections::HashSet::new();
    let mut count = 0usize;
    for assignments in reg.values() {
        for a in assignments {
            wallet_ids.insert(a.wallet_id);
            count += 1;
        }
    }
    (wallet_ids.len(), count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_activate_and_lookup() {
        let reg = AssignmentRegistry::new();
        activate(
            &reg,
            1,
            100,
            serde_json::json!({"mode": "form"}),
            vec!["btc-updown-15m".into()],
            200.0,
            false,
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
        let reg = AssignmentRegistry::new();
        activate(
            &reg,
            1,
            100,
            serde_json::json!({}),
            vec!["btc-15m".into(), "eth-15m".into()],
            100.0,
            false,
            None,
        )
        .await;

        let r = reg.read().await;
        assert!(r.contains_key("btc-15m"));
        assert!(r.contains_key("eth-15m"));
    }

    #[tokio::test]
    async fn test_deactivate() {
        let reg = AssignmentRegistry::new();
        activate(
            &reg,
            1,
            100,
            serde_json::json!({}),
            vec!["btc".into()],
            100.0,
            false,
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
            false,
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
        let reg = AssignmentRegistry::new();
        activate(
            &reg,
            1,
            100,
            serde_json::json!({}),
            vec!["btc".into()],
            100.0,
            false,
            None,
        )
        .await;
        deactivate(&reg, 1, 100).await;

        let r = reg.read().await;
        assert!(!r.contains_key("btc"));
    }

    #[tokio::test]
    async fn test_kill_sets_flag() {
        let reg = AssignmentRegistry::new();
        activate(
            &reg,
            1,
            100,
            serde_json::json!({}),
            vec!["btc".into()],
            100.0,
            false,
            None,
        )
        .await;

        let found = kill(&reg, 1, 100).await;
        assert!(found, "kill should find the assignment");

        let r = reg.read().await;
        let a = &r.get("btc").unwrap()[0];
        assert!(a.is_killed, "assignment should be killed");
    }

    #[tokio::test]
    async fn test_unkill_clears_flag() {
        let reg = AssignmentRegistry::new();
        activate(
            &reg,
            1,
            100,
            serde_json::json!({}),
            vec!["btc".into()],
            100.0,
            false,
            None,
        )
        .await;

        kill(&reg, 1, 100).await;
        let found = unkill(&reg, 1, 100).await;
        assert!(found, "unkill should find the assignment");

        let r = reg.read().await;
        let a = &r.get("btc").unwrap()[0];
        assert!(!a.is_killed, "assignment should not be killed");
    }

    #[tokio::test]
    async fn test_kill_returns_false_for_unknown() {
        let reg = AssignmentRegistry::new();
        let found = kill(&reg, 999, 999).await;
        assert!(!found, "kill should return false for unknown assignment");
    }
}
