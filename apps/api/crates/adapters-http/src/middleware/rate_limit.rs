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

fn client_ip(request: &Request) -> String {
    request
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
        .unwrap_or("unknown")
        .to_string()
}

/// Rate-limits login attempts by client IP.
pub async fn login_rate_limit(
    axum::extract::State(limiter): axum::extract::State<RateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let key = format!("login:{}", client_ip(&request));
    limiter.check(&key)?;
    Ok(next.run(request).await)
}

/// Rate-limits transfer requests by authenticated user ID.
pub async fn transfer_rate_limit(
    axum::extract::State(limiter): axum::extract::State<RateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let user_id = request
        .extensions()
        .get::<super::auth::AuthenticatedUser>()
        .map(|u| u.user_id.to_string())
        .unwrap_or_else(|| client_ip(&request));

    let key = format!("transfer:{user_id}");
    limiter.check(&key)?;
    Ok(next.run(request).await)
}
