pub mod metrics;
pub mod runner;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

use crate::strategy::Outcome;

const DEFAULT_WINDOW_SIZE: usize = 200;

#[derive(Debug, Clone, Deserialize)]
pub struct BacktestRequest {
    pub strategy_graph: Value,
    pub market_filter: Vec<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub date_from: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub date_to: OffsetDateTime,
    #[serde(default = "default_window_size")]
    pub window_size: usize,
}

fn default_window_size() -> usize {
    DEFAULT_WINDOW_SIZE
}

impl BacktestRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.date_from >= self.date_to {
            return Err("date_from must be before date_to");
        }
        if self.window_size == 0 {
            return Err("window_size must be > 0");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub total_trades: u32,
    pub win_rate: f64,
    pub total_pnl_usdc: f64,
    pub max_drawdown: f64,
    /// Per-trade Sharpe ratio (not annualized). Risk-free rate assumed 0.
    pub sharpe_ratio: f64,
    pub trades: Vec<BacktestTrade>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    Stoploss,
    TakeProfit,
    Signal,
    EndOfData,
    SlotResolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestTrade {
    pub market_id: String,
    pub outcome: Outcome,
    pub side: Side,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub size_usdc: f64,
    pub pnl_usdc: f64,
    #[serde(with = "time::serde::rfc3339")]
    pub entry_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub exit_at: Option<OffsetDateTime>,
    pub exit_reason: Option<ExitReason>,
}

pub fn compute_pnl(entry_price: f64, exit_price: f64, size_usdc: f64) -> f64 {
    if entry_price <= 0.0 {
        return 0.0;
    }
    (exit_price - entry_price) / entry_price * size_usdc
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
                side: Side::Buy,
                entry_price: 0.62,
                exit_price: Some(0.68),
                size_usdc: 50.0,
                pnl_usdc: 4.84,
                entry_at: OffsetDateTime::from_unix_timestamp(1700000450).unwrap(),
                exit_at: Some(OffsetDateTime::from_unix_timestamp(1700000900).unwrap()),
                exit_reason: Some(ExitReason::TakeProfit),
            }],
        };
        let json = serde_json::to_string(&result).unwrap();
        let restored: BacktestResult = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.total_trades, 5);
        assert!((restored.win_rate - 0.6).abs() < f64::EPSILON);
        assert_eq!(restored.trades.len(), 1);
        assert_eq!(restored.trades[0].outcome, Outcome::Up);
        assert_eq!(restored.trades[0].side, Side::Buy);
        assert_eq!(restored.trades[0].exit_reason, Some(ExitReason::TakeProfit));
    }

    #[test]
    fn test_validate_empty_market_filter_is_ok() {
        let req = BacktestRequest {
            strategy_graph: serde_json::json!({}),
            market_filter: vec![],
            date_from: OffsetDateTime::from_unix_timestamp(1700000000).unwrap(),
            date_to: OffsetDateTime::from_unix_timestamp(1700001000).unwrap(),
            window_size: 200,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_date_range() {
        let req = BacktestRequest {
            strategy_graph: serde_json::json!({}),
            market_filter: vec!["btc".into()],
            date_from: OffsetDateTime::from_unix_timestamp(1700001000).unwrap(),
            date_to: OffsetDateTime::from_unix_timestamp(1700000000).unwrap(),
            window_size: 200,
        };
        assert_eq!(req.validate(), Err("date_from must be before date_to"));
    }

    #[test]
    fn test_compute_pnl() {
        // Buy at 0.50, sell at 0.57, size 100 → (0.57-0.50)/0.50*100 = 14.0
        let pnl = compute_pnl(0.50, 0.57, 100.0);
        assert!((pnl - 14.0).abs() < 0.001);

        // Buy at 0.62, sell at 0.53, size 50 → (0.53-0.62)/0.62*50 = -7.26
        let pnl = compute_pnl(0.62, 0.53, 50.0);
        assert!(pnl < 0.0);
    }
}
