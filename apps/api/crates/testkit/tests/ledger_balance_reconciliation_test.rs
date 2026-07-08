//! Ledger ↔ balance projection reconciliation for non-system accounts.
//!
//! # Risk guarded
//! Drift between append-only ledger history and materialized `account_balances`
//! after seed and after multi-hop transfers.
//!
//! # Invariant proven
//! For every non-system account, projection equals
//! [`ficus_testkit::ledger_derived_balance`]; per-transfer sender/recipient
//! ledger sums move by exactly the transfer amount.
//!
//! # Amounts chosen
//! 3_000 / 1_500 / 500 and 750 stay inside seeded wallets so transfers complete
//! and leave a non-trivial multi-entry ledger to reconcile.
//!
//! # Failure meaning
//! Projection ≠ ledger sum means the executor updated one side without the
//! other (or seed funding wrote inconsistent books for user accounts).

use ficus_adapters_persistence::entities::{account_balances, accounts};
use ficus_testkit::{
    execute_transfer, ledger_derived_balance, setup_isolated_test_db, TestAppBuilder,
};
use sea_orm::EntityTrait;

#[tokio::test]
async fn all_accounts_reconcile_after_seed() {
    let (_pg, db, _users) = setup_isolated_test_db().await.expect("setup");

    let balances = account_balances::Entity::find()
        .all(&db)
        .await
        .expect("balances");

    for balance in balances {
        let account = accounts::Entity::find_by_id(balance.account_id)
            .one(&db)
            .await
            .expect("account")
            .expect("account row");
        if account.is_system {
            continue;
        }
        let derived = ledger_derived_balance(&db, balance.account_id)
            .await
            .expect("ledger sum");
        assert_eq!(
            balance.balance_minor, derived,
            "account {} out of sync: projection={}, ledger={}",
            balance.account_id, balance.balance_minor, derived
        );
    }
}

#[tokio::test]
async fn all_accounts_reconcile_after_transfers() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db.clone())
        .with_users(users.clone())
        .build()
        .await
        .expect("app");

    execute_transfer(
        &app,
        users.alice.id,
        "bob",
        3_000,
        "550e8400-e29b-41d4-a716-446655440020",
    )
    .await
    .expect("alice->bob");
    execute_transfer(
        &app,
        users.bob.id,
        "charlie",
        1_500,
        "550e8400-e29b-41d4-a716-446655440021",
    )
    .await
    .expect("bob->charlie");
    execute_transfer(
        &app,
        users.charlie.id,
        "alice",
        500,
        "550e8400-e29b-41d4-a716-446655440022",
    )
    .await
    .expect("charlie->alice");

    let balances = account_balances::Entity::find()
        .all(&db)
        .await
        .expect("balances");

    for balance in balances {
        let account = accounts::Entity::find_by_id(balance.account_id)
            .one(&db)
            .await
            .expect("account")
            .expect("account row");
        if account.is_system {
            continue;
        }
        let derived = ledger_derived_balance(&db, balance.account_id)
            .await
            .expect("ledger sum");
        assert_eq!(
            balance.balance_minor, derived,
            "account {} out of sync after transfers",
            balance.account_id
        );
    }
}

#[tokio::test]
async fn sender_and_recipient_reconcile_per_transfer() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let app = TestAppBuilder::new(&pg.database_url)
        .with_db(db.clone())
        .with_users(users.clone())
        .build()
        .await
        .expect("app");

    let alice_before = ledger_derived_balance(&db, users.alice.account_id)
        .await
        .expect("alice ledger");
    let bob_before = ledger_derived_balance(&db, users.bob.account_id)
        .await
        .expect("bob ledger");

    execute_transfer(
        &app,
        users.alice.id,
        "bob",
        750,
        "550e8400-e29b-41d4-a716-446655440023",
    )
    .await
    .expect("transfer");

    let alice_after = ledger_derived_balance(&db, users.alice.account_id)
        .await
        .expect("alice ledger after");
    let bob_after = ledger_derived_balance(&db, users.bob.account_id)
        .await
        .expect("bob ledger after");

    assert_eq!(alice_after, alice_before - 750);
    assert_eq!(bob_after, bob_before + 750);
}
