//! Environment-backed application configuration.
//!
//! Loads host/port, database URLs, JWT settings, CORS, rate limits, and
//! optional OpenTelemetry / metrics auth flags. Required secrets (`DATABASE_URL`,
//! `JWT_SECRET`) must come from the environment — never hard-code them here.

use std::env;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Bind host for the HTTP listener (default `0.0.0.0`).
    pub host: String,
    /// Bind port for the HTTP listener (default `8080`).
    pub port: u16,
    /// Primary Postgres URL used by the API process.
    pub database_url: String,
    /// Postgres URL used by migrate/seed (falls back to [`Self::database_url`]).
    pub migration_database_url: String,
    /// HMAC secret for JWT access tokens (min 32 characters).
    pub jwt_secret: String,
    /// Access token lifetime in seconds (default `3600`).
    pub jwt_expiry_secs: u64,
    /// Deployment environment label (e.g. `development`, `production`).
    pub environment: String,
    /// Allowed CORS origin list.
    pub cors_origins: Vec<String>,
    /// Login attempts allowed per client per minute.
    pub login_rate_limit_per_min: u32,
    /// Transfer attempts allowed per client per minute.
    pub transfer_rate_limit_per_min: u32,
    /// Optional OTLP exporter endpoint (collector sidecar when set).
    pub otel_endpoint: Option<String>,
    /// OpenTelemetry service name (default `ficus-api`).
    pub otel_service_name: String,
    /// Optional bearer token required for `/metrics`.
    pub metrics_auth_token: Option<String>,
    /// When true, trust proxy headers for client IP / rate limiting.
    pub trust_proxy_headers: bool,
}

impl AppConfig {
    /// Loads configuration from environment, using `.env` when present.
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let host = env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".into());
        let port = env::var("API_PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse()
            .map_err(|_| "API_PORT must be a number")?;
        let database_url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL is required")?;
        let migration_database_url =
            env::var("DATABASE_MIGRATION_URL").unwrap_or_else(|_| database_url.clone());
        let jwt_secret = env::var("JWT_SECRET").map_err(|_| "JWT_SECRET is required")?;
        if jwt_secret.len() < 32 {
            return Err("JWT_SECRET must be at least 32 characters".into());
        }
        let jwt_expiry_secs = env::var("JWT_EXPIRY_SECS")
            .unwrap_or_else(|_| "3600".into())
            .parse()
            .map_err(|_| "JWT_EXPIRY_SECS must be a number")?;
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".into());
        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:8081,http://localhost:19006".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let login_rate_limit_per_min = env::var("LOGIN_RATE_LIMIT_PER_MIN")
            .unwrap_or_else(|_| "10".into())
            .parse()
            .map_err(|_| "LOGIN_RATE_LIMIT_PER_MIN must be a number")?;
        let transfer_rate_limit_per_min = env::var("TRANSFER_RATE_LIMIT_PER_MIN")
            .unwrap_or_else(|_| "30".into())
            .parse()
            .map_err(|_| "TRANSFER_RATE_LIMIT_PER_MIN must be a number")?;
        let otel_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
        let otel_service_name =
            env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "ficus-api".into());
        let metrics_auth_token = env::var("METRICS_AUTH_TOKEN")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let trust_proxy_headers = env::var("TRUST_PROXY_HEADERS")
            .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);

        Ok(Self {
            host,
            port,
            database_url,
            migration_database_url,
            jwt_secret,
            jwt_expiry_secs,
            environment,
            cors_origins,
            login_rate_limit_per_min,
            transfer_rate_limit_per_min,
            otel_endpoint,
            otel_service_name,
            metrics_auth_token,
            trust_proxy_headers,
        })
    }

    /// Returns `host:port` for binding the HTTP listener.
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
