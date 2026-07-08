//! Database-level immutability: ledger and audit rows are append-only.
//!
//! # Risk guarded
//! Direct SQL UPDATE/DELETE rewriting history (fraud / silent mutation).
//!
//! # Invariant proven
//! Postgres triggers reject UPDATE/DELETE on `ledger_entries` and
//! `audit_events` with an append-only error; completed transfer amounts stay
//! positive under constraint checks covered elsewhere in this file.
//!
//! # Amounts chosen
//! Uses seed ledger rows and a tiny 50-unit transfer only to create an audit
//! trail — size is irrelevant; presence of a row to mutate is what matters.
//!
//! # Failure meaning
//! Successful UPDATE/DELETE means immutability triggers are missing or broken.

use ficus_adapters_persistence::entities::{audit_events, ledger_entries};
use ficus_testkit::{execute_transfer, setup_isolated_test_db, TestAppBuilder};
use sea_orm::{ConnectionTrait, DatabaseBackend, EntityTrait, Statement};

#[tokio::test]
async fn ledger_entries_reject_update_and_delete() {
    let (_pg, db, _users) = setup_isolated_test_db().await.expect("setup");
    let entry = ledger_entries::Entity::find()
        .one(&db)
        .await
        .expect("query")
        .expect("seed ledger row");

    let update_err = db
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            format!(
                "UPDATE ledger_entries SET amount_minor = 1 WHERE id = '{}'",
                entry.id
            ),
        ))
        .await
        .expect_err("update must fail");
    assert!(
        update_err.to_string().contains("append-only"),
        "expected append-only trigger, got: {update_err}"
    );

    let delete_err = db
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            format!("DELETE FROM ledger_entries WHERE id = '{}'", entry.id),
        ))
        .await
        .expect_err("delete must fail");
    assert!(delete_err.to_string().contains("append-only"));
}

#[tokio::test]
async fn audit_events_reject_update_and_delete() {
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
        50,
        "550e8400-e29b-41d4-a716-446655440400",
    )
    .await
    .expect("transfer creates audit trail");

    let event = audit_events::Entity::find()
        .one(&db)
        .await
        .expect("query")
        .expect("audit row from transfer");

    let update_err = db
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            format!(
                "UPDATE audit_events SET event_type = 'tampered' WHERE id = '{}'",
                event.id
            ),
        ))
        .await
        .expect_err("update must fail");
    assert!(update_err.to_string().contains("append-only"));

    let delete_err = db
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            format!("DELETE FROM audit_events WHERE id = '{}'", event.id),
        ))
        .await
        .expect_err("delete must fail");
    assert!(delete_err.to_string().contains("append-only"));
}

#[tokio::test]
async fn account_balances_reject_negative_projection() {
    let (_pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let err = db
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            format!(
                "UPDATE account_balances SET balance_minor = -1 WHERE account_id = '{}'",
                users.alice.account_id
            ),
        ))
        .await
        .expect_err("negative balance must fail");
    assert!(
        err.to_string()
            .contains("chk_account_balances_non_negative"),
        "expected check constraint, got: {err}"
    );
}

#[tokio::test]
async fn transfers_reject_non_positive_amount() {
    let (_pg, db, users) = setup_isolated_test_db().await.expect("setup");
    let transfer = ficus_adapters_persistence::entities::transfers::Entity::find()
        .one(&db)
        .await
        .expect("query")
        .expect("seed transfer");

    let err = db
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            format!(
                "UPDATE transfers SET amount_minor = 0 WHERE id = '{}'",
                transfer.id
            ),
        ))
        .await
        .expect_err("zero amount must fail");
    assert!(err.to_string().contains("chk_transfers_amount_positive"));

    let _ = users;
}
