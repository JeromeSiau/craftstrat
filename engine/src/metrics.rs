use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

// ---------------------------------------------------------------------------
// Metric name constants
// ---------------------------------------------------------------------------

pub const TICKS_TOTAL: &str = "craftstrat_ticks_total";
pub const UPTIME_SECONDS: &str = "craftstrat_uptime_seconds";
pub const STRATEGY_EVAL_DURATION: &str = "craftstrat_strategy_eval_duration_seconds";
pub const SIGNALS_TOTAL: &str = "craftstrat_signals_total";
pub const ORDERS_TOTAL: &str = "craftstrat_orders_total";
pub const ORDER_EXEC_DURATION: &str = "craftstrat_order_execution_duration_seconds";
pub const PNL_USDC: &str = "craftstrat_pnl_usdc";
pub const COPY_TRADES_TOTAL: &str = "craftstrat_copy_trades_total";
pub const ACTIVE_WALLETS: &str = "craftstrat_active_wallets";
pub const ACTIVE_ASSIGNMENTS: &str = "craftstrat_active_assignments";
pub const WS_RECONNECTIONS_TOTAL: &str = "craftstrat_ws_reconnections_total";
pub const WS_ERRORS_TOTAL: &str = "craftstrat_ws_errors_total";

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------

pub fn init() -> PrometheusHandle {
    let builder = PrometheusBuilder::new()
        .set_buckets_for_metric(
            metrics_exporter_prometheus::Matcher::Full(STRATEGY_EVAL_DURATION.to_string()),
            &[0.00001, 0.00005, 0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1],
        )
        .expect("failed to set eval buckets")
        .set_buckets_for_metric(
            metrics_exporter_prometheus::Matcher::Full(ORDER_EXEC_DURATION.to_string()),
            &[0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        )
        .expect("failed to set exec buckets");

    let handle = builder
        .install_recorder()
        .expect("failed to install Prometheus metrics recorder");

    describe_metrics();

    handle
}

fn describe_metrics() {
    metrics::describe_counter!(TICKS_TOTAL, "Total number of ticks processed from Kafka");
    metrics::describe_gauge!(UPTIME_SECONDS, "Seconds since the strategy engine started");
    metrics::describe_histogram!(STRATEGY_EVAL_DURATION, "Time to evaluate all strategies for one tick (seconds)");
    metrics::describe_counter!(SIGNALS_TOTAL, "Total trading signals emitted by the strategy engine");
    metrics::describe_counter!(ORDERS_TOTAL, "Total orders submitted to Polymarket");
    metrics::describe_histogram!(ORDER_EXEC_DURATION, "Time to submit an order to Polymarket (seconds)");
    metrics::describe_gauge!(PNL_USDC, "Cumulative realized PnL in USDC");
    metrics::describe_counter!(COPY_TRADES_TOTAL, "Total copy trading orders");
    metrics::describe_gauge!(ACTIVE_WALLETS, "Number of wallets with active strategy assignments");
    metrics::describe_gauge!(ACTIVE_ASSIGNMENTS, "Number of active wallet-strategy assignments");
    metrics::describe_counter!(WS_RECONNECTIONS_TOTAL, "Total WebSocket reconnection attempts");
    metrics::describe_counter!(WS_ERRORS_TOTAL, "Total WebSocket errors by type");
}
