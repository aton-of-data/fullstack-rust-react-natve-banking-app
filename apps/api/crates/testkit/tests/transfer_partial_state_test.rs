//! Verifies failed transfers roll back completely with no partial persistence.

use ficus_domain::errors::DomainError;
use ficus_testkit::{
    count_all_transfers, count_audit_events, count_completed_transfers, execute_transfer,
    orphan_ledger_entries, setup_isolated_test_db, total_balance_minor, TestAppBuilder,
};

#[tokio::test]
async fn failed_transfer_leaves_no_partial_state() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db.clone())
        .with_users(users.clone())
        .build()
        .await
        .expect("app");

    let before_total = total_balance_minor(&db).await.expect("before total");
    let before_transfers = count_all_transfers(&db).await.expect("before transfers");
    let before_completed = count_completed_transfers(&db)
        .await
        .expect("before completed");
    let before_audit = count_audit_events(&db).await.expect("before audit");

    let err = execute_transfer(
        &app,
        users.charlie.id,
        "alice",
        1_000_000,
        "550e8400-e29b-41d4-a716-446655440020",
    )
    .await
    .expect_err("insufficient funds");
    assert_eq!(err, DomainError::InsufficientFunds);

    assert_eq!(
        total_balance_minor(&db).await.expect("after total"),
        before_total
    );
    assert_eq!(
        count_all_transfers(&db).await.expect("after transfers"),
        before_transfers
    );
    assert_eq!(
        count_completed_transfers(&db)
            .await
            .expect("after completed"),
        before_completed
    );
    assert!(
        orphan_ledger_entries(&db)
            .await
            .expect("orphans")
            .is_empty(),
        "failed transfer must not leave orphan ledger rows"
    );

    let declined_audits = count_audit_events(&db).await.expect("audit count");
    assert_eq!(
        declined_audits,
        before_audit + 1,
        "insufficient-funds decline must persist exactly one audit event outside the rolled-back transaction"
    );
}

#[tokio::test]
async fn failed_transfer_does_not_consume_idempotency_key() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users.clone())
        .build()
        .await
        .expect("app");

    let key = "550e8400-e29b-41d4-a716-446655440021";
    let err = execute_transfer(&app, users.charlie.id, "alice", 999_999, key)
        .await
        .expect_err("insufficient");
    assert_eq!(err, DomainError::InsufficientFunds);

    // Same key should be reusable because the failed attempt did not commit idempotency state.
    let err_again = execute_transfer(&app, users.charlie.id, "alice", 999_999, key)
        .await
        .expect_err("still insufficient");
    assert_eq!(err_again, DomainError::InsufficientFunds);
}
