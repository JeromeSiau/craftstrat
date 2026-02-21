use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::Deserialize;

const CACHE_TTL: Duration = Duration::from_secs(60);

// ---------------------------------------------------------------------------
// CachedFee
// ---------------------------------------------------------------------------

struct CachedFee {
    fee_rate_bps: u16,
    fetched_at: Instant,
}

impl CachedFee {
    fn is_fresh(&self) -> bool {
        self.fetched_at.elapsed() < CACHE_TTL
    }
}

// ---------------------------------------------------------------------------
// FeeCache
// ---------------------------------------------------------------------------

pub struct FeeCache {
    cache: RwLock<HashMap<String, CachedFee>>,
    http: reqwest::Client,
    clob_url: String,
}

#[derive(Deserialize)]
struct FeeRateResponse {
    fee_rate_bps: u16,
}

impl FeeCache {
    pub fn new(http: reqwest::Client, clob_url: &str) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            http,
            clob_url: clob_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn get_fee(&self, token_id: &str) -> Result<u16> {
        // Check cache first (read lock).
        {
            let cache = self.cache.read().unwrap();
            if let Some(entry) = cache.get(token_id) {
                if entry.is_fresh() {
                    return Ok(entry.fee_rate_bps);
                }
            }
        }

        // Cache miss or stale — fetch from CLOB API.
        let url = format!("{}/fee-rate?token_id={}", self.clob_url, token_id);
        let resp: FeeRateResponse = self
            .http
            .get(&url)
            .send()
            .await
            .context("fee-rate HTTP request failed")?
            .json()
            .await
            .context("failed to parse fee-rate response")?;

        // Write to cache.
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(
                token_id.to_string(),
                CachedFee {
                    fee_rate_bps: resp.fee_rate_bps,
                    fetched_at: Instant::now(),
                },
            );
        }

        Ok(resp.fee_rate_bps)
    }

    /// Test helper — manually insert a fee into the cache.
    #[cfg(test)]
    pub fn set_fee(&self, token_id: &str, fee: u16) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(
            token_id.to_string(),
            CachedFee {
                fee_rate_bps: fee,
                fetched_at: Instant::now(),
            },
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cache() -> FeeCache {
        FeeCache::new(reqwest::Client::new(), "http://localhost")
    }

    #[test]
    fn test_cache_hit() {
        let cache = make_cache();
        cache.set_fee("token_abc", 200);

        let inner = cache.cache.read().unwrap();
        let entry = inner.get("token_abc").expect("entry should exist");
        assert_eq!(entry.fee_rate_bps, 200);
        assert!(entry.is_fresh());
    }

    #[test]
    fn test_cache_miss_returns_none() {
        let cache = make_cache();

        let inner = cache.cache.read().unwrap();
        assert!(inner.get("nonexistent_token").is_none());
    }

    #[test]
    fn test_cache_expiry() {
        let cache = make_cache();

        // Insert an entry with fetched_at 120 seconds in the past.
        {
            let mut inner = cache.cache.write().unwrap();
            inner.insert(
                "token_old".to_string(),
                CachedFee {
                    fee_rate_bps: 150,
                    fetched_at: Instant::now() - Duration::from_secs(120),
                },
            );
        }

        let inner = cache.cache.read().unwrap();
        let entry = inner.get("token_old").expect("entry should exist");
        assert_eq!(entry.fee_rate_bps, 150);
        assert!(!entry.is_fresh(), "entry should be expired after 120s (TTL is 60s)");
    }
}
