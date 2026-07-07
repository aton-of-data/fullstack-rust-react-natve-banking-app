//! HTTP-level feed and SSE contract tests.

use std::time::Duration;

use ficus_testkit::{
    http_client, http_create_transfer, http_get_feed, http_get_metrics, http_login,
    setup_isolated_test_db, TestAppBuilder,
};
use reqwest::StatusCode;

fn usd_transfer<'a>(
    access_token: Option<&'a str>,
    idempotency_key: Option<&'a str>,
    recipient_username: &'a str,
    amount_minor: i64,
    description: Option<&'a str>,
) -> ficus_testkit::HttpTransferParams<'a> {
    ficus_testkit::HttpTransferParams {
        access_token,
        idempotency_key,
        recipient_username,
        amount_minor,
        currency: "USD",
        description,
    }
}

#[tokio::test]
async fn http_feed_requires_authentication() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users)
        .with_http_server()
        .build()
        .await
        .expect("app");

    let response = http_get_feed(&http_client(), app.base_url().expect("http"), None, None)
        .await
        .expect("feed");
    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn http_completed_transfer_appears_in_feed() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users)
        .with_http_server()
        .build()
        .await
        .expect("app");

    let base_url = app.base_url().expect("http");
    let client = http_client();
    let token = http_login(&client, base_url, "alice", "password123")
        .await
        .expect("login");

    let transfer = http_create_transfer(
        &client,
        base_url,
        usd_transfer(
            Some(&token),
            Some("550e8400-e29b-41d4-a716-446655440200"),
            "bob",
            321,
            Some("feed test"),
        ),
    )
    .await
    .expect("transfer");
    assert_eq!(transfer.status, StatusCode::OK);
    let transfer_id = transfer.json["transfer_id"]
        .as_str()
        .expect("transfer_id")
        .to_string();

    tokio::time::sleep(Duration::from_millis(100)).await;

    let feed = http_get_feed(&client, base_url, Some(&token), None)
        .await
        .expect("feed");
    assert_eq!(feed.status, StatusCode::OK);
    let items = feed.json["items"].as_array().expect("items");
    assert!(
        items
            .iter()
            .any(|item| item["transfer_id"].as_str() == Some(transfer_id.as_str())),
        "completed transfer must appear in feed"
    );

    for item in items {
        assert!(item.get("sender_username").is_some());
        assert!(item.get("recipient_username").is_some());
        assert!(item["amount_minor"].is_string());
        assert!(item.get("password").is_none());
    }
    feed.assert_no_sensitive_leaks();
}

#[tokio::test]
async fn http_feed_stream_returns_sse_content_type() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users)
        .with_http_server()
        .build()
        .await
        .expect("app");

    let base_url = app.base_url().expect("http");
    let client = http_client();
    let token = http_login(&client, base_url, "alice", "password123")
        .await
        .expect("login");

    let response = client
        .get(format!("{base_url}/v1/feed/stream"))
        .bearer_auth(&token)
        .header("Accept", "text/event-stream")
        .timeout(Duration::from_secs(2))
        .send()
        .await
        .expect("sse");

    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    assert!(content_type.contains("text/event-stream"));
}

#[tokio::test]
async fn http_metrics_endpoint_exposes_transfer_counters() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users)
        .with_http_server()
        .build()
        .await
        .expect("app");

    let base_url = app.base_url().expect("http");
    let client = http_client();
    let token = http_login(&client, base_url, "alice", "password123")
        .await
        .expect("login");

    let _ = http_create_transfer(
        &client,
        base_url,
        usd_transfer(
            Some(&token),
            Some("550e8400-e29b-41d4-a716-446655440201"),
            "bob",
            50,
            None,
        ),
    )
    .await
    .expect("transfer");

    let metrics = http_get_metrics(&client, base_url).await.expect("metrics");
    for name in [
        "ficus_http_requests_total",
        "ficus_transfers_total",
        "ficus_login_attempts_total",
    ] {
        assert!(metrics.contains(name), "missing metric {name}");
    }
}
