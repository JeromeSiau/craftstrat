use anyhow::Result;

#[derive(Debug, Clone)]
pub struct SymbolConfig {
    pub binance_symbol: String,
    pub slug_prefix: String,
    pub slot_durations: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub clickhouse_url: String,
    pub kafka_brokers: String,
    pub gamma_api_url: String,
    pub clob_ws_url: String,
    pub binance_api_url: String,
    pub symbols: Vec<SymbolConfig>,
    pub tick_interval_ms: u64,
    pub discovery_interval_secs: u64,
}

fn default_symbols() -> Vec<SymbolConfig> {
    vec![
        SymbolConfig {
            binance_symbol: "BTCUSDT".into(),
            slug_prefix: "btc".into(),
            slot_durations: vec![300, 900, 3600, 14400, 86400],
        },
        SymbolConfig {
            binance_symbol: "ETHUSDT".into(),
            slug_prefix: "eth".into(),
            slot_durations: vec![900],
        },
        SymbolConfig {
            binance_symbol: "SOLUSDT".into(),
            slug_prefix: "sol".into(),
            slot_durations: vec![900],
        },
        SymbolConfig {
            binance_symbol: "XRPUSDT".into(),
            slug_prefix: "xrp".into(),
            slot_durations: vec![900],
        },
    ]
}

fn parse_symbols(env_val: &str) -> Vec<SymbolConfig> {
    let defaults = default_symbols();
    let requested: Vec<&str> = env_val.split(',').map(|s| s.trim()).collect();
    defaults
        .into_iter()
        .filter(|s| requested.iter().any(|r| r.eq_ignore_ascii_case(&s.binance_symbol)))
        .collect()
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        let symbols = match std::env::var("ENGINE_SYMBOLS") {
            Ok(val) => parse_symbols(&val),
            Err(_) => default_symbols(),
        };
        if symbols.is_empty() {
            anyhow::bail!("ENGINE_SYMBOLS matched no known symbols");
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
            symbols,
            tick_interval_ms: 1000,
            discovery_interval_secs: 60,
        })
    }

    pub fn binance_symbols(&self) -> Vec<String> {
        let mut syms: Vec<String> = self.symbols.iter().map(|s| s.binance_symbol.clone()).collect();
        syms.dedup();
        syms
    }
}
