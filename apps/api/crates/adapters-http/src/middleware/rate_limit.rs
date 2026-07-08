//! In-memory sliding-window rate limiting for login and transfers.
//!
//! Limiters are process-local ([`DashMap`]) and suitable for single-instance
//! deployments or as a first line of defense. Multi-instance deployments need
//! an external store for coordinated limits.
//!
//! - [`login_rate_limit`] keys by client IP (honoring proxy headers when
//!   configured).
//! - [`transfer_rate_limit`] keys by authenticated user id when available.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{extract::Request, middleware::Next, response::Response};
use dashmap::DashMap;

use crate::error::ApiError;

/// Sliding-window in-memory rate limiter keyed by arbitrary strings.
#[derive(Clone)]
pub struct RateLimiter {
    limits: Arc<DashMap<String, Vec<Instant>>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    /// Creates a limiter allowing `max_requests` per `window`.
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            limits: Arc::new(DashMap::new()),
            max_requests,
            window,
        }
    }

    /// Returns an error when the key has exceeded its quota.
    ///
    /// On success, records the current timestamp for the key. Excess requests
    /// yield [`ApiError::RateLimited`] (HTTP 429).
    pub fn check(&self, key: &str) -> Result<(), ApiError> {
        let now = Instant::now();
        let cutoff = now - self.window;

        let mut entry = self.limits.entry(key.to_string()).or_default();
        entry.retain(|t| *t > cutoff);

        if entry.len() >= self.max_requests as usize {
            return Err(ApiError::RateLimited);
        }

        entry.push(now);
        Ok(())
    }
}

/// Resolves the client IP for rate-limit keys.
///
/// When `trust_proxy_headers` is true, prefers the first `X-Forwarded-For`
/// hop, then `X-Real-Ip`. Otherwise uses the socket peer from Axum
/// `ConnectInfo`, falling back to `"unknown"`.
fn client_ip(request: &Request, trust_proxy_headers: bool) -> String {
    if trust_proxy_headers {
        if let Some(ip) = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .or_else(|| {
                request
                    .headers()
                    .get("x-real-ip")
                    .and_then(|v| v.to_str().ok())
            })
        {
            return ip.to_string();
        }
    }

    request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|info| info.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Rate-limits login attempts by client IP.
///
/// State is `(RateLimiter, trust_proxy_headers)` from [`crate::create_router`].
/// Exceeding the limit returns 429 before the login handler runs.
pub async fn login_rate_limit(
    axum::extract::State((limiter, trust_proxy_headers)): axum::extract::State<(RateLimiter, bool)>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let key = format!("login:{}", client_ip(&request, trust_proxy_headers));
    limiter.check(&key)?;
    Ok(next.run(request).await)
}

/// Rate-limits transfer requests by authenticated user ID.
///
/// Prefers [`super::auth::AuthenticatedUser`] from extensions (set by
/// [`crate::require_auth`]). Falls back to peer IP if the user extension is
/// missing. Exceeding the limit returns 429.
pub async fn transfer_rate_limit(
    axum::extract::State(limiter): axum::extract::State<RateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let user_id = request
        .extensions()
        .get::<super::auth::AuthenticatedUser>()
        .map(|u| u.user_id.to_string())
        .unwrap_or_else(|| client_ip(&request, false));

    let key = format!("transfer:{user_id}");
    limiter.check(&key)?;
    Ok(next.run(request).await)
}
