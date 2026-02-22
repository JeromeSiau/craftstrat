use anyhow::Result;
use clickhouse::Client;

use super::types::{
    CalibrationPoint, HeatmapCell, StoplossThreshold, Summary, SymbolStats, TimeStats,
};

pub struct StatsParams {
    pub slot_duration: u32,
    pub symbols: Vec<String>,
    pub hours: f64,
}

impl StatsParams {
    /// Build a SQL fragment for the time cutoff, e.g. `AND captured_at >= now() - INTERVAL 168 SECOND`.
    /// We convert hours to seconds for the INTERVAL since bind params don't work with INTERVAL.
    fn cutoff_clause(&self) -> String {
        let seconds = (self.hours * 3600.0) as u64;
        format!("AND captured_at >= now64(3) - INTERVAL {seconds} SECOND")
    }

    /// Build a SQL fragment for symbol filtering.
    /// Returns empty string if no symbols are specified.
    fn symbol_clause(&self) -> String {
        if self.symbols.is_empty() {
            String::new()
        } else {
            let list = self
                .symbols
                .iter()
                .map(|s| format!("'{}'", s.replace('\'', "")))
                .collect::<Vec<_>>()
                .join(", ");
            format!("AND symbol IN ({list})")
        }
    }
}

// ---------------------------------------------------------------------------
// 1. Summary
// ---------------------------------------------------------------------------

pub async fn fetch_summary(client: &Client, params: &StatsParams) -> Result<Summary> {
    #[derive(clickhouse::Row, serde::Deserialize)]
    struct Row {
        total_slots: u64,
        resolved_slots: u64,
        unresolved_slots: u64,
        total_snapshots: u64,
        last_snapshot_at: String,
    }

    let cutoff = params.cutoff_clause();
    let sym = params.symbol_clause();
    let seconds = (params.hours * 3600.0) as u64;

    let sql = format!(
        "SELECT
            count() AS total_slots,
            countIf(winner IS NOT NULL) AS resolved_slots,
            countIf(winner IS NULL) AS unresolved_slots,
            (SELECT count() FROM slot_snapshots
             WHERE slot_duration = ?
                 {cutoff}
                 {sym}) AS total_snapshots,
            (SELECT toString(max(captured_at)) FROM slot_snapshots
             WHERE slot_duration = ?
                 {cutoff}
                 {sym}) AS last_snapshot_at
        FROM (
            SELECT symbol, slot_ts, slot_duration, any(winner) AS winner
            FROM slot_snapshots
            WHERE captured_at >= now64(3) - INTERVAL {seconds} SECOND
                AND slot_duration = ?
                {sym}
            GROUP BY symbol, slot_ts, slot_duration
        )"
    );

    let mut cursor = client
        .query(&sql)
        .bind(params.slot_duration)
        .bind(params.slot_duration)
        .bind(params.slot_duration)
        .fetch::<Row>()?;

    if let Some(row) = cursor.next().await? {
        let last = if row.last_snapshot_at.is_empty() || row.last_snapshot_at == "1970-01-01 00:00:00.000" {
            None
        } else {
            Some(row.last_snapshot_at)
        };
        Ok(Summary {
            total_slots: row.total_slots,
            resolved_slots: row.resolved_slots,
            unresolved_slots: row.unresolved_slots,
            total_snapshots: row.total_snapshots,
            last_snapshot_at: last,
        })
    } else {
        Ok(Summary {
            total_slots: 0,
            resolved_slots: 0,
            unresolved_slots: 0,
            total_snapshots: 0,
            last_snapshot_at: None,
        })
    }
}

// ---------------------------------------------------------------------------
// 2. Heatmap
// ---------------------------------------------------------------------------

pub async fn fetch_heatmap(client: &Client, params: &StatsParams) -> Result<Vec<HeatmapCell>> {
    #[derive(clickhouse::Row, serde::Deserialize)]
    struct Row {
        time_bin: String,
        move_bin: String,
        total: u64,
        wins: u64,
    }

    let cutoff = params.cutoff_clause();
    let sym = params.symbol_clause();

    // Timing bin size depends on slot_duration (in seconds)
    let time_bin_minutes: f64 = match params.slot_duration {
        300 => 1.0,      // 5m slots → 1 min bins
        900 => 2.0,      // 15m slots → 2 min bins
        3600 => 10.0,    // 1h slots → 10 min bins
        14400 => 30.0,   // 4h slots → 30 min bins
        86400 => 240.0,  // 1d slots → 4h bins
        _ => 2.0,        // default
    };

    let sql = format!(
        "WITH slots AS (
            SELECT
                symbol, slot_ts, slot_duration,
                any(winner) AS winner,
                any(minutes_into_slot) AS minutes_into_slot,
                any(dir_move_pct) AS dir_move_pct
            FROM slot_snapshots
            WHERE slot_duration = ?
                {cutoff}
                {sym}
            GROUP BY symbol, slot_ts, slot_duration
        )
        SELECT
            concat(
                toString(floor(minutes_into_slot / {time_bin_minutes}) * {time_bin_minutes}),
                '-',
                toString(floor(minutes_into_slot / {time_bin_minutes}) * {time_bin_minutes} + {time_bin_minutes})
            ) AS time_bin,
            multiIf(
                dir_move_pct < -0.2, '< -0.2',
                dir_move_pct < -0.1, '-0.2 / -0.1',
                dir_move_pct < 0.0,  '-0.1 / 0',
                dir_move_pct < 0.1,  '0 / 0.1',
                dir_move_pct < 0.2,  '0.1 / 0.2',
                '>= 0.2'
            ) AS move_bin,
            count() AS total,
            countIf(winner = 'UP') AS wins
        FROM slots
        WHERE winner IS NOT NULL
        GROUP BY time_bin, move_bin
        ORDER BY time_bin, move_bin"
    );

    let mut cursor = client
        .query(&sql)
        .bind(params.slot_duration)
        .fetch::<Row>()?;

    let mut cells = Vec::new();
    while let Some(row) = cursor.next().await? {
        let win_rate = if row.total < 3 {
            -1.0
        } else {
            row.wins as f64 / row.total as f64
        };
        cells.push(HeatmapCell {
            time_bin: row.time_bin,
            move_bin: row.move_bin,
            total: row.total,
            wins: row.wins,
            win_rate,
        });
    }
    Ok(cells)
}

// ---------------------------------------------------------------------------
// 3. Calibration
// ---------------------------------------------------------------------------

pub async fn fetch_calibration(
    client: &Client,
    params: &StatsParams,
) -> Result<Vec<CalibrationPoint>> {
    #[derive(clickhouse::Row, serde::Deserialize)]
    struct Row {
        bid_bucket: f64,
        avg_bid: f64,
        win_rate: f64,
        sample_count: u64,
    }

    let cutoff = params.cutoff_clause();
    let sym = params.symbol_clause();

    // Calibration time window depends on slot_duration
    let (cal_min, cal_max): (f64, f64) = match params.slot_duration {
        300 => (1.0, 4.0),
        900 => (4.0, 10.0),
        3600 => (10.0, 50.0),
        14400 => (30.0, 180.0),
        86400 => (120.0, 1200.0),
        _ => (4.0, 10.0),
    };

    let sql = format!(
        "SELECT
            round(bid_up * 20) / 20 AS bid_bucket,
            avg(bid_up) AS avg_bid,
            countIf(winner = 'UP') / count() AS win_rate,
            count() AS sample_count
        FROM slot_snapshots
        WHERE slot_duration = ?
            AND winner IS NOT NULL
            AND minutes_into_slot >= {cal_min}
            AND minutes_into_slot <= {cal_max}
            {cutoff}
            {sym}
        GROUP BY bid_bucket
        HAVING count() >= 3
        ORDER BY bid_bucket"
    );

    let mut cursor = client
        .query(&sql)
        .bind(params.slot_duration)
        .fetch::<Row>()?;

    let mut points = Vec::new();
    while let Some(row) = cursor.next().await? {
        points.push(CalibrationPoint {
            bid_bucket: row.bid_bucket,
            avg_bid: row.avg_bid,
            win_rate: row.win_rate,
            sample_count: row.sample_count,
        });
    }
    Ok(points)
}

// ---------------------------------------------------------------------------
// 4. By Symbol
// ---------------------------------------------------------------------------

pub async fn fetch_by_symbol(client: &Client, params: &StatsParams) -> Result<Vec<SymbolStats>> {
    #[derive(clickhouse::Row, serde::Deserialize)]
    struct Row {
        symbol: String,
        total: u64,
        wins: u64,
    }

    let cutoff = params.cutoff_clause();
    let sym = params.symbol_clause();

    let sql = format!(
        "WITH slots AS (
            SELECT symbol, slot_ts, slot_duration, any(winner) AS winner
            FROM slot_snapshots
            WHERE slot_duration = ?
                AND winner IS NOT NULL
                {cutoff}
                {sym}
            GROUP BY symbol, slot_ts, slot_duration
        )
        SELECT
            symbol,
            count() AS total,
            countIf(winner = 'UP') AS wins
        FROM slots
        GROUP BY symbol
        ORDER BY total DESC"
    );

    let mut cursor = client
        .query(&sql)
        .bind(params.slot_duration)
        .fetch::<Row>()?;

    let mut stats = Vec::new();
    while let Some(row) = cursor.next().await? {
        let win_rate = if row.total == 0 {
            0.0
        } else {
            row.wins as f64 / row.total as f64
        };
        stats.push(SymbolStats {
            symbol: row.symbol,
            total: row.total,
            wins: row.wins,
            win_rate,
        });
    }
    Ok(stats)
}

// ---------------------------------------------------------------------------
// 5. Stoploss Sweep
// ---------------------------------------------------------------------------

pub async fn fetch_stoploss_sweep(
    client: &Client,
    params: &StatsParams,
) -> Result<Vec<StoplossThreshold>> {
    #[derive(clickhouse::Row, serde::Deserialize)]
    struct Row {
        threshold: f64,
        triggered: u64,
        true_saves: u64,
        false_exits: u64,
    }

    let sym = params.symbol_clause();
    let seconds = (params.hours * 3600.0) as u64;

    // max_minutes: how far into the slot to look for min-after-peak
    let max_minutes: f64 = match params.slot_duration {
        300 => 4.0,
        900 => 12.0,
        3600 => 50.0,
        14400 => 200.0,
        86400 => 1200.0,
        _ => 12.0,
    };

    let sql = format!(
        "WITH
        peak_minute AS (
            SELECT symbol, slot_ts, slot_duration,
                   min(minutes_into_slot) AS first_peak_min
            FROM slot_snapshots
            WHERE bid_up >= 0.75
                AND minutes_into_slot <= {max_minutes}
                AND captured_at >= now64(3) - INTERVAL {seconds} SECOND
                AND slot_duration = ?
                {sym}
            GROUP BY symbol, slot_ts, slot_duration
        ),
        peaked AS (
            SELECT
                ss.symbol,
                ss.slot_ts,
                any(ss.winner) AS winner,
                min(CASE WHEN ss.minutes_into_slot >= pm.first_peak_min THEN ss.bid_up ELSE NULL END) AS min_bid_after_peak
            FROM slot_snapshots ss
            INNER JOIN peak_minute pm
                ON pm.symbol = ss.symbol AND pm.slot_ts = ss.slot_ts AND pm.slot_duration = ss.slot_duration
            WHERE ss.winner IS NOT NULL
                AND ss.minutes_into_slot <= {max_minutes}
                AND ss.captured_at >= now64(3) - INTERVAL {seconds} SECOND
                AND ss.slot_duration = ?
                {sym}
            GROUP BY ss.symbol, ss.slot_ts
        ),
        thresholds AS (
            SELECT arrayJoin([0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.35, 0.40, 0.45, 0.50,
                              0.55, 0.60, 0.65, 0.70, 0.75, 0.80, 0.85, 0.90, 0.95]) AS t
        )
        SELECT
            t.t AS threshold,
            countIf(peaked.min_bid_after_peak <= t.t) AS triggered,
            countIf(peaked.min_bid_after_peak <= t.t AND peaked.winner = 'DOWN') AS true_saves,
            countIf(peaked.min_bid_after_peak <= t.t AND peaked.winner = 'UP') AS false_exits
        FROM peaked
        CROSS JOIN thresholds t
        GROUP BY t.t
        ORDER BY t.t DESC"
    );

    let mut cursor = client
        .query(&sql)
        .bind(params.slot_duration)
        .bind(params.slot_duration)
        .fetch::<Row>()?;

    let mut results = Vec::new();
    while let Some(row) = cursor.next().await? {
        let precision = if row.triggered == 0 {
            0.0
        } else {
            row.true_saves as f64 / row.triggered as f64
        };
        results.push(StoplossThreshold {
            threshold: row.threshold,
            triggered: row.triggered,
            true_saves: row.true_saves,
            false_exits: row.false_exits,
            precision,
        });
    }
    Ok(results)
}

// ---------------------------------------------------------------------------
// 6. By Hour
// ---------------------------------------------------------------------------

pub async fn fetch_by_hour(client: &Client, params: &StatsParams) -> Result<Vec<TimeStats>> {
    #[derive(clickhouse::Row, serde::Deserialize)]
    struct Row {
        period: u8,
        total: u64,
        wins: u64,
    }

    let cutoff = params.cutoff_clause();
    let sym = params.symbol_clause();

    let sql = format!(
        "WITH slots AS (
            SELECT
                symbol, slot_ts, slot_duration,
                any(winner) AS winner,
                toHour(toDateTime(slot_ts)) AS hour_utc
            FROM slot_snapshots
            WHERE slot_duration = ?
                AND winner IS NOT NULL
                {cutoff}
                {sym}
            GROUP BY symbol, slot_ts, slot_duration
        )
        SELECT
            hour_utc AS period,
            count() AS total,
            countIf(winner = 'UP') AS wins
        FROM slots
        GROUP BY period
        ORDER BY period"
    );

    let mut cursor = client
        .query(&sql)
        .bind(params.slot_duration)
        .fetch::<Row>()?;

    let mut stats = Vec::new();
    while let Some(row) = cursor.next().await? {
        let win_rate = if row.total == 0 {
            0.0
        } else {
            row.wins as f64 / row.total as f64
        };
        stats.push(TimeStats {
            period: row.period,
            total: row.total,
            wins: row.wins,
            win_rate,
        });
    }
    Ok(stats)
}

// ---------------------------------------------------------------------------
// 7. By Day of Week
// ---------------------------------------------------------------------------

pub async fn fetch_by_day(client: &Client, params: &StatsParams) -> Result<Vec<TimeStats>> {
    #[derive(clickhouse::Row, serde::Deserialize)]
    struct Row {
        period: u8,
        total: u64,
        wins: u64,
    }

    let cutoff = params.cutoff_clause();
    let sym = params.symbol_clause();

    // toDayOfWeek returns 1=Monday..7=Sunday; subtract 1 for 0=Monday..6=Sunday
    let sql = format!(
        "WITH slots AS (
            SELECT
                symbol, slot_ts, slot_duration,
                any(winner) AS winner,
                toUInt8(toDayOfWeek(toDateTime(slot_ts)) - 1) AS dow
            FROM slot_snapshots
            WHERE slot_duration = ?
                AND winner IS NOT NULL
                {cutoff}
                {sym}
            GROUP BY symbol, slot_ts, slot_duration
        )
        SELECT
            dow AS period,
            count() AS total,
            countIf(winner = 'UP') AS wins
        FROM slots
        GROUP BY period
        ORDER BY period"
    );

    let mut cursor = client
        .query(&sql)
        .bind(params.slot_duration)
        .fetch::<Row>()?;

    let mut stats = Vec::new();
    while let Some(row) = cursor.next().await? {
        let win_rate = if row.total == 0 {
            0.0
        } else {
            row.wins as f64 / row.total as f64
        };
        stats.push(TimeStats {
            period: row.period,
            total: row.total,
            wins: row.wins,
            win_rate,
        });
    }
    Ok(stats)
}
