use crate::fetcher::models::Tick;

use super::eval::get_field;

pub fn sma(values: &[f64], period: usize) -> f64 {
    if values.is_empty() || period == 0 {
        return 0.0;
    }
    let n = values.len().min(period);
    let sum: f64 = values[values.len() - n..].iter().sum();
    sum / n as f64
}

pub fn ema(values: &[f64], period: usize) -> f64 {
    if values.is_empty() || period == 0 {
        return 0.0;
    }
    let k = 2.0 / (period as f64 + 1.0);
    let mut result = values[0];
    for v in &values[1..] {
        result = v * k + result * (1.0 - k);
    }
    result
}

pub fn rsi(values: &[f64], period: usize) -> f64 {
    if values.len() < 2 || period == 0 {
        return 50.0;
    }
    let changes: Vec<f64> = values.windows(2).map(|w| w[1] - w[0]).collect();
    let n = changes.len().min(period);
    let recent = &changes[changes.len() - n..];
    let avg_gain: f64 = recent.iter().filter(|&&c| c > 0.0).sum::<f64>() / n as f64;
    let avg_loss: f64 = recent
        .iter()
        .filter(|&&c| c < 0.0)
        .map(|c| c.abs())
        .sum::<f64>()
        / n as f64;
    if avg_loss < f64::EPSILON {
        return 100.0;
    }
    if avg_gain < f64::EPSILON {
        return 0.0;
    }
    let rs = avg_gain / avg_loss;
    100.0 - 100.0 / (1.0 + rs)
}

pub fn vwap(ticks: &[Tick], field: &str) -> f64 {
    let mut sum_pv = 0.0;
    let mut sum_v = 0.0;
    for t in ticks {
        let price = get_field(t, field).unwrap_or(0.0);
        let vol = t.market_volume_usd as f64;
        sum_pv += price * vol;
        sum_v += vol;
    }
    if sum_v > 0.0 {
        sum_pv / sum_v
    } else {
        0.0
    }
}

pub fn cross_above(prev_a: f64, curr_a: f64, prev_b: f64, curr_b: f64) -> bool {
    prev_a <= prev_b && curr_a > curr_b
}

pub fn cross_below(prev_a: f64, curr_a: f64, prev_b: f64, curr_b: f64) -> bool {
    prev_a >= prev_b && curr_a < curr_b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_basic() {
        assert!((sma(&[1.0, 2.0, 3.0, 4.0, 5.0], 3) - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_sma_period_larger_than_data() {
        assert!((sma(&[2.0, 4.0], 10) - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_sma_empty() {
        assert!((sma(&[], 5)).abs() < 0.001);
    }

    #[test]
    fn test_ema_basic() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = ema(&values, 3);
        // EMA(3): k = 2/(3+1) = 0.5
        // ema0=1, ema1=1.5, ema2=2.25, ema3=3.125, ema4=4.0625
        assert!((result - 4.0625).abs() < 0.001);
    }

    #[test]
    fn test_ema_single_value() {
        assert!((ema(&[42.0], 5) - 42.0).abs() < 0.001);
    }

    #[test]
    fn test_rsi_all_gains() {
        assert!((rsi(&[1.0, 2.0, 3.0, 4.0, 5.0], 4) - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_rsi_all_losses() {
        assert!((rsi(&[5.0, 4.0, 3.0, 2.0, 1.0], 4) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_rsi_mixed() {
        let values = vec![10.0, 11.0, 10.0, 11.0, 10.0];
        let result = rsi(&values, 4);
        assert!((result - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_rsi_insufficient_data() {
        assert!((rsi(&[5.0], 14) - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_cross_above() {
        assert!(cross_above(0.4, 0.6, 0.5, 0.5));
        assert!(!cross_above(0.6, 0.7, 0.5, 0.5));
        assert!(!cross_above(0.4, 0.3, 0.5, 0.5));
    }

    #[test]
    fn test_cross_below() {
        assert!(cross_below(0.6, 0.4, 0.5, 0.5));
        assert!(!cross_below(0.4, 0.3, 0.5, 0.5));
    }
}
