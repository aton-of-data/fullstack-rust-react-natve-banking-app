use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Request ID response/request header name.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Trace ID response/request header name.
pub const TRACE_ID_HEADER: &str = "x-trace-id";

static REQUEST_ID: HeaderName = HeaderName::from_static(REQUEST_ID_HEADER);
static TRACE_ID: HeaderName = HeaderName::from_static(TRACE_ID_HEADER);

/// Per-request correlation identifiers propagated to handlers and responses.
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub trace_id: String,
}

/// Injects or generates request and trace IDs for every request.
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(&REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let trace_id = request
        .headers()
        .get(&TRACE_ID)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    request.extensions_mut().insert(RequestContext {
        request_id: request_id.clone(),
        trace_id: trace_id.clone(),
    });

    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        headers.insert(REQUEST_ID.clone(), value);
    }
    if let Ok(value) = HeaderValue::from_str(&trace_id) {
        headers.insert(TRACE_ID.clone(), value);
    }
    response
}
