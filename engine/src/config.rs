use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub clickhouse_url: String,
    pub kafka_brokers: String,
    pub gamma_api_url: String,
    pub clob_ws_url: String,
    pub binance_api_url: String,
    pub symbols: Vec<String>,
    pub slot_duration: u32,
    pub tick_interval_ms: u64,
    pub discovery_interval_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
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
            symbols: std::env::var("ENGINE_SYMBOLS")
                .unwrap_or_else(|_| "BTCUSDT".into())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            slot_duration: std::env::var("SLOT_DURATION")
                .unwrap_or_else(|_| "900".into())
                .parse()
                .context("SLOT_DURATION must be u32")?,
            tick_interval_ms: 1000,
            discovery_interval_secs: 60,
        })
    }
}
