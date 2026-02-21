use std::collections::HashMap;

use serde_json::Value;

use super::metrics;
use super::{BacktestResult, BacktestTrade};
use crate::fetcher::models::Tick;
use crate::strategy::interpreter::evaluate;
use crate::strategy::state::{Position, StrategyState};
use crate::strategy::{OrderType, Outcome, Signal};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::test_utils::test_tick;
    use time::OffsetDateTime;

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

        // Tick 1: abs_move_pct = 1.0 -> no signal (below threshold)
        let mut t1 = test_tick();
        t1.abs_move_pct = 1.0;
        engine.process_tick(&t1);

        // Tick 2: abs_move_pct = 4.0 -> Buy signal (above 3.0)
        let mut t2 = test_tick();
        t2.abs_move_pct = 4.0;
        t2.captured_at = OffsetDateTime::from_unix_timestamp(1700000451).unwrap();
        engine.process_tick(&t2);

        // Verify position opened
        let ctx = engine.markets.get("btc-updown-15m-1700000000").unwrap();
        assert!(ctx.open_trade.is_some());
        assert!(ctx.state.position.is_some());

        // Tick 3: mid_up drops to 0.54 -> stoploss triggers (PnL = (0.54-0.62)/0.62*100 = -12.9%)
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

        // Tick 2: mid_up rises enough -> take profit
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
}
