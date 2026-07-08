//! `ficus-api` binary — HTTP API process entrypoint.
//!
//! Loads [`ficus_infrastructure::AppConfig`], initializes telemetry, wires the
//! Axum app via [`ficus_infrastructure::build_app`], and serves until shutdown.

use ficus_infrastructure::{build_app, init_telemetry, shutdown_telemetry, AppConfig};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env().map_err(|e| format!("config error: {e}"))?;
    init_telemetry(&config);

    let app = build_app(&config).await?;
    let addr = config.listen_addr();
    info!(%addr, environment = %config.environment, "starting ficus-api");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    shutdown_telemetry();
    Ok(())
}
