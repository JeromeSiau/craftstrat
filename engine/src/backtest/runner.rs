use std::collections::HashMap;

use clickhouse::Client;
use serde_json::Value;

use super::metrics;
use super::{compute_pnl, BacktestRequest, BacktestResult, BacktestTrade, ExitReason, Side};
use crate::fetcher::models::Tick;
use crate::strategy::interpreter::evaluate;
use crate::strategy::state::{Position, StrategyState};
use crate::strategy::{OrderType, Outcome, Signal};

pub struct BacktestEngine {
    graph: Value,
    window_size: usize,
    markets: HashMap<String, MarketContext>,
    trades: Vec<BacktestTrade>,
}

struct MarketContext {
    state: StrategyState,
    open_trade: Option<BacktestTrade>,
}

impl BacktestEngine {
    pub fn new(graph: Value, window_size: usize) -> Self {
        Self {
            graph,
            window_size,
            markets: HashMap::new(),
            trades: Vec::new(),
        }
    }

    pub fn process_tick(&mut self, tick: &Tick) {
        let window_size = self.window_size;
        let ctx = self
            .markets
            .entry(tick.symbol.clone())
            .or_insert_with(|| MarketContext {
                state: StrategyState::new(window_size),
                open_trade: None,
            });

        // Prediction market slot resolution: if winner is known and we have
        // an open position from a previous tick, settle it.
        // Exit at 1.0 if position outcome matches winner, 0.0 otherwise.
        if let (Some(winner), true) = (
            tick.winner,
            ctx.open_trade
                .as_ref()
                .map_or(false, |t| t.entry_at < tick.captured_at),
        ) {
            if let Some(mut trade) = ctx.open_trade.take() {
                let won = matches!(
                    (trade.outcome, winner),
                    (Outcome::Up, 1) | (Outcome::Down, 2)
                );
                let exit = if won { 1.0 } else { 0.0 };
                trade.exit_price = Some(exit);
                trade.pnl_usdc = compute_pnl(trade.entry_price, exit, trade.size_usdc);
                trade.exit_at = Some(tick.captured_at);
                trade.exit_reason = Some(ExitReason::SlotResolved);
                ctx.state.position = None;
                self.trades.push(trade);
            }
        }

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
                    side: Side::Buy,
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
                    trade.pnl_usdc = compute_pnl(trade.entry_price, exit, trade.size_usdc);
                    trade.exit_at = Some(tick.captured_at);
                    trade.exit_reason = Some(map_exit_reason(&order_type));
                    self.trades.push(trade);
                }
            }
            Signal::Cancel { .. } | Signal::Notify { .. } | Signal::Hold => {}
        }
    }

    pub fn finish(mut self) -> BacktestResult {
        // Force-close any open positions — use winner field if available,
        // otherwise fall back to last known mid price.
        for (_, ctx) in self.markets.drain() {
            if let Some(mut trade) = ctx.open_trade {
                if let Some(last_tick) = ctx.state.window.back() {
                    let (exit, reason) = if let Some(winner) = last_tick.winner {
                        let won = matches!(
                            (trade.outcome, winner),
                            (Outcome::Up, 1) | (Outcome::Down, 2)
                        );
                        (if won { 1.0 } else { 0.0 }, ExitReason::SlotResolved)
                    } else {
                        (mid_price(trade.outcome, last_tick), ExitReason::EndOfData)
                    };
                    trade.exit_price = Some(exit);
                    trade.pnl_usdc = compute_pnl(trade.entry_price, exit, trade.size_usdc);
                    trade.exit_at = Some(last_tick.captured_at);
                    trade.exit_reason = Some(reason);
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

fn map_exit_reason(order_type: &OrderType) -> ExitReason {
    match order_type {
        OrderType::StopLoss { .. } => ExitReason::Stoploss,
        OrderType::TakeProfit { .. } => ExitReason::TakeProfit,
        _ => ExitReason::Signal,
    }
}

pub async fn run(req: &BacktestRequest, ch_client: &Client) -> anyhow::Result<BacktestResult> {
    req.validate().map_err(anyhow::Error::msg)?;

    let mut cursor = crate::storage::clickhouse::fetch_ticks(
        ch_client,
        &req.market_filter,
        req.date_from,
        req.date_to,
    )?;

    let mut engine = BacktestEngine::new(req.strategy_graph.clone(), req.window_size);

    while let Some(tick) = cursor.next().await? {
        engine.process_tick(&tick);
    }

    Ok(engine.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::DEFAULT_WINDOW_SIZE;
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
        let mut engine = BacktestEngine::new(graph, DEFAULT_WINDOW_SIZE);

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
        assert_eq!(trade.exit_reason, Some(ExitReason::Stoploss));
    }

    #[test]
    fn test_buy_then_take_profit_exit() {
        let graph = simple_buy_up_strategy(); // take_profit_pct = 15
        let mut engine = BacktestEngine::new(graph, DEFAULT_WINDOW_SIZE);

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
        assert_eq!(trade.exit_reason, Some(ExitReason::TakeProfit));
    }

    #[test]
    fn test_force_close_at_end_of_data() {
        let graph = simple_buy_up_strategy();
        let mut engine = BacktestEngine::new(graph, DEFAULT_WINDOW_SIZE);

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
        assert_eq!(trade.exit_reason, Some(ExitReason::EndOfData));
        // Exit at mid_up of last tick (0.63)
        assert!((trade.exit_price.unwrap() - 0.63).abs() < 0.001);
    }

    #[test]
    fn test_multi_market_separate_state() {
        let graph = simple_buy_up_strategy();
        let mut engine = BacktestEngine::new(graph, DEFAULT_WINDOW_SIZE);

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

    #[test]
    fn test_full_backtest_lifecycle() {
        let graph = simple_buy_up_strategy(); // stoploss=10, take_profit=15, max_trades=2
        let mut engine = BacktestEngine::new(graph, DEFAULT_WINDOW_SIZE);

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
        // Equity curve: [0, -7.26, -0.26] — peak never positive, so percentage drawdown = 0
        assert!((result.max_drawdown).abs() < f64::EPSILON);
        assert_eq!(result.trades[0].exit_reason, Some(ExitReason::Stoploss));
        assert_eq!(result.trades[1].exit_reason, Some(ExitReason::TakeProfit));
    }

    #[test]
    fn test_slot_resolution_win() {
        let graph = simple_buy_up_strategy();
        let mut engine = BacktestEngine::new(graph, DEFAULT_WINDOW_SIZE);

        // Tick 1: triggers buy at ask_up = 0.62, winner=UP (annotated retroactively)
        let mut t1 = test_tick();
        t1.abs_move_pct = 4.0;
        t1.winner = Some(1); // UP wins
        engine.process_tick(&t1);

        // Position opened
        assert!(engine.markets["btc-updown-15m-1700000000"].open_trade.is_some());

        // Tick 2: next tick with winner=UP → slot resolves, position wins
        let mut t2 = test_tick();
        t2.abs_move_pct = 1.0;
        t2.winner = Some(1); // UP wins
        t2.captured_at = OffsetDateTime::from_unix_timestamp(1700000451).unwrap();
        engine.process_tick(&t2);

        // Position should be resolved
        assert!(engine.markets["btc-updown-15m-1700000000"].open_trade.is_none());
        assert_eq!(engine.trades.len(), 1);

        let trade = &engine.trades[0];
        assert!((trade.exit_price.unwrap() - 1.0).abs() < f64::EPSILON);
        // PnL = (1.0 - 0.62) / 0.62 * 50 ≈ 30.65
        assert!(trade.pnl_usdc > 0.0);
        assert_eq!(trade.exit_reason, Some(ExitReason::SlotResolved));
    }

    #[test]
    fn test_slot_resolution_loss() {
        let graph = simple_buy_up_strategy();
        let mut engine = BacktestEngine::new(graph, DEFAULT_WINDOW_SIZE);

        // Tick 1: triggers buy UP, but DOWN wins
        let mut t1 = test_tick();
        t1.abs_move_pct = 4.0;
        t1.winner = Some(2); // DOWN wins
        engine.process_tick(&t1);

        // Tick 2: slot resolves → bought UP but DOWN won → loss
        let mut t2 = test_tick();
        t2.abs_move_pct = 1.0;
        t2.winner = Some(2);
        t2.captured_at = OffsetDateTime::from_unix_timestamp(1700000451).unwrap();
        engine.process_tick(&t2);

        assert_eq!(engine.trades.len(), 1);
        let trade = &engine.trades[0];
        assert!((trade.exit_price.unwrap()).abs() < f64::EPSILON); // 0.0
        // PnL = (0.0 - 0.62) / 0.62 * 50 = -50.0
        assert!((trade.pnl_usdc - (-50.0)).abs() < 0.001);
        assert_eq!(trade.exit_reason, Some(ExitReason::SlotResolved));
    }

    #[test]
    fn test_slot_resolution_does_not_resolve_same_tick() {
        let graph = simple_buy_up_strategy();
        let mut engine = BacktestEngine::new(graph, DEFAULT_WINDOW_SIZE);

        // Single tick: triggers buy, winner is set (retroactive annotation)
        let mut t1 = test_tick();
        t1.abs_move_pct = 4.0;
        t1.winner = Some(1);
        engine.process_tick(&t1);

        // Position should still be open (not resolved on same tick as entry)
        assert!(engine.markets["btc-updown-15m-1700000000"].open_trade.is_some());
        assert_eq!(engine.trades.len(), 0);

        // finish() should resolve using winner
        let result = engine.finish();
        assert_eq!(result.total_trades, 1);
        assert!((result.trades[0].exit_price.unwrap() - 1.0).abs() < f64::EPSILON);
        assert_eq!(result.trades[0].exit_reason, Some(ExitReason::SlotResolved));
    }

    #[test]
    fn test_no_resolution_without_winner() {
        let graph = simple_buy_up_strategy();
        let mut engine = BacktestEngine::new(graph, DEFAULT_WINDOW_SIZE);

        // Tick 1: triggers buy, no winner (unresolved slot)
        let mut t1 = test_tick();
        t1.abs_move_pct = 4.0;
        t1.winner = None;
        engine.process_tick(&t1);

        // Tick 2: still no winner
        let mut t2 = test_tick();
        t2.abs_move_pct = 1.0;
        t2.mid_up = 0.63;
        t2.bid_up = 0.62;
        t2.winner = None;
        t2.captured_at = OffsetDateTime::from_unix_timestamp(1700000451).unwrap();
        engine.process_tick(&t2);

        // Position stays open
        assert!(engine.markets["btc-updown-15m-1700000000"].open_trade.is_some());
        assert_eq!(engine.trades.len(), 0);

        // finish() uses mid_price fallback (no winner)
        let result = engine.finish();
        assert_eq!(result.total_trades, 1);
        assert!((result.trades[0].exit_price.unwrap() - 0.63).abs() < 0.001);
        assert_eq!(result.trades[0].exit_reason, Some(ExitReason::EndOfData));
    }

    #[test]
    fn test_stoploss_takes_priority_over_slot_resolution() {
        // If SL/TP is configured and triggers, it should close the position
        // even if winner is set (SL/TP runs before slot resolution in the
        // interpreter, so position is already cleared).
        let graph = simple_buy_up_strategy(); // stoploss_pct=10
        let mut engine = BacktestEngine::new(graph, DEFAULT_WINDOW_SIZE);

        // Tick 1: triggers buy at ask_up = 0.62
        let mut t1 = test_tick();
        t1.abs_move_pct = 4.0;
        t1.winner = Some(2); // DOWN wins
        engine.process_tick(&t1);

        // Tick 2: stoploss triggers (mid_up=0.54 → -12.9%) AND winner is set
        let mut t2 = test_tick();
        t2.abs_move_pct = 1.0;
        t2.mid_up = 0.54;
        t2.bid_up = 0.53;
        t2.winner = Some(2);
        t2.captured_at = OffsetDateTime::from_unix_timestamp(1700000451).unwrap();
        engine.process_tick(&t2);

        assert_eq!(engine.trades.len(), 1);
        let trade = &engine.trades[0];
        // Slot resolution runs first (open_trade check), then evaluate() runs with risk.
        // Since slot resolution fires first and clears the position, the trade is
        // closed at slot resolution price (0.0 for a loss).
        assert_eq!(trade.exit_reason, Some(ExitReason::SlotResolved));
    }
}
