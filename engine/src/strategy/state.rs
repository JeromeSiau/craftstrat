use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use super::Outcome;
use crate::fetcher::models::Tick;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub outcome: Outcome,
    pub entry_price: f64,
    pub size_usdc: f64,
    pub entry_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyState {
    pub window: VecDeque<Tick>,
    pub window_size: usize,
    pub position: Option<Position>,
    pub pnl: f64,
    pub trades_this_slot: u32,
    pub current_slot_ts: u32,
    pub indicator_cache: HashMap<String, f64>,
}

impl StrategyState {
    #[allow(dead_code)]
    pub fn new(window_size: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(window_size),
            window_size,
            position: None,
            pnl: 0.0,
            trades_this_slot: 0,
            current_slot_ts: 0,
            indicator_cache: HashMap::new(),
        }
    }

    pub fn push_tick(&mut self, tick: Tick) {
        if self.window.len() >= self.window_size {
            self.window.pop_front();
        }
        self.window.push_back(tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::test_utils::test_tick;

    #[test]
    fn test_state_new() {
        let state = StrategyState::new(100);
        assert_eq!(state.window_size, 100);
        assert!(state.window.is_empty());
        assert!(state.position.is_none());
        assert!((state.pnl).abs() < f64::EPSILON);
    }

    #[test]
    fn test_push_tick_respects_window_size() {
        let mut state = StrategyState::new(3);
        for i in 0..5u32 {
            let mut tick = test_tick();
            tick.slot_ts = i;
            state.push_tick(tick);
        }
        assert_eq!(state.window.len(), 3);
        assert_eq!(state.window.front().unwrap().slot_ts, 2);
        assert_eq!(state.window.back().unwrap().slot_ts, 4);
    }

    #[test]
    fn test_state_serialization_roundtrip() {
        let mut state = StrategyState::new(10);
        state.push_tick(test_tick());
        state.pnl = 42.5;
        state.indicator_cache.insert("ema_20".into(), 0.55);
        let json = serde_json::to_string(&state).unwrap();
        let restored: StrategyState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.window.len(), 1);
        assert!((restored.pnl - 42.5).abs() < f64::EPSILON);
        assert!((restored.indicator_cache["ema_20"] - 0.55).abs() < f64::EPSILON);
    }

}
