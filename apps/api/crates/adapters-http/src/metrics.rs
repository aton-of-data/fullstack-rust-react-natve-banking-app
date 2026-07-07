use std::sync::OnceLock;

use axum::{http::StatusCode, response::IntoResponse};
use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

static PROMETHEUS: OnceLock<PrometheusHandle> = OnceLock::new();

/// Registers Prometheus metric descriptions and installs the exporter.
pub fn init_metrics() -> PrometheusHandle {
    describe_counter!(
        "ficus_http_requests_total",
        "Total HTTP requests handled by route and status"
    );
    describe_histogram!(
        "ficus_http_request_duration_seconds",
        "HTTP request latency in seconds"
    );
    describe_counter!(
        "ficus_login_attempts_total",
        "Login attempts by result (success or failure)"
    );
    describe_counter!(
        "ficus_transfers_total",
        "Transfer requests by outcome (completed, declined, error)"
    );
    describe_counter!(
        "ficus_transfer_idempotency_replay_total",
        "Idempotent transfer replays returning stored response"
    );
    describe_counter!(
        "ficus_transfer_idempotency_conflict_total",
        "Idempotency key reuse with different payload"
    );
    describe_counter!(
        "ficus_transfer_serialization_retry_total",
        "Serialization/deadlock retries during transfer execution"
    );
    describe_counter!(
        "ficus_feed_events_published_total",
        "Feed events published to subscribers"
    );
    describe_gauge!(
        "ficus_sse_connections_active",
        "Active Server-Sent Events feed stream connections"
    );

    PROMETHEUS
        .get_or_init(|| {
            PrometheusBuilder::new()
                .install_recorder()
                .expect("failed to install Prometheus metrics recorder")
        })
        .clone()
}

/// Returns the Prometheus scrape handle.
pub fn prometheus_handle() -> PrometheusHandle {
    PROMETHEUS.get().cloned().unwrap_or_else(init_metrics)
}

/// Records request duration and count for a completed HTTP call.
pub fn record_http_request(route: &str, method: &str, status: StatusCode, duration_secs: f64) {
    let status = status.as_u16().to_string();
    counter!(
        "ficus_http_requests_total",
        "route" => route.to_string(),
        "method" => method.to_string(),
        "status" => status
    )
    .increment(1);
    histogram!("ficus_http_request_duration_seconds", "route" => route.to_string())
        .record(duration_secs);
}

/// Records a login attempt outcome.
pub fn record_login_attempt(success: bool) {
    let result = if success { "success" } else { "failure" };
    counter!("ficus_login_attempts_total", "result" => result).increment(1);
}

/// Records a transfer outcome.
pub fn record_transfer(outcome: &str) {
    let outcome = outcome.to_string();
    counter!("ficus_transfers_total", "outcome" => outcome).increment(1);
}

/// Increments the active SSE connection gauge.
pub fn sse_connection_opened() {
    gauge!("ficus_sse_connections_active").increment(1.0);
}

/// Decrements the active SSE connection gauge.
pub fn sse_connection_closed() {
    gauge!("ficus_sse_connections_active").decrement(1.0);
}

/// Prometheus metrics scrape endpoint handler.
pub async fn metrics_handler() -> impl IntoResponse {
    (StatusCode::OK, prometheus_handle().render())
}
