//! Verifies total funds are conserved across transfers.

use ficus_testkit::{
    execute_transfer, setup_isolated_test_db, total_balance_minor, TestAppBuilder,
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
    let err = execute_transfer(
        &app,
        users.charlie.id,
        "alice",
        1_000_000,
        "550e8400-e29b-41d4-a716-446655440014",
    )
    .await
    .expect_err("insufficient funds");
    assert!(matches!(
        err,
        ficus_domain::errors::DomainError::InsufficientFunds
    ));
    let after = total_balance_minor(&app.db).await.expect("after");
    assert_eq!(before, after);
}
