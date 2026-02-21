# Phase 3 — Strategy Engine Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the Rust strategy engine that consumes ticks from Kafka, evaluates JSON-defined strategies (form + node mode) in parallel across wallet/strategy pairs, and persists state to Redis.

**Architecture:** Kafka consumer reads ticks → AssignmentRegistry maps symbols to active wallet/strategy pairs → Rayon parallel dispatch calls the JSON graph interpreter for each pair → Signals output to an mpsc channel (consumed by execution queue in Phase 4). Strategy state (sliding window, positions, indicator cache) stored in `Arc<std::sync::Mutex<StrategyState>>` per assignment, persisted to Redis every 10s.

**Tech Stack:** Rust, Tokio, rdkafka (StreamConsumer), Rayon, Redis, serde_json

---

## Task 1: Add `ref_price_source` field

Preparatory schema alignment. Add provenance tracking for reference prices (binance, chainlink, pyth).

**Files:**
- Modify: `engine/src/fetcher/models.rs` — add field to Tick
- Modify: `engine/src/fetcher/tick_builder.rs` — populate field
- Modify: `infra/clickhouse/init.sql` — add column

### Step 1: Add field to Tick struct

In `engine/src/fetcher/models.rs`, add `ref_price_source` at the end of the Tick struct (after `ref_price_end`) to match ClickHouse column order:

```rust
    #[serde(rename = "btc_price_end")]
    pub ref_price_end: f32,
    pub ref_price_source: String,
}
```

### Step 2: Update ClickHouse schema

In `infra/clickhouse/init.sql`, add the column at the end (before the closing paren):

```sql
    btc_price_end     Float32,
    ref_price_source  LowCardinality(String)
) ENGINE = MergeTree()
```

For existing ClickHouse instances:
```sql
ALTER TABLE slot_snapshots ADD COLUMN ref_price_source LowCardinality(String) AFTER btc_price_end;
```

### Step 3: Populate in tick_builder

In `engine/src/fetcher/tick_builder.rs`, update `build_tick()` to accept and set the source:

Change the function signature:
```rust
pub fn build_tick(
    market: &ActiveMarket,
    book_up: Option<&OrderBook>,
    book_down: Option<&OrderBook>,
    ref_price: f32,
    ref_price_source: &str,
    now_unix: f64,
) -> Option<Tick> {
```

Add to the Tick construction:
```rust
        ref_price_end: ref_price,
        ref_price_source: ref_price_source.to_string(),
    })
```

Update the call site in `run_tick_builder()`:
```rust
if let Some(tick) = build_tick(market, book_up, book_down, ref_price, "binance", now) {
```

### Step 4: Fix tests

Update all test calls to `build_tick()` — add `"binance"` parameter and `ref_price_source` in any manual Tick construction.

### Step 5: Verify compilation

```bash
cd engine && cargo build 2>&1
```

Expected: compiles cleanly.

### Step 6: Commit

```bash
git add engine/src/fetcher/models.rs engine/src/fetcher/tick_builder.rs infra/clickhouse/init.sql
git commit -m "feat: add ref_price_source field for price provenance tracking"
```

---

## Task 2: Add Deserialize to Tick + Kafka consumer module

**Files:**
- Modify: `engine/src/fetcher/models.rs` — add `Deserialize` derive
- Create: `engine/src/kafka/consumer.rs`
- Modify: `engine/src/kafka/mod.rs` — register consumer module

### Step 1: Add Deserialize to Tick

In `engine/src/fetcher/models.rs`, change the Tick derive:

```rust
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct Tick {
```

### Step 2: Write Kafka consumer

Create `engine/src/kafka/consumer.rs`:

```rust
use anyhow::Result;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};

pub fn create_consumer(brokers: &str, group_id: &str) -> Result<StreamConsumer> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("auto.offset.reset", "latest")
        .set("enable.auto.commit", "true")
        .set("enable.auto.offset.store", "true")
        .create()?;
    consumer.subscribe(&["ticks"])?;
    tracing::info!(group_id, "kafka_consumer_created");
    Ok(consumer)
}
```

### Step 3: Register module

In `engine/src/kafka/mod.rs`:

```rust
pub mod consumer;
pub mod producer;
```

### Step 4: Verify compilation

```bash
cd engine && cargo build 2>&1
```

### Step 5: Commit

```bash
git add engine/src/fetcher/models.rs engine/src/kafka/consumer.rs engine/src/kafka/mod.rs
git commit -m "feat: add Kafka consumer and Tick deserialization"
```

---

## Task 3: Core strategy types

**Files:**
- Create: `engine/src/strategy/mod.rs`
- Create: `engine/src/strategy/state.rs`
- Modify: `engine/src/main.rs` — register module

### Step 1: Write tests for core types

Create `engine/src/strategy/mod.rs`:

```rust
pub mod state;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Signal {
    Buy {
        outcome: Outcome,
        size_usdc: f64,
        order_type: OrderType,
    },
    Sell {
        outcome: Outcome,
        size_usdc: f64,
        order_type: OrderType,
    },
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit { price: f64 },
    StopLoss { trigger_price: f64 },
    TakeProfit { trigger_price: f64 },
}

pub struct EngineOutput {
    pub wallet_id: u64,
    pub strategy_id: u64,
    pub symbol: String,
    pub signal: Signal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_serialization_roundtrip() {
        let signal = Signal::Buy {
            outcome: Outcome::Up,
            size_usdc: 50.0,
            order_type: OrderType::Market,
        };
        let json = serde_json::to_string(&signal).unwrap();
        let deserialized: Signal = serde_json::from_str(&json).unwrap();
        match deserialized {
            Signal::Buy { outcome, size_usdc, .. } => {
                assert_eq!(outcome, Outcome::Up);
                assert!((size_usdc - 50.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected Buy signal"),
        }
    }

    #[test]
    fn test_outcome_equality() {
        assert_eq!(Outcome::Up, Outcome::Up);
        assert_ne!(Outcome::Up, Outcome::Down);
    }
}
```

### Step 2: Write StrategyState and Position

Create `engine/src/strategy/state.rs`:

```rust
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
            bid_up: 0.60, ask_up: 0.62,
            bid_down: 0.38, ask_down: 0.40,
            bid_size_up: 100.0, ask_size_up: 80.0,
            bid_size_down: 90.0, ask_size_down: 70.0,
            spread_up: 0.02, spread_down: 0.02,
            bid_up_l2: 0.58, ask_up_l2: 0.65,
            bid_up_l3: 0.55, ask_up_l3: 0.68,
            bid_down_l2: 0.36, ask_down_l2: 0.42,
            bid_down_l3: 0.34, ask_down_l3: 0.44,
            mid_up: 0.61, mid_down: 0.39,
            size_ratio_up: 1.25, size_ratio_down: 1.29,
            ref_price: 50500.0,
            dir_move_pct: 1.0, abs_move_pct: 1.0,
            hour_utc: 14, day_of_week: 2,
            market_volume_usd: 0.0,
            winner: None,
            ref_price_start: 50000.0,
            ref_price_end: 50500.0,
            ref_price_source: "binance".into(),
        }
    }
}
```

### Step 3: Register strategy module

In `engine/src/main.rs`, add at top:

```rust
mod strategy;
```

### Step 4: Run tests

```bash
cd engine && cargo test strategy
```

Expected: all tests pass.

### Step 5: Commit

```bash
git add engine/src/strategy/mod.rs engine/src/strategy/state.rs engine/src/main.rs
git commit -m "feat: add core strategy types — Signal, Outcome, OrderType, StrategyState"
```

---

## Task 4: Tick field accessor + stateless comparators

**Files:**
- Create: `engine/src/strategy/eval.rs`
- Modify: `engine/src/strategy/mod.rs` — register module

### Step 1: Write failing tests

Create `engine/src/strategy/eval.rs`:

```rust
use crate::fetcher::models::Tick;

pub fn get_field(tick: &Tick, name: &str) -> Option<f64> {
    todo!()
}

pub fn evaluate_op(value: f64, operator: &str, target: &serde_json::Value) -> bool {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_field_stateless() {
        let tick = test_tick();
        assert!((get_field(&tick, "abs_move_pct").unwrap() - 1.0).abs() < 0.001);
        assert!((get_field(&tick, "pct_into_slot").unwrap() - 0.5).abs() < 0.001);
        assert!((get_field(&tick, "spread_up").unwrap() - 0.02).abs() < 0.001);
        assert!((get_field(&tick, "mid_up").unwrap() - 0.61).abs() < 0.001);
        assert!((get_field(&tick, "hour_utc").unwrap() - 14.0).abs() < 0.001);
    }

    #[test]
    fn test_get_field_aliases() {
        let tick = test_tick();
        let a = get_field(&tick, "ref_price").unwrap();
        let b = get_field(&tick, "chainlink_price").unwrap();
        assert!((a - b).abs() < 0.001);
    }

    #[test]
    fn test_get_field_unknown() {
        let tick = test_tick();
        assert!(get_field(&tick, "nonexistent").is_none());
    }

    #[test]
    fn test_evaluate_op_gt() {
        assert!(evaluate_op(5.0, ">", &serde_json::json!(3.0)));
        assert!(!evaluate_op(3.0, ">", &serde_json::json!(5.0)));
    }

    #[test]
    fn test_evaluate_op_between() {
        assert!(evaluate_op(0.3, "between", &serde_json::json!([0.1, 0.5])));
        assert!(!evaluate_op(0.8, "between", &serde_json::json!([0.1, 0.5])));
        // boundaries inclusive
        assert!(evaluate_op(0.1, "between", &serde_json::json!([0.1, 0.5])));
    }

    #[test]
    fn test_evaluate_op_all_operators() {
        assert!(evaluate_op(5.0, ">=", &serde_json::json!(5.0)));
        assert!(evaluate_op(3.0, "<", &serde_json::json!(5.0)));
        assert!(evaluate_op(3.0, "<=", &serde_json::json!(3.0)));
        assert!(evaluate_op(3.0, "==", &serde_json::json!(3.0)));
        assert!(evaluate_op(3.0, "!=", &serde_json::json!(5.0)));
    }

    fn test_tick() -> Tick {
        Tick {
            captured_at: time::OffsetDateTime::from_unix_timestamp(1700000450).unwrap(),
            symbol: "btc-updown-15m-1700000000".into(),
            slot_ts: 1700000000, slot_duration: 900,
            minutes_into_slot: 7.5, pct_into_slot: 0.5,
            bid_up: 0.60, ask_up: 0.62,
            bid_down: 0.38, ask_down: 0.40,
            bid_size_up: 100.0, ask_size_up: 80.0,
            bid_size_down: 90.0, ask_size_down: 70.0,
            spread_up: 0.02, spread_down: 0.02,
            bid_up_l2: 0.58, ask_up_l2: 0.65,
            bid_up_l3: 0.55, ask_up_l3: 0.68,
            bid_down_l2: 0.36, ask_down_l2: 0.42,
            bid_down_l3: 0.34, ask_down_l3: 0.44,
            mid_up: 0.61, mid_down: 0.39,
            size_ratio_up: 1.25, size_ratio_down: 1.29,
            ref_price: 50500.0,
            dir_move_pct: 1.0, abs_move_pct: 1.0,
            hour_utc: 14, day_of_week: 2,
            market_volume_usd: 0.0,
            winner: None,
            ref_price_start: 50000.0, ref_price_end: 50500.0,
            ref_price_source: "binance".into(),
        }
    }
}
```

### Step 2: Run tests to verify they fail

```bash
cd engine && cargo test strategy::eval 2>&1
```

Expected: FAIL with `not yet implemented`.

### Step 3: Implement get_field

```rust
pub fn get_field(tick: &Tick, name: &str) -> Option<f64> {
    match name {
        "abs_move_pct" => Some(tick.abs_move_pct as f64),
        "dir_move_pct" => Some(tick.dir_move_pct as f64),
        "spread_up" => Some(tick.spread_up as f64),
        "spread_down" => Some(tick.spread_down as f64),
        "size_ratio_up" => Some(tick.size_ratio_up as f64),
        "size_ratio_down" => Some(tick.size_ratio_down as f64),
        "pct_into_slot" => Some(tick.pct_into_slot as f64),
        "minutes_into_slot" => Some(tick.minutes_into_slot as f64),
        "mid_up" => Some(tick.mid_up as f64),
        "mid_down" => Some(tick.mid_down as f64),
        "bid_up" => Some(tick.bid_up as f64),
        "ask_up" => Some(tick.ask_up as f64),
        "bid_down" => Some(tick.bid_down as f64),
        "ask_down" => Some(tick.ask_down as f64),
        "bid_size_up" => Some(tick.bid_size_up as f64),
        "ask_size_up" => Some(tick.ask_size_up as f64),
        "bid_size_down" => Some(tick.bid_size_down as f64),
        "ask_size_down" => Some(tick.ask_size_down as f64),
        "ref_price" | "chainlink_price" => Some(tick.ref_price as f64),
        "hour_utc" => Some(tick.hour_utc as f64),
        "day_of_week" => Some(tick.day_of_week as f64),
        "market_volume_usd" => Some(tick.market_volume_usd as f64),
        _ => None,
    }
}
```

### Step 4: Implement evaluate_op

```rust
pub fn evaluate_op(value: f64, operator: &str, target: &serde_json::Value) -> bool {
    match operator {
        ">" => target.as_f64().map_or(false, |t| value > t),
        ">=" => target.as_f64().map_or(false, |t| value >= t),
        "<" => target.as_f64().map_or(false, |t| value < t),
        "<=" => target.as_f64().map_or(false, |t| value <= t),
        "==" => target.as_f64().map_or(false, |t| (value - t).abs() < f64::EPSILON),
        "!=" => target.as_f64().map_or(false, |t| (value - t).abs() >= f64::EPSILON),
        "between" => {
            if let Some(arr) = target.as_array() {
                let lo = arr.first().and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
                let hi = arr.get(1).and_then(|v| v.as_f64()).unwrap_or(f64::MAX);
                value >= lo && value <= hi
            } else {
                false
            }
        }
        _ => false,
    }
}
```

### Step 5: Register module

In `engine/src/strategy/mod.rs`, add:

```rust
pub mod eval;
```

### Step 6: Run tests

```bash
cd engine && cargo test strategy::eval
```

Expected: all pass.

### Step 7: Commit

```bash
git add engine/src/strategy/eval.rs engine/src/strategy/mod.rs
git commit -m "feat: add tick field accessor and stateless comparators"
```

---

## Task 5: Stateful indicators

**Files:**
- Create: `engine/src/strategy/indicators.rs`
- Modify: `engine/src/strategy/mod.rs` — register module

### Step 1: Write failing tests

Create `engine/src/strategy/indicators.rs`:

```rust
use crate::fetcher::models::Tick;
use super::eval::get_field;

pub fn ema(values: &[f64], period: usize) -> f64 { todo!() }
pub fn sma(values: &[f64], period: usize) -> f64 { todo!() }
pub fn rsi(values: &[f64], period: usize) -> f64 { todo!() }
pub fn vwap(ticks: &[Tick], field: &str) -> f64 { todo!() }
pub fn cross_above(prev_a: f64, curr_a: f64, prev_b: f64, curr_b: f64) -> bool { todo!() }
pub fn cross_below(prev_a: f64, curr_a: f64, prev_b: f64, curr_b: f64) -> bool { todo!() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_basic() {
        assert!((sma(&[1.0, 2.0, 3.0, 4.0, 5.0], 3) - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_sma_period_larger_than_data() {
        assert!((sma(&[2.0, 4.0], 10) - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_sma_empty() {
        assert!((sma(&[], 5)).abs() < 0.001);
    }

    #[test]
    fn test_ema_basic() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = ema(&values, 3);
        // EMA(3): k = 0.5
        // ema0=1, ema1=1.5, ema2=2.25, ema3=3.125, ema4=4.0625
        assert!((result - 4.0625).abs() < 0.001);
    }

    #[test]
    fn test_ema_single_value() {
        assert!((ema(&[42.0], 5) - 42.0).abs() < 0.001);
    }

    #[test]
    fn test_rsi_all_gains() {
        // all positive changes → RSI = 100
        assert!((rsi(&[1.0, 2.0, 3.0, 4.0, 5.0], 4) - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_rsi_all_losses() {
        // all negative changes → RSI = 0
        assert!((rsi(&[5.0, 4.0, 3.0, 2.0, 1.0], 4) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_rsi_mixed() {
        // 50/50 gains and losses of equal magnitude → RSI = 50
        let values = vec![10.0, 11.0, 10.0, 11.0, 10.0];
        let result = rsi(&values, 4);
        assert!((result - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_rsi_insufficient_data() {
        assert!((rsi(&[5.0], 14) - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_cross_above() {
        assert!(cross_above(0.4, 0.6, 0.5, 0.5)); // a crosses above b
        assert!(!cross_above(0.6, 0.7, 0.5, 0.5)); // a was already above
        assert!(!cross_above(0.4, 0.3, 0.5, 0.5)); // a still below
    }

    #[test]
    fn test_cross_below() {
        assert!(cross_below(0.6, 0.4, 0.5, 0.5)); // a crosses below b
        assert!(!cross_below(0.4, 0.3, 0.5, 0.5)); // a was already below
    }
}
```

### Step 2: Run tests to verify they fail

```bash
cd engine && cargo test strategy::indicators 2>&1
```

Expected: FAIL with `not yet implemented`.

### Step 3: Implement all indicators

```rust
pub fn sma(values: &[f64], period: usize) -> f64 {
    if values.is_empty() || period == 0 {
        return 0.0;
    }
    let n = values.len().min(period);
    let sum: f64 = values[values.len() - n..].iter().sum();
    sum / n as f64
}

pub fn ema(values: &[f64], period: usize) -> f64 {
    if values.is_empty() || period == 0 {
        return 0.0;
    }
    let k = 2.0 / (period as f64 + 1.0);
    let mut result = values[0];
    for v in &values[1..] {
        result = v * k + result * (1.0 - k);
    }
    result
}

pub fn rsi(values: &[f64], period: usize) -> f64 {
    if values.len() < 2 || period == 0 {
        return 50.0;
    }
    let changes: Vec<f64> = values.windows(2).map(|w| w[1] - w[0]).collect();
    let n = changes.len().min(period);
    let recent = &changes[changes.len() - n..];
    let avg_gain: f64 = recent.iter().filter(|&&c| c > 0.0).sum::<f64>() / n as f64;
    let avg_loss: f64 = recent
        .iter()
        .filter(|&&c| c < 0.0)
        .map(|c| c.abs())
        .sum::<f64>()
        / n as f64;
    if avg_loss < f64::EPSILON {
        return 100.0;
    }
    if avg_gain < f64::EPSILON {
        return 0.0;
    }
    let rs = avg_gain / avg_loss;
    100.0 - 100.0 / (1.0 + rs)
}

pub fn vwap(ticks: &[Tick], field: &str) -> f64 {
    let mut sum_pv = 0.0;
    let mut sum_v = 0.0;
    for t in ticks {
        let price = get_field(t, field).unwrap_or(0.0);
        let vol = t.market_volume_usd as f64;
        sum_pv += price * vol;
        sum_v += vol;
    }
    if sum_v > 0.0 {
        sum_pv / sum_v
    } else {
        0.0
    }
}

pub fn cross_above(prev_a: f64, curr_a: f64, prev_b: f64, curr_b: f64) -> bool {
    prev_a <= prev_b && curr_a > curr_b
}

pub fn cross_below(prev_a: f64, curr_a: f64, prev_b: f64, curr_b: f64) -> bool {
    prev_a >= prev_b && curr_a < curr_b
}
```

### Step 4: Register module

In `engine/src/strategy/mod.rs`, add:

```rust
pub mod indicators;
```

### Step 5: Run tests

```bash
cd engine && cargo test strategy::indicators
```

Expected: all pass.

### Step 6: Commit

```bash
git add engine/src/strategy/indicators.rs engine/src/strategy/mod.rs
git commit -m "feat: add stateful indicators — EMA, SMA, RSI, VWAP, cross detection"
```

---

## Task 6: JSON graph interpreter — form mode

**Files:**
- Create: `engine/src/strategy/interpreter.rs`
- Modify: `engine/src/strategy/mod.rs` — register module

### Step 1: Write failing tests

Create `engine/src/strategy/interpreter.rs` with test-first approach. Key tests:

```rust
use serde_json::Value;

use super::eval::{evaluate_op, get_field};
use super::indicators;
use super::state::{Position, StrategyState};
use super::{OrderType, Outcome, Signal};
use crate::fetcher::models::Tick;

pub fn evaluate(graph: &Value, tick: &Tick, state: &mut StrategyState) -> Signal {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tick() -> Tick { /* same helper as Task 3 */ }

    fn simple_form_graph() -> Value {
        serde_json::json!({
            "mode": "form",
            "conditions": [{
                "type": "AND",
                "rules": [
                    { "indicator": "abs_move_pct", "operator": ">", "value": 0.5 },
                    { "indicator": "pct_into_slot", "operator": "between", "value": [0.1, 0.6] }
                ]
            }],
            "action": {
                "signal": "buy",
                "outcome": "UP",
                "size_mode": "fixed",
                "size_usdc": 50,
                "order_type": "market"
            },
            "risk": {
                "stoploss_pct": 30,
                "take_profit_pct": 80,
                "max_position_usdc": 200,
                "max_trades_per_slot": 1
            }
        })
    }

    #[test]
    fn test_form_conditions_met_produces_buy() {
        let graph = simple_form_graph();
        let tick = test_tick(); // abs_move_pct=1.0 > 0.5 ✓, pct_into_slot=0.5 in [0.1,0.6] ✓
        let mut state = StrategyState::new(100);
        let signal = evaluate(&graph, &tick, &mut state);
        match signal {
            Signal::Buy { outcome, size_usdc, order_type } => {
                assert_eq!(outcome, Outcome::Up);
                assert!((size_usdc - 50.0).abs() < f64::EPSILON);
                assert!(matches!(order_type, OrderType::Market));
            }
            _ => panic!("expected Buy, got {:?}", signal),
        }
    }

    #[test]
    fn test_form_conditions_not_met_holds() {
        let graph = simple_form_graph();
        let mut tick = test_tick();
        tick.abs_move_pct = 0.2; // < 0.5, fails condition
        let mut state = StrategyState::new(100);
        assert!(matches!(evaluate(&graph, &tick, &mut state), Signal::Hold));
    }

    #[test]
    fn test_form_max_trades_per_slot() {
        let graph = simple_form_graph();
        let tick = test_tick();
        let mut state = StrategyState::new(100);
        // First trade should succeed
        assert!(matches!(evaluate(&graph, &tick, &mut state), Signal::Buy { .. }));
        // Simulate that the trade was filled — set position and increment trade count
        state.position = Some(Position {
            outcome: Outcome::Up,
            entry_price: 0.62,
            size_usdc: 50.0,
            entry_at: 1700000450,
        });
        // Second tick: already in position → Hold (even if conditions met)
        assert!(matches!(evaluate(&graph, &tick, &mut state), Signal::Hold));
    }

    #[test]
    fn test_form_stoploss_triggers_sell() {
        let graph = simple_form_graph(); // stoploss_pct: 30
        let tick = test_tick(); // mid_up = 0.61
        let mut state = StrategyState::new(100);
        state.position = Some(Position {
            outcome: Outcome::Up,
            entry_price: 0.90, // entered at 0.90, current mid_up 0.61 → down 32% > 30%
            size_usdc: 50.0,
            entry_at: 1700000000,
        });
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(signal, Signal::Sell { order_type: OrderType::StopLoss { .. }, .. }));
    }

    #[test]
    fn test_form_take_profit_triggers_sell() {
        let graph = simple_form_graph(); // take_profit_pct: 80
        let tick = test_tick(); // mid_up = 0.61
        let mut state = StrategyState::new(100);
        state.position = Some(Position {
            outcome: Outcome::Up,
            entry_price: 0.30, // entered at 0.30, current mid_up 0.61 → up 103% > 80%
            size_usdc: 50.0,
            entry_at: 1700000000,
        });
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(signal, Signal::Sell { order_type: OrderType::TakeProfit { .. }, .. }));
    }

    #[test]
    fn test_form_slot_change_resets_trade_count() {
        let graph = simple_form_graph();
        let mut state = StrategyState::new(100);
        state.trades_this_slot = 1;
        state.current_slot_ts = 1700000000;

        let mut tick = test_tick();
        tick.slot_ts = 1700000900; // new slot
        let signal = evaluate(&graph, &tick, &mut state);
        // trade count reset → should produce Buy
        assert!(matches!(signal, Signal::Buy { .. }));
        assert_eq!(state.current_slot_ts, 1700000900);
    }

    #[test]
    fn test_form_stateful_indicator_ema() {
        let graph = serde_json::json!({
            "mode": "form",
            "conditions": [{
                "type": "AND",
                "rules": [{
                    "indicator": { "fn": "EMA", "period": 3, "field": "mid_up" },
                    "operator": ">",
                    "value": 0.5
                }]
            }],
            "action": { "signal": "buy", "outcome": "UP", "size_usdc": 25, "order_type": "market" },
            "risk": {}
        });
        let mut state = StrategyState::new(100);
        // Push some ticks to build window
        for mid in [0.55, 0.58, 0.60] {
            let mut t = test_tick();
            t.mid_up = mid;
            state.push_tick(t);
        }
        let tick = test_tick(); // mid_up = 0.61
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(signal, Signal::Buy { .. }));
    }

    #[test]
    fn test_form_or_group() {
        let graph = serde_json::json!({
            "mode": "form",
            "conditions": [{
                "type": "OR",
                "rules": [
                    { "indicator": "abs_move_pct", "operator": ">", "value": 10.0 },
                    { "indicator": "pct_into_slot", "operator": ">", "value": 0.3 }
                ]
            }],
            "action": { "signal": "buy", "outcome": "DOWN", "size_usdc": 30, "order_type": "market" },
            "risk": {}
        });
        let tick = test_tick(); // abs_move_pct=1.0 < 10 but pct_into_slot=0.5 > 0.3
        let mut state = StrategyState::new(100);
        let signal = evaluate(&graph, &tick, &mut state);
        match signal {
            Signal::Buy { outcome, .. } => assert_eq!(outcome, Outcome::Down),
            _ => panic!("expected Buy Down"),
        }
    }
}
```

### Step 2: Run tests to verify they fail

```bash
cd engine && cargo test strategy::interpreter 2>&1
```

### Step 3: Implement the form mode interpreter

Replace the `todo!()` in `evaluate()` and add helper functions:

```rust
pub fn evaluate(graph: &Value, tick: &Tick, state: &mut StrategyState) -> Signal {
    state.push_tick(tick.clone());

    // Reset trades counter on new slot
    if tick.slot_ts != state.current_slot_ts {
        state.trades_this_slot = 0;
        state.current_slot_ts = tick.slot_ts;
    }

    let mode = graph["mode"].as_str().unwrap_or("form");
    match mode {
        "form" => evaluate_form(graph, tick, state),
        "node" => evaluate_node(graph, tick, state),
        _ => Signal::Hold,
    }
}

fn evaluate_form(graph: &Value, tick: &Tick, state: &mut StrategyState) -> Signal {
    // 1. Check risk management on open position
    if let Some(ref pos) = state.position {
        if let Some(signal) = check_risk(graph, tick, pos) {
            return signal;
        }
        return Signal::Hold; // in position, no risk trigger → hold
    }

    // 2. Check max trades per slot
    let max_trades = graph["risk"]["max_trades_per_slot"].as_u64().unwrap_or(u64::MAX) as u32;
    if state.trades_this_slot >= max_trades {
        return Signal::Hold;
    }

    // 3. Evaluate entry conditions (OR across groups, AND/OR within groups)
    let conditions = &graph["conditions"];
    if evaluate_conditions(conditions, tick, state) {
        let signal = build_action_signal(&graph["action"]);
        state.trades_this_slot += 1;
        signal
    } else {
        Signal::Hold
    }
}

fn evaluate_conditions(conditions: &Value, tick: &Tick, state: &mut StrategyState) -> bool {
    let Some(groups) = conditions.as_array() else {
        return false;
    };
    // Any group matching → true (implicit OR across groups)
    groups.iter().any(|group| {
        let group_type = group["type"].as_str().unwrap_or("AND");
        let Some(rules) = group["rules"].as_array() else {
            return false;
        };
        match group_type {
            "OR" => rules.iter().any(|rule| evaluate_rule(rule, tick, state)),
            _ => rules.iter().all(|rule| evaluate_rule(rule, tick, state)),
        }
    })
}

fn evaluate_rule(rule: &Value, tick: &Tick, state: &mut StrategyState) -> bool {
    let indicator = &rule["indicator"];

    // Resolve indicator value (stateless field or stateful function)
    let value = match resolve_indicator(indicator, tick, state) {
        Some(v) => v,
        None => return false,
    };

    let operator = rule["operator"].as_str().unwrap_or("==");
    evaluate_op(value, operator, &rule["value"])
}

fn resolve_indicator(indicator: &Value, tick: &Tick, state: &StrategyState) -> Option<f64> {
    if let Some(name) = indicator.as_str() {
        return get_field(tick, name);
    }

    let obj = indicator.as_object()?;
    let func = obj.get("fn")?.as_str()?;
    let field = obj.get("field").and_then(|v| v.as_str()).unwrap_or("mid_up");

    let values: Vec<f64> = state
        .window
        .iter()
        .filter_map(|t| get_field(t, field))
        .collect();

    match func {
        "EMA" => {
            let period = obj.get("period").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
            Some(indicators::ema(&values, period))
        }
        "SMA" => {
            let period = obj.get("period").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
            Some(indicators::sma(&values, period))
        }
        "RSI" => {
            let period = obj.get("period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
            Some(indicators::rsi(&values, period))
        }
        "VWAP" => Some(indicators::vwap(
            state.window.make_contiguous(),
            field,
        )),
        _ => None,
    }
}

fn check_risk(graph: &Value, tick: &Tick, pos: &Position) -> Option<Signal> {
    let risk = &graph["risk"];
    let current_price = match pos.outcome {
        Outcome::Up => get_field(tick, "mid_up").unwrap_or(0.0),
        Outcome::Down => get_field(tick, "mid_down").unwrap_or(0.0),
    };

    if pos.entry_price <= 0.0 || current_price <= 0.0 {
        return None;
    }

    let pnl_pct = (current_price - pos.entry_price) / pos.entry_price * 100.0;

    // Stoploss: price dropped below threshold
    if let Some(sl) = risk["stoploss_pct"].as_f64() {
        if pnl_pct <= -sl {
            return Some(Signal::Sell {
                outcome: pos.outcome,
                size_usdc: pos.size_usdc,
                order_type: OrderType::StopLoss {
                    trigger_price: current_price,
                },
            });
        }
    }

    // Take profit: price rose above threshold
    if let Some(tp) = risk["take_profit_pct"].as_f64() {
        if pnl_pct >= tp {
            return Some(Signal::Sell {
                outcome: pos.outcome,
                size_usdc: pos.size_usdc,
                order_type: OrderType::TakeProfit {
                    trigger_price: current_price,
                },
            });
        }
    }

    None
}

fn build_action_signal(action: &Value) -> Signal {
    let outcome = match action["outcome"].as_str().unwrap_or("UP") {
        "DOWN" => Outcome::Down,
        _ => Outcome::Up,
    };
    let size_usdc = action["size_usdc"].as_f64().unwrap_or(10.0);
    let order_type = match action["order_type"].as_str().unwrap_or("market") {
        "limit" => OrderType::Limit {
            price: action["limit_price"].as_f64().unwrap_or(0.0),
        },
        _ => OrderType::Market,
    };
    let signal_type = action["signal"].as_str().unwrap_or("buy");
    match signal_type {
        "sell" => Signal::Sell { outcome, size_usdc, order_type },
        _ => Signal::Buy { outcome, size_usdc, order_type },
    }
}

fn evaluate_node(_graph: &Value, _tick: &Tick, _state: &mut StrategyState) -> Signal {
    todo!("node mode implemented in Task 7")
}
```

### Step 4: Register module

In `engine/src/strategy/mod.rs`, add:

```rust
pub mod interpreter;
```

### Step 5: Run tests

```bash
cd engine && cargo test strategy::interpreter
```

Expected: all pass.

### Step 6: Commit

```bash
git add engine/src/strategy/interpreter.rs engine/src/strategy/mod.rs
git commit -m "feat: add JSON graph interpreter — form mode with risk management"
```

---

## Task 7: JSON graph interpreter — node mode

**Files:**
- Modify: `engine/src/strategy/interpreter.rs` — implement `evaluate_node()`

### Step 1: Write failing tests

Add to the `tests` module in `interpreter.rs`:

```rust
    #[test]
    fn test_node_simple_graph() {
        // Same logic as form: abs_move_pct > 3.0 AND pct_into_slot between [0.1, 0.6] → buy UP
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "input",      "data": { "field": "pct_into_slot" } },
                { "id": "n3", "type": "comparator", "data": { "operator": ">", "value": 0.5 } },
                { "id": "n4", "type": "comparator", "data": { "operator": "between", "value": [0.1, 0.6] } },
                { "id": "n5", "type": "logic",      "data": { "operator": "AND" } },
                { "id": "n6", "type": "action",     "data": { "signal": "buy", "outcome": "UP", "size_usdc": 50 } }
            ],
            "edges": [
                { "source": "n1", "target": "n3" },
                { "source": "n2", "target": "n4" },
                { "source": "n3", "target": "n5" },
                { "source": "n4", "target": "n5" },
                { "source": "n5", "target": "n6" }
            ]
        });
        let tick = test_tick(); // abs_move_pct=1.0>0.5, pct_into_slot=0.5 in [0.1,0.6]
        let mut state = StrategyState::new(100);
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(signal, Signal::Buy { .. }));
    }

    #[test]
    fn test_node_condition_fails() {
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 10.0 } },
                { "id": "n3", "type": "action",     "data": { "signal": "buy", "outcome": "UP", "size_usdc": 25 } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" }
            ]
        });
        let tick = test_tick(); // abs_move_pct=1.0 < 10.0
        let mut state = StrategyState::new(100);
        assert!(matches!(evaluate(&graph, &tick, &mut state), Signal::Hold));
    }

    #[test]
    fn test_node_with_indicator() {
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "indicator",  "data": { "fn": "EMA", "period": 3, "field": "mid_up" } },
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 0.5 } },
                { "id": "n3", "type": "action",     "data": { "signal": "buy", "outcome": "DOWN", "size_usdc": 30 } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" }
            ]
        });
        let mut state = StrategyState::new(100);
        for mid in [0.55, 0.58, 0.60] {
            let mut t = test_tick();
            t.mid_up = mid;
            state.push_tick(t);
        }
        let tick = test_tick();
        let signal = evaluate(&graph, &tick, &mut state);
        assert!(matches!(signal, Signal::Buy { outcome: Outcome::Down, .. }));
    }

    #[test]
    fn test_node_or_logic() {
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
                { "id": "n2", "type": "input",      "data": { "field": "pct_into_slot" } },
                { "id": "n3", "type": "comparator", "data": { "operator": ">", "value": 10.0 } },
                { "id": "n4", "type": "comparator", "data": { "operator": ">", "value": 0.3 } },
                { "id": "n5", "type": "logic",      "data": { "operator": "OR" } },
                { "id": "n6", "type": "action",     "data": { "signal": "buy", "outcome": "UP", "size_usdc": 20 } }
            ],
            "edges": [
                { "source": "n1", "target": "n3" },
                { "source": "n2", "target": "n4" },
                { "source": "n3", "target": "n5" },
                { "source": "n4", "target": "n5" },
                { "source": "n5", "target": "n6" }
            ]
        });
        let tick = test_tick(); // abs_move_pct=1.0<10 fails, pct_into_slot=0.5>0.3 passes → OR = true
        let mut state = StrategyState::new(100);
        assert!(matches!(evaluate(&graph, &tick, &mut state), Signal::Buy { .. }));
    }
```

### Step 2: Implement evaluate_node

Replace the `todo!()` in `evaluate_node`:

```rust
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
enum NodeValue {
    Number(f64),
    Bool(bool),
}

fn evaluate_node(graph: &Value, tick: &Tick, state: &mut StrategyState) -> Signal {
    let Some(nodes) = graph["nodes"].as_array() else {
        return Signal::Hold;
    };
    let Some(edges) = graph["edges"].as_array() else {
        return Signal::Hold;
    };

    // Build adjacency and in-degree for topological sort
    let node_ids: Vec<&str> = nodes.iter().filter_map(|n| n["id"].as_str()).collect();
    let mut in_degree: HashMap<&str, usize> = node_ids.iter().map(|&id| (id, 0)).collect();
    let mut adj: HashMap<&str, Vec<&str>> = node_ids.iter().map(|&id| (id, vec![])).collect();
    let mut inputs_for: HashMap<&str, Vec<&str>> = node_ids.iter().map(|&id| (id, vec![])).collect();

    for edge in edges {
        let Some(src) = edge["source"].as_str() else { continue };
        let Some(tgt) = edge["target"].as_str() else { continue };
        *in_degree.entry(tgt).or_insert(0) += 1;
        adj.entry(src).or_default().push(tgt);
        inputs_for.entry(tgt).or_default().push(src);
    }

    // Kahn's topological sort
    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();
    let mut order: Vec<&str> = Vec::new();
    while let Some(id) = queue.pop_front() {
        order.push(id);
        for &next in adj.get(id).unwrap_or(&vec![]) {
            let deg = in_degree.get_mut(next).unwrap();
            *deg -= 1;
            if *deg == 0 {
                queue.push_back(next);
            }
        }
    }

    // Index nodes by ID
    let node_map: HashMap<&str, &Value> = nodes
        .iter()
        .filter_map(|n| n["id"].as_str().map(|id| (id, n)))
        .collect();

    // Evaluate in topological order
    let mut values: HashMap<&str, NodeValue> = HashMap::new();

    for &id in &order {
        let Some(node) = node_map.get(id) else { continue };
        let node_type = node["type"].as_str().unwrap_or("");
        let data = &node["data"];

        let result = match node_type {
            "input" => {
                let field = data["field"].as_str().unwrap_or("");
                get_field(tick, field)
                    .map(NodeValue::Number)
                    .unwrap_or(NodeValue::Number(0.0))
            }
            "indicator" => {
                let indicator_val = resolve_indicator(data, tick, state);
                NodeValue::Number(indicator_val.unwrap_or(0.0))
            }
            "comparator" => {
                let input_ids = inputs_for.get(id).unwrap();
                let input_val = input_ids
                    .first()
                    .and_then(|&src_id| values.get(src_id))
                    .and_then(|v| match v {
                        NodeValue::Number(n) => Some(*n),
                        _ => None,
                    })
                    .unwrap_or(0.0);
                let op = data["operator"].as_str().unwrap_or("==");
                NodeValue::Bool(evaluate_op(input_val, op, &data["value"]))
            }
            "logic" => {
                let op = data["operator"].as_str().unwrap_or("AND");
                let input_ids = inputs_for.get(id).unwrap();
                let bools: Vec<bool> = input_ids
                    .iter()
                    .filter_map(|&src_id| values.get(src_id))
                    .map(|v| match v {
                        NodeValue::Bool(b) => *b,
                        NodeValue::Number(n) => *n != 0.0,
                    })
                    .collect();
                let result = match op {
                    "OR" => bools.iter().any(|&b| b),
                    _ => bools.iter().all(|&b| b),
                };
                NodeValue::Bool(result)
            }
            "action" => {
                let input_ids = inputs_for.get(id).unwrap();
                let triggered = input_ids
                    .iter()
                    .filter_map(|&src_id| values.get(src_id))
                    .all(|v| match v {
                        NodeValue::Bool(b) => *b,
                        NodeValue::Number(n) => *n != 0.0,
                    });
                if triggered {
                    return build_action_signal(data);
                }
                NodeValue::Bool(false)
            }
            _ => NodeValue::Number(0.0),
        };

        values.insert(id, result);
    }

    Signal::Hold
}
```

### Step 3: Run tests

```bash
cd engine && cargo test strategy::interpreter
```

Expected: all pass (form + node mode tests).

### Step 4: Commit

```bash
git add engine/src/strategy/interpreter.rs
git commit -m "feat: add JSON graph interpreter — node mode with topological sort"
```

---

## Task 8: Assignment registry

**Files:**
- Create: `engine/src/strategy/registry.rs`
- Modify: `engine/src/strategy/mod.rs` — register module

### Step 1: Write tests

Create `engine/src/strategy/registry.rs`:

```rust
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
    // Remove empty market entries
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
            &reg, 1, 100,
            serde_json::json!({"mode": "form"}),
            vec!["btc-updown-15m".into()],
            200.0, None,
        ).await;

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
            &reg, 1, 100,
            serde_json::json!({}),
            vec!["btc-15m".into(), "eth-15m".into()],
            100.0, None,
        ).await;

        let r = reg.read().await;
        assert!(r.contains_key("btc-15m"));
        assert!(r.contains_key("eth-15m"));
    }

    #[tokio::test]
    async fn test_deactivate() {
        let reg = new_registry();
        activate(&reg, 1, 100, serde_json::json!({}), vec!["btc".into()], 100.0, None).await;
        activate(&reg, 2, 200, serde_json::json!({}), vec!["btc".into()], 100.0, None).await;

        deactivate(&reg, 1, 100).await;

        let r = reg.read().await;
        let assignments = r.get("btc").unwrap();
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].wallet_id, 2);
    }

    #[tokio::test]
    async fn test_deactivate_removes_empty_entries() {
        let reg = new_registry();
        activate(&reg, 1, 100, serde_json::json!({}), vec!["btc".into()], 100.0, None).await;
        deactivate(&reg, 1, 100).await;

        let r = reg.read().await;
        assert!(!r.contains_key("btc"));
    }
}
```

### Step 2: Register module

In `engine/src/strategy/mod.rs`, add:

```rust
pub mod registry;
```

### Step 3: Run tests

```bash
cd engine && cargo test strategy::registry
```

Expected: all pass.

### Step 4: Commit

```bash
git add engine/src/strategy/registry.rs engine/src/strategy/mod.rs
git commit -m "feat: add AssignmentRegistry for wallet/strategy pair management"
```

---

## Task 9: Engine dispatch loop

Kafka consumer → registry lookup → Rayon parallel dispatch → signal output.

**Files:**
- Create: `engine/src/strategy/engine.rs`
- Modify: `engine/src/strategy/mod.rs` — register module

### Step 1: Implement engine loop

Create `engine/src/strategy/engine.rs`:

```rust
use anyhow::Result;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::Message;
use rayon::prelude::*;
use tokio::sync::mpsc;

use super::interpreter;
use super::registry::AssignmentRegistry;
use super::{EngineOutput, Signal};
use crate::fetcher::models::Tick;
use crate::kafka;

pub async fn run(
    brokers: &str,
    registry: AssignmentRegistry,
    signal_tx: mpsc::Sender<EngineOutput>,
) -> Result<()> {
    let consumer = kafka::consumer::create_consumer(brokers, "strategy-engine")?;
    tracing::info!("strategy_engine_started");

    loop {
        let message = match consumer.recv().await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(error = %e, "kafka_recv_error");
                continue;
            }
        };

        let Some(payload) = message.payload() else {
            continue;
        };
        let Ok(payload_str) = std::str::from_utf8(payload) else {
            continue;
        };
        let tick: Tick = match serde_json::from_str(payload_str) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(error = %e, "tick_deserialize_failed");
                continue;
            }
        };

        // Read lock → clone assignments for this symbol → release lock
        let assignments = {
            let reg = registry.read().await;
            reg.get(&tick.symbol).cloned().unwrap_or_default()
        };

        if assignments.is_empty() {
            continue;
        }

        // Rayon parallel dispatch
        let signals: Vec<EngineOutput> = assignments
            .par_iter()
            .filter_map(|a| {
                let mut state = a.state.lock().unwrap();
                let signal = interpreter::evaluate(&a.graph, &tick, &mut state);
                match signal {
                    Signal::Hold => None,
                    s => Some(EngineOutput {
                        wallet_id: a.wallet_id,
                        strategy_id: a.strategy_id,
                        symbol: tick.symbol.clone(),
                        signal: s,
                    }),
                }
            })
            .collect();

        for output in signals {
            tracing::info!(
                wallet_id = output.wallet_id,
                strategy_id = output.strategy_id,
                symbol = %output.symbol,
                signal = ?output.signal,
                "strategy_signal"
            );
            if signal_tx.send(output).await.is_err() {
                tracing::info!("signal_channel_closed");
                return Ok(());
            }
        }
    }
}
```

### Step 2: Register module

In `engine/src/strategy/mod.rs`, add:

```rust
pub mod engine;
```

### Step 3: Verify compilation

```bash
cd engine && cargo build 2>&1
```

### Step 4: Commit

```bash
git add engine/src/strategy/engine.rs engine/src/strategy/mod.rs
git commit -m "feat: add strategy engine dispatch loop with Kafka consumer and Rayon"
```

---

## Task 10: Redis state persistence

**Files:**
- Create: `engine/src/storage/redis.rs`
- Modify: `engine/src/storage/mod.rs` — register module
- Modify: `engine/src/config.rs` — add `redis_url` field

### Step 1: Add redis_url to Config

In `engine/src/config.rs`, add field to `Config` struct:

```rust
pub struct Config {
    // ... existing fields ...
    pub redis_url: String,
}
```

And in `from_env()`:

```rust
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
```

### Step 2: Implement Redis state persistence

Create `engine/src/storage/redis.rs`:

```rust
use anyhow::Result;
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
    for assignments in reg.values() {
        for a in assignments {
            let state = a.state.lock().unwrap();
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

pub async fn run_state_persister(
    redis_url: &str,
    registry: AssignmentRegistry,
) -> Result<()> {
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
```

### Step 3: Register module

In `engine/src/storage/mod.rs`:

```rust
pub mod clickhouse;
pub mod redis;
```

### Step 4: Write serialization test

Add to `engine/src/storage/redis.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
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
```

### Step 5: Run tests

```bash
cd engine && cargo test storage::redis
```

### Step 6: Commit

```bash
git add engine/src/storage/redis.rs engine/src/storage/mod.rs engine/src/config.rs
git commit -m "feat: add Redis state persistence for strategy state"
```

---

## Task 11: Wire into main.rs + smoke test

**Files:**
- Modify: `engine/src/main.rs` — spawn strategy engine + Redis persister

### Step 1: Add strategy engine tasks to main.rs

Add after the existing task spawns (after task 6 — Kafka publisher):

```rust
    // 7. Strategy engine — Kafka consumer → evaluate → signal output
    let engine_registry = strategy::registry::new_registry();
    let (signal_tx, mut signal_rx) = tokio::sync::mpsc::channel::<strategy::EngineOutput>(256);

    let eng_brokers = cfg.kafka_brokers.clone();
    let eng_registry = engine_registry.clone();
    tasks.spawn(async move {
        strategy::engine::run(&eng_brokers, eng_registry, signal_tx).await
    });

    // 8. Signal logger (placeholder — replaced by execution queue in Phase 4)
    tasks.spawn(async move {
        while let Some(output) = signal_rx.recv().await {
            tracing::info!(
                wallet_id = output.wallet_id,
                strategy_id = output.strategy_id,
                symbol = %output.symbol,
                signal = ?output.signal,
                "engine_signal_output"
            );
        }
        Ok(())
    });

    // 9. Redis state persister
    let redis_registry = engine_registry.clone();
    let redis_url = cfg.redis_url.clone();
    tasks.spawn(async move {
        storage::redis::run_state_persister(&redis_url, redis_registry).await
    });
```

### Step 2: Add test assignment for smoke testing

Add a temporary test assignment after registry creation (remove once Phase 6 internal API exists):

```rust
    // TEMPORARY: activate a test assignment for smoke testing
    // Remove when Phase 6 internal API is implemented
    strategy::registry::activate(
        &engine_registry,
        0, // test wallet_id
        0, // test strategy_id
        serde_json::json!({
            "mode": "form",
            "conditions": [{
                "type": "AND",
                "rules": [
                    { "indicator": "abs_move_pct", "operator": ">", "value": 2.0 },
                    { "indicator": "pct_into_slot", "operator": "between", "value": [0.1, 0.5] }
                ]
            }],
            "action": { "signal": "buy", "outcome": "UP", "size_usdc": 10, "order_type": "market" },
            "risk": { "stoploss_pct": 30, "take_profit_pct": 80, "max_trades_per_slot": 1 }
        }),
        vec![], // empty = subscribe to all symbols (engine will match against registry keys)
        100.0,
        None,
    ).await;
```

**Note:** The test assignment uses empty `markets` vec which won't match any symbols. To truly smoke-test, you'd need to know an active symbol (e.g., `"btc-updown-15m-XXXXXXXXXX"`). This can be set after observing which symbols the fetcher discovers. A better approach: subscribe to a wildcard or use a catch-all. For now, the empty assignment validates compilation and startup — real testing happens when Phase 6 provides proper activation.

### Step 3: Verify full compilation

```bash
cd engine && cargo build 2>&1
```

Expected: compiles cleanly.

### Step 4: Run all tests

```bash
cd engine && cargo test 2>&1
```

Expected: all strategy tests pass.

### Step 5: Commit

```bash
git add engine/src/main.rs
git commit -m "feat: wire strategy engine, signal logger, and Redis persister into main loop"
```

---

## Summary

| Task | Description | Key files | Tests |
|------|-------------|-----------|-------|
| 1 | `ref_price_source` field | `models.rs`, `tick_builder.rs`, `init.sql` | existing tick_builder tests |
| 2 | Kafka consumer + Tick Deserialize | `consumer.rs`, `models.rs` | compilation only |
| 3 | Core types (Signal, State, Position) | `strategy/mod.rs`, `state.rs` | serialization roundtrip |
| 4 | Field accessor + comparators | `strategy/eval.rs` | 6 tests |
| 5 | Indicators (EMA/SMA/RSI/VWAP/cross) | `strategy/indicators.rs` | 10 tests |
| 6 | Form mode interpreter | `strategy/interpreter.rs` | 8 tests |
| 7 | Node mode interpreter | `strategy/interpreter.rs` | 4 tests |
| 8 | Assignment registry | `strategy/registry.rs` | 4 tests |
| 9 | Engine dispatch loop | `strategy/engine.rs` | compilation only |
| 10 | Redis state persistence | `storage/redis.rs` | serialization test |
| 11 | Wire into main.rs | `main.rs` | full `cargo test` |

**Final directory structure added:**
```
engine/src/
├── strategy/
│   ├── mod.rs           # Signal, Outcome, OrderType, EngineOutput
│   ├── state.rs         # StrategyState, Position
│   ├── eval.rs          # get_field(), evaluate_op()
│   ├── indicators.rs    # ema, sma, rsi, vwap, cross_above, cross_below
│   ├── interpreter.rs   # evaluate() — form + node mode dispatch
│   ├── registry.rs      # AssignmentRegistry, Assignment, activate/deactivate
│   └── engine.rs        # run() — Kafka consumer → Rayon dispatch → signal output
├── kafka/
│   ├── consumer.rs      # NEW
│   └── producer.rs
└── storage/
    ├── clickhouse.rs
    └── redis.rs         # NEW
```
