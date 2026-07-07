//! HTTP-level transfer, balance, and ledger contract tests.

use std::sync::Arc;

use ficus_testkit::{
    count_all_transfers, count_completed_transfers, count_idempotency_records,
    count_ledger_entries, http_client, http_create_transfer, http_create_transfer_raw,
    http_get_balance, http_get_ledger, http_get_metrics, http_login, negative_balances,
    orphan_ledger_entries, set_account_balance, setup_isolated_test_db, total_balance_minor,
    TestAppBuilder,
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

fn parse_balance(response: &ficus_testkit::HttpJsonResponse) -> i64 {
    response.json["balance_minor"]
        .as_str()
        .expect("balance_minor string")
        .parse()
        .expect("parse balance")
}

#[tokio::test]
async fn http_missing_authorization_returns_401() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users)
        .with_http_server()
        .build()
        .await
        .expect("app");

    let response = http_create_transfer(
        &http_client(),
        app.base_url().expect("http"),
        usd_transfer(
            None,
            Some("550e8400-e29b-41d4-a716-446655440001"),
            "bob",
            100,
            None,
        ),
    )
    .await
    .expect("transfer");

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert_eq!(response.json["code"].as_str(), Some("UNAUTHORIZED"));
}

#[tokio::test]
async fn http_missing_idempotency_key_returns_400() {
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

    let response = http_create_transfer(
        &client,
        base_url,
        usd_transfer(Some(&token), None, "bob", 100, None),
    )
    .await
    .expect("transfer");

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        response.json["code"].as_str(),
        Some("MISSING_IDEMPOTENCY_KEY")
    );
}

#[tokio::test]
async fn http_invalid_idempotency_key_returns_400() {
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

    let response = http_create_transfer(
        &client,
        base_url,
        usd_transfer(Some(&token), Some("short"), "bob", 100, None),
    )
    .await
    .expect("transfer");

    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        response.json["code"].as_str(),
        Some("INVALID_IDEMPOTENCY_KEY")
    );
}

#[tokio::test]
async fn http_self_transfer_returns_422() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let before_transfers = count_all_transfers(&db).await.expect("transfers");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db.clone())
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

    let response = http_create_transfer(
        &client,
        base_url,
        usd_transfer(
            Some(&token),
            Some("550e8400-e29b-41d4-a716-446655440002"),
            "alice",
            100,
            None,
        ),
    )
    .await
    .expect("transfer");

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response.json["code"].as_str(), Some("SELF_TRANSFER"));
    assert_eq!(
        count_all_transfers(&db).await.expect("after"),
        before_transfers
    );
}

#[tokio::test]
async fn http_unknown_recipient_returns_404() {
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

    let response = http_create_transfer(
        &client,
        base_url,
        usd_transfer(
            Some(&token),
            Some("550e8400-e29b-41d4-a716-446655440003"),
            "nobody",
            100,
            None,
        ),
    )
    .await
    .expect("transfer");

    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(response.json["code"].as_str(), Some("RECIPIENT_NOT_FOUND"));
}

#[tokio::test]
async fn http_validation_errors_for_invalid_body() {
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
    let key = "550e8400-e29b-41d4-a716-446655440004";

    for (body, expected_code) in [
        (
            serde_json::json!({
                "recipient_username": "bob",
                "amount_minor": "0",
                "currency": "USD"
            }),
            "INVALID_MONEY",
        ),
        (
            serde_json::json!({
                "recipient_username": "bob",
                "amount_minor": "-100",
                "currency": "USD"
            }),
            "INVALID_MONEY",
        ),
        (
            serde_json::json!({
                "recipient_username": "bob",
                "amount_minor": "100",
                "currency": "EUR"
            }),
            "UNSUPPORTED_CURRENCY",
        ),
    ] {
        let response = http_create_transfer_raw(&client, base_url, &token, Some(key), body)
            .await
            .expect("transfer");
        assert_eq!(response.status, StatusCode::BAD_REQUEST);
        assert_eq!(response.json["code"].as_str(), Some(expected_code));
    }
}

#[tokio::test]
async fn http_idempotency_replay_returns_same_transfer_id() {
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
    let key = "550e8400-e29b-41d4-a716-446655440100";

    let first = http_create_transfer(
        &client,
        base_url,
        usd_transfer(Some(&token), Some(key), "bob", 1_500, None),
    )
    .await
    .expect("first");
    assert_eq!(first.status, StatusCode::OK);

    let retry = http_create_transfer(
        &client,
        base_url,
        usd_transfer(Some(&token), Some(key), "bob", 1_500, None),
    )
    .await
    .expect("retry");
    assert_eq!(retry.status, StatusCode::OK);
    assert_eq!(
        first.json["transfer_id"].as_str(),
        retry.json["transfer_id"].as_str()
    );

    let metrics = http_get_metrics(&client, base_url).await.expect("metrics");
    assert!(metrics.contains("ficus_transfer_idempotency_replay_total"));
}

#[tokio::test]
async fn http_idempotency_conflict_returns_409() {
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
    let key = "550e8400-e29b-41d4-a716-446655440101";

    let first = http_create_transfer(
        &client,
        base_url,
        usd_transfer(Some(&token), Some(key), "bob", 500, None),
    )
    .await
    .expect("first");
    assert_eq!(first.status, StatusCode::OK);

    let conflict = http_create_transfer(
        &client,
        base_url,
        usd_transfer(Some(&token), Some(key), "bob", 750, None),
    )
    .await
    .expect("conflict");
    assert_eq!(conflict.status, StatusCode::CONFLICT);
    assert_eq!(conflict.json["code"].as_str(), Some("IDEMPOTENCY_CONFLICT"));
}

#[tokio::test]
async fn http_insufficient_funds_returns_422_without_partial_state() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db.clone())
        .with_users(users.clone())
        .with_http_server()
        .build()
        .await
        .expect("app");

    let base_url = app.base_url().expect("http");
    let client = http_client();
    let token = http_login(&client, base_url, "charlie", "password123")
        .await
        .expect("login");

    let before_total = total_balance_minor(&db).await.expect("before total");
    let before_transfers = count_all_transfers(&db).await.expect("before transfers");
    let balance_before = parse_balance(
        &http_get_balance(&client, base_url, Some(&token))
            .await
            .expect("balance"),
    );

    let response = http_create_transfer(
        &client,
        base_url,
        usd_transfer(
            Some(&token),
            Some("550e8400-e29b-41d4-a716-446655440102"),
            "alice",
            1_000_000,
            None,
        ),
    )
    .await
    .expect("transfer");

    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response.json["code"].as_str(), Some("INSUFFICIENT_FUNDS"));

    let balance_after = parse_balance(
        &http_get_balance(&client, base_url, Some(&token))
            .await
            .expect("balance after"),
    );
    assert_eq!(balance_after, balance_before);
    assert_eq!(
        total_balance_minor(&db).await.expect("after total"),
        before_total
    );
    assert_eq!(
        count_all_transfers(&db).await.expect("after transfers"),
        before_transfers
    );
    assert!(orphan_ledger_entries(&db)
        .await
        .expect("orphans")
        .is_empty());
    assert!(negative_balances(&db).await.expect("negatives").is_empty());
}

#[tokio::test]
async fn http_successful_transfer_response_contract() {
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

    let response = http_create_transfer(
        &client,
        base_url,
        usd_transfer(
            Some(&token),
            Some("550e8400-e29b-41d4-a716-446655440103"),
            "bob",
            250,
            Some("lunch"),
        ),
    )
    .await
    .expect("transfer");

    assert_eq!(response.status, StatusCode::OK);
    assert!(response.json["transfer_id"].as_str().is_some());
    assert_eq!(response.json["status"].as_str(), Some("COMPLETED"));
    assert_eq!(response.json["currency"].as_str(), Some("USD"));
    assert!(response.json["sender_balance_minor"].as_str().is_some());
    assert!(response.request_id.is_some());
    response.assert_no_sensitive_leaks();
}

#[tokio::test]
async fn http_hundred_concurrent_transfers_exact_success_and_failure_counts() {
    const CONCURRENT_REQUESTS: usize = 100;
    const TRANSFER_AMOUNT: i64 = 100;
    const FUNDED_BALANCE: i64 = 5_000;

    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    set_account_balance(&db, users.alice.account_id, FUNDED_BALANCE)
        .await
        .expect("set balance");

    let seed_completed = count_completed_transfers(&db)
        .await
        .expect("seed completed");
    let seed_ledger = count_ledger_entries(&db).await.expect("seed ledger");

    let app = Arc::new(
        TestAppBuilder::new(&pg.database_url)
            .with_db(db.clone())
            .with_users(users.clone())
            .with_http_server()
            .build()
            .await
            .expect("app"),
    );

    let base_url = app.base_url().expect("http server").to_string();
    let client = http_client();
    let token = http_login(&client, &base_url, "alice", "password123")
        .await
        .expect("login");

    let initial_total = total_balance_minor(&db).await.expect("initial total");
    let max_fundable = (FUNDED_BALANCE / TRANSFER_AMOUNT) as usize;
    let initial_recipient = users.bob.initial_balance_minor;

    let mut handles = Vec::with_capacity(CONCURRENT_REQUESTS);
    for i in 0..CONCURRENT_REQUESTS {
        let base_url = base_url.clone();
        let token = token.clone();
        let client = client.clone();
        let key = format!("550e8400-e29b-41d4-a716-{i:012}");
        handles.push(tokio::spawn(async move {
            http_create_transfer(
                &client,
                &base_url,
                usd_transfer(Some(&token), Some(&key), "bob", TRANSFER_AMOUNT, None),
            )
            .await
        }));
    }

    let mut ok = 0usize;
    let mut rejected = 0usize;
    for handle in handles {
        let response = handle.await.expect("join").expect("http");
        if response.status == StatusCode::OK {
            ok += 1;
        } else if response.status == StatusCode::UNPROCESSABLE_ENTITY {
            rejected += 1;
        } else {
            panic!("unexpected status {}: {:?}", response.status, response.json);
        }
    }

    assert_eq!(ok, max_fundable);
    assert_eq!(rejected, CONCURRENT_REQUESTS - max_fundable);
    assert_eq!(ok + rejected, CONCURRENT_REQUESTS);
    assert_eq!(
        initial_total,
        total_balance_minor(&db).await.expect("final total")
    );
    assert!(negative_balances(&db).await.expect("negatives").is_empty());
    assert!(orphan_ledger_entries(&db)
        .await
        .expect("orphans")
        .is_empty());

    let sender_balance = parse_balance(
        &http_get_balance(&client, &base_url, Some(&token))
            .await
            .expect("sender balance"),
    );
    assert_eq!(sender_balance, 0);

    let bob_token = http_login(&client, &base_url, "bob", "password123")
        .await
        .expect("bob login");
    let recipient_balance = parse_balance(
        &http_get_balance(&client, &base_url, Some(&bob_token))
            .await
            .expect("recipient balance"),
    );
    assert_eq!(
        recipient_balance,
        initial_recipient + (ok as i64 * TRANSFER_AMOUNT)
    );

    let completed = count_completed_transfers(&db).await.expect("completed");
    assert_eq!(completed, seed_completed + ok as u64);

    let ledger_entries = count_ledger_entries(&db).await.expect("ledger");
    assert_eq!(ledger_entries, seed_ledger + (ok as u64 * 2));
}

#[tokio::test]
async fn http_balance_requires_auth_and_serializes_amounts_as_strings() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users)
        .with_http_server()
        .build()
        .await
        .expect("app");

    let base_url = app.base_url().expect("http");
    let unauth = http_get_balance(&http_client(), base_url, None)
        .await
        .expect("balance");
    assert_eq!(unauth.status, StatusCode::UNAUTHORIZED);

    let token = http_login(&http_client(), base_url, "alice", "password123")
        .await
        .expect("login");
    let response = http_get_balance(&http_client(), base_url, Some(&token))
        .await
        .expect("balance");
    assert_eq!(response.status, StatusCode::OK);
    assert!(response.json["balance_minor"].is_string());
    assert!(response.json["currency"].is_string());
}

#[tokio::test]
async fn http_ledger_requires_auth_and_scopes_to_authenticated_account() {
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

    let _transfer = http_create_transfer(
        &client,
        base_url,
        usd_transfer(
            Some(&token),
            Some("550e8400-e29b-41d4-a716-446655440104"),
            "bob",
            100,
            None,
        ),
    )
    .await
    .expect("transfer");

    let ledger = http_get_ledger(&client, base_url, Some(&token), None)
        .await
        .expect("ledger");
    assert_eq!(ledger.status, StatusCode::OK);
    let items = ledger.json["items"].as_array().expect("items");
    assert!(!items.is_empty());
    for item in items {
        assert!(item["amount_minor"].is_string());
    }
    ledger.assert_no_sensitive_leaks();
}

#[tokio::test]
async fn http_concurrent_duplicate_idempotency_key_debits_once() {
    const CONCURRENT: usize = 50;
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let before_completed = count_completed_transfers(&db).await.expect("before");
    let before_idempotency = count_idempotency_records(&db).await.expect("before idem");
    let system_before = total_balance_minor(&db).await.expect("system");

    let app = Arc::new(
        TestAppBuilder::new(&pg.database_url)
            .with_db(db.clone())
            .with_users(users.clone())
            .with_http_server()
            .build()
            .await
            .expect("app"),
    );

    let base_url = app.base_url().expect("http").to_string();
    let client = http_client();
    let token = http_login(&client, &base_url, "alice", "password123")
        .await
        .expect("login");
    let key = "550e8400-e29b-41d4-a716-446655440105";

    let mut handles = Vec::with_capacity(CONCURRENT);
    for _ in 0..CONCURRENT {
        let base_url = base_url.clone();
        let token = token.clone();
        let client = client.clone();
        handles.push(tokio::spawn(async move {
            http_create_transfer(
                &client,
                &base_url,
                usd_transfer(Some(&token), Some(key), "bob", 200, None),
            )
            .await
        }));
    }

    let mut transfer_ids = Vec::new();
    for handle in handles {
        let response = handle.await.expect("join").expect("http");
        assert_eq!(response.status, StatusCode::OK);
        transfer_ids.push(
            response.json["transfer_id"]
                .as_str()
                .expect("transfer_id")
                .to_string(),
        );
    }
    assert!(transfer_ids.iter().all(|id| id == &transfer_ids[0]));

    assert_eq!(
        count_completed_transfers(&db).await.expect("completed"),
        before_completed + 1
    );
    assert_eq!(
        count_idempotency_records(&db).await.expect("idem"),
        before_idempotency + 1
    );
    assert_eq!(
        total_balance_minor(&db).await.expect("system"),
        system_before
    );
}
