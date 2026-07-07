//! HTTP helpers for full-stack API integration tests.

use reqwest::{Client, StatusCode};
use serde_json::Value;

/// Builds a default HTTP client for integration tests.
pub fn http_client() -> Client {
    Client::builder()
        .build()
        .expect("failed to build reqwest client")
}

/// HTTP response wrapper with status, headers, and JSON body.
pub struct HttpJsonResponse {
    /// HTTP status code.
    pub status: StatusCode,
    /// Parsed JSON body (empty object when body is absent).
    pub json: Value,
    /// Raw response text for non-JSON bodies.
    pub text: String,
    /// Value of `x-request-id` when present.
    pub request_id: Option<String>,
    /// Value of `x-trace-id` when present.
    pub trace_id: Option<String>,
}

impl HttpJsonResponse {
    /// Returns the machine-readable error code when present.
    pub fn error_code(&self) -> Option<&str> {
        self.json["code"].as_str()
    }

    /// Asserts the response does not leak sensitive substrings.
    pub fn assert_no_sensitive_leaks(&self) {
        let serialized = self.text.to_lowercase();
        for forbidden in [
            "password123",
            "password_hash",
            "bearer ",
            "authorization:",
            "postgres://",
            "jwt_secret",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "response leaked sensitive data ({forbidden}): {}",
                self.text
            );
        }
    }
}

async fn parse_response(response: reqwest::Response) -> Result<HttpJsonResponse, String> {
    let status = response.status();
    let request_id = response
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let trace_id = response
        .headers()
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let text = response
        .text()
        .await
        .map_err(|err| format!("read body failed: {err}"))?;
    let json = serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({}));
    Ok(HttpJsonResponse {
        status,
        json,
        text,
        request_id,
        trace_id,
    })
}

/// Authenticates via `POST /v1/auth/login` and returns the bearer token.
pub async fn http_login(
    client: &Client,
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<String, String> {
    let response = http_login_response(client, base_url, username, password).await?;
    if !response.status.is_success() {
        return Err(format!("login failed with status {}", response.status));
    }
    response.json["access_token"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "login response missing access_token".to_string())
}

/// Authenticates via `POST /v1/auth/login` and returns the full HTTP response.
pub async fn http_login_response(
    client: &Client,
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<HttpJsonResponse, String> {
    let response = client
        .post(format!("{base_url}/v1/auth/login"))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .await
        .map_err(|err| format!("login request failed: {err}"))?;
    parse_response(response).await
}

/// Returns the authenticated user profile via `GET /v1/auth/me`.
pub async fn http_get_me(
    client: &Client,
    base_url: &str,
    access_token: Option<&str>,
) -> Result<HttpJsonResponse, String> {
    let mut request = client.get(format!("{base_url}/v1/auth/me"));
    if let Some(token) = access_token {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .await
        .map_err(|err| format!("me request failed: {err}"))?;
    parse_response(response).await
}

/// Logs out via `POST /v1/auth/logout`.
pub async fn http_logout(
    client: &Client,
    base_url: &str,
    access_token: Option<&str>,
) -> Result<HttpJsonResponse, String> {
    let mut request = client.post(format!("{base_url}/v1/auth/logout"));
    if let Some(token) = access_token {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .await
        .map_err(|err| format!("logout request failed: {err}"))?;
    parse_response(response).await
}

/// Parameters for HTTP transfer creation in integration tests.
pub struct HttpTransferParams<'a> {
    /// Optional bearer token.
    pub access_token: Option<&'a str>,
    /// Optional idempotency key header.
    pub idempotency_key: Option<&'a str>,
    /// Recipient username.
    pub recipient_username: &'a str,
    /// Amount in minor units.
    pub amount_minor: i64,
    /// ISO currency code.
    pub currency: &'a str,
    /// Optional transfer description.
    pub description: Option<&'a str>,
}

/// Creates a transfer via `POST /v1/transfers`.
pub async fn http_create_transfer(
    client: &Client,
    base_url: &str,
    params: HttpTransferParams<'_>,
) -> Result<HttpJsonResponse, String> {
    let mut body = serde_json::json!({
        "recipient_username": params.recipient_username,
        "amount_minor": params.amount_minor.to_string(),
        "currency": params.currency,
    });
    if let Some(desc) = params.description {
        body["description"] = serde_json::json!(desc);
    }

    let mut request = client.post(format!("{base_url}/v1/transfers")).json(&body);

    if let Some(token) = params.access_token {
        request = request.bearer_auth(token);
    }
    if let Some(key) = params.idempotency_key {
        request = request.header("Idempotency-Key", key);
    }

    let response = request
        .send()
        .await
        .map_err(|err| format!("transfer request failed: {err}"))?;
    parse_response(response).await
}

/// Fetches the authenticated user's balance via `GET /v1/accounts/me/balance`.
pub async fn http_get_balance(
    client: &Client,
    base_url: &str,
    access_token: Option<&str>,
) -> Result<HttpJsonResponse, String> {
    let mut request = client.get(format!("{base_url}/v1/accounts/me/balance"));
    if let Some(token) = access_token {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .await
        .map_err(|err| format!("balance request failed: {err}"))?;
    parse_response(response).await
}

/// Fetches ledger entries via `GET /v1/accounts/me/ledger`.
pub async fn http_get_ledger(
    client: &Client,
    base_url: &str,
    access_token: Option<&str>,
    cursor: Option<&str>,
) -> Result<HttpJsonResponse, String> {
    let mut request = client.get(format!("{base_url}/v1/accounts/me/ledger"));
    if let Some(token) = access_token {
        request = request.bearer_auth(token);
    }
    if let Some(cursor) = cursor {
        request = request.query(&[("cursor", cursor)]);
    }
    let response = request
        .send()
        .await
        .map_err(|err| format!("ledger request failed: {err}"))?;
    parse_response(response).await
}

/// Fetches the global feed via `GET /v1/feed`.
pub async fn http_get_feed(
    client: &Client,
    base_url: &str,
    access_token: Option<&str>,
    cursor: Option<&str>,
) -> Result<HttpJsonResponse, String> {
    let mut request = client.get(format!("{base_url}/v1/feed"));
    if let Some(token) = access_token {
        request = request.bearer_auth(token);
    }
    if let Some(cursor) = cursor {
        request = request.query(&[("cursor", cursor)]);
    }
    let response = request
        .send()
        .await
        .map_err(|err| format!("feed request failed: {err}"))?;
    parse_response(response).await
}

/// Fetches Prometheus metrics via `GET /metrics`.
pub async fn http_get_metrics(client: &Client, base_url: &str) -> Result<String, String> {
    http_get_metrics_with_auth(client, base_url, None)
        .await
        .map(|(_status, body)| body)
}

/// Fetches Prometheus metrics with an optional bearer token.
pub async fn http_get_metrics_with_auth(
    client: &Client,
    base_url: &str,
    bearer_token: Option<&str>,
) -> Result<(reqwest::StatusCode, String), String> {
    let mut request = client.get(format!("{base_url}/metrics"));
    if let Some(token) = bearer_token {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .await
        .map_err(|err| format!("metrics request failed: {err}"))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|err| format!("metrics body read failed: {err}"))?;
    Ok((status, text))
}

/// Posts a raw JSON transfer body for validation contract tests.
pub async fn http_create_transfer_raw(
    client: &Client,
    base_url: &str,
    access_token: &str,
    idempotency_key: Option<&str>,
    body: Value,
) -> Result<HttpJsonResponse, String> {
    let mut request = client
        .post(format!("{base_url}/v1/transfers"))
        .bearer_auth(access_token)
        .json(&body);
    if let Some(key) = idempotency_key {
        request = request.header("Idempotency-Key", key);
    }
    let response = request
        .send()
        .await
        .map_err(|err| format!("transfer request failed: {err}"))?;
    parse_response(response).await
}
