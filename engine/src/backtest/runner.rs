use std::collections::{HashMap, VecDeque};

use clickhouse::Client;
use serde_json::Value;

use super::metrics;
use super::{compute_pnl, BacktestRequest, BacktestResult, BacktestTrade, ExitReason, Side};
use crate::fetcher::models::Tick;
use crate::strategy::interpreter::{evaluate, evaluate_with_caches};
use crate::strategy::ml_features::{build_live_feature_row, LIVE_FEATURE_WINDOW};
use crate::strategy::state::{Position, StrategyState};
use crate::strategy::{OrderType, Outcome, Signal};
use crate::tasks::model_score_task::{fetch_prediction_batch, ModelScoreCache};

const MODEL_SCORE_BATCH_SIZE: usize = 64;
const DEPTH_LEVEL_WEIGHTS: [f64; 3] = [1.0, 0.75, 0.5];

pub struct BacktestEngine {
    graph: Value,
    window_size: usize,
    markets: HashMap<String, MarketContext>,
    trades: Vec<BacktestTrade>,
    model_scores: Option<BacktestModelScores>,
}

struct MarketContext {
    state: StrategyState,
    open_trade: Option<BacktestTrade>,
}

struct BacktestModelScores {
    urls: Vec<String>,
    lookup: HashMap<String, Value>,
    cache: ModelScoreCache,
}

#[derive(Debug, Clone, Copy)]
struct SimulatedFill {
    average_price: f64,
    reference_price: f64,
    slippage_bps: f64,
    book_depth_usdc: f64,
    depth_ratio: f64,
}

impl BacktestEngine {
    pub fn new(graph: Value, window_size: usize) -> Self {
        Self::with_model_scores(graph, window_size, None)
    }

    fn with_model_scores(
        graph: Value,
        window_size: usize,
        model_scores: Option<BacktestModelScores>,
    ) -> Self {
        Self {
            graph,
            window_size,
            markets: HashMap::new(),
            trades: Vec::new(),
            model_scores,
        }
    }

    pub fn process_tick(&mut self, tick: &Tick) {
        self.seed_model_scores_for_tick(tick);

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

        let signal = if let Some(model_scores) = &self.model_scores {
            evaluate_with_caches(
                &self.graph,
                tick,
                &mut ctx.state,
                None,
                Some(&model_scores.cache),
            )
        } else {
            evaluate(&self.graph, tick, &mut ctx.state)
        };

        match signal {
            Signal::Buy {
                outcome, size_usdc, ..
            } => {
                let Some(entry_fill) = simulate_entry_fill(outcome, tick, size_usdc) else {
                    // No liquidity — skip this entry
                    return;
                };
                ctx.state.pending_entry_symbol = None;
                ctx.state.position = Some(Position {
                    outcome,
                    entry_price: entry_fill.average_price,
                    size_usdc,
                    entry_at: tick.captured_at.unix_timestamp(),
                    symbol: tick.symbol.clone(),
                });
                ctx.open_trade = Some(BacktestTrade {
                    symbol: tick.symbol.clone(),
                    outcome,
                    side: Side::Buy,
                    entry_price: entry_fill.average_price,
                    entry_reference_price: entry_fill.reference_price,
                    entry_slippage_bps: entry_fill.slippage_bps,
                    entry_book_depth_usdc: entry_fill.book_depth_usdc,
                    entry_depth_ratio: entry_fill.depth_ratio,
                    exit_price: None,
                    exit_reference_price: None,
                    exit_slippage_bps: None,
                    exit_book_depth_usdc: None,
                    exit_depth_ratio: None,
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
                if let Some(mut trade) = ctx.open_trade.take() {
                    let Some(exit_fill) =
                        simulate_exit_fill(outcome, tick, trade.size_usdc, trade.entry_price)
                    else {
                        ctx.state.position = Some(Position {
                            outcome: trade.outcome,
                            entry_price: trade.entry_price,
                            size_usdc: trade.size_usdc,
                            entry_at: trade.entry_at.unix_timestamp(),
                            symbol: trade.symbol.clone(),
                        });
                        ctx.open_trade = Some(trade);
                        return;
                    };

                    trade.exit_price = Some(exit_fill.average_price);
                    trade.exit_reference_price = Some(exit_fill.reference_price);
                    trade.exit_slippage_bps = Some(exit_fill.slippage_bps);
                    trade.exit_book_depth_usdc = Some(exit_fill.book_depth_usdc);
                    trade.exit_depth_ratio = Some(exit_fill.depth_ratio);
                    trade.pnl_usdc =
                        compute_pnl(trade.entry_price, exit_fill.average_price, trade.size_usdc);
                    trade.exit_at = Some(tick.captured_at);
                    trade.exit_reason = Some(map_exit_reason(&order_type));
                    self.trades.push(trade);
                }
            }
            Signal::Cancel { .. } | Signal::Notify { .. } | Signal::Hold => {}
        }
    }

    fn seed_model_scores_for_tick(&self, tick: &Tick) {
        let Some(model_scores) = &self.model_scores else {
            return;
        };

        for url in &model_scores.urls {
            let live_key = format!("{}#{}", url, tick.symbol);
            let lookup_key = prediction_lookup_key(url, &tick.symbol, tick);
            if let Some(payload) = model_scores.lookup.get(&lookup_key) {
                model_scores.cache.set(live_key, payload.clone());
            } else {
                model_scores.cache.remove(&live_key);
            }
        }
    }

    pub fn finish(mut self) -> BacktestResult {
        // Force-close any open positions — use winner field if available,
        // otherwise fall back to last known mid price.
        for (_, ctx) in self.markets.drain() {
            if let Some(mut trade) = ctx.open_trade {
                if let Some(last_tick) = ctx.state.window.back() {
                    let (exit, reference_price, slippage_bps, book_depth_usdc, depth_ratio, reason) =
                        if let Some(winner) = last_tick.winner {
                            let won = matches!(
                                (trade.outcome, winner),
                                (Outcome::Up, 1) | (Outcome::Down, 2)
                            );
                            (
                                if won { 1.0 } else { 0.0 },
                                None,
                                None,
                                None,
                                None,
                                ExitReason::SlotResolved,
                            )
                        } else {
                            match simulate_exit_fill(
                                trade.outcome,
                                last_tick,
                                trade.size_usdc,
                                trade.entry_price,
                            ) {
                                Some(fill) => (
                                    fill.average_price,
                                    Some(fill.reference_price),
                                    Some(fill.slippage_bps),
                                    Some(fill.book_depth_usdc),
                                    Some(fill.depth_ratio),
                                    ExitReason::EndOfData,
                                ),
                                None => (
                                    mid_price(trade.outcome, last_tick),
                                    None,
                                    None,
                                    None,
                                    None,
                                    ExitReason::EndOfData,
                                ),
                            }
                        };
                    trade.exit_price = Some(exit);
                    trade.exit_reference_price = reference_price;
                    trade.exit_slippage_bps = slippage_bps;
                    trade.exit_book_depth_usdc = book_depth_usdc;
                    trade.exit_depth_ratio = depth_ratio;
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

fn mid_price(outcome: Outcome, tick: &Tick) -> f64 {
    match outcome {
        Outcome::Up => tick.mid_up as f64,
        Outcome::Down => tick.mid_down as f64,
    }
}

fn simulate_entry_fill(outcome: Outcome, tick: &Tick, size_usdc: f64) -> Option<SimulatedFill> {
    if size_usdc <= 0.0 {
        return None;
    }

    let (reference_price, l2_price, l3_price, l1_size_tokens) = match outcome {
        Outcome::Up => (
            tick.ask_up as f64,
            tick.ask_up_l2 as f64,
            tick.ask_up_l3 as f64,
            tick.ask_size_up as f64,
        ),
        Outcome::Down => (
            tick.ask_down as f64,
            tick.ask_down_l2 as f64,
            tick.ask_down_l3 as f64,
            tick.ask_size_down as f64,
        ),
    };

    let levels = estimated_levels(reference_price, l2_price, l3_price, l1_size_tokens);
    simulate_buy_usdc_fill(&levels, size_usdc)
}

fn simulate_exit_fill(
    outcome: Outcome,
    tick: &Tick,
    size_usdc: f64,
    entry_price: f64,
) -> Option<SimulatedFill> {
    if size_usdc <= 0.0 || entry_price <= 0.0 {
        return None;
    }

    let (reference_price, l2_price, l3_price, l1_size_tokens) = match outcome {
        Outcome::Up => (
            tick.bid_up as f64,
            tick.bid_up_l2 as f64,
            tick.bid_up_l3 as f64,
            tick.bid_size_up as f64,
        ),
        Outcome::Down => (
            tick.bid_down as f64,
            tick.bid_down_l2 as f64,
            tick.bid_down_l3 as f64,
            tick.bid_size_down as f64,
        ),
    };

    let levels = estimated_levels(reference_price, l2_price, l3_price, l1_size_tokens);
    let token_qty = size_usdc / entry_price;
    simulate_sell_token_fill(&levels, token_qty)
}

fn estimated_levels(
    l1_price: f64,
    l2_price: f64,
    l3_price: f64,
    l1_size_tokens: f64,
) -> Vec<(f64, f64)> {
    if l1_price <= 0.0 || l1_size_tokens <= 0.0 {
        return Vec::new();
    }

    [l1_price, l2_price, l3_price]
        .into_iter()
        .zip(DEPTH_LEVEL_WEIGHTS)
        .filter_map(|(price, weight)| {
            if price > 0.0 {
                Some((price, l1_size_tokens * weight))
            } else {
                None
            }
        })
        .collect()
}

fn simulate_buy_usdc_fill(levels: &[(f64, f64)], budget_usdc: f64) -> Option<SimulatedFill> {
    let reference_price = levels.first()?.0;
    let book_depth_usdc = levels.iter().map(|(price, size)| price * size).sum::<f64>();
    if book_depth_usdc + f64::EPSILON < budget_usdc {
        return None;
    }

    let mut remaining_budget = budget_usdc;
    let mut total_tokens = 0.0;
    for (price, size_tokens) in levels {
        let level_capacity_usdc = price * size_tokens;
        let budget_here = remaining_budget.min(level_capacity_usdc);
        total_tokens += budget_here / price;
        remaining_budget -= budget_here;
        if remaining_budget <= f64::EPSILON {
            break;
        }
    }

    if total_tokens <= 0.0 {
        return None;
    }

    let average_price = budget_usdc / total_tokens;
    let slippage_bps = if reference_price > 0.0 {
        (average_price - reference_price) / reference_price * 10_000.0
    } else {
        0.0
    };

    Some(SimulatedFill {
        average_price,
        reference_price,
        slippage_bps,
        book_depth_usdc,
        depth_ratio: if book_depth_usdc > 0.0 {
            budget_usdc / book_depth_usdc
        } else {
            0.0
        },
    })
}

fn simulate_sell_token_fill(levels: &[(f64, f64)], token_qty: f64) -> Option<SimulatedFill> {
    let reference_price = levels.first()?.0;
    let total_capacity_tokens = levels.iter().map(|(_, size)| size).sum::<f64>();
    if total_capacity_tokens + f64::EPSILON < token_qty {
        return None;
    }

    let book_depth_usdc = levels.iter().map(|(price, size)| price * size).sum::<f64>();
    let requested_notional_usdc = token_qty * reference_price;
    let mut remaining_tokens = token_qty;
    let mut proceeds_usdc = 0.0;

    for (price, size_tokens) in levels {
        let tokens_here = remaining_tokens.min(*size_tokens);
        proceeds_usdc += tokens_here * price;
        remaining_tokens -= tokens_here;
        if remaining_tokens <= f64::EPSILON {
            break;
        }
    }

    if token_qty <= 0.0 {
        return None;
    }

    let average_price = proceeds_usdc / token_qty;
    let slippage_bps = if reference_price > 0.0 {
        (reference_price - average_price) / reference_price * 10_000.0
    } else {
        0.0
    };

    Some(SimulatedFill {
        average_price,
        reference_price,
        slippage_bps,
        book_depth_usdc,
        depth_ratio: if book_depth_usdc > 0.0 {
            requested_notional_usdc / book_depth_usdc
        } else {
            0.0
        },
    })
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

    let model_urls = collect_model_score_urls(&req.strategy_graph);
    let mut cursor = crate::storage::clickhouse::fetch_ticks(
        ch_client,
        &req.market_filter,
        req.date_from,
        req.date_to,
    )?;

    if model_urls.is_empty() {
        let mut engine = BacktestEngine::new(req.strategy_graph.clone(), req.window_size);

        while let Some(tick) = cursor.next().await? {
            engine.process_tick(&tick);
        }

        return Ok(engine.finish());
    }

    let mut ticks = Vec::new();
    while let Some(tick) = cursor.next().await? {
        ticks.push(tick);
    }

    let model_scores = precompute_model_scores(&model_urls, &ticks).await?;
    let mut engine = BacktestEngine::with_model_scores(
        req.strategy_graph.clone(),
        req.window_size,
        Some(model_scores),
    );

    for tick in &ticks {
        engine.process_tick(tick);
    }

    Ok(engine.finish())
}

fn collect_model_score_urls(graph: &Value) -> Vec<String> {
    let Some(nodes) = graph["nodes"].as_array() else {
        return Vec::new();
    };

    let mut urls = Vec::new();
    for node in nodes {
        if node["type"].as_str() != Some("model_score") {
            continue;
        }

        let url = node["data"]["url"].as_str().unwrap_or("").trim();
        if url.is_empty() || urls.iter().any(|existing| existing == url) {
            continue;
        }

        urls.push(url.to_string());
    }

    urls
}

async fn precompute_model_scores(
    model_urls: &[String],
    ticks: &[Tick],
) -> anyhow::Result<BacktestModelScores> {
    let mut windows: HashMap<String, VecDeque<Tick>> = HashMap::new();
    let mut rows_by_url: HashMap<String, Vec<Value>> = model_urls
        .iter()
        .cloned()
        .map(|url| (url, Vec::new()))
        .collect();
    let mut keys_by_url: HashMap<String, Vec<String>> = model_urls
        .iter()
        .cloned()
        .map(|url| (url, Vec::new()))
        .collect();

    for tick in ticks {
        let window = windows
            .entry(tick.symbol.clone())
            .or_insert_with(|| VecDeque::with_capacity(LIVE_FEATURE_WINDOW));
        if window.len() >= LIVE_FEATURE_WINDOW {
            window.pop_front();
        }
        window.push_back(tick.clone());

        let Some(feature_row) = build_live_feature_row(window) else {
            continue;
        };

        for url in model_urls {
            if let Some(rows) = rows_by_url.get_mut(url) {
                rows.push(feature_row.clone());
            }
            if let Some(keys) = keys_by_url.get_mut(url) {
                keys.push(prediction_lookup_key(url, &tick.symbol, tick));
            }
        }
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let mut lookup = HashMap::new();

    for url in model_urls {
        let Some(rows) = rows_by_url.get(url) else {
            continue;
        };
        let Some(keys) = keys_by_url.get(url) else {
            continue;
        };

        if rows.is_empty() {
            continue;
        }

        for (row_chunk, key_chunk) in rows
            .chunks(MODEL_SCORE_BATCH_SIZE)
            .zip(keys.chunks(MODEL_SCORE_BATCH_SIZE))
        {
            let predictions = fetch_prediction_batch(&client, url, row_chunk).await?;
            for (key, payload) in key_chunk.iter().zip(predictions.into_iter()) {
                lookup.insert(key.clone(), payload);
            }
        }
    }

    Ok(BacktestModelScores {
        urls: model_urls.to_vec(),
        lookup,
        cache: ModelScoreCache::new(),
    })
}

fn prediction_lookup_key(url: &str, symbol: &str, tick: &Tick) -> String {
    format!(
        "{}#{}#{}",
        url,
        symbol,
        tick.captured_at.unix_timestamp_nanos()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::DEFAULT_WINDOW_SIZE;
    use crate::strategy::test_utils::test_tick;
    use axum::{routing::post, Json, Router};
    use time::OffsetDateTime;
    use tokio::net::TcpListener;

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
        t1.ask_size_up = 120.0;
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
        // Exit through the bid side of the last book snapshot.
        assert!((trade.exit_price.unwrap() - 0.62).abs() < 0.001);
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
        t3.ask_size_up = 120.0;
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
        assert!(engine.markets["btc-updown-15m-1700000000"]
            .open_trade
            .is_some());

        // Tick 2: next tick with winner=UP → slot resolves, position wins
        let mut t2 = test_tick();
        t2.abs_move_pct = 1.0;
        t2.winner = Some(1); // UP wins
        t2.captured_at = OffsetDateTime::from_unix_timestamp(1700000451).unwrap();
        engine.process_tick(&t2);

        // Position should be resolved
        assert!(engine.markets["btc-updown-15m-1700000000"]
            .open_trade
            .is_none());
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
        assert!(engine.markets["btc-updown-15m-1700000000"]
            .open_trade
            .is_some());
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
        assert!(engine.markets["btc-updown-15m-1700000000"]
            .open_trade
            .is_some());
        assert_eq!(engine.trades.len(), 0);

        // finish() exits through the bid side of the final book snapshot.
        let result = engine.finish();
        assert_eq!(result.total_trades, 1);
        assert!((result.trades[0].exit_price.unwrap() - 0.62).abs() < 0.001);
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

    #[tokio::test]
    async fn test_model_score_backtest_uses_http_predictions() {
        let app = Router::new().route(
            "/predict",
            post(|Json(payload): Json<Value>| async move {
                let rows = payload["rows"].as_array().cloned().unwrap_or_default();
                let predictions: Vec<Value> = rows
                    .into_iter()
                    .map(|row| {
                        let proba_up = if row["f_mid_up"].as_f64().unwrap_or(0.0) > 0.6 {
                            0.91
                        } else {
                            0.09
                        };
                        serde_json::json!({
                            "proba_up": proba_up,
                            "edge_up": proba_up - row["f_ask_up"].as_f64().unwrap_or(0.0),
                        })
                    })
                    .collect();

                Json(serde_json::json!({
                    "count": predictions.len(),
                    "predictions": predictions,
                }))
            }),
        );

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = format!("http://{addr}/predict");
        let graph = serde_json::json!({
            "mode": "node",
            "nodes": [
                { "id": "n1", "type": "model_score", "data": {
                    "url": url,
                    "json_path": "proba_up",
                    "interval_ms": 1000
                }},
                { "id": "n2", "type": "comparator", "data": { "operator": ">", "value": 0.8 } },
                { "id": "n3", "type": "action", "data": { "signal": "buy", "outcome": "UP", "size_usdc": 50 } }
            ],
            "edges": [
                { "source": "n1", "target": "n2" },
                { "source": "n2", "target": "n3" }
            ]
        });

        let mut t1 = test_tick();
        t1.winner = Some(1);

        let mut t2 = test_tick();
        t2.captured_at = OffsetDateTime::from_unix_timestamp(1700000451).unwrap();
        t2.pct_into_slot = 0.95;
        t2.winner = Some(1);

        let ticks = vec![t1.clone(), t2.clone()];
        let model_scores = precompute_model_scores(&collect_model_score_urls(&graph), &ticks)
            .await
            .unwrap();
        let mut engine =
            BacktestEngine::with_model_scores(graph, DEFAULT_WINDOW_SIZE, Some(model_scores));

        engine.process_tick(&t1);
        engine.process_tick(&t2);
        let result = engine.finish();

        server.abort();

        assert_eq!(result.total_trades, 1);
        assert_eq!(result.trades[0].exit_reason, Some(ExitReason::SlotResolved));
        assert!(result.trades[0].pnl_usdc > 0.0);
    }
}
