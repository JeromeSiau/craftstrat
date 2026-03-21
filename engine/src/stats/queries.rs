use anyhow::Result;
use clickhouse::Client;

use super::types::{
    CalibrationPoint, HeatmapCell, MlDatasetRow, StoplossThreshold, Summary, SymbolStats, TimeStats,
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
    /// Matches on the short symbol prefix extracted from the full slug
    /// (e.g. 'BTC' matches 'btc-updown-15m-1771910100').
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
            format!("AND upper(splitByChar('-', symbol)[1]) IN ({list})")
        }
    }
}

pub struct MlDatasetParams {
    pub slot_duration: u32,
    pub symbols: Vec<String>,
    pub hours: f64,
    pub sample_every: u32,
    pub limit: u32,
    pub offset: u32,
}

impl MlDatasetParams {
    fn cutoff_clause(&self) -> String {
        let seconds = (self.hours * 3600.0) as u64;
        format!("AND captured_at >= now64(3) - INTERVAL {seconds} SECOND")
    }

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
            format!("AND upper(splitByChar('-', symbol)[1]) IN ({list})")
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
            countIf(slot_winner IS NOT NULL) AS resolved_slots,
            countIf(slot_winner IS NULL) AS unresolved_slots,
            (SELECT count() FROM slot_snapshots
             WHERE slot_duration = ?
                 {cutoff}
                 {sym}) AS total_snapshots,
            (SELECT toString(max(captured_at)) FROM slot_snapshots
             WHERE slot_duration = ?
                 {cutoff}
                 {sym}) AS last_snapshot_at
        FROM (
            SELECT symbol, slot_ts, slot_duration, any(winner) AS slot_winner
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
        let last = if row.last_snapshot_at.is_empty()
            || row.last_snapshot_at == "1970-01-01 00:00:00.000"
        {
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
        300 => 1.0,     // 5m slots → 1 min bins
        900 => 2.0,     // 15m slots → 2 min bins
        3600 => 10.0,   // 1h slots → 10 min bins
        14400 => 30.0,  // 4h slots → 30 min bins
        86400 => 240.0, // 1d slots → 4h bins
        _ => 2.0,       // default
    };

    let sql = format!(
        "SELECT
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
        FROM slot_snapshots
        WHERE slot_duration = ?
            AND winner IS NOT NULL
            {cutoff}
            {sym}
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
        short_symbol: String,
        total: u64,
        wins: u64,
    }

    let cutoff = params.cutoff_clause();
    let sym = params.symbol_clause();

    let sql = format!(
        "WITH slots AS (
            SELECT symbol, slot_ts, slot_duration, any(winner) AS slot_winner
            FROM slot_snapshots
            WHERE slot_duration = ?
                AND winner IS NOT NULL
                {cutoff}
                {sym}
            GROUP BY symbol, slot_ts, slot_duration
        )
        SELECT
            upper(splitByChar('-', symbol)[1]) AS short_symbol,
            count() AS total,
            countIf(slot_winner = 'UP') AS wins
        FROM slots
        GROUP BY short_symbol
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
            symbol: row.short_symbol,
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
                any(ss.winner) AS slot_winner,
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
            countIf(peaked.min_bid_after_peak <= t.t AND peaked.slot_winner = 'DOWN') AS true_saves,
            countIf(peaked.min_bid_after_peak <= t.t AND peaked.slot_winner = 'UP') AS false_exits
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
                any(winner) AS slot_winner,
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
            countIf(slot_winner = 'UP') AS wins
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
                any(winner) AS slot_winner,
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
            countIf(slot_winner = 'UP') AS wins
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
// 8. ML Dataset
// ---------------------------------------------------------------------------

pub async fn fetch_ml_dataset(
    client: &Client,
    params: &MlDatasetParams,
) -> Result<Vec<MlDatasetRow>> {
    let cutoff = params.cutoff_clause();
    let sym = params.symbol_clause();
    let sample_every = params.sample_every;
    let limit = params.limit;
    let offset = params.offset;

    let sql = format!(
        r#"
        SELECT
            concat(replaceAll(toString(captured_at), ' ', 'T'), 'Z') AS captured_at,
            symbol,
            slot_ts,
            slot_duration,
            target_up,
            f_mid_up,
            f_mid_down,
            f_bid_up,
            f_ask_up,
            f_bid_down,
            f_ask_down,
            f_spread_up_rel,
            f_spread_down_rel,
            f_cross_sum_mid,
            f_cross_sum_bid,
            f_cross_sum_ask,
            f_parity_gap_up,
            f_l1_imbalance_up,
            f_l1_imbalance_down,
            f_size_ratio_up,
            f_size_ratio_down,
            f_bid_gap_up_12,
            f_bid_gap_up_23,
            f_ask_gap_up_12,
            f_ask_gap_up_23,
            f_bid_gap_down_12,
            f_bid_gap_down_23,
            f_ask_gap_down_12,
            f_ask_gap_down_23,
            f_minutes_into_slot,
            f_pct_into_slot,
            f_pct_into_slot_sq,
            f_log_volume,
            f_hour_sin,
            f_hour_cos,
            f_dow_sin,
            f_dow_cos,
            f_dir_move_pct,
            f_abs_move_pct,
            f_ref_move_from_start,
            f_d_mid_up_1,
            f_d_spread_up_1,
            f_d_imbalance_up_1,
            f_d_ref_1,
            f_mid_up_vs_ma5
        FROM (
            SELECT
                captured_at,
                symbol,
                slot_ts,
                slot_duration,
                toUInt8(winner = 'UP') AS target_up,
                toFloat64(mid_up) AS f_mid_up,
                toFloat64(mid_down) AS f_mid_down,
                toFloat64(bid_up) AS f_bid_up,
                toFloat64(ask_up) AS f_ask_up,
                toFloat64(bid_down) AS f_bid_down,
                toFloat64(ask_down) AS f_ask_down,
                if(mid_up > 0, toFloat64(spread_up) / toFloat64(mid_up), 0.0) AS f_spread_up_rel,
                if(mid_down > 0, toFloat64(spread_down) / toFloat64(mid_down), 0.0) AS f_spread_down_rel,
                toFloat64(mid_up + mid_down - 1) AS f_cross_sum_mid,
                toFloat64(bid_up + bid_down - 1) AS f_cross_sum_bid,
                toFloat64(ask_up + ask_down - 1) AS f_cross_sum_ask,
                toFloat64(mid_up - (1 - mid_down)) AS f_parity_gap_up,
                imbalance_up AS f_l1_imbalance_up,
                imbalance_down AS f_l1_imbalance_down,
                toFloat64(size_ratio_up) AS f_size_ratio_up,
                toFloat64(size_ratio_down) AS f_size_ratio_down,
                toFloat64(bid_up - bid_up_l2) AS f_bid_gap_up_12,
                toFloat64(bid_up_l2 - bid_up_l3) AS f_bid_gap_up_23,
                toFloat64(ask_up_l2 - ask_up) AS f_ask_gap_up_12,
                toFloat64(ask_up_l3 - ask_up_l2) AS f_ask_gap_up_23,
                toFloat64(bid_down - bid_down_l2) AS f_bid_gap_down_12,
                toFloat64(bid_down_l2 - bid_down_l3) AS f_bid_gap_down_23,
                toFloat64(ask_down_l2 - ask_down) AS f_ask_gap_down_12,
                toFloat64(ask_down_l3 - ask_down_l2) AS f_ask_gap_down_23,
                toFloat64(minutes_into_slot) AS f_minutes_into_slot,
                toFloat64(pct_into_slot) AS f_pct_into_slot,
                toFloat64(pct_into_slot * pct_into_slot) AS f_pct_into_slot_sq,
                log1p(toFloat64(market_volume_usd)) AS f_log_volume,
                sin(2 * pi() * toFloat64(hour_utc) / 24.0) AS f_hour_sin,
                cos(2 * pi() * toFloat64(hour_utc) / 24.0) AS f_hour_cos,
                sin(2 * pi() * toFloat64(day_of_week) / 7.0) AS f_dow_sin,
                cos(2 * pi() * toFloat64(day_of_week) / 7.0) AS f_dow_cos,
                toFloat64(dir_move_pct) AS f_dir_move_pct,
                toFloat64(abs_move_pct) AS f_abs_move_pct,
                if(
                    btc_price_start > 0,
                    toFloat64(chainlink_price / btc_price_start - 1),
                    0.0
                ) AS f_ref_move_from_start,
                toFloat64(mid_up - lagInFrame(mid_up, 1, mid_up) OVER slot_order) AS f_d_mid_up_1,
                toFloat64(spread_up - lagInFrame(spread_up, 1, spread_up) OVER slot_order) AS f_d_spread_up_1,
                imbalance_up - lagInFrame(imbalance_up, 1, imbalance_up) OVER slot_order AS f_d_imbalance_up_1,
                if(
                    chainlink_price > 0
                    AND lagInFrame(chainlink_price, 1, chainlink_price) OVER slot_order > 0,
                    log(
                        toFloat64(chainlink_price)
                        / toFloat64(
                            lagInFrame(chainlink_price, 1, chainlink_price) OVER slot_order
                        )
                    ),
                    0.0
                ) AS f_d_ref_1,
                toFloat64(mid_up - avg(mid_up) OVER slot_ma5) AS f_mid_up_vs_ma5,
                row_number() OVER slot_order AS rn
            FROM (
                SELECT
                    *,
                    if(
                        bid_size_up + ask_size_up > 0,
                        toFloat64((bid_size_up - ask_size_up) / (bid_size_up + ask_size_up)),
                        0.0
                    ) AS imbalance_up,
                    if(
                        bid_size_down + ask_size_down > 0,
                        toFloat64((bid_size_down - ask_size_down) / (bid_size_down + ask_size_down)),
                        0.0
                    ) AS imbalance_down
                FROM slot_snapshots
                WHERE slot_duration = ?
                    AND winner IS NOT NULL
                    {cutoff}
                    {sym}
            )
            WINDOW
                slot_order AS (
                    PARTITION BY symbol, slot_ts, slot_duration
                    ORDER BY captured_at
                    ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
                ),
                slot_ma5 AS (
                    PARTITION BY symbol, slot_ts, slot_duration
                    ORDER BY captured_at
                    ROWS BETWEEN 4 PRECEDING AND CURRENT ROW
                )
        )
        WHERE
            f_pct_into_slot BETWEEN 0.05 AND 0.90
            AND f_bid_up > 0
            AND f_ask_up > 0
            AND f_bid_down > 0
            AND f_ask_down > 0
            AND f_spread_up_rel >= 0
            AND f_spread_down_rel >= 0
            AND f_spread_up_rel <= 0.25
            AND f_spread_down_rel <= 0.25
            AND modulo(rn - 1, {sample_every}) = 0
        ORDER BY captured_at ASC, symbol ASC, slot_ts ASC
        LIMIT {limit} OFFSET {offset}
        "#
    );

    let mut cursor = client
        .query(&sql)
        .bind(params.slot_duration)
        .fetch::<MlDatasetRow>()?;

    let mut rows = Vec::new();
    while let Some(row) = cursor.next().await? {
        rows.push(row);
    }
    Ok(rows)
}
