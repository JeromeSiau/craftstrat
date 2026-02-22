use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use crate::api::error::ApiError;
use crate::api::state::ApiState;
use crate::stats::queries::{self, StatsParams};
use crate::stats::types::SlotStatsResponse;

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

pub async fn slots(
    State(state): State<Arc<ApiState>>,
    Query(q): Query<StatsQuery>,
) -> Result<Json<SlotStatsResponse>, ApiError> {
    let symbols: Vec<String> = q
        .symbols
        .map(|s| {
            s.split(',')
                .map(|v| v.trim().to_uppercase())
                .filter(|v| !v.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let params = StatsParams {
        slot_duration: q.slot_duration,
        symbols,
        hours: q.hours.clamp(1.0, 2160.0),
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
