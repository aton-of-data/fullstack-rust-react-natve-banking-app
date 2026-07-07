//! HTTP contract tests for Prometheus metrics endpoint authorization.

use ficus_testkit::{
    http_client, http_get_metrics, http_get_metrics_with_auth, setup_isolated_test_db,
    TestAppBuilder,
};
use reqwest::StatusCode;

#[tokio::test]
async fn http_metrics_open_in_test_environment() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users)
        .with_http_server()
        .with_environment("test")
        .build()
        .await
        .expect("app");

    let base_url = app.base_url().expect("http");
    let body = http_get_metrics(&http_client(), base_url)
        .await
        .expect("metrics");
    assert!(body.contains("ficus_http_requests_total"));
}

#[tokio::test]
async fn http_metrics_disabled_in_ci_without_token() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users)
        .with_http_server()
        .with_environment("ci")
        .build()
        .await
        .expect("app");

    let base_url = app.base_url().expect("http");
    let (status, body) = http_get_metrics_with_auth(&http_client(), base_url, None)
        .await
        .expect("metrics");
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("metrics endpoint disabled"));
}

#[tokio::test]
async fn http_metrics_requires_bearer_in_ci_when_token_configured() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users)
        .with_http_server()
        .with_environment("ci")
        .with_metrics_auth_token("ci-metrics-secret-token")
        .build()
        .await
        .expect("app");

    let base_url = app.base_url().expect("http");
    let client = http_client();

    let (unauthorized, _) = http_get_metrics_with_auth(&client, base_url, None)
        .await
        .expect("metrics");
    assert_eq!(unauthorized, StatusCode::UNAUTHORIZED);

    let (wrong, _) = http_get_metrics_with_auth(&client, base_url, Some("wrong-token"))
        .await
        .expect("metrics");
    assert_eq!(wrong, StatusCode::UNAUTHORIZED);

    let (authorized, body) =
        http_get_metrics_with_auth(&client, base_url, Some("ci-metrics-secret-token"))
            .await
            .expect("metrics");
    assert_eq!(authorized, StatusCode::OK);
    assert!(body.contains("ficus_http_requests_total"));
}
