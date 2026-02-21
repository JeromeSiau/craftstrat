use anyhow::Result;

#[derive(Debug, Clone)]
pub enum MarketSource {
    CryptoUpDown {
        binance_symbol: String,
        slug_prefix: String,
        slot_durations: Vec<u32>,
    },
    #[allow(dead_code)]
    Custom {
        slug: String,
        ref_price_symbol: Option<String>,
    },
}

impl MarketSource {
    pub fn binance_symbol(&self) -> Option<&str> {
        match self {
            Self::CryptoUpDown { binance_symbol, .. } => Some(binance_symbol),
            Self::Custom { ref_price_symbol, .. } => ref_price_symbol.as_deref(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub clickhouse_url: String,
    pub kafka_brokers: String,
    pub gamma_api_url: String,
    pub clob_ws_url: String,
    pub binance_api_url: String,
    pub redis_url: String,
    pub sources: Vec<MarketSource>,
    pub tick_interval_ms: u64,
    pub discovery_interval_secs: u64,
}

fn default_sources() -> Vec<MarketSource> {
    vec![
        MarketSource::CryptoUpDown {
            binance_symbol: "BTCUSDT".into(),
            slug_prefix: "btc".into(),
            slot_durations: vec![300, 900, 3600, 14400, 86400],
        },
        MarketSource::CryptoUpDown {
            binance_symbol: "ETHUSDT".into(),
            slug_prefix: "eth".into(),
            slot_durations: vec![900],
        },
        MarketSource::CryptoUpDown {
            binance_symbol: "SOLUSDT".into(),
            slug_prefix: "sol".into(),
            slot_durations: vec![900],
        },
        MarketSource::CryptoUpDown {
            binance_symbol: "XRPUSDT".into(),
            slug_prefix: "xrp".into(),
            slot_durations: vec![900],
        },
    ]
}

fn parse_sources(env_val: &str) -> Vec<MarketSource> {
    let defaults = default_sources();
    let requested: Vec<&str> = env_val.split(',').map(|s| s.trim()).collect();
    defaults
        .into_iter()
        .filter(|s| {
            s.binance_symbol()
                .map(|bs| requested.iter().any(|r| r.eq_ignore_ascii_case(bs)))
                .unwrap_or(false)
        })
        .collect()
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        let sources = match std::env::var("ENGINE_SYMBOLS") {
            Ok(val) => parse_sources(&val),
            Err(_) => default_sources(),
        };
        if sources.is_empty() {
            anyhow::bail!("ENGINE_SYMBOLS matched no known sources");
        }
        Ok(Self {
            clickhouse_url: std::env::var("CLICKHOUSE_URL")
                .unwrap_or_else(|_| "http://localhost:8123".into()),
            kafka_brokers: std::env::var("KAFKA_BROKERS")
                .unwrap_or_else(|_| "localhost:9092".into()),
            gamma_api_url: std::env::var("GAMMA_API_URL")
                .unwrap_or_else(|_| "https://gamma-api.polymarket.com".into()),
            clob_ws_url: std::env::var("CLOB_WS_URL")
                .unwrap_or_else(|_| "wss://ws-subscriptions-clob.polymarket.com/ws/market".into()),
            binance_api_url: std::env::var("BINANCE_API_URL")
                .unwrap_or_else(|_| "https://api.binance.com/api/v3/ticker/price".into()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
            sources,
            tick_interval_ms: 1000,
            discovery_interval_secs: 60,
        })
    }

    pub fn binance_symbols(&self) -> Vec<String> {
        let mut syms: Vec<String> = self
            .sources
            .iter()
            .filter_map(|s| s.binance_symbol().map(String::from))
            .collect();
        syms.sort();
        syms.dedup();
        syms
    }
}
