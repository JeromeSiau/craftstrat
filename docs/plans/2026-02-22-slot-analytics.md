# Slot Analytics Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replicate the Python Streamlit "ML" dashboard as a new Analytics page in CraftStrat, showing 7 visualizations: KPIs, Win Rate Heatmap, Market Calibration, WR by Symbol, Stop-Loss Sweep, WR by Hour, WR by Day.

**Architecture:** New Rust endpoint `GET /internal/stats/slots` runs 7 ClickHouse aggregation queries in parallel via `tokio::join!`, returns a single JSON response. Laravel `AnalyticsController` calls it via `EngineService` and passes data to an Inertia React page. Frontend uses Recharts for all charts.

**Tech Stack:** Rust (Axum, clickhouse crate), Laravel 12, Inertia.js v2, React 19, Recharts, Tailwind CSS v4.

**Key data difference from Python project:** CraftStrat has a single `slot_snapshots` table in ClickHouse with a `winner` column (`Nullable(Enum8('UP'=1, 'DOWN'=2))`) instead of separate `slot_snapshots` + `slot_resolutions` tables. The `winner` column is denormalized into every snapshot row, so slot-level queries must deduplicate via `GROUP BY (symbol, slot_ts, slot_duration)`.

---

### Task 1: Rust stats module — types

**Files:**
- Create: `engine/src/stats/mod.rs`
- Create: `engine/src/stats/types.rs`

**Step 1: Create the types file**

Create `engine/src/stats/types.rs`:

```rust
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
```

**Step 2: Create the mod.rs**

Create `engine/src/stats/mod.rs`:

```rust
pub mod queries;
pub mod types;
```

**Step 3: Register the module in the engine crate**

Modify `engine/src/main.rs` — add `mod stats;` alongside the other module declarations.

**Step 4: Verify it compiles**

Run: `cd /Users/jerome/Projets/web/php/oddex/engine && cargo check`
Expected: compiles with no errors (unused warnings OK).

**Step 5: Commit**

```bash
git add engine/src/stats/
git commit -m "feat(stats): add slot analytics types module"
```

---

### Task 2: Rust stats module — ClickHouse queries

**Files:**
- Create: `engine/src/stats/queries.rs`

**Step 1: Implement the queries module**

Create `engine/src/stats/queries.rs`. This module contains 7 query functions that each run a single ClickHouse SQL query and return typed results. All queries filter by `slot_duration` and optionally by `symbols`.

Key ClickHouse SQL translation notes from the Python MySQL queries:
- `SUM(resolved_up = 1)` → `countIf(winner = 'UP')`
- `SUM(resolved_up IS NOT NULL)` → `countIf(winner IS NOT NULL)`
- `HOUR(FROM_UNIXTIME(slot_ts))` → `toHour(toDateTime(slot_ts))`
- `WEEKDAY(FROM_UNIXTIME(slot_ts))` → `toDayOfWeek(toDateTime(slot_ts)) - 1` (0=Mon..6=Sun)
- Slot-level dedup: subquery `GROUP BY symbol, slot_ts, slot_duration` with `any(winner)` since winner is denormalized

```rust
use anyhow::Result;
use clickhouse::Client;

use super::types::*;

/// Params shared across all queries
pub struct StatsParams {
    pub slot_duration: u32,
    pub symbols: Vec<String>,
    pub hours: f64,
}

impl StatsParams {
    fn cutoff_seconds(&self) -> f64 {
        self.hours * 3600.0
    }

    /// Returns (max_minutes for heatmap, time bin size in minutes)
    fn timing_config(&self) -> (f32, f32) {
        match self.slot_duration {
            300 => (5.0, 1.0),      // 5m: 1-min bins
            900 => (12.0, 2.0),     // 15m: 2-min bins
            3600 => (50.0, 10.0),   // 1h: 10-min bins
            14400 => (200.0, 30.0), // 4h: 30-min bins
            86400 => (1320.0, 240.0), // 1d: 4-hour bins
            _ => (12.0, 2.0),       // default to 15m pattern
        }
    }

    /// Returns (cal_min, cal_max) calibration time window
    fn calibration_window(&self) -> (f32, f32) {
        match self.slot_duration {
            300 => (1.0, 4.0),
            900 => (4.0, 10.0),
            3600 => (10.0, 50.0),
            14400 => (30.0, 180.0),
            86400 => (120.0, 1200.0),
            _ => (4.0, 10.0),
        }
    }

    fn symbol_filter_sql(&self) -> String {
        if self.symbols.is_empty() {
            String::new()
        } else {
            let placeholders: Vec<&str> = self.symbols.iter().map(|_| "?").collect();
            format!("AND symbol IN ({})", placeholders.join(", "))
        }
    }

    fn bind_common<'a>(&'a self, mut query: clickhouse::query::Query) -> clickhouse::query::Query {
        // Bind cutoff
        query = query.bind(self.cutoff_seconds());
        query = query.bind(self.slot_duration);
        for s in &self.symbols {
            query = query.bind(s.as_str());
        }
        query
    }
}

pub async fn fetch_summary(client: &Client, params: &StatsParams) -> Result<Summary> {
    let sym_filter = params.symbol_filter_sql();
    let sql = format!(
        "SELECT
            count() AS total_slots,
            countIf(winner IS NOT NULL) AS resolved_slots,
            countIf(winner IS NULL) AS unresolved_slots
        FROM (
            SELECT symbol, slot_ts, slot_duration, any(winner) AS winner
            FROM slot_snapshots
            WHERE captured_at >= now() - INTERVAL ? SECOND
              AND slot_duration = ?
              {sym_filter}
            GROUP BY symbol, slot_ts, slot_duration
        )"
    );
    let mut query = client.query(&sql);
    query = params.bind_common(query);

    #[derive(clickhouse::Row, serde::Deserialize)]
    struct Row {
        total_slots: u64,
        resolved_slots: u64,
        unresolved_slots: u64,
    }
    let row = query.fetch_one::<Row>().await?;

    // Snapshot count + last snapshot
    let sym_filter2 = params.symbol_filter_sql();
    let sql2 = format!(
        "SELECT count() AS cnt, max(captured_at) AS last_ts
         FROM slot_snapshots
         WHERE captured_at >= now() - INTERVAL ? SECOND
           AND slot_duration = ?
           {sym_filter2}"
    );
    let mut q2 = client.query(&sql2);
    q2 = params.bind_common(q2);

    #[derive(clickhouse::Row, serde::Deserialize)]
    struct SnapRow {
        cnt: u64,
        #[serde(with = "clickhouse::serde::time::datetime64::millis::option")]
        last_ts: Option<time::OffsetDateTime>,
    }
    let snap = q2.fetch_one::<SnapRow>().await?;

    Ok(Summary {
        total_slots: row.total_slots,
        resolved_slots: row.resolved_slots,
        unresolved_slots: row.unresolved_slots,
        total_snapshots: snap.cnt,
        last_snapshot_at: snap.last_ts.map(|t| {
            t.format(&time::format_description::well_known::Rfc3339).unwrap_or_default()
        }),
    })
}

pub async fn fetch_heatmap(client: &Client, params: &StatsParams) -> Result<Vec<HeatmapCell>> {
    let (max_minutes, bin_size) = params.timing_config();
    let sym_filter = params.symbol_filter_sql();

    // Build timing CASE dynamically based on bin size
    let num_bins = (max_minutes / bin_size) as usize;
    let mut timing_cases = String::from("CASE\n");
    for i in 0..num_bins {
        let lo = i as f32 * bin_size;
        let hi = lo + bin_size;
        if i == num_bins - 1 {
            timing_cases.push_str(&format!(
                "    ELSE '{}-{}'\n", lo as u32, hi as u32
            ));
        } else {
            timing_cases.push_str(&format!(
                "    WHEN minutes_into_slot < {} THEN '{}-{}'\n", hi, lo as u32, hi as u32
            ));
        }
    }
    timing_cases.push_str("END");

    let sql = format!(
        "SELECT
            {timing_cases} AS time_bin,
            CASE
                WHEN dir_move_pct < -0.2 THEN '< -0.2'
                WHEN dir_move_pct < -0.1 THEN '-0.2/-0.1'
                WHEN dir_move_pct < 0    THEN '-0.1/0'
                WHEN dir_move_pct < 0.1  THEN '0/0.1'
                WHEN dir_move_pct < 0.2  THEN '0.1/0.2'
                ELSE '> 0.2'
            END AS move_bin,
            count() AS total,
            countIf(winner = 'UP') AS wins
        FROM slot_snapshots
        WHERE winner IS NOT NULL
          AND captured_at >= now() - INTERVAL ? SECOND
          AND slot_duration = ?
          AND minutes_into_slot <= {max_minutes}
          {sym_filter}
        GROUP BY time_bin, move_bin"
    );

    let mut query = client.query(&sql);
    query = params.bind_common(query);

    #[derive(clickhouse::Row, serde::Deserialize)]
    struct Row {
        time_bin: String,
        move_bin: String,
        total: u64,
        wins: u64,
    }

    let mut cursor = query.fetch::<Row>()?;
    let mut results = Vec::new();
    while let Some(row) = cursor.next().await? {
        let win_rate = if row.total >= 3 {
            (row.wins as f64 / row.total as f64) * 100.0
        } else {
            -1.0 // sentinel for < 3 samples
        };
        results.push(HeatmapCell {
            time_bin: row.time_bin,
            move_bin: row.move_bin,
            total: row.total,
            wins: row.wins,
            win_rate,
        });
    }
    Ok(results)
}

pub async fn fetch_calibration(client: &Client, params: &StatsParams) -> Result<Vec<CalibrationPoint>> {
    let (cal_min, cal_max) = params.calibration_window();
    let sym_filter = params.symbol_filter_sql();

    let sql = format!(
        "SELECT
            round(bid_up * 20) / 20 AS bid_bucket,
            avg(bid_up) AS avg_bid,
            count() AS total,
            countIf(winner = 'UP') AS wins
        FROM slot_snapshots
        WHERE winner IS NOT NULL
          AND bid_up BETWEEN 0.10 AND 0.95
          AND minutes_into_slot BETWEEN {cal_min} AND {cal_max}
          AND captured_at >= now() - INTERVAL ? SECOND
          AND slot_duration = ?
          {sym_filter}
        GROUP BY bid_bucket
        HAVING total >= 3
        ORDER BY bid_bucket"
    );

    let mut query = client.query(&sql);
    query = params.bind_common(query);

    #[derive(clickhouse::Row, serde::Deserialize)]
    struct Row {
        bid_bucket: f64,
        avg_bid: f64,
        total: u64,
        wins: u64,
    }

    let mut cursor = query.fetch::<Row>()?;
    let mut results = Vec::new();
    while let Some(row) = cursor.next().await? {
        results.push(CalibrationPoint {
            bid_bucket: row.bid_bucket,
            avg_bid: row.avg_bid,
            win_rate: (row.wins as f64 / row.total as f64) * 100.0,
            sample_count: row.total,
        });
    }
    Ok(results)
}

pub async fn fetch_by_symbol(client: &Client, params: &StatsParams) -> Result<Vec<SymbolStats>> {
    let sym_filter = params.symbol_filter_sql();

    let sql = format!(
        "SELECT symbol, count() AS total, countIf(winner = 'UP') AS wins
        FROM (
            SELECT symbol, slot_ts, slot_duration, any(winner) AS winner
            FROM slot_snapshots
            WHERE winner IS NOT NULL
              AND captured_at >= now() - INTERVAL ? SECOND
              AND slot_duration = ?
              {sym_filter}
            GROUP BY symbol, slot_ts, slot_duration
        )
        GROUP BY symbol
        ORDER BY symbol"
    );

    let mut query = client.query(&sql);
    query = params.bind_common(query);

    #[derive(clickhouse::Row, serde::Deserialize)]
    struct Row {
        symbol: String,
        total: u64,
        wins: u64,
    }

    let mut cursor = query.fetch::<Row>()?;
    let mut results = Vec::new();
    while let Some(row) = cursor.next().await? {
        let win_rate = if row.total > 0 {
            (row.wins as f64 / row.total as f64) * 100.0
        } else {
            0.0
        };
        results.push(SymbolStats {
            symbol: row.symbol,
            total: row.total,
            wins: row.wins,
            win_rate,
        });
    }
    Ok(results)
}

pub async fn fetch_stoploss_sweep(client: &Client, params: &StatsParams) -> Result<Vec<StoplossThreshold>> {
    let (max_minutes, _) = params.timing_config();
    let sym_filter = params.symbol_filter_sql();
    let peak: f64 = 0.75;

    let sql = format!(
        "WITH
        peak_minute AS (
            SELECT symbol, slot_ts, slot_duration,
                   min(minutes_into_slot) AS first_peak_min
            FROM slot_snapshots
            WHERE bid_up >= {peak}
              AND minutes_into_slot <= {max_minutes}
              AND captured_at >= now() - INTERVAL ? SECOND
              AND slot_duration = ?
              {sym_filter}
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
              AND ss.captured_at >= now() - INTERVAL ? SECOND
              AND ss.slot_duration = ?
              {sym_filter}
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

    let mut query = client.query(&sql);
    // peak_minute CTE binds: cutoff, slot_duration, symbols
    query = query.bind(params.cutoff_seconds());
    query = query.bind(params.slot_duration);
    for s in &params.symbols {
        query = query.bind(s.as_str());
    }
    // peaked CTE binds: cutoff, slot_duration, symbols
    query = query.bind(params.cutoff_seconds());
    query = query.bind(params.slot_duration);
    for s in &params.symbols {
        query = query.bind(s.as_str());
    }

    #[derive(clickhouse::Row, serde::Deserialize)]
    struct Row {
        threshold: f64,
        triggered: u64,
        true_saves: u64,
        false_exits: u64,
    }

    let mut cursor = query.fetch::<Row>()?;
    let mut results = Vec::new();
    while let Some(row) = cursor.next().await? {
        let precision = if row.triggered > 0 {
            (row.true_saves as f64 / row.triggered as f64) * 100.0
        } else {
            0.0
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

pub async fn fetch_by_hour(client: &Client, params: &StatsParams) -> Result<Vec<TimeStats>> {
    let sym_filter = params.symbol_filter_sql();

    let sql = format!(
        "SELECT hour_utc AS period, count() AS total, countIf(winner = 'UP') AS wins
        FROM (
            SELECT toHour(toDateTime(slot_ts)) AS hour_utc, any(winner) AS winner
            FROM slot_snapshots
            WHERE winner IS NOT NULL
              AND captured_at >= now() - INTERVAL ? SECOND
              AND slot_duration = ?
              {sym_filter}
            GROUP BY symbol, slot_ts, slot_duration, hour_utc
        )
        GROUP BY period
        ORDER BY period"
    );

    let mut query = client.query(&sql);
    query = params.bind_common(query);

    #[derive(clickhouse::Row, serde::Deserialize)]
    struct Row {
        period: u8,
        total: u64,
        wins: u64,
    }

    let mut cursor = query.fetch::<Row>()?;
    let mut results = Vec::new();
    while let Some(row) = cursor.next().await? {
        results.push(TimeStats {
            period: row.period,
            total: row.total,
            wins: row.wins,
            win_rate: if row.total > 0 { (row.wins as f64 / row.total as f64) * 100.0 } else { 0.0 },
        });
    }
    Ok(results)
}

pub async fn fetch_by_day(client: &Client, params: &StatsParams) -> Result<Vec<TimeStats>> {
    let sym_filter = params.symbol_filter_sql();

    // toDayOfWeek returns 1=Monday..7=Sunday, we subtract 1 for 0=Monday..6=Sunday
    let sql = format!(
        "SELECT day_of_week AS period, count() AS total, countIf(winner = 'UP') AS wins
        FROM (
            SELECT (toDayOfWeek(toDateTime(slot_ts)) - 1) AS day_of_week, any(winner) AS winner
            FROM slot_snapshots
            WHERE winner IS NOT NULL
              AND captured_at >= now() - INTERVAL ? SECOND
              AND slot_duration = ?
              {sym_filter}
            GROUP BY symbol, slot_ts, slot_duration, day_of_week
        )
        GROUP BY period
        ORDER BY period"
    );

    let mut query = client.query(&sql);
    query = params.bind_common(query);

    #[derive(clickhouse::Row, serde::Deserialize)]
    struct Row {
        period: u8,
        total: u64,
        wins: u64,
    }

    let mut cursor = query.fetch::<Row>()?;
    let mut results = Vec::new();
    while let Some(row) = cursor.next().await? {
        results.push(TimeStats {
            period: row.period,
            total: row.total,
            wins: row.wins,
            win_rate: if row.total > 0 { (row.wins as f64 / row.total as f64) * 100.0 } else { 0.0 },
        });
    }
    Ok(results)
}
```

> **Note:** The `bind_common` helper in `StatsParams` may need adjustment since ClickHouse's `INTERVAL ? SECOND` might not support bind params for interval values. If compilation or runtime fails, switch to string interpolation for the cutoff value (safe since it's a computed f64, not user input). The same applies to `max_minutes`, `cal_min`, `cal_max` which are already interpolated.

**Step 2: Verify it compiles**

Run: `cd /Users/jerome/Projets/web/php/oddex/engine && cargo check`
Expected: compiles (may have unused warnings).

**Step 3: Commit**

```bash
git add engine/src/stats/queries.rs
git commit -m "feat(stats): add ClickHouse aggregation queries for slot analytics"
```

---

### Task 3: Rust stats handler — Axum endpoint

**Files:**
- Create: `engine/src/api/handlers/stats.rs`
- Modify: `engine/src/api/handlers/mod.rs` (add `pub mod stats;`)
- Modify: `engine/src/api/mod.rs` (add route)

**Step 1: Create the handler**

Create `engine/src/api/handlers/stats.rs`:

```rust
use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::api::error::ApiError;
use crate::api::state::ApiState;
use crate::stats::queries::{self, StatsParams};
use crate::stats::types::SlotStatsResponse;

#[derive(Deserialize)]
pub struct StatsQuery {
    pub slot_duration: u32,
    #[serde(default)]
    pub symbols: Option<String>, // comma-separated: "BTC,ETH"
    #[serde(default = "default_hours")]
    pub hours: f64,
}

fn default_hours() -> f64 {
    168.0 // 7 days
}

pub async fn slots(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<StatsQuery>,
) -> Result<Json<SlotStatsResponse>, ApiError> {
    let symbols: Vec<String> = q.symbols
        .map(|s| s.split(',').map(|v| v.trim().to_uppercase()).filter(|v| !v.is_empty()).collect())
        .unwrap_or_default();

    let params = StatsParams {
        slot_duration: q.slot_duration,
        symbols,
        hours: q.hours.clamp(1.0, 2160.0),
    };

    let (summary, heatmap, calibration, by_symbol, stoploss_sweep, by_hour, by_day) = tokio::try_join!(
        queries::fetch_summary(&state.ch, &params),
        queries::fetch_heatmap(&state.ch, &params),
        queries::fetch_calibration(&state.ch, &params),
        queries::fetch_by_symbol(&state.ch, &params),
        queries::fetch_stoploss_sweep(&state.ch, &params),
        queries::fetch_by_hour(&state.ch, &params),
        queries::fetch_by_day(&state.ch, &params),
    )
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(SlotStatsResponse {
        summary,
        heatmap,
        calibration,
        by_symbol,
        stoploss_sweep,
        by_hour,
        by_day,
    }))
}
```

**Step 2: Register the handler module**

Add to `engine/src/api/handlers/mod.rs`:

```rust
pub mod stats;
```

**Step 3: Register the route**

Add to `engine/src/api/mod.rs` in the `router()` function, before `.with_state(state)`:

```rust
.route("/internal/stats/slots", get(handlers::stats::slots))
```

**Step 4: Verify it compiles**

Run: `cd /Users/jerome/Projets/web/php/oddex/engine && cargo check`
Expected: compiles.

**Step 5: Commit**

```bash
git add engine/src/api/handlers/stats.rs engine/src/api/handlers/mod.rs engine/src/api/mod.rs
git commit -m "feat(stats): add GET /internal/stats/slots endpoint"
```

---

### Task 4: Laravel — EngineService method + AnalyticsController + route

**Files:**
- Modify: `web/app/Services/EngineService.php` (add `slotStats` method)
- Create: `web/app/Http/Controllers/AnalyticsController.php` (via artisan)
- Modify: `web/routes/web.php` (add route)

**Step 1: Add method to EngineService**

Add to `web/app/Services/EngineService.php` after the `engineStatus()` method:

```php
public function slotStats(int $slotDuration, array $symbols = [], float $hours = 168.0): array
{
    return $this->client()
        ->get('/internal/stats/slots', array_filter([
            'slot_duration' => $slotDuration,
            'symbols' => !empty($symbols) ? implode(',', $symbols) : null,
            'hours' => $hours,
        ]))
        ->throw()
        ->json();
}
```

**Step 2: Create AnalyticsController**

Run: `cd /Users/jerome/Projets/web/php/oddex/web && php artisan make:controller AnalyticsController --no-interaction`

Then edit `web/app/Http/Controllers/AnalyticsController.php`:

```php
<?php

namespace App\Http\Controllers;

use App\Services\EngineService;
use Illuminate\Http\Request;
use Inertia\Inertia;
use Inertia\Response;

class AnalyticsController extends Controller
{
    public function index(Request $request, EngineService $engine): Response
    {
        $slotDuration = (int) $request->query('slot_duration', 900);
        $symbols = $request->query('symbols')
            ? array_filter(explode(',', $request->query('symbols')))
            : [];
        $hours = (float) $request->query('hours', 168.0);

        try {
            $stats = $engine->slotStats($slotDuration, $symbols, $hours);
        } catch (\Illuminate\Http\Client\RequestException) {
            $stats = null;
        }

        return Inertia::render('analytics/index', [
            'stats' => $stats,
            'filters' => [
                'slot_duration' => $slotDuration,
                'symbols' => $symbols,
                'hours' => $hours,
            ],
        ]);
    }
}
```

**Step 3: Add the route**

Add to `web/routes/web.php` inside the `Route::middleware(['auth', 'verified'])` group, after the Billing routes:

```php
// Analytics
Route::get('analytics', [AnalyticsController::class, 'index'])->name('analytics.index');
```

Don't forget to add the `use` statement at the top of the file:

```php
use App\Http\Controllers\AnalyticsController;
```

**Step 4: Run Pint**

Run: `cd /Users/jerome/Projets/web/php/oddex/web && vendor/bin/pint --dirty --format agent`

**Step 5: Commit**

```bash
git add web/app/Services/EngineService.php web/app/Http/Controllers/AnalyticsController.php web/routes/web.php
git commit -m "feat(analytics): add AnalyticsController with engine integration"
```

---

### Task 5: Laravel tests — AnalyticsController

**Files:**
- Create: `web/tests/Feature/AnalyticsControllerTest.php`
- Create: `web/tests/Unit/Services/EngineServiceSlotStatsTest.php`

**Step 1: Create the feature test**

Run: `cd /Users/jerome/Projets/web/php/oddex/web && php artisan make:test --pest AnalyticsControllerTest --no-interaction`

Edit `web/tests/Feature/AnalyticsControllerTest.php`:

```php
<?php

use App\Models\User;
use Illuminate\Support\Facades\Http;
use Inertia\Testing\AssertableInertia as Assert;

beforeEach(function () {
    $this->withoutVite();
    $this->user = User::factory()->create();
});

it('displays analytics page with stats from engine', function () {
    Http::fake(['*/internal/stats/slots*' => Http::response([
        'summary' => [
            'total_slots' => 100,
            'resolved_slots' => 90,
            'unresolved_slots' => 10,
            'total_snapshots' => 5000,
            'last_snapshot_at' => '2026-02-22T12:00:00Z',
        ],
        'heatmap' => [],
        'calibration' => [],
        'by_symbol' => [
            ['symbol' => 'BTC', 'total' => 50, 'wins' => 26, 'win_rate' => 52.0],
        ],
        'stoploss_sweep' => [],
        'by_hour' => [],
        'by_day' => [],
    ])]);

    $this->actingAs($this->user)
        ->get(route('analytics.index'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('analytics/index', false)
            ->has('stats')
            ->has('filters')
        );
});

it('passes filters to the engine', function () {
    Http::fake(['*/internal/stats/slots*' => Http::response([
        'summary' => ['total_slots' => 0, 'resolved_slots' => 0, 'unresolved_slots' => 0, 'total_snapshots' => 0, 'last_snapshot_at' => null],
        'heatmap' => [], 'calibration' => [], 'by_symbol' => [],
        'stoploss_sweep' => [], 'by_hour' => [], 'by_day' => [],
    ])]);

    $this->actingAs($this->user)
        ->get(route('analytics.index', ['slot_duration' => 300, 'symbols' => 'BTC,ETH']))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->where('filters.slot_duration', 300)
            ->where('filters.symbols', ['BTC', 'ETH'])
        );

    Http::assertSent(fn ($request) => str_contains($request->url(), 'slot_duration=300')
        && str_contains($request->url(), 'symbols=BTC%2CETH')
    );
});

it('handles engine errors gracefully', function () {
    Http::fake(['*/internal/stats/slots*' => Http::response([], 500)]);

    $this->actingAs($this->user)
        ->get(route('analytics.index'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('analytics/index', false)
            ->where('stats', null)
        );
});

it('requires authentication', function () {
    $this->get(route('analytics.index'))->assertRedirect('/login');
});
```

**Step 2: Run the tests**

Run: `cd /Users/jerome/Projets/web/php/oddex/web && php artisan test --compact --filter=AnalyticsController`
Expected: 4 tests pass.

**Step 3: Commit**

```bash
git add web/tests/Feature/AnalyticsControllerTest.php
git commit -m "test(analytics): add AnalyticsController feature tests"
```

---

### Task 6: TypeScript types for analytics data

**Files:**
- Modify: `web/resources/js/types/models.ts`

**Step 1: Add analytics types**

Add to the end of `web/resources/js/types/models.ts`:

```typescript
// Slot Analytics types
export interface SlotAnalyticsSummary {
    total_slots: number;
    resolved_slots: number;
    unresolved_slots: number;
    total_snapshots: number;
    last_snapshot_at: string | null;
}

export interface HeatmapCell {
    time_bin: string;
    move_bin: string;
    total: number;
    wins: number;
    win_rate: number;
}

export interface CalibrationPoint {
    bid_bucket: number;
    avg_bid: number;
    win_rate: number;
    sample_count: number;
}

export interface SymbolStats {
    symbol: string;
    total: number;
    wins: number;
    win_rate: number;
}

export interface StoplossThreshold {
    threshold: number;
    triggered: number;
    true_saves: number;
    false_exits: number;
    precision: number;
}

export interface TimeStats {
    period: number;
    total: number;
    wins: number;
    win_rate: number;
}

export interface SlotAnalyticsData {
    summary: SlotAnalyticsSummary;
    heatmap: HeatmapCell[];
    calibration: CalibrationPoint[];
    by_symbol: SymbolStats[];
    stoploss_sweep: StoplossThreshold[];
    by_hour: TimeStats[];
    by_day: TimeStats[];
}

export interface AnalyticsFilters {
    slot_duration: number;
    symbols: string[];
    hours: number;
}
```

**Step 2: Commit**

```bash
git add web/resources/js/types/models.ts
git commit -m "feat(analytics): add TypeScript types for slot analytics"
```

---

### Task 7: React chart components — WinRateBarChart (reusable)

**Files:**
- Create: `web/resources/js/components/charts/win-rate-bar-chart.tsx`

This reusable component will be used for WR by Symbol, WR by Hour, and WR by Day.

**Step 1: Create the component**

Create `web/resources/js/components/charts/win-rate-bar-chart.tsx`:

```tsx
import { Bar, BarChart, CartesianGrid, Cell, ReferenceLine, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts';

interface WinRateBarChartProps {
    data: Array<{ label: string; winRate: number; total: number }>;
    height?: number;
}

export function WinRateBarChart({ data, height = 300 }: WinRateBarChartProps) {
    return (
        <ResponsiveContainer width="100%" height={height}>
            <BarChart data={data}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
                <XAxis dataKey="label" tick={{ fontSize: 12 }} />
                <YAxis domain={[0, 100]} tick={{ fontSize: 12 }} tickFormatter={(v) => `${v}%`} />
                <Tooltip
                    formatter={(value: number, _name: string, props: { payload: { total: number } }) => [
                        `${value.toFixed(1)}% (n=${props.payload.total})`,
                        'Win Rate',
                    ]}
                    contentStyle={{
                        background: 'hsl(var(--background))',
                        border: '1px solid hsl(var(--border))',
                    }}
                />
                <ReferenceLine y={50} stroke="hsl(var(--muted-foreground))" strokeDasharray="3 3" />
                <Bar dataKey="winRate" radius={[4, 4, 0, 0]}>
                    {data.map((entry, i) => (
                        <Cell
                            key={i}
                            fill={entry.winRate >= 50 ? 'hsl(var(--chart-2))' : 'hsl(var(--chart-5))'}
                        />
                    ))}
                </Bar>
            </BarChart>
        </ResponsiveContainer>
    );
}
```

**Step 2: Commit**

```bash
git add web/resources/js/components/charts/win-rate-bar-chart.tsx
git commit -m "feat(analytics): add reusable WinRateBarChart component"
```

---

### Task 8: React chart components — WinRateHeatmap

**Files:**
- Create: `web/resources/js/components/charts/win-rate-heatmap.tsx`

The heatmap is a custom grid (not a standard Recharts chart). We use a simple CSS grid with colored cells.

**Step 1: Create the component**

Create `web/resources/js/components/charts/win-rate-heatmap.tsx`:

```tsx
import type { HeatmapCell } from '@/types/models';

interface WinRateHeatmapProps {
    data: HeatmapCell[];
}

const MOVE_BINS = ['> 0.2', '0.1/0.2', '0/0.1', '-0.1/0', '-0.2/-0.1', '< -0.2'];

function cellColor(winRate: number, total: number): string {
    if (total < 3 || winRate < 0) return 'bg-muted text-muted-foreground/50';
    if (winRate >= 65) return 'bg-emerald-600 text-white';
    if (winRate >= 55) return 'bg-emerald-500/70 text-white';
    if (winRate >= 50) return 'bg-emerald-400/40 text-foreground';
    if (winRate >= 45) return 'bg-red-400/40 text-foreground';
    if (winRate >= 35) return 'bg-red-500/70 text-white';
    return 'bg-red-600 text-white';
}

export function WinRateHeatmap({ data }: WinRateHeatmapProps) {
    const timeBins = [...new Set(data.map((d) => d.time_bin))].sort((a, b) => {
        const numA = parseInt(a.split('-')[0]);
        const numB = parseInt(b.split('-')[0]);
        return numA - numB;
    });

    const lookup = new Map<string, HeatmapCell>();
    for (const cell of data) {
        lookup.set(`${cell.time_bin}|${cell.move_bin}`, cell);
    }

    return (
        <div className="overflow-x-auto">
            <table className="w-full border-collapse text-sm">
                <thead>
                    <tr>
                        <th className="p-2 text-left text-xs font-medium text-muted-foreground">Move \ Time</th>
                        {timeBins.map((bin) => (
                            <th key={bin} className="p-2 text-center text-xs font-medium text-muted-foreground">
                                {bin}m
                            </th>
                        ))}
                    </tr>
                </thead>
                <tbody>
                    {MOVE_BINS.map((moveBin) => (
                        <tr key={moveBin}>
                            <td className="p-2 text-xs font-medium text-muted-foreground whitespace-nowrap">
                                {moveBin}%
                            </td>
                            {timeBins.map((timeBin) => {
                                const cell = lookup.get(`${timeBin}|${moveBin}`);
                                const wr = cell?.win_rate ?? -1;
                                const total = cell?.total ?? 0;
                                return (
                                    <td
                                        key={timeBin}
                                        className={`p-2 text-center text-xs font-mono rounded-sm ${cellColor(wr, total)}`}
                                        title={`WR: ${wr >= 0 ? wr.toFixed(1) : 'N/A'}% | n=${total}`}
                                    >
                                        {total >= 3 ? (
                                            <>
                                                {wr.toFixed(0)}%
                                                <span className="block text-[10px] opacity-70">n={total}</span>
                                            </>
                                        ) : (
                                            <span className="opacity-40">-</span>
                                        )}
                                    </td>
                                );
                            })}
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}
```

**Step 2: Commit**

```bash
git add web/resources/js/components/charts/win-rate-heatmap.tsx
git commit -m "feat(analytics): add WinRateHeatmap component"
```

---

### Task 9: React chart components — CalibrationChart

**Files:**
- Create: `web/resources/js/components/charts/calibration-chart.tsx`

**Step 1: Create the component**

Create `web/resources/js/components/charts/calibration-chart.tsx`:

```tsx
import { CartesianGrid, Line, LineChart, ReferenceLine, ResponsiveContainer, Scatter, ScatterChart, Tooltip, XAxis, YAxis, ZAxis } from 'recharts';
import type { CalibrationPoint } from '@/types/models';

interface CalibrationChartProps {
    data: CalibrationPoint[];
}

export function CalibrationChart({ data }: CalibrationChartProps) {
    const chartData = data.map((d) => ({
        impliedProb: d.avg_bid * 100,
        actualWinRate: d.win_rate,
        sampleCount: d.sample_count,
    }));

    return (
        <ResponsiveContainer width="100%" height={300}>
            <ScatterChart>
                <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
                <XAxis
                    dataKey="impliedProb"
                    type="number"
                    domain={[10, 95]}
                    tick={{ fontSize: 12 }}
                    label={{ value: 'Market P(Up) %', position: 'insideBottom', offset: -5, fontSize: 12 }}
                />
                <YAxis
                    dataKey="actualWinRate"
                    type="number"
                    domain={[10, 95]}
                    tick={{ fontSize: 12 }}
                    label={{ value: 'Actual WR %', angle: -90, position: 'insideLeft', fontSize: 12 }}
                />
                <ZAxis dataKey="sampleCount" range={[40, 400]} />
                <Tooltip
                    formatter={(value: number, name: string) => [
                        `${value.toFixed(1)}%`,
                        name === 'impliedProb' ? 'Market P(Up)' : 'Actual WR',
                    ]}
                    contentStyle={{
                        background: 'hsl(var(--background))',
                        border: '1px solid hsl(var(--border))',
                    }}
                />
                <ReferenceLine
                    segment={[{ x: 10, y: 10 }, { x: 95, y: 95 }]}
                    stroke="hsl(var(--muted-foreground))"
                    strokeDasharray="3 3"
                    label={{ value: 'Perfect calibration', position: 'end', fontSize: 11 }}
                />
                <Scatter data={chartData} fill="hsl(var(--chart-1))" />
            </ScatterChart>
        </ResponsiveContainer>
    );
}
```

**Step 2: Commit**

```bash
git add web/resources/js/components/charts/calibration-chart.tsx
git commit -m "feat(analytics): add CalibrationChart component"
```

---

### Task 10: React chart components — StoplossSweepChart

**Files:**
- Create: `web/resources/js/components/charts/stoploss-sweep-chart.tsx`

**Step 1: Create the component**

Create `web/resources/js/components/charts/stoploss-sweep-chart.tsx`:

```tsx
import { Bar, CartesianGrid, ComposedChart, Line, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts';
import type { StoplossThreshold } from '@/types/models';

interface StoplossSweepChartProps {
    data: StoplossThreshold[];
}

export function StoplossSweepChart({ data }: StoplossSweepChartProps) {
    const chartData = data.map((d) => ({
        threshold: d.threshold.toFixed(2),
        trueSaves: d.true_saves,
        falseExits: d.false_exits,
        precision: d.precision,
    }));

    return (
        <ResponsiveContainer width="100%" height={350}>
            <ComposedChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
                <XAxis dataKey="threshold" tick={{ fontSize: 11 }} />
                <YAxis yAxisId="left" tick={{ fontSize: 12 }} label={{ value: 'Count', angle: -90, position: 'insideLeft', fontSize: 12 }} />
                <YAxis
                    yAxisId="right"
                    orientation="right"
                    domain={[0, 100]}
                    tick={{ fontSize: 12 }}
                    tickFormatter={(v) => `${v}%`}
                    label={{ value: 'Precision', angle: 90, position: 'insideRight', fontSize: 12 }}
                />
                <Tooltip
                    contentStyle={{
                        background: 'hsl(var(--background))',
                        border: '1px solid hsl(var(--border))',
                    }}
                />
                <Bar yAxisId="left" dataKey="trueSaves" stackId="a" fill="hsl(var(--chart-2))" name="True Saves" />
                <Bar yAxisId="left" dataKey="falseExits" stackId="a" fill="hsl(var(--chart-5))" name="False Exits" radius={[4, 4, 0, 0]} />
                <Line yAxisId="right" type="monotone" dataKey="precision" stroke="hsl(var(--chart-1))" strokeWidth={2} dot={false} name="Precision %" />
            </ComposedChart>
        </ResponsiveContainer>
    );
}
```

**Step 2: Commit**

```bash
git add web/resources/js/components/charts/stoploss-sweep-chart.tsx
git commit -m "feat(analytics): add StoplossSweepChart component"
```

---

### Task 11: React page — Analytics index

**Files:**
- Create: `web/resources/js/pages/analytics/index.tsx`

**Step 1: Create the page**

Create `web/resources/js/pages/analytics/index.tsx`:

```tsx
import { Head, router } from '@inertiajs/react';
import { BarChart3, Clock, Database, Layers, TrendingUp } from 'lucide-react';
import AppLayout from '@/layouts/app-layout';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import MetricCard from '@/components/metric-card';
import { WinRateHeatmap } from '@/components/charts/win-rate-heatmap';
import { WinRateBarChart } from '@/components/charts/win-rate-bar-chart';
import { CalibrationChart } from '@/components/charts/calibration-chart';
import { StoplossSweepChart } from '@/components/charts/stoploss-sweep-chart';
import { index as analyticsIndex } from '@/actions/App/Http/Controllers/AnalyticsController';
import type { BreadcrumbItem } from '@/types';
import type { SlotAnalyticsData, AnalyticsFilters } from '@/types/models';

const DURATION_OPTIONS = [
    { value: '300', label: '5 min' },
    { value: '900', label: '15 min' },
    { value: '3600', label: '1 hour' },
    { value: '14400', label: '4 hours' },
    { value: '86400', label: '1 day' },
];

const SYMBOL_OPTIONS = ['BTC', 'ETH', 'SOL', 'XRP'];

const DAY_LABELS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Analytics', href: analyticsIndex.url() },
];

interface Props {
    stats: SlotAnalyticsData | null;
    filters: AnalyticsFilters;
}

function dataAge(lastSnapshotAt: string | null): string {
    if (!lastSnapshotAt) return 'N/A';
    const diffMs = Date.now() - new Date(lastSnapshotAt).getTime();
    const minutes = Math.floor(diffMs / 60000);
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    return `${hours}h ${minutes % 60}m ago`;
}

export default function AnalyticsIndex({ stats, filters }: Props) {
    function updateFilter(key: string, value: string) {
        router.get(
            analyticsIndex.url(),
            { ...filters, [key]: value },
            { preserveState: true, preserveScroll: true },
        );
    }

    function toggleSymbol(symbol: string) {
        const current = filters.symbols;
        const next = current.includes(symbol)
            ? current.filter((s) => s !== symbol)
            : [...current, symbol];
        updateFilter('symbols', next.join(','));
    }

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Slot Analytics" />

            <div className="space-y-6 p-6">
                {/* Header + Filters */}
                <div className="flex flex-wrap items-center justify-between gap-4">
                    <h1 className="text-2xl font-bold tracking-tight">Slot Analytics</h1>
                    <div className="flex items-center gap-3">
                        <Select
                            value={String(filters.slot_duration)}
                            onValueChange={(v) => updateFilter('slot_duration', v)}
                        >
                            <SelectTrigger className="w-[130px]">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                {DURATION_OPTIONS.map((opt) => (
                                    <SelectItem key={opt.value} value={opt.value}>
                                        {opt.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>

                        <div className="flex gap-1">
                            {SYMBOL_OPTIONS.map((sym) => (
                                <button
                                    key={sym}
                                    onClick={() => toggleSymbol(sym)}
                                    className={`rounded-md px-3 py-1.5 text-xs font-medium transition-colors ${
                                        filters.symbols.length === 0 || filters.symbols.includes(sym)
                                            ? 'bg-primary text-primary-foreground'
                                            : 'bg-muted text-muted-foreground hover:bg-muted/80'
                                    }`}
                                >
                                    {sym}
                                </button>
                            ))}
                        </div>
                    </div>
                </div>

                {!stats ? (
                    <Card>
                        <CardContent className="py-12 text-center text-muted-foreground">
                            Unable to load analytics data. The engine may be unavailable.
                        </CardContent>
                    </Card>
                ) : (
                    <>
                        {/* KPIs */}
                        <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
                            <MetricCard label="Total Slots" value={stats.summary.total_slots.toLocaleString()} icon={Layers} />
                            <MetricCard label="In Progress" value={stats.summary.unresolved_slots.toLocaleString()} icon={Clock} />
                            <MetricCard label="Snapshots" value={stats.summary.total_snapshots.toLocaleString()} icon={Database} />
                            <MetricCard label="Data Age" value={dataAge(stats.summary.last_snapshot_at)} icon={TrendingUp} />
                        </div>

                        {/* Heatmap */}
                        <Card>
                            <CardHeader>
                                <CardTitle>Win Rate: Entry Timing x Price Move</CardTitle>
                            </CardHeader>
                            <CardContent>
                                {stats.heatmap.length > 0 ? (
                                    <WinRateHeatmap data={stats.heatmap} />
                                ) : (
                                    <p className="py-8 text-center text-sm text-muted-foreground">No heatmap data available.</p>
                                )}
                            </CardContent>
                        </Card>

                        {/* Calibration + By Symbol */}
                        <div className="grid gap-4 md:grid-cols-2">
                            <Card>
                                <CardHeader>
                                    <CardTitle>Market Calibration</CardTitle>
                                </CardHeader>
                                <CardContent>
                                    {stats.calibration.length > 0 ? (
                                        <CalibrationChart data={stats.calibration} />
                                    ) : (
                                        <p className="py-8 text-center text-sm text-muted-foreground">No calibration data.</p>
                                    )}
                                </CardContent>
                            </Card>
                            <Card>
                                <CardHeader>
                                    <CardTitle>Win Rate by Symbol</CardTitle>
                                </CardHeader>
                                <CardContent>
                                    {stats.by_symbol.length > 0 ? (
                                        <WinRateBarChart
                                            data={stats.by_symbol.map((s) => ({
                                                label: s.symbol,
                                                winRate: s.win_rate,
                                                total: s.total,
                                            }))}
                                        />
                                    ) : (
                                        <p className="py-8 text-center text-sm text-muted-foreground">No symbol data.</p>
                                    )}
                                </CardContent>
                            </Card>
                        </div>

                        {/* Stoploss Sweep */}
                        <Card>
                            <CardHeader>
                                <CardTitle>Stop-Loss Threshold Sweep</CardTitle>
                            </CardHeader>
                            <CardContent>
                                {stats.stoploss_sweep.length > 0 ? (
                                    <StoplossSweepChart data={stats.stoploss_sweep} />
                                ) : (
                                    <p className="py-8 text-center text-sm text-muted-foreground">No stop-loss data.</p>
                                )}
                            </CardContent>
                        </Card>

                        {/* By Hour + By Day */}
                        <div className="grid gap-4 md:grid-cols-2">
                            <Card>
                                <CardHeader>
                                    <CardTitle>Win Rate by Hour (UTC)</CardTitle>
                                </CardHeader>
                                <CardContent>
                                    {stats.by_hour.length > 0 ? (
                                        <WinRateBarChart
                                            data={stats.by_hour.map((h) => ({
                                                label: `${h.period}h`,
                                                winRate: h.win_rate,
                                                total: h.total,
                                            }))}
                                        />
                                    ) : (
                                        <p className="py-8 text-center text-sm text-muted-foreground">No hourly data.</p>
                                    )}
                                </CardContent>
                            </Card>
                            <Card>
                                <CardHeader>
                                    <CardTitle>Win Rate by Day</CardTitle>
                                </CardHeader>
                                <CardContent>
                                    {stats.by_day.length > 0 ? (
                                        <WinRateBarChart
                                            data={stats.by_day.map((d) => ({
                                                label: DAY_LABELS[d.period] ?? `Day ${d.period}`,
                                                winRate: d.win_rate,
                                                total: d.total,
                                            }))}
                                        />
                                    ) : (
                                        <p className="py-8 text-center text-sm text-muted-foreground">No daily data.</p>
                                    )}
                                </CardContent>
                            </Card>
                        </div>

                        {/* Summary footer */}
                        <p className="text-center text-xs text-muted-foreground">
                            {stats.summary.resolved_slots.toLocaleString()} resolved slots |{' '}
                            Overall WR:{' '}
                            {stats.by_symbol.length > 0
                                ? (
                                      (stats.by_symbol.reduce((sum, s) => sum + s.wins, 0) /
                                          Math.max(1, stats.by_symbol.reduce((sum, s) => sum + s.total, 0))) *
                                      100
                                  ).toFixed(1)
                                : 'N/A'}
                            % | Last {filters.hours}h
                        </p>
                    </>
                )}
            </div>
        </AppLayout>
    );
}
```

**Step 2: Generate Wayfinder routes**

Run: `cd /Users/jerome/Projets/web/php/oddex/web && npx @laravel/wayfinder-gen`

(This generates the `@/actions/App/Http/Controllers/AnalyticsController` import used by the page.)

**Step 3: Verify build**

Run: `cd /Users/jerome/Projets/web/php/oddex/web && npm run build`
Expected: builds with no errors.

**Step 4: Commit**

```bash
git add web/resources/js/pages/analytics/
git commit -m "feat(analytics): add Analytics page with all 7 visualizations"
```

---

### Task 12: Sidebar navigation — add Analytics entry

**Files:**
- Modify: `web/resources/js/components/app-sidebar.tsx`

**Step 1: Add the Analytics nav item**

In `web/resources/js/components/app-sidebar.tsx`:

1. Add import at the top, alongside other icon imports:
```tsx
import { BarChart3 } from 'lucide-react';
```

2. Add import for the Wayfinder action:
```tsx
import { index as analyticsIndex } from '@/actions/App/Http/Controllers/AnalyticsController';
```

3. Add to `mainNavItems` array, between `Backtests` and `Billing`:
```tsx
{ title: 'Analytics', href: analyticsIndex.url(), icon: BarChart3 },
```

**Step 2: Verify build**

Run: `cd /Users/jerome/Projets/web/php/oddex/web && npm run build`
Expected: builds with no errors.

**Step 3: Commit**

```bash
git add web/resources/js/components/app-sidebar.tsx
git commit -m "feat(analytics): add Analytics entry to sidebar navigation"
```

---

### Task 13: Final verification

**Step 1: Run all PHP tests**

Run: `cd /Users/jerome/Projets/web/php/oddex/web && php artisan test --compact`
Expected: all tests pass.

**Step 2: Run Pint on all dirty files**

Run: `cd /Users/jerome/Projets/web/php/oddex/web && vendor/bin/pint --dirty --format agent`

**Step 3: Verify full frontend build**

Run: `cd /Users/jerome/Projets/web/php/oddex/web && npm run build`
Expected: builds with no errors.

**Step 4: Verify Rust engine compiles**

Run: `cd /Users/jerome/Projets/web/php/oddex/engine && cargo check`
Expected: compiles with no errors.

**Step 5: Final commit if any formatting changes**

```bash
git add -A && git commit -m "style: apply pint formatting"
```
