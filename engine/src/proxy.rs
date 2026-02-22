use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// HttpPool — round-robin proxy rotation for reqwest clients
// ---------------------------------------------------------------------------

struct HttpPoolInner {
    direct: reqwest::Client,
    proxied: Vec<reqwest::Client>,
    counter: AtomicUsize,
}

/// A pool of HTTP clients with optional proxy rotation.
///
/// Polymarket blocks certain geolocations, so requests to their APIs
/// (CLOB, Gamma, Data API) are routed through rotating proxies.
/// Binance and internal calls use the direct (non-proxied) client.
#[derive(Clone)]
pub struct HttpPool {
    inner: Arc<HttpPoolInner>,
}

impl HttpPool {
    /// Build a pool from a list of proxy URLs.
    ///
    /// Each proxy gets its own `reqwest::Client` so connections are
    /// established through different exit IPs. If `proxy_urls` is empty,
    /// `proxied()` falls back to the direct client.
    pub fn new(proxy_urls: &[String], timeout: Duration) -> anyhow::Result<Self> {
        let direct = reqwest::Client::builder()
            .timeout(timeout)
            .build()?;

        let mut proxied = Vec::with_capacity(proxy_urls.len());
        for url in proxy_urls {
            let proxy = reqwest::Proxy::all(url)
                .map_err(|e| anyhow::anyhow!("invalid proxy URL {url}: {e}"))?;
            let client = reqwest::Client::builder()
                .timeout(timeout)
                .proxy(proxy)
                .build()?;
            proxied.push(client);
        }

        if !proxied.is_empty() {
            tracing::info!(count = proxied.len(), "proxy_pool_initialized");
        }

        Ok(Self {
            inner: Arc::new(HttpPoolInner {
                direct,
                proxied,
                counter: AtomicUsize::new(0),
            }),
        })
    }

    /// Get a client for Polymarket API calls (round-robin proxied).
    ///
    /// Falls back to the direct client when no proxies are configured.
    pub fn proxied(&self) -> &reqwest::Client {
        if self.inner.proxied.is_empty() {
            return &self.inner.direct;
        }
        let idx = self.inner.counter.fetch_add(1, Ordering::Relaxed) % self.inner.proxied.len();
        &self.inner.proxied[idx]
    }

    /// Get the direct (non-proxied) client for Binance / internal calls.
    pub fn direct(&self) -> &reqwest::Client {
        &self.inner.direct
    }

    /// Number of proxies in the pool.
    pub fn proxy_count(&self) -> usize {
        self.inner.proxied.len()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_proxies_falls_back_to_direct() {
        let pool = HttpPool::new(&[], Duration::from_secs(10)).unwrap();
        assert_eq!(pool.proxy_count(), 0);
        // proxied() should not panic and return the direct client
        let _ = pool.proxied();
    }

    #[test]
    fn test_round_robin_cycles() {
        // Use dummy URLs — we won't actually send requests
        let proxies = vec![
            "http://proxy1.example.com:8080".to_string(),
            "http://proxy2.example.com:8080".to_string(),
            "http://proxy3.example.com:8080".to_string(),
        ];
        let pool = HttpPool::new(&proxies, Duration::from_secs(10)).unwrap();
        assert_eq!(pool.proxy_count(), 3);

        // Verify round-robin by checking the counter advances
        let _ = pool.proxied(); // idx 0
        let _ = pool.proxied(); // idx 1
        let _ = pool.proxied(); // idx 2
        let _ = pool.proxied(); // idx 0 again (wraps)

        let counter = pool.inner.counter.load(Ordering::Relaxed);
        assert_eq!(counter, 4);
    }

    #[test]
    fn test_clone_shares_state() {
        let proxies = vec![
            "http://proxy1.example.com:8080".to_string(),
            "http://proxy2.example.com:8080".to_string(),
        ];
        let pool = HttpPool::new(&proxies, Duration::from_secs(10)).unwrap();
        let pool2 = pool.clone();

        let _ = pool.proxied(); // counter -> 1
        let _ = pool2.proxied(); // counter -> 2 (shared Arc)

        let counter = pool.inner.counter.load(Ordering::Relaxed);
        assert_eq!(counter, 2);
    }
}
