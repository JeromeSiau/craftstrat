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

    fn test_tick() -> Tick {
        Tick {
            captured_at: time::OffsetDateTime::from_unix_timestamp(1700000450).unwrap(),
            symbol: "btc-updown-15m-1700000000".into(),
            slot_ts: 1700000000,
            slot_duration: 900,
            minutes_into_slot: 7.5,
            pct_into_slot: 0.5,
            bid_up: 0.60,
            ask_up: 0.62,
            bid_down: 0.38,
            ask_down: 0.40,
            bid_size_up: 100.0,
            ask_size_up: 80.0,
            bid_size_down: 90.0,
            ask_size_down: 70.0,
            spread_up: 0.02,
            spread_down: 0.02,
            bid_up_l2: 0.58,
            ask_up_l2: 0.65,
            bid_up_l3: 0.55,
            ask_up_l3: 0.68,
            bid_down_l2: 0.36,
            ask_down_l2: 0.42,
            bid_down_l3: 0.34,
            ask_down_l3: 0.44,
            mid_up: 0.61,
            mid_down: 0.39,
            size_ratio_up: 1.25,
            size_ratio_down: 1.29,
            ref_price: 50500.0,
            dir_move_pct: 1.0,
            abs_move_pct: 1.0,
            hour_utc: 14,
            day_of_week: 2,
            market_volume_usd: 0.0,
            winner: None,
            ref_price_start: 50000.0,
            ref_price_end: 50500.0,
            ref_price_source: "binance".into(),
        }
    }
}
