//! HTTP-level authentication contract tests.

use ficus_adapters_persistence::entities::audit_events;
use ficus_testkit::{
    count_audit_events, http_client, http_get_me, http_login_response, http_logout,
    setup_isolated_test_db, TestAppBuilder,
};
use reqwest::StatusCode;
use sea_orm::EntityTrait;

#[tokio::test]
async fn http_login_success_returns_token_and_safe_user_payload() {
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
    let response = http_login_response(&client, base_url, "alice", "password123")
        .await
        .expect("login");

    assert_eq!(response.status, StatusCode::OK);
    assert!(response.json["access_token"].as_str().is_some());
    assert!(response.json["user_id"].as_str().is_some());
    assert_eq!(response.json["username"].as_str(), Some("alice"));
    assert!(response.request_id.is_some());
    response.assert_no_sensitive_leaks();
}

#[tokio::test]
async fn http_login_invalid_credentials_returns_generic_error() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let before_audit = count_audit_events(&db).await.expect("audit count");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db.clone())
        .with_users(users)
        .with_http_server()
        .build()
        .await
        .expect("app");

    let base_url = app.base_url().expect("http");
    let client = http_client();
    let response = http_login_response(&client, base_url, "alice", "wrong-password")
        .await
        .expect("login");

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(response.json["code"].as_str(), Some("INVALID_CREDENTIALS"));
    assert!(
        !response.text.contains("password_hash"),
        "must not expose password hash"
    );
    response.assert_no_sensitive_leaks();

    let after_audit = count_audit_events(&db).await.expect("audit after");
    assert!(
        after_audit > before_audit,
        "login failure should append audit event"
    );
}

#[tokio::test]
async fn http_me_missing_token_returns_401() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users)
        .with_http_server()
        .build()
        .await
        .expect("app");

    let response = http_get_me(&http_client(), app.base_url().expect("http"), None)
        .await
        .expect("me");
    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(response.json["code"].as_str(), Some("UNAUTHORIZED"));
}

#[tokio::test]
async fn http_me_invalid_token_returns_401() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users)
        .with_http_server()
        .build()
        .await
        .expect("app");

    let response = http_get_me(
        &http_client(),
        app.base_url().expect("http"),
        Some("not-a-valid-jwt"),
    )
    .await
    .expect("me");
    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn http_me_returns_safe_profile() {
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
    let token = ficus_testkit::http_login(&client, base_url, "bob", "password123")
        .await
        .expect("login");

    let response = http_get_me(&client, base_url, Some(&token))
        .await
        .expect("me");
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.json["username"].as_str(), Some("bob"));
    assert!(response.json["user_id"].as_str().is_some());
    response.assert_no_sensitive_leaks();
}

#[tokio::test]
async fn http_logout_requires_authentication() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users)
        .with_http_server()
        .build()
        .await
        .expect("app");

    let response = http_logout(&http_client(), app.base_url().expect("http"), None)
        .await
        .expect("logout");
    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn http_login_rate_limit_returns_429() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users)
        .with_http_server()
        .with_login_rate_limit(2)
        .build()
        .await
        .expect("app");

    let base_url = app.base_url().expect("http");
    let client = http_client();

    let first = http_login_response(&client, base_url, "alice", "password123")
        .await
        .expect("first");
    assert_eq!(first.status, StatusCode::OK);

    let second = http_login_response(&client, base_url, "alice", "password123")
        .await
        .expect("second");
    assert_eq!(second.status, StatusCode::OK);

    let third = http_login_response(&client, base_url, "alice", "password123")
        .await
        .expect("third");
    assert_eq!(third.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(third.json["code"].as_str(), Some("RATE_LIMITED"));
}

#[tokio::test]
async fn http_login_success_audit_event_has_correlation_fields() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let before = count_audit_events(&db).await.expect("before");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db.clone())
        .with_users(users)
        .with_http_server()
        .build()
        .await
        .expect("app");

    let base_url = app.base_url().expect("http");
    let response = http_login_response(&http_client(), base_url, "alice", "password123")
        .await
        .expect("login");
    assert_eq!(response.status, StatusCode::OK);

    let events = audit_events::Entity::find().all(&db).await.expect("audit");
    assert!(events.len() as u64 > before);
    let latest = events.last().expect("latest audit");
    assert!(!latest.request_id.is_empty());
    assert!(!latest.trace_id.is_empty());
    assert!(!latest.event_type.is_empty());
}
