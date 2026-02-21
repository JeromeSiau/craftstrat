# Phase 5 — Backtest Engine Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add backtesting to the Rust engine — replay historical ClickHouse ticks through the existing `evaluate()` pipeline with realistic fill simulation, producing performance metrics.

**Architecture:** No new abstraction layer — `evaluate(graph, tick, state)` is already tick-source agnostic. A `BacktestEngine` struct processes ticks one-by-one (testable with `Vec<Tick>`, production uses ClickHouse async cursor). Multi-market support via `HashMap<String, MarketContext>` keyed by symbol. Metrics computed from collected trades at the end.

**Tech Stack:** Rust, `clickhouse` crate (async cursor for reading), existing `strategy::interpreter::evaluate()`, existing `Tick` struct.

---

## Key Design Decisions

- **No Strategy trait** — `evaluate()` is the stable contract. Trait deferred indefinitely.
- **Streaming cursor** — ClickHouse rows streamed via async cursor, constant memory.
- **Realistic fills** — Buy at ask, sell at bid. Spread cost naturally included.
- **Force-close at end** — Open positions closed at last mid price, `exit_reason = "end_of_data"`.
- **Per-market state** — Each symbol gets its own `StrategyState`. Matches live engine behavior.

## Important Context

**How `evaluate()` manages position** (see `engine/src/strategy/interpreter/mod.rs:19-54`):
- When `state.position` is `Some`: checks risk (stoploss/take_profit). If triggered → clears `state.position`, returns `Sell`. Otherwise → returns `Hold`.
- When `state.position` is `None`: evaluates conditions, may return `Buy`.
- `evaluate()` does **NOT** set `state.position` on Buy — that's the caller's job (execution pipeline in live, runner in backtest).

**Fill price mapping:**

| Action | Outcome::Up | Outcome::Down |
|--------|------------|---------------|
| Buy entry | `tick.ask_up` | `tick.ask_down` |
| Sell exit | `tick.bid_up` | `tick.bid_down` |
| End-of-data close | `tick.mid_up` | `tick.mid_down` |

---

## Task 1: Backtest Types

**Files:**
- Create: `engine/src/backtest/mod.rs`
- Modify: `engine/src/main.rs:1-8` (add `mod backtest;`)

### Step 1: Create `engine/src/backtest/mod.rs` with types

```rust
pub mod metrics;
pub mod runner;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

use crate::strategy::Outcome;

#[derive(Debug, Clone)]
pub struct BacktestRequest {
    pub strategy_graph: Value,
    pub market_filter: Vec<String>,
    pub date_from: OffsetDateTime,
    pub date_to: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub total_trades: u32,
    pub win_rate: f64,
    pub total_pnl_usdc: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub trades: Vec<BacktestTrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestTrade {
    pub market_id: String,
    pub outcome: Outcome,
    pub side: String,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub size_usdc: f64,
    pub pnl_usdc: f64,
    #[serde(with = "time::serde::rfc3339")]
    pub entry_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub exit_at: Option<OffsetDateTime>,
    pub exit_reason: Option<String>,
}
```

### Step 2: Create empty `engine/src/backtest/metrics.rs` and `engine/src/backtest/runner.rs`

```rust
// metrics.rs — placeholder
use super::BacktestTrade;
```

```rust
// runner.rs — placeholder
use super::BacktestTrade;
```

### Step 3: Add `mod backtest;` to `engine/src/main.rs`

Add `mod backtest;` after the existing module declarations (line 1-8).

### Step 4: Verify compilation

Run: `cargo check` in `engine/`
Expected: compiles with no errors (warnings OK)

### Step 5: Write serialization test in `engine/src/backtest/mod.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::Outcome;

    #[test]
    fn test_backtest_result_serialization_roundtrip() {
        let result = BacktestResult {
            total_trades: 5,
            win_rate: 0.6,
            total_pnl_usdc: 42.5,
            max_drawdown: 0.15,
            sharpe_ratio: 1.2,
            trades: vec![BacktestTrade {
                market_id: "btc-updown-15m-1700000000".into(),
                outcome: Outcome::Up,
                side: "buy".into(),
                entry_price: 0.62,
                exit_price: Some(0.68),
                size_usdc: 50.0,
                pnl_usdc: 4.84,
                entry_at: OffsetDateTime::from_unix_timestamp(1700000450).unwrap(),
                exit_at: Some(OffsetDateTime::from_unix_timestamp(1700000900).unwrap()),
                exit_reason: Some("take_profit".into()),
            }],
        };
        let json = serde_json::to_string(&result).unwrap();
        let restored: BacktestResult = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.total_trades, 5);
        assert!((restored.win_rate - 0.6).abs() < f64::EPSILON);
        assert_eq!(restored.trades.len(), 1);
        assert_eq!(restored.trades[0].outcome, Outcome::Up);
    }
}
```

### Step 6: Run test

Run: `cargo test -p oddex-engine test_backtest_result_serialization_roundtrip`
Expected: PASS

### Step 7: Commit

```bash
git add engine/src/backtest/
git add engine/src/main.rs
git commit -m "feat(backtest): add backtest types — BacktestRequest, BacktestResult, BacktestTrade"
```

---

## Task 2: Metrics Computation

**Files:**
- Modify: `engine/src/backtest/metrics.rs`

### Step 1: Write failing tests

```rust
// engine/src/backtest/metrics.rs

use super::{BacktestResult, BacktestTrade};

pub fn compute(trades: Vec<BacktestTrade>) -> BacktestResult {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::Outcome;
    use time::OffsetDateTime;

    fn trade(pnl: f64, size: f64) -> BacktestTrade {
        BacktestTrade {
            market_id: "test".into(),
            outcome: Outcome::Up,
            side: "buy".into(),
            entry_price: 0.50,
            exit_price: Some(if pnl >= 0.0 { 0.50 + pnl / size * 0.50 } else { 0.50 + pnl / size * 0.50 }),
            size_usdc: size,
            pnl_usdc: pnl,
            entry_at: OffsetDateTime::from_unix_timestamp(1700000000).unwrap(),
            exit_at: Some(OffsetDateTime::from_unix_timestamp(1700000900).unwrap()),
            exit_reason: Some("signal".into()),
        }
    }

    #[test]
    fn test_compute_empty_trades() {
        let result = compute(vec![]);
        assert_eq!(result.total_trades, 0);
        assert!((result.win_rate).abs() < f64::EPSILON);
        assert!((result.total_pnl_usdc).abs() < f64::EPSILON);
        assert!((result.max_drawdown).abs() < f64::EPSILON);
        assert!((result.sharpe_ratio).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_single_win() {
        let result = compute(vec![trade(10.0, 100.0)]);
        assert_eq!(result.total_trades, 1);
        assert!((result.win_rate - 1.0).abs() < f64::EPSILON);
        assert!((result.total_pnl_usdc - 10.0).abs() < f64::EPSILON);
        assert!((result.max_drawdown).abs() < f64::EPSILON); // no drawdown with single win
    }

    #[test]
    fn test_compute_win_loss_win() {
        let result = compute(vec![
            trade(10.0, 100.0),
            trade(-5.0, 100.0),
            trade(8.0, 100.0),
        ]);
        assert_eq!(result.total_trades, 3);
        assert!((result.win_rate - 2.0 / 3.0).abs() < 0.001);
        assert!((result.total_pnl_usdc - 13.0).abs() < f64::EPSILON);
        // Equity curve: [0, 10, 5, 13] → peak 10, trough 5, drawdown = 5/10 = 0.5
        assert!((result.max_drawdown - 0.5).abs() < 0.001);
        assert!(result.sharpe_ratio > 0.0); // positive overall
    }

    #[test]
    fn test_compute_all_losses() {
        let result = compute(vec![
            trade(-10.0, 100.0),
            trade(-5.0, 100.0),
        ]);
        assert_eq!(result.total_trades, 2);
        assert!((result.win_rate).abs() < f64::EPSILON);
        assert!((result.total_pnl_usdc - (-15.0)).abs() < f64::EPSILON);
        assert!(result.sharpe_ratio < 0.0); // negative sharpe
    }

    #[test]
    fn test_compute_ignores_unclosed_trades() {
        let mut unclosed = trade(0.0, 100.0);
        unclosed.exit_price = None;
        unclosed.exit_at = None;
        unclosed.exit_reason = None;

        let result = compute(vec![
            trade(10.0, 100.0),
            unclosed,
        ]);
        // Only closed trade counts
        assert_eq!(result.total_trades, 1);
        assert!((result.total_pnl_usdc - 10.0).abs() < f64::EPSILON);
    }
}
```

### Step 2: Verify tests fail

Run: `cargo test -p oddex-engine backtest::metrics`
Expected: FAIL with `not yet implemented`

### Step 3: Implement `compute()`, `compute_max_drawdown()`, `compute_sharpe()`

```rust
use super::{BacktestResult, BacktestTrade};

pub fn compute(trades: Vec<BacktestTrade>) -> BacktestResult {
    let closed: Vec<&BacktestTrade> = trades.iter().filter(|t| t.exit_price.is_some()).collect();
    let total_trades = closed.len() as u32;

    if total_trades == 0 {
        return BacktestResult {
            total_trades: 0,
            win_rate: 0.0,
            total_pnl_usdc: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            trades,
        };
    }

    let wins = closed.iter().filter(|t| t.pnl_usdc > 0.0).count();
    let win_rate = wins as f64 / total_trades as f64;
    let total_pnl_usdc: f64 = closed.iter().map(|t| t.pnl_usdc).sum();
    let max_drawdown = compute_max_drawdown(&closed);
    let sharpe_ratio = compute_sharpe(&closed);

    BacktestResult {
        total_trades,
        win_rate,
        total_pnl_usdc,
        max_drawdown,
        sharpe_ratio,
        trades,
    }
}

fn compute_max_drawdown(trades: &[&BacktestTrade]) -> f64 {
    let mut peak = 0.0_f64;
    let mut equity = 0.0_f64;
    let mut max_dd = 0.0_f64;

    for trade in trades {
        equity += trade.pnl_usdc;
        if equity > peak {
            peak = equity;
        }
        if peak > 0.0 {
            let dd = (peak - equity) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }
    max_dd
}

fn compute_sharpe(trades: &[&BacktestTrade]) -> f64 {
    if trades.len() < 2 {
        return 0.0;
    }
    let pnls: Vec<f64> = trades.iter().map(|t| t.pnl_usdc).collect();
    let n = pnls.len() as f64;
    let mean = pnls.iter().sum::<f64>() / n;
    let variance = pnls.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();
    if std_dev < f64::EPSILON {
        return 0.0;
    }
    mean / std_dev
}
```

### Step 4: Run tests

Run: `cargo test -p oddex-engine backtest::metrics`
Expected: all 5 tests PASS

### Step 5: Commit

```bash
git add engine/src/backtest/metrics.rs
git commit -m "feat(backtest): add metrics computation — win_rate, max_drawdown, sharpe_ratio"
```

---

## Task 3: BacktestEngine Core

**Files:**
- Modify: `engine/src/backtest/runner.rs`

**Context:**
- `evaluate()` at `engine/src/strategy/interpreter/mod.rs:19` — does NOT set `state.position` on Buy, only clears it on risk Sell.
- The runner must set `state.position` when a Buy signal is produced so `evaluate()` knows we're in a position on next tick.
- `test_tick()` at `engine/src/strategy/test_utils.rs:3` — creates default tick with `ask_up=0.62`, `bid_up=0.60`, `mid_up=0.61`.

### Step 1: Write failing test — buy + stoploss exit

```rust
// engine/src/backtest/runner.rs

use std::collections::HashMap;
use serde_json::Value;
use time::OffsetDateTime;

use super::{BacktestResult, BacktestTrade};
use crate::fetcher::models::Tick;
use crate::strategy::interpreter::evaluate;
use crate::strategy::state::{Position, StrategyState};
use crate::strategy::{Outcome, Signal};

pub struct BacktestEngine {
    graph: Value,
    markets: HashMap<String, MarketContext>,
    trades: Vec<BacktestTrade>,
}

struct MarketContext {
    state: StrategyState,
    open_trade: Option<BacktestTrade>,
}

impl BacktestEngine {
    pub fn new(graph: Value) -> Self {
        todo!()
    }

    pub fn process_tick(&mut self, tick: &Tick) {
        todo!()
    }

    pub fn finish(mut self) -> BacktestResult {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::test_utils::test_tick;

    fn simple_buy_up_strategy() -> Value {
        serde_json::json!({
            "mode": "form",
            "conditions": [{
                "type": "AND",
                "rules": [{
                    "indicator": "abs_move_pct",
                    "operator": ">",
                    "value": 3.0
                }]
            }],
            "action": {
                "signal": "buy",
                "outcome": "UP",
                "size_mode": "fixed",
                "size_usdc": 50,
                "order_type": "market"
            },
            "risk": {
                "stoploss_pct": 10,
                "take_profit_pct": 15,
                "max_trades_per_slot": 2
            }
        })
    }

    #[test]
    fn test_buy_then_stoploss_exit() {
        let graph = simple_buy_up_strategy();
        let mut engine = BacktestEngine::new(graph);

        // Tick 1: abs_move_pct = 1.0 → no signal (below threshold)
        let mut t1 = test_tick();
        t1.abs_move_pct = 1.0;
        engine.process_tick(&t1);

        // Tick 2: abs_move_pct = 4.0 → Buy signal (above 3.0)
        let mut t2 = test_tick();
        t2.abs_move_pct = 4.0;
        t2.captured_at = OffsetDateTime::from_unix_timestamp(1700000451).unwrap();
        engine.process_tick(&t2);

        // Should have opened a position — verify via internal state
        let ctx = engine.markets.get("btc-updown-15m-1700000000").unwrap();
        assert!(ctx.open_trade.is_some());
        assert!(ctx.state.position.is_some());

        // Tick 3: mid_up drops to 0.54 → stoploss triggers (PnL = (0.54-0.62)/0.62*100 = -12.9%)
        let mut t3 = test_tick();
        t3.abs_move_pct = 1.0;
        t3.mid_up = 0.54;
        t3.bid_up = 0.53;
        t3.captured_at = OffsetDateTime::from_unix_timestamp(1700000452).unwrap();
        engine.process_tick(&t3);

        // Position should be closed
        let ctx = engine.markets.get("btc-updown-15m-1700000000").unwrap();
        assert!(ctx.open_trade.is_none());
        assert!(ctx.state.position.is_none());
        assert_eq!(engine.trades.len(), 1);

        let trade = &engine.trades[0];
        assert!((trade.entry_price - 0.62).abs() < 0.001); // ask_up
        assert!((trade.exit_price.unwrap() - 0.53).abs() < 0.001); // bid_up
        assert!(trade.pnl_usdc < 0.0); // loss
        assert_eq!(trade.exit_reason.as_deref(), Some("stoploss"));
    }
}
```

### Step 2: Verify test fails

Run: `cargo test -p oddex-engine backtest::runner::tests::test_buy_then_stoploss_exit`
Expected: FAIL with `not yet implemented`

### Step 3: Implement BacktestEngine

```rust
use std::collections::HashMap;
use serde_json::Value;
use time::OffsetDateTime;

use super::metrics;
use super::{BacktestResult, BacktestTrade};
use crate::fetcher::models::Tick;
use crate::strategy::interpreter::evaluate;
use crate::strategy::state::{Position, StrategyState};
use crate::strategy::{Outcome, OrderType, Signal};

pub struct BacktestEngine {
    graph: Value,
    markets: HashMap<String, MarketContext>,
    trades: Vec<BacktestTrade>,
}

struct MarketContext {
    state: StrategyState,
    open_trade: Option<BacktestTrade>,
}

impl BacktestEngine {
    pub fn new(graph: Value) -> Self {
        Self {
            graph,
            markets: HashMap::new(),
            trades: Vec::new(),
        }
    }

    pub fn process_tick(&mut self, tick: &Tick) {
        let ctx = self
            .markets
            .entry(tick.symbol.clone())
            .or_insert_with(|| MarketContext {
                state: StrategyState::new(200),
                open_trade: None,
            });

        let signal = evaluate(&self.graph, tick, &mut ctx.state);

        match signal {
            Signal::Buy {
                outcome,
                size_usdc,
                ..
            } => {
                let entry_price = ask_price(outcome, tick);
                ctx.state.position = Some(Position {
                    outcome,
                    entry_price,
                    size_usdc,
                    entry_at: tick.captured_at.unix_timestamp(),
                });
                ctx.open_trade = Some(BacktestTrade {
                    market_id: tick.symbol.clone(),
                    outcome,
                    side: "buy".into(),
                    entry_price,
                    exit_price: None,
                    size_usdc,
                    pnl_usdc: 0.0,
                    entry_at: tick.captured_at,
                    exit_at: None,
                    exit_reason: None,
                });
            }
            Signal::Sell {
                outcome,
                order_type,
                ..
            } => {
                // evaluate() already cleared state.position
                if let Some(mut trade) = ctx.open_trade.take() {
                    let exit = bid_price(outcome, tick);
                    trade.exit_price = Some(exit);
                    trade.pnl_usdc =
                        (exit - trade.entry_price) / trade.entry_price * trade.size_usdc;
                    trade.exit_at = Some(tick.captured_at);
                    trade.exit_reason = Some(exit_reason(&order_type));
                    self.trades.push(trade);
                }
            }
            Signal::Hold => {}
        }
    }

    pub fn finish(mut self) -> BacktestResult {
        // Force-close any open positions at last known mid price
        for (_, ctx) in self.markets.drain() {
            if let Some(mut trade) = ctx.open_trade {
                if let Some(last_tick) = ctx.state.window.back() {
                    let exit = mid_price(trade.outcome, last_tick);
                    trade.exit_price = Some(exit);
                    trade.pnl_usdc =
                        (exit - trade.entry_price) / trade.entry_price * trade.size_usdc;
                    trade.exit_at = Some(last_tick.captured_at);
                    trade.exit_reason = Some("end_of_data".into());
                    self.trades.push(trade);
                }
            }
        }
        metrics::compute(self.trades)
    }
}

fn ask_price(outcome: Outcome, tick: &Tick) -> f64 {
    match outcome {
        Outcome::Up => tick.ask_up as f64,
        Outcome::Down => tick.ask_down as f64,
    }
}

fn bid_price(outcome: Outcome, tick: &Tick) -> f64 {
    match outcome {
        Outcome::Up => tick.bid_up as f64,
        Outcome::Down => tick.bid_down as f64,
    }
}

fn mid_price(outcome: Outcome, tick: &Tick) -> f64 {
    match outcome {
        Outcome::Up => tick.mid_up as f64,
        Outcome::Down => tick.mid_down as f64,
    }
}

fn exit_reason(order_type: &OrderType) -> String {
    match order_type {
        OrderType::StopLoss { .. } => "stoploss".into(),
        OrderType::TakeProfit { .. } => "take_profit".into(),
        _ => "signal".into(),
    }
}
```

### Step 4: Run stoploss test

Run: `cargo test -p oddex-engine backtest::runner::tests::test_buy_then_stoploss_exit`
Expected: PASS

### Step 5: Write test — take_profit exit

```rust
    #[test]
    fn test_buy_then_take_profit_exit() {
        let graph = simple_buy_up_strategy(); // take_profit_pct = 15
        let mut engine = BacktestEngine::new(graph);

        // Tick 1: triggers buy (abs_move_pct > 3.0)
        let mut t1 = test_tick();
        t1.abs_move_pct = 4.0;
        t1.ask_up = 0.50; // entry price
        t1.mid_up = 0.49;
        engine.process_tick(&t1);

        // Tick 2: mid_up rises enough → take profit
        // PnL% = (0.58 - 0.50) / 0.50 * 100 = 16% > 15% take_profit
        let mut t2 = test_tick();
        t2.abs_move_pct = 1.0;
        t2.mid_up = 0.58;
        t2.bid_up = 0.57;
        t2.captured_at = OffsetDateTime::from_unix_timestamp(1700000451).unwrap();
        engine.process_tick(&t2);

        assert_eq!(engine.trades.len(), 1);
        let trade = &engine.trades[0];
        assert!((trade.entry_price - 0.50).abs() < 0.001);
        assert!((trade.exit_price.unwrap() - 0.57).abs() < 0.001);
        assert!(trade.pnl_usdc > 0.0);
        assert_eq!(trade.exit_reason.as_deref(), Some("take_profit"));
    }
```

### Step 6: Run take_profit test

Run: `cargo test -p oddex-engine backtest::runner::tests::test_buy_then_take_profit_exit`
Expected: PASS (logic already implemented)

### Step 7: Write test — force-close at end of data

```rust
    #[test]
    fn test_force_close_at_end_of_data() {
        let graph = simple_buy_up_strategy();
        let mut engine = BacktestEngine::new(graph);

        // Tick 1: triggers buy
        let mut t1 = test_tick();
        t1.abs_move_pct = 4.0;
        engine.process_tick(&t1);

        // Tick 2: hold (still in position, no risk trigger)
        let mut t2 = test_tick();
        t2.abs_move_pct = 1.0;
        t2.mid_up = 0.63; // slight profit, within risk bounds
        t2.bid_up = 0.62;
        t2.captured_at = OffsetDateTime::from_unix_timestamp(1700000451).unwrap();
        engine.process_tick(&t2);

        // finish() should force-close
        let result = engine.finish();
        assert_eq!(result.total_trades, 1);
        let trade = &result.trades[0];
        assert!(trade.exit_price.is_some());
        assert_eq!(trade.exit_reason.as_deref(), Some("end_of_data"));
        // Exit at mid_up of last tick (0.63)
        assert!((trade.exit_price.unwrap() - 0.63).abs() < 0.001);
    }
```

### Step 8: Run force-close test

Run: `cargo test -p oddex-engine backtest::runner::tests::test_force_close_at_end_of_data`
Expected: PASS

### Step 9: Write test — multi-market isolation

```rust
    #[test]
    fn test_multi_market_separate_state() {
        let graph = simple_buy_up_strategy();
        let mut engine = BacktestEngine::new(graph);

        // Buy on market A
        let mut t1 = test_tick();
        t1.symbol = "btc-15m-AAA".into();
        t1.abs_move_pct = 4.0;
        t1.ask_up = 0.50;
        t1.mid_up = 0.49;
        engine.process_tick(&t1);

        // Buy on market B (separate state, should also trigger)
        let mut t2 = test_tick();
        t2.symbol = "eth-15m-BBB".into();
        t2.abs_move_pct = 4.0;
        t2.ask_up = 0.60;
        t2.mid_up = 0.59;
        t2.captured_at = OffsetDateTime::from_unix_timestamp(1700000451).unwrap();
        engine.process_tick(&t2);

        assert_eq!(engine.markets.len(), 2);
        assert!(engine.markets["btc-15m-AAA"].open_trade.is_some());
        assert!(engine.markets["eth-15m-BBB"].open_trade.is_some());

        // Both should force-close at end
        let result = engine.finish();
        assert_eq!(result.total_trades, 2);
    }
```

### Step 10: Run all runner tests

Run: `cargo test -p oddex-engine backtest::runner`
Expected: all 4 tests PASS

### Step 11: Commit

```bash
git add engine/src/backtest/runner.rs
git commit -m "feat(backtest): add BacktestEngine — tick-by-tick processing with position tracking"
```

---

## Task 4: ClickHouse Tick Reader + Async Entry Point

**Files:**
- Modify: `engine/src/storage/clickhouse.rs` (add `fetch_ticks` function)
- Modify: `engine/src/backtest/runner.rs` (add async `run` function)

**Context:** The `clickhouse` crate at `engine/Cargo.toml:11` already has the `time` feature for `OffsetDateTime` support. Querying uses `client.query(sql).bind(val).fetch::<Row>()` which returns an async `RowCursor`.

### Step 1: Add `fetch_ticks` to `engine/src/storage/clickhouse.rs`

Add after the existing `run_writer` function:

```rust
use clickhouse::RowCursor;

pub fn fetch_ticks(
    client: &Client,
    symbols: &[String],
    date_from: time::OffsetDateTime,
    date_to: time::OffsetDateTime,
) -> Result<RowCursor<Tick>> {
    let placeholders: Vec<&str> = symbols.iter().map(|_| "?").collect();
    let sql = format!(
        "SELECT ?fields FROM slot_snapshots WHERE symbol IN ({}) AND captured_at >= ? AND captured_at <= ? ORDER BY captured_at ASC",
        placeholders.join(", ")
    );
    let mut query = client.query(&sql);
    for s in symbols {
        query = query.bind(s.as_str());
    }
    query = query.bind(date_from).bind(date_to);
    Ok(query.fetch::<Tick>()?)
}
```

### Step 2: Add async `run` to `engine/src/backtest/runner.rs`

Add at the bottom of runner.rs (before the `#[cfg(test)]` block):

```rust
use clickhouse::Client;
use super::BacktestRequest;

pub async fn run(req: &BacktestRequest, ch_client: &Client) -> anyhow::Result<BacktestResult> {
    let mut cursor = crate::storage::clickhouse::fetch_ticks(
        ch_client,
        &req.market_filter,
        req.date_from,
        req.date_to,
    )?;

    let mut engine = BacktestEngine::new(req.strategy_graph.clone());

    while let Some(tick) = cursor.next().await? {
        engine.process_tick(&tick);
    }

    Ok(engine.finish())
}
```

### Step 3: Verify compilation

Run: `cargo check -p oddex-engine`
Expected: compiles (may have warnings about unused imports, that's fine)

### Step 4: Commit

```bash
git add engine/src/storage/clickhouse.rs engine/src/backtest/runner.rs
git commit -m "feat(backtest): add ClickHouse tick reader + async run() entry point"
```

---

## Task 5: Full Integration Test

**Files:**
- Modify: `engine/src/backtest/runner.rs` (add integration test)

### Step 1: Write integration test — full strategy lifecycle

This test verifies the entire pipeline: multiple ticks, buy entry, risk exit, second entry, take_profit exit, metrics computation.

```rust
    #[test]
    fn test_full_backtest_lifecycle() {
        let graph = simple_buy_up_strategy(); // stoploss=10, take_profit=15, max_trades=2
        let mut engine = BacktestEngine::new(graph);

        // --- Trade 1: Buy → Stoploss ---

        // Tick 1: triggers buy at ask_up = 0.62
        let mut t1 = test_tick();
        t1.abs_move_pct = 4.0;
        engine.process_tick(&t1);

        // Tick 2: stoploss triggers (mid_up=0.54 → PnL=-12.9%)
        let mut t2 = test_tick();
        t2.abs_move_pct = 1.0;
        t2.mid_up = 0.54;
        t2.bid_up = 0.53;
        t2.captured_at = OffsetDateTime::from_unix_timestamp(1700000451).unwrap();
        engine.process_tick(&t2);

        // --- Trade 2: Buy → Take Profit ---

        // Tick 3: triggers buy at ask_up = 0.50
        let mut t3 = test_tick();
        t3.abs_move_pct = 4.0;
        t3.ask_up = 0.50;
        t3.mid_up = 0.49;
        t3.captured_at = OffsetDateTime::from_unix_timestamp(1700000452).unwrap();
        engine.process_tick(&t3);

        // Tick 4: take profit triggers (mid_up=0.58 → PnL=+16%)
        let mut t4 = test_tick();
        t4.abs_move_pct = 1.0;
        t4.mid_up = 0.58;
        t4.bid_up = 0.57;
        t4.captured_at = OffsetDateTime::from_unix_timestamp(1700000453).unwrap();
        engine.process_tick(&t4);

        // --- Verify results ---
        let result = engine.finish();

        assert_eq!(result.total_trades, 2);
        // Trade 1: loss, Trade 2: win → win_rate = 0.5
        assert!((result.win_rate - 0.5).abs() < 0.001);
        // Trade 1 PnL: (0.53 - 0.62) / 0.62 * 50 = -7.26
        // Trade 2 PnL: (0.57 - 0.50) / 0.50 * 50 = 7.0
        // Total: ~ -0.26
        assert!(result.total_pnl_usdc < 0.0); // slight net loss
        assert!(result.max_drawdown > 0.0);    // had a drawdown after trade 1
        assert_eq!(result.trades[0].exit_reason.as_deref(), Some("stoploss"));
        assert_eq!(result.trades[1].exit_reason.as_deref(), Some("take_profit"));
    }
```

### Step 2: Run all backtest tests

Run: `cargo test -p oddex-engine backtest`
Expected: all tests PASS (types: 1, metrics: 5, runner: 5)

### Step 3: Final commit

```bash
git add engine/src/backtest/
git commit -m "feat(backtest): add full lifecycle integration test"
```

---

## Summary

| Task | Files | Tests |
|------|-------|-------|
| 1. Types | `backtest/mod.rs`, `main.rs` | 1 (serialization) |
| 2. Metrics | `backtest/metrics.rs` | 5 (empty, single, win-loss, all-loss, unclosed) |
| 3. Engine | `backtest/runner.rs` | 4 (stoploss, take_profit, force-close, multi-market) |
| 4. ClickHouse reader | `storage/clickhouse.rs`, `backtest/runner.rs` | 0 (compilation check) |
| 5. Integration | `backtest/runner.rs` | 1 (full lifecycle) |
| **Total** | **4 files created, 2 modified** | **11 tests** |

**What's NOT in this phase** (deferred to Phase 6):
- Axum `POST /internal/backtest/run` endpoint (Phase 6: Internal API)
- Laravel `BacktestController` calling the Axum endpoint (Phase 7)
