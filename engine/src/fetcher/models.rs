use clickhouse::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub price: f32,
    pub size: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Default)]
pub struct OrderBook {
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

impl OrderBook {
    pub fn best_bid(&self) -> Option<&Level> {
        self.bids.first()
    }

    pub fn best_ask(&self) -> Option<&Level> {
        self.asks.first()
    }

    pub fn level_n_bid(&self, n: usize) -> Option<&Level> {
        self.bids.get(n)
    }

    pub fn level_n_ask(&self, n: usize) -> Option<&Level> {
        self.asks.get(n)
    }

    pub fn merge_level(&mut self, price: f32, size: f32, side: Side) {
        let levels = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        if size == 0.0 {
            levels.retain(|l| (l.price - price).abs() > f32::EPSILON);
        } else if let Some(existing) = levels.iter_mut().find(|l| (l.price - price).abs() < f32::EPSILON) {
            existing.size = size;
        } else {
            levels.push(Level { price, size });
        }
        match side {
            Side::Buy => levels.sort_by(|a, b| b.price.total_cmp(&a.price)),
            Side::Sell => levels.sort_by(|a, b| a.price.total_cmp(&b.price)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActiveMarket {
    pub condition_id: String,
    pub slug: String,
    pub binance_symbol: Option<String>,
    pub slot_ts: u32,
    pub slot_duration: u32,
    pub end_time: f64,
    pub token_up: String,
    pub token_down: String,
    pub ref_price_start: Option<f32>,
}

#[derive(Debug, Clone, Row, Serialize)]
pub struct Tick {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub captured_at: time::OffsetDateTime,
    pub symbol: String,
    pub slot_ts: u32,
    pub slot_duration: u32,
    pub minutes_into_slot: f32,
    pub pct_into_slot: f32,
    pub bid_up: f32,
    pub ask_up: f32,
    pub bid_down: f32,
    pub ask_down: f32,
    pub bid_size_up: f32,
    pub ask_size_up: f32,
    pub bid_size_down: f32,
    pub ask_size_down: f32,
    pub spread_up: f32,
    pub spread_down: f32,
    pub bid_up_l2: f32,
    pub ask_up_l2: f32,
    pub bid_up_l3: f32,
    pub ask_up_l3: f32,
    pub bid_down_l2: f32,
    pub ask_down_l2: f32,
    pub bid_down_l3: f32,
    pub ask_down_l3: f32,
    pub mid_up: f32,
    pub mid_down: f32,
    pub size_ratio_up: f32,
    pub size_ratio_down: f32,
    #[serde(rename = "chainlink_price")]
    pub ref_price: f32,
    pub dir_move_pct: f32,
    pub abs_move_pct: f32,
    pub hour_utc: u8,
    pub day_of_week: u8,
    pub market_volume_usd: f32,
    pub winner: Option<i8>, // 1 = UP, 2 = DOWN, None = not resolved
    #[serde(rename = "btc_price_start")]
    pub ref_price_start: f32,
    #[serde(rename = "btc_price_end")]
    pub ref_price_end: f32,
    pub ref_price_source: String,
}
