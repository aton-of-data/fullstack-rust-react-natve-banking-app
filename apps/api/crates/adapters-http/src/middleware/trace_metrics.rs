//! HTTP request metrics middleware.
//!
//! After the inner handler finishes, records route, method, status, and
//! duration into Prometheus via [`crate::metrics::record_http_request`].
//! Uses Axum [`MatchedPath`] when available so dynamic routes aggregate under
//! their template path instead of raw URLs.

use std::time::Instant;

use axum::{extract::Request, middleware::Next, response::Response};

use crate::metrics::record_http_request;

/// Records HTTP request duration and status after the inner handler completes.
///
/// Does not inspect request bodies or auth headers. Failures in metric
/// recording must never alter the response (this middleware always returns the
/// handler response).
pub async fn trace_metrics_middleware(request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let route = request
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let started = Instant::now();
    let response = next.run(request).await;
    let duration = started.elapsed().as_secs_f64();
    record_http_request(&route, &method, response.status(), duration);
    response
}
