use clickhouse::Row;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Serialize)]
pub struct SlotStatsResponse {
    pub summary: Summary,
    pub heatmap: Vec<HeatmapCell>,
    pub calibration: Vec<CalibrationPoint>,
    pub by_symbol: Vec<SymbolStats>,
    pub stoploss_sweep: Vec<StoplossThreshold>,
    pub by_hour: Vec<TimeStats>,
    pub by_day: Vec<TimeStats>,
}

#[derive(Debug, Serialize)]
pub struct MlDatasetResponse {
    pub row_count: usize,
    pub rows: Vec<MlDatasetRow>,
}

#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct MlDatasetRow {
    #[serde(with = "clickhouse::serde::time::datetime64::millis")]
    pub captured_at: OffsetDateTime,
    pub symbol: String,
    pub slot_ts: u32,
    pub slot_duration: u32,
    pub target_up: u8,
    pub f_mid_up: f64,
    pub f_mid_down: f64,
    pub f_bid_up: f64,
    pub f_ask_up: f64,
    pub f_bid_down: f64,
    pub f_ask_down: f64,
    pub f_spread_up_rel: f64,
    pub f_spread_down_rel: f64,
    pub f_cross_sum_mid: f64,
    pub f_cross_sum_bid: f64,
    pub f_cross_sum_ask: f64,
    pub f_parity_gap_up: f64,
    pub f_l1_imbalance_up: f64,
    pub f_l1_imbalance_down: f64,
    pub f_size_ratio_up: f64,
    pub f_size_ratio_down: f64,
    pub f_bid_gap_up_12: f64,
    pub f_bid_gap_up_23: f64,
    pub f_ask_gap_up_12: f64,
    pub f_ask_gap_up_23: f64,
    pub f_bid_gap_down_12: f64,
    pub f_bid_gap_down_23: f64,
    pub f_ask_gap_down_12: f64,
    pub f_ask_gap_down_23: f64,
    pub f_minutes_into_slot: f64,
    pub f_pct_into_slot: f64,
    pub f_pct_into_slot_sq: f64,
    pub f_log_volume: f64,
    pub f_hour_sin: f64,
    pub f_hour_cos: f64,
    pub f_dow_sin: f64,
    pub f_dow_cos: f64,
    pub f_dir_move_pct: f64,
    pub f_abs_move_pct: f64,
    pub f_ref_move_from_start: f64,
    pub f_d_mid_up_1: f64,
    pub f_d_spread_up_1: f64,
    pub f_d_imbalance_up_1: f64,
    pub f_d_ref_1: f64,
    pub f_mid_up_vs_ma5: f64,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub total_slots: u64,
    pub resolved_slots: u64,
    pub unresolved_slots: u64,
    pub total_snapshots: u64,
    pub last_snapshot_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HeatmapCell {
    pub time_bin: String,
    pub move_bin: String,
    pub total: u64,
    pub wins: u64,
    pub win_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct CalibrationPoint {
    pub bid_bucket: f64,
    pub avg_bid: f64,
    pub win_rate: f64,
    pub sample_count: u64,
}

#[derive(Debug, Serialize)]
pub struct SymbolStats {
    pub symbol: String,
    pub total: u64,
    pub wins: u64,
    pub win_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct StoplossThreshold {
    pub threshold: f64,
    pub triggered: u64,
    pub true_saves: u64,
    pub false_exits: u64,
    pub precision: f64,
}

#[derive(Debug, Serialize)]
pub struct TimeStats {
    pub period: u8,
    pub total: u64,
    pub wins: u64,
    pub win_rate: f64,
}
