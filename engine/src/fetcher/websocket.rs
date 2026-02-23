use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use metrics::counter;

use super::models::{Level, OrderBook, Side};

#[derive(Clone)]
pub struct OrderBookCache(Arc<RwLock<HashMap<String, OrderBook>>>);

impl OrderBookCache {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }
}

impl std::ops::Deref for OrderBookCache {
    type Target = RwLock<HashMap<String, OrderBook>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub enum WsCommand {
    Subscribe { token_ids: Vec<String> },
    Unsubscribe { token_ids: Vec<String> },
}

pub async fn run_ws_feed(
    ws_url: String,
    books: OrderBookCache,
    mut cmd_rx: tokio::sync::mpsc::Receiver<WsCommand>,
) {
    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);
    let mut subscribed: HashSet<String> = HashSet::new();
    let mut consecutive_reconnects: u32 = 0;

    loop {
        tracing::info!("clob_ws_connecting");
        let connected_at = Instant::now();

        match connect_and_stream(&ws_url, &books, &mut cmd_rx, &mut subscribed).await {
            Ok(_) => {
                tracing::warn!("clob_ws_disconnected");
                counter!(crate::metrics::WS_RECONNECTIONS_TOTAL, "reason" => "disconnected")
                    .increment(1);
            }
            Err(e) => {
                let error_type = classify_ws_error(&e);
                tracing::warn!(error = %e, error_type, "clob_ws_error");
                counter!(crate::metrics::WS_ERRORS_TOTAL, "error_type" => error_type)
                    .increment(1);
                counter!(crate::metrics::WS_RECONNECTIONS_TOTAL, "reason" => "error")
                    .increment(1);
            }
        }

        books.write().await.clear();

        if connected_at.elapsed() > Duration::from_secs(60) {
            backoff = Duration::from_secs(1);
            consecutive_reconnects = 0;
        } else {
            consecutive_reconnects += 1;
        }

        tracing::warn!(
            consecutive = consecutive_reconnects,
            backoff_ms = backoff.as_millis() as u64,
            "clob_ws_reconnecting"
        );

        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(max_backoff);
    }
}

fn classify_ws_error(error: &anyhow::Error) -> &'static str {
    let msg = error.to_string();
    if msg.contains("Connection reset") {
        "connection_reset"
    } else if msg.contains("imeout") {
        "timeout"
    } else {
        "other"
    }
}

async fn connect_and_stream(
    ws_url: &str,
    books: &OrderBookCache,
    cmd_rx: &mut tokio::sync::mpsc::Receiver<WsCommand>,
    subscribed: &mut HashSet<String>,
) -> Result<()> {
    let (ws, _) = connect_async(ws_url).await?;
    let (mut write, mut read) = ws.split();
    tracing::info!(tokens = subscribed.len(), "clob_ws_connected");

    if !subscribed.is_empty() {
        let tokens: Vec<&String> = subscribed.iter().collect();
        let msg = serde_json::json!({
            "assets_ids": tokens,
            "type": "market",
            "custom_feature_enabled": true,
        });
        write.send(Message::Text(msg.to_string().into())).await?;
    }

    let mut last_update = Instant::now();
    let stale_threshold = Duration::from_secs(60);
    let mut ping_interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if text.trim() == "PONG" { continue; }
                        process_message(&text, books).await;
                        last_update = Instant::now();
                    }
                    Some(Ok(Message::Close(_))) | None => return Ok(()),
                    Some(Err(e)) => return Err(e.into()),
                    _ => {}
                }
            }
            _ = ping_interval.tick() => {
                if last_update.elapsed() > stale_threshold && !subscribed.is_empty() {
                    tracing::warn!("clob_ws_stale");
                    return Ok(());
                }
                // Timeout the ping to avoid hanging on half-open connections
                match tokio::time::timeout(
                    Duration::from_secs(5),
                    write.send(Message::Text("PING".into())),
                ).await {
                    Ok(Ok(_)) => {}
                    Ok(Err(e)) => return Err(e.into()),
                    Err(_) => {
                        tracing::warn!("clob_ws_ping_timeout");
                        return Ok(());
                    }
                }
            }
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(WsCommand::Subscribe { token_ids }) => {
                        subscribed.extend(token_ids.iter().cloned());
                        let msg = serde_json::json!({
                            "assets_ids": token_ids,
                            "type": "market",
                            "custom_feature_enabled": true,
                        });
                        write.send(Message::Text(msg.to_string().into())).await?;
                    }
                    Some(WsCommand::Unsubscribe { token_ids }) => {
                        for t in &token_ids {
                            subscribed.remove(t);
                        }
                        let msg = serde_json::json!({
                            "assets_ids": token_ids,
                            "operation": "unsubscribe",
                        });
                        write.send(Message::Text(msg.to_string().into())).await?;
                    }
                    None => return Ok(()),
                }
            }
        }
    }
}

async fn process_message(text: &str, books: &OrderBookCache) {
    let Ok(data) = serde_json::from_str::<serde_json::Value>(text) else {
        return;
    };

    let events: Vec<&serde_json::Value> = if let Some(arr) = data.as_array() {
        arr.iter().collect()
    } else {
        vec![&data]
    };

    let mut cache = books.write().await;
    for event in events {
        match event.get("event_type").and_then(|v| v.as_str()) {
            Some("book") => handle_book_snapshot(event, &mut cache),
            Some("price_change") => handle_price_change(event, &mut cache),
            _ => {}
        }
    }
}

fn handle_book_snapshot(event: &serde_json::Value, cache: &mut HashMap<String, OrderBook>) {
    let Some(token_id) = event.get("asset_id").and_then(|v| v.as_str()) else {
        return;
    };
    let bids = parse_levels(event.get("bids"), true);
    let asks = parse_levels(event.get("asks"), false);
    cache.insert(token_id.to_string(), OrderBook { bids, asks });
}

fn handle_price_change(event: &serde_json::Value, cache: &mut HashMap<String, OrderBook>) {
    let Some(changes) = event.get("price_changes").and_then(|v| v.as_array()) else {
        return;
    };
    for change in changes {
        let Some(token_id) = change.get("asset_id").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(book) = cache.get_mut(token_id) else {
            continue;
        };
        let price = parse_f32(change.get("price"));
        let size = parse_f32(change.get("size"));
        let side = match change.get("side").and_then(|v| v.as_str()) {
            Some("BUY") => Side::Buy,
            Some("SELL") => Side::Sell,
            _ => continue,
        };
        book.merge_level(price, size, side);
    }
}

fn parse_levels(val: Option<&serde_json::Value>, descending: bool) -> Vec<Level> {
    let Some(arr) = val.and_then(|v| v.as_array()) else {
        return vec![];
    };
    let mut levels: Vec<Level> = arr
        .iter()
        .filter_map(|item| {
            let price = parse_f32(item.get("price"));
            let size = parse_f32(item.get("size"));
            if size > 0.0 { Some(Level { price, size }) } else { None }
        })
        .collect();
    if descending {
        levels.sort_by(|a, b| b.price.total_cmp(&a.price));
    } else {
        levels.sort_by(|a, b| a.price.total_cmp(&b.price));
    }
    levels
}

fn parse_f32(val: Option<&serde_json::Value>) -> f32 {
    val.and_then(|v| {
        v.as_str()
            .and_then(|s| s.parse::<f32>().ok())
            .or_else(|| v.as_f64().map(|f| f as f32))
    })
    .unwrap_or(0.0)
}
