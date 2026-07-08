//! Money conservation: total projected balances never change on success or fail.
//!
//! # Risk guarded
//! Creating/destroying money via unbalanced ledger updates or failed transfers
//! that still mutate projections.
//!
//! # Invariant proven
//! [`ficus_testkit::total_balance_minor`] is identical before/after single and
//! sequenced user↔user transfers; insufficient-funds attempts leave totals,
//! transfer counts, and orphan-ledger scans unchanged.
//!
//! # Amounts chosen
//! Values well within seeded balances (e.g. 2_500 alice→bob); failure case uses
//! 1_000_000 from charlie to force `InsufficientFunds` without touching money.
//!
//! # Failure meaning
//! `before != after` means debit/credit application or rollback is incorrect.

use ficus_domain::errors::DomainError;
use ficus_testkit::{
    count_all_transfers, count_completed_transfers, execute_transfer, orphan_ledger_entries,
    setup_isolated_test_db, total_balance_minor, TestAppBuilder,
};

#[tokio::test]
async fn single_transfer_conserves_total_funds() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users.clone())
        .build()
        .await
        .expect("app");

    let before = total_balance_minor(&app.db).await.expect("before");
    execute_transfer(
        &app,
        users.alice.id,
        "bob",
        2_500,
        "550e8400-e29b-41d4-a716-446655440010",
    )
    .await
    .expect("transfer");
    let after = total_balance_minor(&app.db).await.expect("after");

    assert_eq!(before, after);
}

#[tokio::test]
async fn sequence_of_transfers_conserves_total_funds() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users.clone())
        .build()
        .await
        .expect("app");

    let before = total_balance_minor(&app.db).await.expect("before");

    let transfers = [
        (
            users.alice.id,
            "bob",
            1_000_i64,
            "550e8400-e29b-41d4-a716-446655440011",
        ),
        (
            users.bob.id,
            "charlie",
            500_i64,
            "550e8400-e29b-41d4-a716-446655440012",
        ),
        (
            users.charlie.id,
            "alice",
            250_i64,
            "550e8400-e29b-41d4-a716-446655440013",
        ),
    ];

    for (sender, recipient, amount, key) in transfers {
        execute_transfer(&app, sender, recipient, amount, key)
            .await
            .expect("transfer");
        let mid = total_balance_minor(&app.db).await.expect("mid total");
        assert_eq!(before, mid, "conservation violated after {recipient}");
    }

    let after = total_balance_minor(&app.db).await.expect("after");
    assert_eq!(before, after);
}

#[tokio::test]
async fn failed_transfer_does_not_change_total_funds() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db)
        .with_users(users.clone())
        .build()
        .await
        .expect("app");

    let before = total_balance_minor(&app.db).await.expect("before");
    let before_transfers = count_all_transfers(&app.db)
        .await
        .expect("before transfers");
    let before_completed = count_completed_transfers(&app.db)
        .await
        .expect("before completed");
    let err = execute_transfer(
        &app,
        users.charlie.id,
        "alice",
        1_000_000,
        "550e8400-e29b-41d4-a716-446655440014",
    )
    .await
    .expect_err("insufficient funds");
    assert!(matches!(err, DomainError::InsufficientFunds));
    let after = total_balance_minor(&app.db).await.expect("after");
    assert_eq!(before, after);
    assert_eq!(
        count_all_transfers(&app.db).await.expect("after transfers"),
        before_transfers
    );
    assert_eq!(
        count_completed_transfers(&app.db)
            .await
            .expect("after completed"),
        before_completed
    );
    assert!(orphan_ledger_entries(&app.db)
        .await
        .expect("orphans")
        .is_empty());
}
