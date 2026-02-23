use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::proxy::HttpPool;
use crate::strategy::registry::AssignmentRegistry;

// ---------------------------------------------------------------------------
// ApiFetchCache — shared cache for external API values
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct CacheEntry {
    value: f64,
    updated_at: Instant,
}

#[derive(Clone)]
pub struct ApiFetchCache(Arc<RwLock<HashMap<String, CacheEntry>>>);

impl ApiFetchCache {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    /// Read a cached value. Returns 0.0 if not found or expired.
    pub fn get(&self, key: &str, max_age_secs: u64) -> f64 {
        let guard = self.0.read().unwrap_or_else(|e| e.into_inner());
        guard
            .get(key)
            .filter(|entry| entry.updated_at.elapsed().as_secs() < max_age_secs)
            .map(|entry| entry.value)
            .unwrap_or(0.0)
    }

    pub(crate) fn set(&self, key: String, value: f64) {
        let mut guard = self.0.write().unwrap_or_else(|e| e.into_inner());
        guard.insert(
            key,
            CacheEntry {
                value,
                updated_at: Instant::now(),
            },
        );
    }
}

// ---------------------------------------------------------------------------
// FetchTarget — a unique URL + json_path to poll
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct FetchTarget {
    url: String,
    json_path: String,
    interval_secs: u64,
}

impl FetchTarget {
    fn cache_key(&self) -> String {
        format!("{}#{}", self.url, self.json_path)
    }
}

// ---------------------------------------------------------------------------
// Background polling task
// ---------------------------------------------------------------------------

pub async fn run(
    registry: AssignmentRegistry,
    cache: ApiFetchCache,
    http: HttpPool,
) -> anyhow::Result<()> {
    tracing::info!("api_fetch_task_started");
    let mut last_fetch: HashMap<String, Instant> = HashMap::new();

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        // Collect unique fetch targets from all active assignments
        let targets = collect_targets(&registry).await;
        if targets.is_empty() {
            continue;
        }

        for target in &targets {
            let key = target.cache_key();
            let should_fetch = last_fetch
                .get(&key)
                .map(|t| t.elapsed().as_secs() >= target.interval_secs)
                .unwrap_or(true);

            if !should_fetch {
                continue;
            }

            let client = http.direct();
            match fetch_value(&client, &target.url, &target.json_path).await {
                Ok(value) => {
                    cache.set(key.clone(), value);
                    last_fetch.insert(key, Instant::now());
                    tracing::debug!(url = %target.url, value, "api_fetch_updated");
                }
                Err(e) => {
                    tracing::warn!(url = %target.url, error = %e, "api_fetch_failed");
                    // Keep old cached value — it will expire via max_age
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// collect_targets — scan registry for api_fetch nodes
// ---------------------------------------------------------------------------

async fn collect_targets(registry: &AssignmentRegistry) -> Vec<FetchTarget> {
    let reg = registry.read().await;
    let mut seen = std::collections::HashSet::new();
    let mut targets = Vec::new();

    for assignments in reg.values() {
        for a in assignments {
            if let Some(nodes) = a.graph["nodes"].as_array() {
                for node in nodes {
                    if node["type"].as_str() != Some("api_fetch") {
                        continue;
                    }
                    let data = &node["data"];
                    let url = data["url"].as_str().unwrap_or("").to_string();
                    let json_path = data["json_path"].as_str().unwrap_or("").to_string();
                    let interval_secs = data["interval_secs"].as_u64().unwrap_or(60).max(30);

                    if url.is_empty() || json_path.is_empty() {
                        continue;
                    }

                    let key = format!("{}#{}", url, json_path);
                    if seen.insert(key) {
                        targets.push(FetchTarget {
                            url,
                            json_path,
                            interval_secs,
                        });
                    }
                }
            }
        }
    }
    targets
}

// ---------------------------------------------------------------------------
// fetch_value — HTTP GET + JSONPath extraction
// ---------------------------------------------------------------------------

async fn fetch_value(
    client: &reqwest::Client,
    url: &str,
    json_path: &str,
) -> anyhow::Result<f64> {
    let resp = client
        .get(url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;

    let body = resp.bytes().await?;
    if body.len() > 10_240 {
        anyhow::bail!("response too large: {} bytes", body.len());
    }

    let json: serde_json::Value = serde_json::from_slice(&body)?;
    extract_json_path(&json, json_path)
        .ok_or_else(|| anyhow::anyhow!("json_path '{}' not found or not numeric", json_path))
}

/// Simple JSONPath extraction supporting dot notation (e.g. "main.temp", "data.0.price").
fn extract_json_path(value: &serde_json::Value, path: &str) -> Option<f64> {
    let path = path.strip_prefix("$.").unwrap_or(path);
    let mut current = value;
    for segment in path.split('.') {
        if let Ok(index) = segment.parse::<usize>() {
            current = current.get(index)?;
        } else {
            current = current.get(segment)?;
        }
    }
    current.as_f64()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_get_set() {
        let cache = ApiFetchCache::new();
        cache.set("key1".into(), 42.5);
        assert!((cache.get("key1", 60) - 42.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_miss_returns_zero() {
        let cache = ApiFetchCache::new();
        assert!((cache.get("missing", 60)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_extract_json_path_simple() {
        let json = serde_json::json!({"main": {"temp": 22.5}});
        assert_eq!(extract_json_path(&json, "main.temp"), Some(22.5));
    }

    #[test]
    fn test_extract_json_path_with_dollar_prefix() {
        let json = serde_json::json!({"data": {"price": 1.5}});
        assert_eq!(extract_json_path(&json, "$.data.price"), Some(1.5));
    }

    #[test]
    fn test_extract_json_path_array_index() {
        let json = serde_json::json!({"data": [{"price": 10.0}, {"price": 20.0}]});
        assert_eq!(extract_json_path(&json, "data.1.price"), Some(20.0));
    }

    #[test]
    fn test_extract_json_path_not_found() {
        let json = serde_json::json!({"foo": "bar"});
        assert_eq!(extract_json_path(&json, "baz.qux"), None);
    }

    #[test]
    fn test_extract_json_path_not_numeric() {
        let json = serde_json::json!({"name": "hello"});
        assert_eq!(extract_json_path(&json, "name"), None);
    }

    #[test]
    fn test_fetch_target_cache_key() {
        let target = FetchTarget {
            url: "https://api.example.com/data".into(),
            json_path: "main.temp".into(),
            interval_secs: 60,
        };
        assert_eq!(
            target.cache_key(),
            "https://api.example.com/data#main.temp"
        );
    }
}
