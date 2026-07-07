use std::time::Instant;

use axum::{extract::Request, middleware::Next, response::Response};

use crate::metrics::record_http_request;

/// Records HTTP request duration and status after the inner handler completes.
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
