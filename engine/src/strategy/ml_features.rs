use std::collections::VecDeque;

use serde_json::{Map, Value};
use time::format_description::well_known::Rfc3339;

use crate::fetcher::models::Tick;

pub const LIVE_FEATURE_WINDOW: usize = 5;

pub fn build_live_feature_row(window: &VecDeque<Tick>) -> Option<Value> {
    let tick = window.back()?;
    let prev = window.iter().rev().nth(1).unwrap_or(tick);

    let imbalance_up = imbalance(tick.bid_size_up as f64, tick.ask_size_up as f64);
    let imbalance_down = imbalance(tick.bid_size_down as f64, tick.ask_size_down as f64);
    let prev_imbalance_up = imbalance(prev.bid_size_up as f64, prev.ask_size_up as f64);

    let f_mid_up = tick.mid_up as f64;
    let f_mid_down = tick.mid_down as f64;
    let f_bid_up = tick.bid_up as f64;
    let f_ask_up = tick.ask_up as f64;
    let f_bid_down = tick.bid_down as f64;
    let f_ask_down = tick.ask_down as f64;
    let f_spread_up_rel = if tick.mid_up > 0.0 {
        tick.spread_up as f64 / tick.mid_up as f64
    } else {
        0.0
    };
    let f_spread_down_rel = if tick.mid_down > 0.0 {
        tick.spread_down as f64 / tick.mid_down as f64
    } else {
        0.0
    };

    if !(0.05..=0.90).contains(&(tick.pct_into_slot as f64))
        || tick.bid_up <= 0.0
        || tick.ask_up <= 0.0
        || tick.bid_down <= 0.0
        || tick.ask_down <= 0.0
        || !(0.0..=0.25).contains(&f_spread_up_rel)
        || !(0.0..=0.25).contains(&f_spread_down_rel)
    {
        return None;
    }

    let avg_mid_up =
        window.iter().map(|entry| entry.mid_up as f64).sum::<f64>() / window.len() as f64;
    let f_d_ref_1 = if tick.ref_price > 0.0 && prev.ref_price > 0.0 {
        (tick.ref_price as f64 / prev.ref_price as f64).ln()
    } else {
        0.0
    };

    let captured_at = tick
        .captured_at
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

    let mut row = Map::new();
    row.insert("captured_at".into(), Value::String(captured_at));
    row.insert("symbol".into(), Value::String(tick.symbol.clone()));
    row.insert("slot_ts".into(), Value::from(tick.slot_ts));
    row.insert("slot_duration".into(), Value::from(tick.slot_duration));
    row.insert("f_mid_up".into(), Value::from(f_mid_up));
    row.insert("f_mid_down".into(), Value::from(f_mid_down));
    row.insert("f_bid_up".into(), Value::from(f_bid_up));
    row.insert("f_ask_up".into(), Value::from(f_ask_up));
    row.insert("f_bid_down".into(), Value::from(f_bid_down));
    row.insert("f_ask_down".into(), Value::from(f_ask_down));
    row.insert("f_spread_up_rel".into(), Value::from(f_spread_up_rel));
    row.insert("f_spread_down_rel".into(), Value::from(f_spread_down_rel));
    row.insert(
        "f_cross_sum_mid".into(),
        Value::from(f_mid_up + f_mid_down - 1.0),
    );
    row.insert(
        "f_cross_sum_bid".into(),
        Value::from(f_bid_up + f_bid_down - 1.0),
    );
    row.insert(
        "f_cross_sum_ask".into(),
        Value::from(f_ask_up + f_ask_down - 1.0),
    );
    row.insert(
        "f_parity_gap_up".into(),
        Value::from(f_mid_up - (1.0 - f_mid_down)),
    );
    row.insert("f_l1_imbalance_up".into(), Value::from(imbalance_up));
    row.insert("f_l1_imbalance_down".into(), Value::from(imbalance_down));
    row.insert(
        "f_size_ratio_up".into(),
        Value::from(tick.size_ratio_up as f64),
    );
    row.insert(
        "f_size_ratio_down".into(),
        Value::from(tick.size_ratio_down as f64),
    );
    row.insert(
        "f_bid_gap_up_12".into(),
        Value::from((tick.bid_up - tick.bid_up_l2) as f64),
    );
    row.insert(
        "f_bid_gap_up_23".into(),
        Value::from((tick.bid_up_l2 - tick.bid_up_l3) as f64),
    );
    row.insert(
        "f_ask_gap_up_12".into(),
        Value::from((tick.ask_up_l2 - tick.ask_up) as f64),
    );
    row.insert(
        "f_ask_gap_up_23".into(),
        Value::from((tick.ask_up_l3 - tick.ask_up_l2) as f64),
    );
    row.insert(
        "f_bid_gap_down_12".into(),
        Value::from((tick.bid_down - tick.bid_down_l2) as f64),
    );
    row.insert(
        "f_bid_gap_down_23".into(),
        Value::from((tick.bid_down_l2 - tick.bid_down_l3) as f64),
    );
    row.insert(
        "f_ask_gap_down_12".into(),
        Value::from((tick.ask_down_l2 - tick.ask_down) as f64),
    );
    row.insert(
        "f_ask_gap_down_23".into(),
        Value::from((tick.ask_down_l3 - tick.ask_down_l2) as f64),
    );
    row.insert(
        "f_minutes_into_slot".into(),
        Value::from(tick.minutes_into_slot as f64),
    );
    row.insert(
        "f_pct_into_slot".into(),
        Value::from(tick.pct_into_slot as f64),
    );
    row.insert(
        "f_pct_into_slot_sq".into(),
        Value::from((tick.pct_into_slot as f64).powi(2)),
    );
    row.insert(
        "f_log_volume".into(),
        Value::from((1.0 + tick.market_volume_usd as f64).ln()),
    );
    row.insert(
        "f_hour_sin".into(),
        Value::from((2.0 * std::f64::consts::PI * tick.hour_utc as f64 / 24.0).sin()),
    );
    row.insert(
        "f_hour_cos".into(),
        Value::from((2.0 * std::f64::consts::PI * tick.hour_utc as f64 / 24.0).cos()),
    );
    row.insert(
        "f_dow_sin".into(),
        Value::from((2.0 * std::f64::consts::PI * tick.day_of_week as f64 / 7.0).sin()),
    );
    row.insert(
        "f_dow_cos".into(),
        Value::from((2.0 * std::f64::consts::PI * tick.day_of_week as f64 / 7.0).cos()),
    );
    row.insert(
        "f_dir_move_pct".into(),
        Value::from(tick.dir_move_pct as f64),
    );
    row.insert(
        "f_abs_move_pct".into(),
        Value::from(tick.abs_move_pct as f64),
    );
    row.insert(
        "f_ref_move_from_start".into(),
        Value::from(if tick.ref_price_start > 0.0 {
            tick.ref_price as f64 / tick.ref_price_start as f64 - 1.0
        } else {
            0.0
        }),
    );
    row.insert(
        "f_d_mid_up_1".into(),
        Value::from((tick.mid_up - prev.mid_up) as f64),
    );
    row.insert(
        "f_d_spread_up_1".into(),
        Value::from((tick.spread_up - prev.spread_up) as f64),
    );
    row.insert(
        "f_d_imbalance_up_1".into(),
        Value::from(imbalance_up - prev_imbalance_up),
    );
    row.insert("f_d_ref_1".into(), Value::from(f_d_ref_1));
    row.insert("f_mid_up_vs_ma5".into(), Value::from(f_mid_up - avg_mid_up));

    Some(Value::Object(row))
}

fn imbalance(bid_size: f64, ask_size: f64) -> f64 {
    let denom = bid_size + ask_size;
    if denom > 0.0 {
        (bid_size - ask_size) / denom
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::test_utils::test_tick;

    #[test]
    fn builds_expected_live_feature_row() {
        let mut earlier = test_tick();
        earlier.mid_up = 0.59;
        earlier.spread_up = 0.03;
        earlier.ref_price = 50_250.0;
        earlier.bid_size_up = 90.0;
        earlier.ask_size_up = 70.0;

        let mut current = test_tick();
        current.captured_at = time::OffsetDateTime::from_unix_timestamp(1_700_000_480).unwrap();

        let mut window = VecDeque::new();
        window.push_back(earlier);
        window.push_back(current);

        let row = build_live_feature_row(&window).expect("feature row");
        let object = row.as_object().expect("object row");

        assert_eq!(
            object["symbol"],
            Value::String("btc-updown-15m-1700000000".into())
        );
        assert_eq!(object["slot_duration"], Value::from(900_u32));
        assert!((object["f_mid_up"].as_f64().unwrap() - 0.61).abs() < 0.001);
        assert!((object["f_d_mid_up_1"].as_f64().unwrap() - 0.02).abs() < 0.001);
        assert!((object["f_mid_up_vs_ma5"].as_f64().unwrap() - 0.01).abs() < 0.001);
    }

    #[test]
    fn rejects_rows_outside_training_filter() {
        let mut tick = test_tick();
        tick.pct_into_slot = 0.95;

        let mut window = VecDeque::new();
        window.push_back(tick);

        assert!(build_live_feature_row(&window).is_none());
    }
}
