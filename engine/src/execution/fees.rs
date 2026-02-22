use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::proxy::HttpPool;

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
    http: HttpPool,
    clob_url: String,
}

#[derive(Deserialize)]
struct FeeRateResponse {
    fee_rate_bps: u16,
}

impl FeeCache {
    pub fn new(http: HttpPool, clob_url: &str) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            http,
            clob_url: clob_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn get_fee(&self, token_id: &str) -> Result<u16> {
        // Check cache first (read lock).
        {
            let cache = self.cache.read().await;
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
            .proxied()
            .get(&url)
            .send()
            .await
            .context("fee-rate HTTP request failed")?
            .json()
            .await
            .context("failed to parse fee-rate response")?;

        // Write to cache.
        {
            let mut cache = self.cache.write().await;
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
    pub async fn set_fee(&self, token_id: &str, fee: u16) {
        let mut cache = self.cache.write().await;
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
        let pool = HttpPool::new(&[], std::time::Duration::from_secs(10)).unwrap();
        FeeCache::new(pool, "http://localhost")
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = make_cache();
        cache.set_fee("token_abc", 200).await;

        let inner = cache.cache.read().await;
        let entry = inner.get("token_abc").expect("entry should exist");
        assert_eq!(entry.fee_rate_bps, 200);
        assert!(entry.is_fresh());
    }

    #[tokio::test]
    async fn test_cache_miss_returns_none() {
        let cache = make_cache();

        let inner = cache.cache.read().await;
        assert!(inner.get("nonexistent_token").is_none());
    }

    #[tokio::test]
    async fn test_cache_expiry() {
        let cache = make_cache();

        // Insert an entry with fetched_at 120 seconds in the past.
        {
            let mut inner = cache.cache.write().await;
            inner.insert(
                "token_old".to_string(),
                CachedFee {
                    fee_rate_bps: 150,
                    fetched_at: Instant::now() - Duration::from_secs(120),
                },
            );
        }

        let inner = cache.cache.read().await;
        let entry = inner.get("token_old").expect("entry should exist");
        assert_eq!(entry.fee_rate_bps, 150);
        assert!(!entry.is_fresh(), "entry should be expired after 120s (TTL is 60s)");
    }
}
