use super::Side;

pub fn fill_slippage_pct(
    side: Side,
    reference_price: Option<f64>,
    filled_price: Option<f64>,
) -> Option<f64> {
    let reference_price = reference_price?;
    let filled_price = filled_price?;
    if reference_price <= 0.0 || filled_price <= 0.0 {
        return None;
    }

    Some(match side {
        Side::Buy => (filled_price - reference_price) / reference_price,
        Side::Sell => (reference_price - filled_price) / reference_price,
    })
}

pub fn fill_slippage_bps(
    side: Side,
    reference_price: Option<f64>,
    filled_price: Option<f64>,
) -> Option<f64> {
    fill_slippage_pct(side, reference_price, filled_price).map(|pct| pct * 10_000.0)
}

pub fn markout_bps_60s(
    side: Side,
    filled_price: Option<f64>,
    markout_price: Option<f64>,
) -> Option<f64> {
    let filled_price = filled_price?;
    let markout_price = markout_price?;
    if filled_price <= 0.0 || markout_price <= 0.0 {
        return None;
    }

    Some(match side {
        Side::Buy => (markout_price - filled_price) / filled_price * 10_000.0,
        Side::Sell => (filled_price - markout_price) / filled_price * 10_000.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buy_fill_slippage_is_positive_when_fill_is_worse() {
        let bps = fill_slippage_bps(Side::Buy, Some(0.62), Some(0.625)).unwrap();

        assert!(bps > 0.0);
    }

    #[test]
    fn sell_fill_slippage_is_positive_when_fill_is_worse() {
        let bps = fill_slippage_bps(Side::Sell, Some(0.62), Some(0.615)).unwrap();

        assert!(bps > 0.0);
    }

    #[test]
    fn buy_markout_is_negative_when_price_falls_after_fill() {
        let bps = markout_bps_60s(Side::Buy, Some(0.62), Some(0.60)).unwrap();

        assert!(bps < 0.0);
    }

    #[test]
    fn sell_markout_is_positive_when_price_falls_after_fill() {
        let bps = markout_bps_60s(Side::Sell, Some(0.62), Some(0.60)).unwrap();

        assert!(bps > 0.0);
    }
}
