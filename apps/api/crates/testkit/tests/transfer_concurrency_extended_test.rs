//! Extended concurrency tests — cross-account contention and inverse-direction transfers.

use std::sync::Arc;

use ficus_domain::errors::DomainError;
use ficus_testkit::{
    execute_transfer, negative_balances, orphan_ledger_entries, reconcile_all_accounts,
    setup_isolated_test_db, total_balance_minor, TestAppBuilder,
};

#[tokio::test]
async fn cross_account_contention_many_senders_one_recipient() {
    const PER_SENDER: i64 = 100;
    const TRANSFERS_PER_SENDER: usize = 5;

    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let system_before = total_balance_minor(&db).await.expect("system");
    let bob_before = users.bob.initial_balance_minor;

    let app = Arc::new(
        TestAppBuilder::new(&pg.database_url)
            .with_db(db.clone())
            .with_users(users.clone())
            .build()
            .await
            .expect("app"),
    );

    let mut handles = Vec::new();
    for (sender, prefix) in [(users.alice.id, "a"), (users.charlie.id, "c")] {
        for i in 0..TRANSFERS_PER_SENDER {
            let app = app.clone();
            let key = format!("550e8400-e29b-41d4-a716-{prefix}{i:011}");
            handles.push(tokio::spawn(async move {
                execute_transfer(&app, sender, "bob", PER_SENDER, &key).await
            }));
        }
    }

    let mut successes = 0usize;
    for handle in handles {
        match handle.await.expect("join") {
            Ok(_) => successes += 1,
            Err(e) => panic!("unexpected failure: {e:?}"),
        }
    }

    assert_eq!(successes, TRANSFERS_PER_SENDER * 2);
    assert!(negative_balances(&db).await.expect("neg").is_empty());
    assert!(orphan_ledger_entries(&db)
        .await
        .expect("orphans")
        .is_empty());
    assert!(reconcile_all_accounts(&db)
        .await
        .expect("reconcile")
        .is_empty());
    assert_eq!(
        total_balance_minor(&db).await.expect("system"),
        system_before
    );

    let bob_after = app
        .accounts
        .get_balance(users.bob.account_id)
        .await
        .expect("bob")
        .balance_minor;
    assert_eq!(bob_after, bob_before + (successes as i64 * PER_SENDER));
}

#[tokio::test]
async fn inverse_direction_transfers_do_not_deadlock() {
    const ROUNDS: usize = 25;
    const AMOUNT: i64 = 10;

    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let system_before = total_balance_minor(&db).await.expect("system");

    let app = Arc::new(
        TestAppBuilder::new(&pg.database_url)
            .with_db(db.clone())
            .with_users(users.clone())
            .build()
            .await
            .expect("app"),
    );

    let mut handles = Vec::with_capacity(ROUNDS * 2);
    for i in 0..ROUNDS {
        let app_ab = app.clone();
        let app_ba = app.clone();
        let key_ab = format!("550e8400-e29b-41d4-a716-ab{i:010}");
        let key_ba = format!("550e8400-e29b-41d4-a716-ba{i:010}");
        handles.push(tokio::spawn(async move {
            execute_transfer(&app_ab, users.alice.id, "bob", AMOUNT, &key_ab).await
        }));
        handles.push(tokio::spawn(async move {
            execute_transfer(&app_ba, users.bob.id, "alice", AMOUNT, &key_ba).await
        }));
    }

    let mut ok = 0usize;
    let mut err = 0usize;
    for handle in handles {
        match handle.await.expect("join") {
            Ok(_) => ok += 1,
            Err(DomainError::InsufficientFunds) => err += 1,
            Err(other) => panic!("unexpected error: {other:?}"),
        }
    }

    assert_eq!(ok + err, ROUNDS * 2);
    assert!(negative_balances(&db).await.expect("neg").is_empty());
    assert!(orphan_ledger_entries(&db)
        .await
        .expect("orphans")
        .is_empty());
    assert_eq!(
        total_balance_minor(&db).await.expect("system"),
        system_before
    );
    assert!(reconcile_all_accounts(&db)
        .await
        .expect("reconcile")
        .is_empty());
}

#[tokio::test]
async fn self_transfer_rejected_at_service_layer() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let system_before = total_balance_minor(&db).await.expect("system");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db.clone())
        .with_users(users.clone())
        .build()
        .await
        .expect("app");

    let err = execute_transfer(
        &app,
        users.alice.id,
        "alice",
        100,
        "550e8400-e29b-41d4-a716-446655440300",
    )
    .await
    .expect_err("self transfer");
    assert_eq!(err, DomainError::SelfTransfer);
    assert_eq!(
        total_balance_minor(&db).await.expect("system"),
        system_before
    );
    assert!(orphan_ledger_entries(&db)
        .await
        .expect("orphans")
        .is_empty());
}
