use chrono::{Datelike, Timelike};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::models::{ActiveMarket, OrderBook, Tick};
use super::websocket::OrderBookCache;

pub type PriceCache = Arc<RwLock<HashMap<String, f64>>>;

pub fn build_tick(
    market: &ActiveMarket,
    book_up: Option<&OrderBook>,
    book_down: Option<&OrderBook>,
    ref_price: f32,
    now_unix: f64,
) -> Option<Tick> {
    let minutes_into_slot = (now_unix - market.slot_ts as f64) / 60.0;
    let slot_minutes = market.slot_duration as f32 / 60.0;

    if minutes_into_slot < 0.0 || minutes_into_slot > (slot_minutes as f64 + 0.5) {
        return None;
    }

    let pct_into_slot = minutes_into_slot as f32 / slot_minutes;

    let (bid_up, ask_up, bid_sz_up, ask_sz_up) = extract_l1(book_up);
    let (bid_up_l2, ask_up_l2) = extract_ln(book_up, 1);
    let (bid_up_l3, ask_up_l3) = extract_ln(book_up, 2);

    let (bid_down, ask_down, bid_sz_down, ask_sz_down) = extract_l1(book_down);
    let (bid_down_l2, ask_down_l2) = extract_ln(book_down, 1);
    let (bid_down_l3, ask_down_l3) = extract_ln(book_down, 2);

    let price_start = market.ref_price_start.unwrap_or(ref_price);
    let dir_move_pct = if price_start > 0.0 {
        (ref_price - price_start) / price_start * 100.0
    } else {
        0.0
    };

    let dt = chrono::DateTime::from_timestamp(now_unix as i64, 0)?;
    let captured_at = time::OffsetDateTime::from_unix_timestamp(now_unix as i64).ok()?;

    Some(Tick {
        captured_at,
        symbol: market.slug.clone(),
        slot_ts: market.slot_ts,
        slot_duration: market.slot_duration,
        minutes_into_slot: minutes_into_slot as f32,
        pct_into_slot,
        bid_up,
        ask_up,
        bid_down,
        ask_down,
        bid_size_up: bid_sz_up,
        ask_size_up: ask_sz_up,
        bid_size_down: bid_sz_down,
        ask_size_down: ask_sz_down,
        spread_up: ask_up - bid_up,
        spread_down: ask_down - bid_down,
        bid_up_l2,
        ask_up_l2,
        bid_up_l3,
        ask_up_l3,
        bid_down_l2,
        ask_down_l2,
        bid_down_l3,
        ask_down_l3,
        mid_up: if bid_up > 0.0 && ask_up > 0.0 { (bid_up + ask_up) / 2.0 } else { 0.0 },
        mid_down: if bid_down > 0.0 && ask_down > 0.0 { (bid_down + ask_down) / 2.0 } else { 0.0 },
        size_ratio_up: safe_div(bid_sz_up, ask_sz_up),
        size_ratio_down: safe_div(bid_sz_down, ask_sz_down),
        chainlink_price: ref_price,
        dir_move_pct,
        abs_move_pct: dir_move_pct.abs(),
        hour_utc: dt.hour() as u8,
        day_of_week: dt.weekday().num_days_from_monday() as u8,
        market_volume_usd: 0.0,
        winner: None,
        btc_price_start: price_start,
        btc_price_end: ref_price,
    })
}

fn extract_l1(book: Option<&OrderBook>) -> (f32, f32, f32, f32) {
    let Some(b) = book else { return (0.0, 0.0, 0.0, 0.0) };
    (
        b.best_bid().map(|l| l.price).unwrap_or(0.0),
        b.best_ask().map(|l| l.price).unwrap_or(0.0),
        b.best_bid().map(|l| l.size).unwrap_or(0.0),
        b.best_ask().map(|l| l.size).unwrap_or(0.0),
    )
}

fn extract_ln(book: Option<&OrderBook>, n: usize) -> (f32, f32) {
    let Some(b) = book else { return (0.0, 0.0) };
    (
        b.level_n_bid(n).map(|l| l.price).unwrap_or(0.0),
        b.level_n_ask(n).map(|l| l.price).unwrap_or(0.0),
    )
}

fn safe_div(a: f32, b: f32) -> f32 {
    if b > 0.0 { a / b } else { 0.0 }
}

pub async fn run_tick_builder(
    books: OrderBookCache,
    markets: Arc<RwLock<HashMap<String, ActiveMarket>>>,
    prices: PriceCache,
    tick_tx: tokio::sync::mpsc::Sender<Tick>,
    interval: Duration,
) {
    let mut ticker = tokio::time::interval(interval);
    loop {
        ticker.tick().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let active = markets.read().await;
        let book_cache = books.read().await;
        let price_cache = prices.read().await;

        for market in active.values() {
            let ref_price = price_cache
                .get(&market.binance_symbol)
                .copied()
                .unwrap_or(0.0) as f32;
            if ref_price <= 0.0 { continue; }

            let book_up = book_cache.get(&market.token_up);
            let book_down = book_cache.get(&market.token_down);
            if book_up.is_none() && book_down.is_none() { continue; }

            if let Some(tick) = build_tick(market, book_up, book_down, ref_price, now) {
                if tick_tx.send(tick).await.is_err() {
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetcher::models::{ActiveMarket, Level, OrderBook, Side};

    fn market(slot_ts: u32) -> ActiveMarket {
        ActiveMarket {
            condition_id: "0xabc".into(),
            slug: "btc-updown-15m-1700000000".into(),
            binance_symbol: "BTCUSDT".into(),
            slot_ts,
            slot_duration: 900,
            end_time: (slot_ts + 900) as f64,
            token_up: "tok_up".into(),
            token_down: "tok_down".into(),
            ref_price_start: Some(50000.0),
        }
    }

    fn book(bids: &[(f32, f32)], asks: &[(f32, f32)]) -> OrderBook {
        OrderBook {
            bids: bids.iter().map(|&(p, s)| Level { price: p, size: s }).collect(),
            asks: asks.iter().map(|&(p, s)| Level { price: p, size: s }).collect(),
        }
    }

    #[test]
    fn test_build_tick_mid_slot() {
        let m = market(1700000000);
        let up = book(&[(0.60, 100.0), (0.58, 50.0), (0.55, 30.0)],
                       &[(0.62, 80.0), (0.65, 40.0), (0.68, 20.0)]);
        let down = book(&[(0.38, 90.0)], &[(0.40, 70.0)]);
        let now = 1700000000.0 + 450.0;

        let t = build_tick(&m, Some(&up), Some(&down), 50500.0, now).unwrap();
        assert!((t.pct_into_slot - 0.5).abs() < 0.01);
        assert!((t.bid_up - 0.60).abs() < 0.001);
        assert!((t.ask_up - 0.62).abs() < 0.001);
        assert!((t.spread_up - 0.02).abs() < 0.001);
        assert!((t.mid_up - 0.61).abs() < 0.001);
        assert!((t.bid_up_l2 - 0.58).abs() < 0.001);
        assert!((t.dir_move_pct - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_outside_slot_returns_none() {
        let m = market(1700000000);
        let b = book(&[(0.5, 10.0)], &[(0.6, 10.0)]);
        assert!(build_tick(&m, Some(&b), Some(&b), 50000.0, 1699999999.0).is_none());
        assert!(build_tick(&m, Some(&b), Some(&b), 50000.0, 1700000960.0).is_none());
    }

    #[test]
    fn test_empty_books_zero_prices() {
        let m = market(1700000000);
        let empty = OrderBook::default();
        let t = build_tick(&m, Some(&empty), Some(&empty), 50000.0, 1700000100.0).unwrap();
        assert!((t.bid_up).abs() < 0.001);
        assert!((t.mid_up).abs() < 0.001);
    }

    #[test]
    fn test_negative_move_pct() {
        let m = market(1700000000);
        let b = book(&[(0.5, 10.0)], &[(0.6, 10.0)]);
        let t = build_tick(&m, Some(&b), Some(&b), 49000.0, 1700000100.0).unwrap();
        assert!((t.dir_move_pct - (-2.0)).abs() < 0.01);
        assert!((t.abs_move_pct - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_orderbook_merge() {
        let mut b = OrderBook::default();
        b.merge_level(0.50, 100.0, Side::Buy);
        b.merge_level(0.48, 50.0, Side::Buy);
        assert_eq!(b.bids.len(), 2);
        assert!((b.best_bid().unwrap().price - 0.50).abs() < 0.001);

        b.merge_level(0.50, 200.0, Side::Buy);
        assert!((b.best_bid().unwrap().size - 200.0).abs() < 0.001);

        b.merge_level(0.50, 0.0, Side::Buy);
        assert_eq!(b.bids.len(), 1);
        assert!((b.best_bid().unwrap().price - 0.48).abs() < 0.001);
    }
}
