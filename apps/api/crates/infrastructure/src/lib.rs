//! Ficus infrastructure — composition root, config, auth, telemetry.

pub mod auth;
pub mod config;
pub mod readiness;
pub mod telemetry;
pub mod wiring;

pub use config::AppConfig;
pub use telemetry::{init_telemetry, shutdown_telemetry};
pub use wiring::build_app;
