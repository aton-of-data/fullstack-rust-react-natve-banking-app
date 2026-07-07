//! Mandatory idempotency scenarios for transfer retries.

use ficus_domain::errors::DomainError;
use ficus_domain::idempotency::request_fingerprint;
use ficus_testkit::{
    execute_transfer, setup_isolated_test_db, total_balance_minor, TestAppBuilder,
};

#[tokio::test]
async fn replay_same_key_same_payload_returns_original_transfer() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users.clone())
        .build()
        .await
        .expect("app");

    let key = "550e8400-e29b-41d4-a716-446655440001";
    let before = total_balance_minor(&app.db).await.expect("total");

    let first = execute_transfer(&app, users.alice.id, "bob", 1_000, key)
        .await
        .expect("first transfer");
    let second = execute_transfer(&app, users.alice.id, "bob", 1_000, key)
        .await
        .expect("replay");

    assert_eq!(first.id, second.id);
    assert_eq!(first.amount_minor, second.amount_minor);
    assert_eq!(
        before,
        total_balance_minor(&app.db).await.expect("total after")
    );
}

#[tokio::test]
async fn same_key_different_amount_returns_conflict() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users.clone())
        .build()
        .await
        .expect("app");

    let key = "550e8400-e29b-41d4-a716-446655440002";
    execute_transfer(&app, users.alice.id, "bob", 500, key)
        .await
        .expect("first");

    let err = execute_transfer(&app, users.alice.id, "bob", 750, key)
        .await
        .expect_err("conflict expected");
    assert_eq!(err, DomainError::IdempotencyConflict);
}

#[tokio::test]
async fn same_key_different_recipient_returns_conflict() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users.clone())
        .build()
        .await
        .expect("app");

    let key = "550e8400-e29b-41d4-a716-446655440003";
    execute_transfer(&app, users.alice.id, "bob", 500, key)
        .await
        .expect("first");

    let err = execute_transfer(&app, users.alice.id, "charlie", 500, key)
        .await
        .expect_err("conflict expected");
    assert_eq!(err, DomainError::IdempotencyConflict);
}

#[tokio::test]
async fn invalid_idempotency_key_is_rejected() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users.clone())
        .build()
        .await
        .expect("app");

    let err = app
        .transfer_service
        .transfer(
            users.alice.id,
            "bob",
            "100",
            "USD",
            None,
            "short",
            "req-1",
            "trace-1",
        )
        .await
        .expect_err("invalid key");
    assert_eq!(err, DomainError::InvalidIdempotencyKey);
}

#[tokio::test]
async fn concurrent_duplicate_key_only_transfers_once() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = std::sync::Arc::new(
        TestAppBuilder::new(&pg.database_url)
            .with_db(db)
            .with_users(users.clone())
            .build()
            .await
            .expect("app"),
    );

    let key = "550e8400-e29b-41d4-a716-446655440004";
    let before = total_balance_minor(&app.db).await.expect("total");
    let mut handles = Vec::new();

    for i in 0..50 {
        let app = app.clone();
        let sender = users.alice.id;
        handles.push(tokio::spawn(async move {
            execute_transfer(&app, sender, "bob", 200, key).await
        }));
        let _ = i;
    }

    let mut transfer_ids = Vec::new();
    let mut successes = 0usize;
    let mut conflicts_or_ok = 0usize;

    for handle in handles {
        match handle.await.expect("join") {
            Ok(record) => {
                successes += 1;
                transfer_ids.push(record.id);
            }
            Err(DomainError::IdempotencyConflict) => conflicts_or_ok += 1,
            Err(other) => panic!("unexpected error: {other:?}"),
        }
    }

    assert!(successes >= 1, "at least one transfer should succeed");
    transfer_ids.sort();
    transfer_ids.dedup();
    assert_eq!(
        transfer_ids.len(),
        1,
        "all successful responses must share one transfer id"
    );
    assert_eq!(
        before,
        total_balance_minor(&app.db).await.expect("total after")
    );
    let _ = conflicts_or_ok;
}

#[tokio::test]
async fn fingerprint_is_stable_for_identical_payload() {
    let sender = uuid::Uuid::new_v4().to_string();
    let fp1 = request_fingerprint(&sender, "bob", "1000", "USD", "");
    let fp2 = request_fingerprint(&sender, "bob", "1000", "USD", "");
    assert_eq!(fp1, fp2);
}
