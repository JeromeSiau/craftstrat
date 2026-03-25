use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::sync::broadcast;

use crate::fetcher::models::Tick;
use crate::proxy::HttpPool;
use crate::strategy::bandit;
use crate::strategy::ml_features::{build_live_feature_row, LIVE_FEATURE_WINDOW};
use crate::strategy::registry::{Assignment, AssignmentRegistry};
use crate::tasks::json_path::extract_json_path;

#[derive(Debug, Clone)]
struct CacheEntry {
    payload: Value,
    updated_at: Instant,
}

#[derive(Clone)]
pub struct ModelScoreCache(Arc<RwLock<HashMap<String, CacheEntry>>>);

impl ModelScoreCache {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn get_number(&self, key: &str, max_age_ms: u64, json_path: &str) -> f64 {
        let guard = self.0.read().unwrap_or_else(|e| e.into_inner());
        guard
            .get(key)
            .filter(|entry| entry.updated_at.elapsed().as_millis() < max_age_ms as u128)
            .and_then(|entry| extract_json_path(&entry.payload, json_path))
            .and_then(|value| {
                value
                    .as_f64()
                    .or_else(|| value.as_bool().map(|flag| if flag { 1.0 } else { 0.0 }))
            })
            .unwrap_or(0.0)
    }

    pub(crate) fn set(&self, key: String, payload: Value) {
        let mut guard = self.0.write().unwrap_or_else(|e| e.into_inner());
        guard.insert(
            key,
            CacheEntry {
                payload,
                updated_at: Instant::now(),
            },
        );
    }

    pub(crate) fn remove(&self, key: &str) {
        let mut guard = self.0.write().unwrap_or_else(|e| e.into_inner());
        guard.remove(key);
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ModelTarget {
    url: String,
    interval_ms: u64,
}

impl ModelTarget {
    fn cache_key(&self, symbol: &str) -> String {
        format!("{}#{}", self.url, symbol)
    }
}

pub async fn run(
    registry: AssignmentRegistry,
    cache: ModelScoreCache,
    http: HttpPool,
    mut tick_rx: broadcast::Receiver<Tick>,
) -> anyhow::Result<()> {
    tracing::info!("model_score_task_started");

    let client = http.direct().clone();
    let mut last_scored: HashMap<String, Instant> = HashMap::new();
    let mut windows: HashMap<String, VecDeque<Tick>> = HashMap::new();

    loop {
        let tick = match tick_rx.recv().await {
            Ok(tick) => tick,
            Err(broadcast::error::RecvError::Closed) => return Ok(()),
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::warn!(skipped, "model_score_tick_lagged");
                continue;
            }
        };

        let market_prefix = tick
            .symbol
            .rfind('-')
            .map(|pos| &tick.symbol[..pos])
            .unwrap_or(&tick.symbol);

        let targets = {
            let reg = registry.read().await;
            reg.get(market_prefix)
                .map(|assignments| collect_targets(assignments.as_slice()))
                .unwrap_or_default()
        };

        if targets.is_empty() {
            continue;
        }

        let window = windows
            .entry(tick.symbol.clone())
            .or_insert_with(|| VecDeque::with_capacity(LIVE_FEATURE_WINDOW));
        if window.len() >= LIVE_FEATURE_WINDOW {
            window.pop_front();
        }
        window.push_back(tick.clone());

        let Some(feature_row) = build_live_feature_row(window) else {
            for target in &targets {
                cache.remove(&target.cache_key(&tick.symbol));
            }
            continue;
        };

        for target in targets {
            let cache_key = target.cache_key(&tick.symbol);
            let should_score = last_scored
                .get(&cache_key)
                .map(|instant| instant.elapsed() >= Duration::from_millis(target.interval_ms))
                .unwrap_or(true);

            if !should_score {
                continue;
            }

            match fetch_payload(&client, &target.url, &feature_row).await {
                Ok(payload) => {
                    cache.set(cache_key.clone(), payload);
                    last_scored.insert(cache_key, Instant::now());
                }
                Err(error) => {
                    tracing::warn!(url = %target.url, symbol = %tick.symbol, error = %error, "model_score_failed");
                }
            }
        }
    }
}

fn collect_targets(assignments: &[Assignment]) -> Vec<ModelTarget> {
    let mut intervals = HashMap::<String, u64>::new();

    for assignment in assignments {
        if assignment.is_killed {
            continue;
        }

        let Some(nodes) = assignment.graph["nodes"].as_array() else {
            continue;
        };

        for node in nodes {
            if node["type"].as_str() != Some("model_score") {
                continue;
            }

            let data = &node["data"];
            let url = data["url"].as_str().unwrap_or("").trim().to_string();
            let interval_ms = data["interval_ms"].as_u64().unwrap_or(2_000).max(1_000);

            if url.is_empty() {
                continue;
            }

            intervals
                .entry(url)
                .and_modify(|existing| *existing = (*existing).min(interval_ms))
                .or_insert(interval_ms);
        }

        for (url, interval_ms) in bandit::collect_model_targets(&assignment.graph) {
            if url.is_empty() {
                continue;
            }

            intervals
                .entry(url)
                .and_modify(|existing| *existing = (*existing).min(interval_ms))
                .or_insert(interval_ms.max(1_000));
        }
    }

    intervals
        .into_iter()
        .map(|(url, interval_ms)| ModelTarget { url, interval_ms })
        .collect()
}

async fn fetch_payload(client: &reqwest::Client, url: &str, row: &Value) -> anyhow::Result<Value> {
    let mut predictions = fetch_prediction_batch(client, url, std::slice::from_ref(row)).await?;
    predictions
        .pop()
        .ok_or_else(|| anyhow::anyhow!("model response did not contain a prediction"))
}

pub(crate) async fn fetch_prediction_batch(
    client: &reqwest::Client,
    url: &str,
    rows: &[Value],
) -> anyhow::Result<Vec<Value>> {
    let response = client
        .post(url)
        .timeout(Duration::from_secs(3))
        .json(&serde_json::json!({ "rows": rows }))
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("model returned HTTP {}", response.status());
    }

    let body = response.bytes().await?;
    if body.len() > 65_536 {
        anyhow::bail!("response too large: {} bytes", body.len());
    }

    let payload: Value = serde_json::from_slice(&body)?;
    if !payload.is_object() {
        anyhow::bail!("model response must be a JSON object");
    }

    let predictions = payload
        .get("predictions")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("model response must contain a predictions array"))?;

    if predictions.len() != rows.len() {
        anyhow::bail!(
            "model response length mismatch: expected {}, got {}",
            rows.len(),
            predictions.len()
        );
    }

    Ok(predictions.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_get_set_with_json_path() {
        let cache = ModelScoreCache::new();
        cache.set(
            "https://ml.example.com/predict#btc-updown-15m-1".into(),
            serde_json::json!({
                "proba_up": 0.71,
                "meta": { "edge_up": 0.08 }
            }),
        );

        assert!(
            (cache.get_number(
                "https://ml.example.com/predict#btc-updown-15m-1",
                5_000,
                "proba_up"
            ) - 0.71)
                .abs()
                < f64::EPSILON
        );
        assert!(
            (cache.get_number(
                "https://ml.example.com/predict#btc-updown-15m-1",
                5_000,
                "meta.edge_up"
            ) - 0.08)
                .abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn test_extract_json_path_array_index() {
        let payload = serde_json::json!({
            "predictions": [
                { "proba_up": 0.42 },
                { "proba_up": 0.63 }
            ]
        });

        assert_eq!(
            extract_json_path(&payload, "predictions.1.proba_up").and_then(Value::as_f64),
            Some(0.63)
        );
    }

    #[test]
    fn test_extract_json_path_bool_as_number() {
        let payload = serde_json::json!({
            "take_trade": true,
            "nested": { "take_down": false }
        });

        assert_eq!(
            extract_json_path(&payload, "take_trade")
                .and_then(|value| value.as_bool().map(|flag| if flag { 1.0 } else { 0.0 })),
            Some(1.0)
        );
        assert_eq!(
            extract_json_path(&payload, "nested.take_down")
                .and_then(|value| value.as_bool().map(|flag| if flag { 1.0 } else { 0.0 })),
            Some(0.0)
        );
    }

    #[test]
    fn test_collect_targets_deduplicates_urls() {
        let assignments = vec![Assignment {
            wallet_id: 1,
            strategy_id: 10,
            graph: serde_json::json!({
                "nodes": [
                    { "type": "model_score", "data": { "url": "https://ml.example.com/predict", "interval_ms": 2000 } },
                    { "type": "model_score", "data": { "url": "https://ml.example.com/predict", "interval_ms": 5000 } }
                ]
            }),
            markets: vec!["btc-updown-15m".into()],
            is_paper: false,
            is_killed: false,
            state: Arc::new(std::sync::Mutex::new(
                crate::strategy::state::StrategyState::new(16),
            )),
        }];

        let targets = collect_targets(&assignments);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].url, "https://ml.example.com/predict");
        assert_eq!(targets[0].interval_ms, 2_000);
    }

    #[test]
    fn test_collect_targets_includes_bandit_entry_url() {
        let assignments = vec![Assignment {
            wallet_id: 1,
            strategy_id: 10,
            graph: serde_json::json!({
                "mode": "node",
                "nodes": [],
                "edges": [],
                "bandit": {
                    "entry": {
                        "enabled": true,
                        "url": "https://ml.example.com/predict",
                        "interval_ms": 3_000,
                        "profiles": [
                            { "id": "balanced", "min_value": 0.02 }
                        ]
                    }
                }
            }),
            markets: vec!["btc-updown-15m".into()],
            is_paper: false,
            is_killed: false,
            state: Arc::new(std::sync::Mutex::new(
                crate::strategy::state::StrategyState::new(16),
            )),
        }];

        let targets = collect_targets(&assignments);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].url, "https://ml.example.com/predict");
        assert_eq!(targets[0].interval_ms, 3_000);
    }
}
