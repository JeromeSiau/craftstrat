use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use time::Duration;

use crate::api::error::ApiError;
use crate::api::state::ApiState;
use crate::stats::queries::{self, MlDatasetParams, StatsParams};
use crate::stats::types::{MlDatasetResponse, SlotStatsResponse};

#[derive(Deserialize)]
pub struct StatsQuery {
    pub slot_duration: u32,
    #[serde(default)]
    pub symbols: Option<String>,
    #[serde(default = "default_hours")]
    pub hours: f64,
}

fn default_hours() -> f64 {
    168.0
}

fn default_ml_hours() -> f64 {
    24.0 * 30.0
}

fn default_sample_every() -> u32 {
    1
}

fn default_limit() -> u32 {
    10_000
}

fn parse_symbols(symbols: Option<String>) -> Vec<String> {
    symbols
        .map(|s| {
            s.split(',')
                .map(|v| v.trim().to_lowercase())
                .filter(|v| !v.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn validate_slot_duration(slot_duration: u32) -> Result<(), ApiError> {
    match slot_duration {
        300 | 900 | 3600 | 14400 | 86400 => Ok(()),
        _ => Err(ApiError::Validation(
            "slot_duration must be one of 300, 900, 3600, 14400, 86400".into(),
        )),
    }
}

fn validate_hours(hours: f64) -> Result<(), ApiError> {
    if !hours.is_finite() || hours < 1.0 || hours > Duration::days(365 * 5).whole_hours() as f64 {
        return Err(ApiError::Validation(
            "hours must be a finite value between 1 and 43800".into(),
        ));
    }

    Ok(())
}

pub async fn slots(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<StatsQuery>,
) -> Result<Json<SlotStatsResponse>, ApiError> {
    validate_slot_duration(q.slot_duration)?;
    validate_hours(q.hours)?;

    let symbols = parse_symbols(q.symbols);

    let params = StatsParams {
        slot_duration: q.slot_duration,
        symbols,
        hours: q.hours,
    };

    let (summary, heatmap, calibration, by_symbol, stoploss_sweep, by_hour, by_day) =
        tokio::try_join!(
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

#[derive(Deserialize)]
pub struct MlDatasetQuery {
    pub slot_duration: u32,
    #[serde(default)]
    pub symbols: Option<String>,
    #[serde(default = "default_ml_hours")]
    pub hours: f64,
    #[serde(default = "default_sample_every")]
    pub sample_every: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

pub async fn ml_dataset(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<MlDatasetQuery>,
) -> Result<Json<MlDatasetResponse>, ApiError> {
    validate_slot_duration(q.slot_duration)?;
    validate_hours(q.hours)?;

    if q.sample_every == 0 {
        return Err(ApiError::Validation("sample_every must be >= 1".into()));
    }
    if q.limit == 0 || q.limit > 50_000 {
        return Err(ApiError::Validation(
            "limit must be between 1 and 50000".into(),
        ));
    }

    let params = MlDatasetParams {
        slot_duration: q.slot_duration,
        symbols: parse_symbols(q.symbols),
        hours: q.hours,
        sample_every: q.sample_every,
        limit: q.limit,
        offset: q.offset,
    };

    let rows = queries::fetch_ml_dataset(&state.ch, &params)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(MlDatasetResponse {
        row_count: rows.len(),
        rows,
    }))
}

#[cfg(test)]
mod tests {
    use super::parse_symbols;

    #[test]
    fn parse_symbols_normalizes_to_lowercase_prefixes() {
        assert_eq!(
            parse_symbols(Some(" BTC,eth , Sol ".into())),
            vec!["btc", "eth", "sol"]
        );
    }
}
