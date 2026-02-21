pub mod metrics;
pub mod runner;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

use crate::strategy::Outcome;

#[derive(Debug, Clone)]
pub struct BacktestRequest {
    pub strategy_graph: Value,
    pub market_filter: Vec<String>,
    pub date_from: OffsetDateTime,
    pub date_to: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub total_trades: u32,
    pub win_rate: f64,
    pub total_pnl_usdc: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub trades: Vec<BacktestTrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestTrade {
    pub market_id: String,
    pub outcome: Outcome,
    pub side: String,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub size_usdc: f64,
    pub pnl_usdc: f64,
    #[serde(with = "time::serde::rfc3339")]
    pub entry_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub exit_at: Option<OffsetDateTime>,
    pub exit_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::Outcome;

    #[test]
    fn test_backtest_result_serialization_roundtrip() {
        let result = BacktestResult {
            total_trades: 5,
            win_rate: 0.6,
            total_pnl_usdc: 42.5,
            max_drawdown: 0.15,
            sharpe_ratio: 1.2,
            trades: vec![BacktestTrade {
                market_id: "btc-updown-15m-1700000000".into(),
                outcome: Outcome::Up,
                side: "buy".into(),
                entry_price: 0.62,
                exit_price: Some(0.68),
                size_usdc: 50.0,
                pnl_usdc: 4.84,
                entry_at: OffsetDateTime::from_unix_timestamp(1700000450).unwrap(),
                exit_at: Some(OffsetDateTime::from_unix_timestamp(1700000900).unwrap()),
                exit_reason: Some("take_profit".into()),
            }],
        };
        let json = serde_json::to_string(&result).unwrap();
        let restored: BacktestResult = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.total_trades, 5);
        assert!((restored.win_rate - 0.6).abs() < f64::EPSILON);
        assert_eq!(restored.trades.len(), 1);
        assert_eq!(restored.trades[0].outcome, Outcome::Up);
    }
}
