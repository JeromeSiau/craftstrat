use anyhow::Result;
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

        // Read lock -> clone assignments for this symbol -> release lock
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
