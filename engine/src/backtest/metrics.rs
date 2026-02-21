use super::{BacktestResult, BacktestTrade};

pub fn compute(trades: Vec<BacktestTrade>) -> BacktestResult {
    let closed: Vec<&BacktestTrade> = trades.iter().filter(|t| t.exit_price.is_some()).collect();
    let total_trades = closed.len() as u32;

    if total_trades == 0 {
        return BacktestResult {
            total_trades: 0,
            win_rate: 0.0,
            total_pnl_usdc: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            trades,
        };
    }

    let wins = closed.iter().filter(|t| t.pnl_usdc > 0.0).count();
    let win_rate = wins as f64 / total_trades as f64;
    let total_pnl_usdc: f64 = closed.iter().map(|t| t.pnl_usdc).sum();
    let max_drawdown = compute_max_drawdown(&closed);
    let sharpe_ratio = compute_sharpe(&closed);

    BacktestResult {
        total_trades,
        win_rate,
        total_pnl_usdc,
        max_drawdown,
        sharpe_ratio,
        trades,
    }
}

fn compute_max_drawdown(trades: &[&BacktestTrade]) -> f64 {
    let mut peak = 0.0_f64;
    let mut equity = 0.0_f64;
    let mut max_dd = 0.0_f64;

    for trade in trades {
        equity += trade.pnl_usdc;
        if equity > peak {
            peak = equity;
        }
        if peak > 0.0 {
            let dd = (peak - equity) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }
    max_dd
}

fn compute_sharpe(trades: &[&BacktestTrade]) -> f64 {
    if trades.len() < 2 {
        return 0.0;
    }
    let pnls: Vec<f64> = trades.iter().map(|t| t.pnl_usdc).collect();
    let n = pnls.len() as f64;
    let mean = pnls.iter().sum::<f64>() / n;
    let variance = pnls.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();
    if std_dev < f64::EPSILON {
        return 0.0;
    }
    mean / std_dev
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::Outcome;
    use time::OffsetDateTime;

    fn trade(pnl: f64, size: f64) -> BacktestTrade {
        BacktestTrade {
            market_id: "test".into(),
            outcome: Outcome::Up,
            side: "buy".into(),
            entry_price: 0.50,
            exit_price: Some(if pnl >= 0.0 { 0.50 + pnl / size * 0.50 } else { 0.50 + pnl / size * 0.50 }),
            size_usdc: size,
            pnl_usdc: pnl,
            entry_at: OffsetDateTime::from_unix_timestamp(1700000000).unwrap(),
            exit_at: Some(OffsetDateTime::from_unix_timestamp(1700000900).unwrap()),
            exit_reason: Some("signal".into()),
        }
    }

    #[test]
    fn test_compute_empty_trades() {
        let result = compute(vec![]);
        assert_eq!(result.total_trades, 0);
        assert!((result.win_rate).abs() < f64::EPSILON);
        assert!((result.total_pnl_usdc).abs() < f64::EPSILON);
        assert!((result.max_drawdown).abs() < f64::EPSILON);
        assert!((result.sharpe_ratio).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_single_win() {
        let result = compute(vec![trade(10.0, 100.0)]);
        assert_eq!(result.total_trades, 1);
        assert!((result.win_rate - 1.0).abs() < f64::EPSILON);
        assert!((result.total_pnl_usdc - 10.0).abs() < f64::EPSILON);
        assert!((result.max_drawdown).abs() < f64::EPSILON); // no drawdown with single win
    }

    #[test]
    fn test_compute_win_loss_win() {
        let result = compute(vec![
            trade(10.0, 100.0),
            trade(-5.0, 100.0),
            trade(8.0, 100.0),
        ]);
        assert_eq!(result.total_trades, 3);
        assert!((result.win_rate - 2.0 / 3.0).abs() < 0.001);
        assert!((result.total_pnl_usdc - 13.0).abs() < f64::EPSILON);
        // Equity curve: [0, 10, 5, 13] -> peak 10, trough 5, drawdown = 5/10 = 0.5
        assert!((result.max_drawdown - 0.5).abs() < 0.001);
        assert!(result.sharpe_ratio > 0.0); // positive overall
    }

    #[test]
    fn test_compute_all_losses() {
        let result = compute(vec![
            trade(-10.0, 100.0),
            trade(-5.0, 100.0),
        ]);
        assert_eq!(result.total_trades, 2);
        assert!((result.win_rate).abs() < f64::EPSILON);
        assert!((result.total_pnl_usdc - (-15.0)).abs() < f64::EPSILON);
        assert!(result.sharpe_ratio < 0.0); // negative sharpe
    }

    #[test]
    fn test_compute_ignores_unclosed_trades() {
        let mut unclosed = trade(0.0, 100.0);
        unclosed.exit_price = None;
        unclosed.exit_at = None;
        unclosed.exit_reason = None;

        let result = compute(vec![
            trade(10.0, 100.0),
            unclosed,
        ]);
        // Only closed trade counts
        assert_eq!(result.total_trades, 1);
        assert!((result.total_pnl_usdc - 10.0).abs() < f64::EPSILON);
    }
}
