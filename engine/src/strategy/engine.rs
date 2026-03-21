use anyhow::Result;
use metrics::{counter, gauge, histogram};
use rayon::prelude::*;
use rdkafka::Message;
use tokio::sync::mpsc;

use super::interpreter;
use super::registry::AssignmentRegistry;
use super::{EngineOutput, OrderType, Outcome, Signal};
use crate::fetcher::models::Tick;
use crate::kafka;
use crate::metrics as m;
use crate::tasks::api_fetch_task::ApiFetchCache;
use crate::tasks::model_score_task::ModelScoreCache;

pub async fn run(
    brokers: &str,
    registry: AssignmentRegistry,
    api_cache: ApiFetchCache,
    model_score_cache: ModelScoreCache,
    signal_tx: mpsc::Sender<EngineOutput>,
) -> Result<()> {
    let consumer = kafka::consumer::create_consumer(brokers, "strategy-engine", &["ticks"])?;
    tracing::info!("strategy_engine_started");

    let engine_start = std::time::Instant::now();
    let mut tick_count: u64 = 0;

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

        counter!(m::TICKS_TOTAL).increment(1);
        tick_count += 1;
        if tick_count % 100 == 0 {
            gauge!(m::UPTIME_SECONDS).set(engine_start.elapsed().as_secs_f64());
        }

        // Read lock -> clone assignments for this symbol -> release lock
        // Tick symbols include a timestamp suffix (e.g. "btc-updown-15m-1772135100")
        // but registry keys are prefixes (e.g. "btc-updown-15m"), so strip the
        // trailing timestamp segment for the lookup.
        let market_prefix = tick
            .symbol
            .rfind('-')
            .map(|pos| &tick.symbol[..pos])
            .unwrap_or(&tick.symbol);
        let assignments = {
            let reg = registry.read().await;
            reg.get(market_prefix).cloned().unwrap_or_default()
        };

        if assignments.is_empty() {
            continue;
        }

        // Rayon parallel dispatch
        let eval_start = std::time::Instant::now();
        let signals: Vec<EngineOutput> = assignments
            .par_iter()
            .filter_map(|a| {
                if a.is_killed {
                    return None;
                }
                let mut state = match a.state.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => {
                        tracing::warn!(
                            wallet_id = a.wallet_id,
                            strategy_id = a.strategy_id,
                            "mutex_poisoned_recovering"
                        );
                        poisoned.into_inner()
                    }
                };
                let signal = interpreter::evaluate_with_caches(
                    &a.graph,
                    &tick,
                    &mut state,
                    Some(&api_cache),
                    Some(&model_score_cache),
                );
                match signal {
                    Signal::Hold => None,
                    s => {
                        let reference_price = execution_reference_price(&s, &tick);

                        Some(EngineOutput {
                            wallet_id: a.wallet_id,
                            strategy_id: a.strategy_id,
                            symbol: tick.symbol.clone(),
                            signal: s,
                            reference_price,
                            is_paper: a.is_paper,
                        })
                    }
                }
            })
            .collect();
        histogram!(m::STRATEGY_EVAL_DURATION).record(eval_start.elapsed().as_secs_f64());

        for output in signals {
            let signal_type = match &output.signal {
                Signal::Buy { .. } => "buy",
                Signal::Sell { .. } => "sell",
                Signal::Cancel { .. } => "cancel",
                Signal::Notify { .. } => "notify",
                Signal::Hold => "hold",
            };
            counter!(m::SIGNALS_TOTAL, "signal" => signal_type).increment(1);

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

fn execution_reference_price(signal: &Signal, tick: &Tick) -> Option<f64> {
    match signal {
        Signal::Buy {
            outcome,
            order_type,
            ..
        } => Some(match order_type {
            OrderType::Market => market_buy_price(*outcome, tick),
            OrderType::Limit { price } => *price,
            OrderType::StopLoss { trigger_price } => *trigger_price,
            OrderType::TakeProfit { trigger_price } => *trigger_price,
        }),
        Signal::Sell {
            outcome,
            order_type,
            ..
        } => Some(match order_type {
            OrderType::Market => market_sell_price(*outcome, tick),
            OrderType::Limit { price } => *price,
            OrderType::StopLoss { trigger_price } => *trigger_price,
            OrderType::TakeProfit { trigger_price } => *trigger_price,
        }),
        Signal::Cancel { .. } | Signal::Notify { .. } | Signal::Hold => None,
    }
}

fn market_buy_price(outcome: Outcome, tick: &Tick) -> f64 {
    match outcome {
        Outcome::Up => tick.ask_up as f64,
        Outcome::Down => tick.ask_down as f64,
    }
}

fn market_sell_price(outcome: Outcome, tick: &Tick) -> f64 {
    match outcome {
        Outcome::Up => tick.bid_up as f64,
        Outcome::Down => tick.bid_down as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::test_utils::test_tick;

    #[test]
    fn market_buy_signals_capture_ask_price() {
        let tick = test_tick();
        let signal = Signal::Buy {
            outcome: Outcome::Up,
            size_usdc: 1.0,
            order_type: OrderType::Market,
        };

        let reference_price = execution_reference_price(&signal, &tick);

        assert!((reference_price.unwrap_or_default() - 0.62).abs() < 1e-6);
    }

    #[test]
    fn market_sell_signals_capture_bid_price() {
        let tick = test_tick();
        let signal = Signal::Sell {
            outcome: Outcome::Down,
            size_usdc: 1.0,
            order_type: OrderType::Market,
        };

        let reference_price = execution_reference_price(&signal, &tick);

        assert!((reference_price.unwrap_or_default() - 0.38).abs() < 1e-6);
    }
}
