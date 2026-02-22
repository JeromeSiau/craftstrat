use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

pub fn init() -> PrometheusHandle {
    PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus metrics recorder")
}
