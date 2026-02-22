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
    // Execution config
    pub database_url: String,
    pub clob_api_url: String,
    pub data_api_url: String,
    pub builder_api_key: String,
    pub builder_secret: String,
    pub builder_passphrase: String,
    pub encryption_key: String,
    pub max_orders_per_day: u32,
    pub neg_risk: bool,
    pub api_port: u16,
    pub proxy_urls: Vec<String>,
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
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://craftstrat:craftstrat_secret@localhost:5432/craftstrat".into()),
            clob_api_url: std::env::var("POLYMARKET_CLOB_URL")
                .unwrap_or_else(|_| "https://clob.polymarket.com".into()),
            data_api_url: std::env::var("POLYMARKET_DATA_API_URL")
                .unwrap_or_else(|_| "https://data-api.polymarket.com".into()),
            builder_api_key: std::env::var("POLYMARKET_BUILDER_API_KEY")
                .unwrap_or_default(),
            builder_secret: std::env::var("POLYMARKET_BUILDER_SECRET")
                .unwrap_or_default(),
            builder_passphrase: std::env::var("POLYMARKET_BUILDER_PASSPHRASE")
                .unwrap_or_default(),
            encryption_key: std::env::var("ENCRYPTION_KEY")
                .unwrap_or_default(),
            max_orders_per_day: std::env::var("ENGINE_MAX_ORDERS_PER_DAY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            neg_risk: std::env::var("ENGINE_NEG_RISK")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(true), // updown markets use NegRiskCtfExchange by default
            api_port: std::env::var("INTERNAL_API_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            proxy_urls: std::env::var("PROXY_LIST")
                .ok()
                .map(|v| {
                    v.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
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
