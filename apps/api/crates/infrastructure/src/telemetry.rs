use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::AppConfig;

/// Initializes structured JSON logging and optional OpenTelemetry tracing.
pub fn init_telemetry(config: &AppConfig) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,ficus_api=debug"));

    if config.otel_endpoint.is_some() {
        tracing::info!(
            endpoint = config.otel_endpoint.as_deref().unwrap_or_default(),
            service = %config.otel_service_name,
            "OpenTelemetry endpoint configured; export via collector sidecar in full profile"
        );
    }

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

/// Shuts down telemetry exporters (no-op when OTLP is not wired in-process).
pub fn shutdown_telemetry() {}
