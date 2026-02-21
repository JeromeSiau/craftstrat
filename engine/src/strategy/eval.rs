use crate::fetcher::models::Tick;

/// Extract a numeric field value from a Tick by name.
/// Supports all stateless indicators from the spec + aliases.
pub fn get_field(tick: &Tick, name: &str) -> Option<f64> {
    match name {
        "abs_move_pct" => Some(tick.abs_move_pct as f64),
        "dir_move_pct" => Some(tick.dir_move_pct as f64),
        "spread_up" => Some(tick.spread_up as f64),
        "spread_down" => Some(tick.spread_down as f64),
        "size_ratio_up" => Some(tick.size_ratio_up as f64),
        "size_ratio_down" => Some(tick.size_ratio_down as f64),
        "pct_into_slot" => Some(tick.pct_into_slot as f64),
        "minutes_into_slot" => Some(tick.minutes_into_slot as f64),
        "mid_up" => Some(tick.mid_up as f64),
        "mid_down" => Some(tick.mid_down as f64),
        "bid_up" => Some(tick.bid_up as f64),
        "ask_up" => Some(tick.ask_up as f64),
        "bid_down" => Some(tick.bid_down as f64),
        "ask_down" => Some(tick.ask_down as f64),
        "bid_size_up" => Some(tick.bid_size_up as f64),
        "ask_size_up" => Some(tick.ask_size_up as f64),
        "bid_size_down" => Some(tick.bid_size_down as f64),
        "ask_size_down" => Some(tick.ask_size_down as f64),
        "bid_up_l2" => Some(tick.bid_up_l2 as f64),
        "ask_up_l2" => Some(tick.ask_up_l2 as f64),
        "bid_up_l3" => Some(tick.bid_up_l3 as f64),
        "ask_up_l3" => Some(tick.ask_up_l3 as f64),
        "bid_down_l2" => Some(tick.bid_down_l2 as f64),
        "ask_down_l2" => Some(tick.ask_down_l2 as f64),
        "bid_down_l3" => Some(tick.bid_down_l3 as f64),
        "ask_down_l3" => Some(tick.ask_down_l3 as f64),
        "ref_price" | "chainlink_price" => Some(tick.ref_price as f64),
        "hour_utc" => Some(tick.hour_utc as f64),
        "day_of_week" => Some(tick.day_of_week as f64),
        "market_volume_usd" => Some(tick.market_volume_usd as f64),
        _ => None,
    }
}

/// Evaluate a comparison operator between a value and a JSON target.
pub fn evaluate_op(value: f64, operator: &str, target: &serde_json::Value) -> bool {
    match operator {
        ">" => target.as_f64().map_or(false, |t| value > t),
        ">=" => target.as_f64().map_or(false, |t| value >= t),
        "<" => target.as_f64().map_or(false, |t| value < t),
        "<=" => target.as_f64().map_or(false, |t| value <= t),
        "==" => target.as_f64().map_or(false, |t| (value - t).abs() < 1e-6),
        "!=" => target.as_f64().map_or(false, |t| (value - t).abs() >= 1e-6),
        "between" => {
            if let Some(arr) = target.as_array() {
                let lo = arr.first().and_then(|v| v.as_f64()).unwrap_or(f64::MIN);
                let hi = arr.get(1).and_then(|v| v.as_f64()).unwrap_or(f64::MAX);
                value >= lo && value <= hi
            } else {
                false
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_field_stateless() {
        let tick = test_tick();
        assert!((get_field(&tick, "abs_move_pct").unwrap() - 1.0).abs() < 0.001);
        assert!((get_field(&tick, "pct_into_slot").unwrap() - 0.5).abs() < 0.001);
        assert!((get_field(&tick, "spread_up").unwrap() - 0.02).abs() < 0.001);
        assert!((get_field(&tick, "mid_up").unwrap() - 0.61).abs() < 0.001);
        assert!((get_field(&tick, "hour_utc").unwrap() - 14.0).abs() < 0.001);
    }

    #[test]
    fn test_get_field_aliases() {
        let tick = test_tick();
        let a = get_field(&tick, "ref_price").unwrap();
        let b = get_field(&tick, "chainlink_price").unwrap();
        assert!((a - b).abs() < 0.001);
    }

    #[test]
    fn test_get_field_l2_l3() {
        let tick = test_tick();
        assert!((get_field(&tick, "bid_up_l2").unwrap() - 0.58).abs() < 0.001);
        assert!((get_field(&tick, "ask_up_l2").unwrap() - 0.65).abs() < 0.001);
        assert!((get_field(&tick, "bid_up_l3").unwrap() - 0.55).abs() < 0.001);
        assert!((get_field(&tick, "bid_down_l2").unwrap() - 0.36).abs() < 0.001);
        assert!((get_field(&tick, "ask_down_l3").unwrap() - 0.44).abs() < 0.001);
    }

    #[test]
    fn test_get_field_unknown() {
        let tick = test_tick();
        assert!(get_field(&tick, "nonexistent").is_none());
    }

    #[test]
    fn test_evaluate_op_gt() {
        assert!(evaluate_op(5.0, ">", &serde_json::json!(3.0)));
        assert!(!evaluate_op(3.0, ">", &serde_json::json!(5.0)));
    }

    #[test]
    fn test_evaluate_op_between() {
        assert!(evaluate_op(0.3, "between", &serde_json::json!([0.1, 0.5])));
        assert!(!evaluate_op(0.8, "between", &serde_json::json!([0.1, 0.5])));
        assert!(evaluate_op(0.1, "between", &serde_json::json!([0.1, 0.5])));
    }

    #[test]
    fn test_evaluate_op_all_operators() {
        assert!(evaluate_op(5.0, ">=", &serde_json::json!(5.0)));
        assert!(evaluate_op(3.0, "<", &serde_json::json!(5.0)));
        assert!(evaluate_op(3.0, "<=", &serde_json::json!(3.0)));
        assert!(evaluate_op(3.0, "==", &serde_json::json!(3.0)));
        assert!(evaluate_op(3.0, "!=", &serde_json::json!(5.0)));
    }

    fn test_tick() -> Tick {
        Tick {
            captured_at: time::OffsetDateTime::from_unix_timestamp(1700000450).unwrap(),
            symbol: "btc-updown-15m-1700000000".into(),
            slot_ts: 1700000000,
            slot_duration: 900,
            minutes_into_slot: 7.5,
            pct_into_slot: 0.5,
            bid_up: 0.60,
            ask_up: 0.62,
            bid_down: 0.38,
            ask_down: 0.40,
            bid_size_up: 100.0,
            ask_size_up: 80.0,
            bid_size_down: 90.0,
            ask_size_down: 70.0,
            spread_up: 0.02,
            spread_down: 0.02,
            bid_up_l2: 0.58,
            ask_up_l2: 0.65,
            bid_up_l3: 0.55,
            ask_up_l3: 0.68,
            bid_down_l2: 0.36,
            ask_down_l2: 0.42,
            bid_down_l3: 0.34,
            ask_down_l3: 0.44,
            mid_up: 0.61,
            mid_down: 0.39,
            size_ratio_up: 1.25,
            size_ratio_down: 1.29,
            ref_price: 50500.0,
            dir_move_pct: 1.0,
            abs_move_pct: 1.0,
            hour_utc: 14,
            day_of_week: 2,
            market_volume_usd: 0.0,
            winner: None,
            ref_price_start: 50000.0,
            ref_price_end: 50500.0,
            ref_price_source: "binance".into(),
        }
    }
}
