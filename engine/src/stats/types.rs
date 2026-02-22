use serde::Serialize;

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
