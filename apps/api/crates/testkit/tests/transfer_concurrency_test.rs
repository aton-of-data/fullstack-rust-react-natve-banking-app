//! 100 concurrent transfers from one funded account — ten mandatory assertions.

use std::sync::Arc;

use ficus_adapters_persistence::entities::{ledger_entries, transfers};
use ficus_domain::errors::DomainError;
use ficus_testkit::{
    count_completed_transfers, execute_transfer, negative_balances, orphan_ledger_entries,
    set_account_balance, setup_isolated_test_db, signed_ledger_amount, total_balance_minor,
    TestAppBuilder,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

const CONCURRENT_REQUESTS: usize = 100;
const TRANSFER_AMOUNT: i64 = 100;
const FUNDED_BALANCE: i64 = 5_000;

#[tokio::test]
async fn hundred_concurrent_transfers_from_same_account() {
    let (pg, db, users) = setup_isolated_test_db().await.expect("setup");
    set_account_balance(&db, users.alice.account_id, FUNDED_BALANCE)
        .await
        .expect("set balance");

    let initial_total = total_balance_minor(&db).await.expect("initial total");
    let initial_sender = FUNDED_BALANCE;
    let initial_recipient = users.bob.initial_balance_minor;

    let seed_completed = count_completed_transfers(&db)
        .await
        .expect("seed completed");
    let app = Arc::new(
        TestAppBuilder::new(&pg.database_url)
            .with_db(db.clone())
            .with_users(users.clone())
            .build()
            .await
            .expect("app"),
    );

    let mut handles = Vec::with_capacity(CONCURRENT_REQUESTS);
    for i in 0..CONCURRENT_REQUESTS {
        let app = app.clone();
        let sender = users.alice.id;
        let key = format!("550e8400-e29b-41d4-a716-{i:012}");
        handles.push(tokio::spawn(async move {
            execute_transfer(&app, sender, "bob", TRANSFER_AMOUNT, &key).await
        }));
    }

    let mut ok = Vec::new();
    let mut err = Vec::new();
    for handle in handles {
        match handle.await.expect("join") {
            Ok(record) => ok.push(record),
            Err(e) => err.push(e),
        }
    }

    // 1. No account balance goes negative.
    let negatives = negative_balances(&db).await.expect("negative scan");
    assert!(
        negatives.is_empty(),
        "negative balances found: {negatives:?}"
    );

    // 2. Total money in the system is conserved.
    let final_total = total_balance_minor(&db).await.expect("final total");
    assert_eq!(initial_total, final_total);

    // 3. Only fundable transfers succeed.
    let max_fundable = (initial_sender / TRANSFER_AMOUNT) as usize;
    assert_eq!(
        ok.len(),
        max_fundable,
        "expected exactly {max_fundable} successes, got {}",
        ok.len()
    );
    assert_eq!(
        err.len(),
        CONCURRENT_REQUESTS - max_fundable,
        "expected {} failures",
        CONCURRENT_REQUESTS - max_fundable
    );

    // 4. Failed transfers leave no partial state (no orphan ledger rows).
    let orphans = orphan_ledger_entries(&db).await.expect("orphans");
    assert!(orphans.is_empty(), "orphan ledger entries: {orphans:?}");

    // 5. Every request resolves to success or expected failure.
    assert_eq!(ok.len() + err.len(), CONCURRENT_REQUESTS);
    assert!(
        err.iter()
            .all(|e| matches!(e, DomainError::InsufficientFunds)),
        "unexpected errors: {err:?}"
    );

    // 6. Successful transfer count matches newly completed rows.
    let completed = count_completed_transfers(&db)
        .await
        .expect("completed count");
    assert_eq!(completed, seed_completed + ok.len() as u64);

    // 7. Sender balance decremented only by successful transfers.
    let sender_balance = app
        .accounts
        .get_balance(users.alice.account_id)
        .await
        .expect("sender balance");
    let expected_sender = initial_sender - (ok.len() as i64 * TRANSFER_AMOUNT);
    assert_eq!(sender_balance.balance_minor, expected_sender);

    // 8. Recipient balance increased only by successful transfers.
    let recipient_balance = app
        .accounts
        .get_balance(users.bob.account_id)
        .await
        .expect("recipient balance");
    let expected_recipient = initial_recipient + (ok.len() as i64 * TRANSFER_AMOUNT);
    assert_eq!(recipient_balance.balance_minor, expected_recipient);

    // 9. Each successful transfer has a balanced debit/credit ledger pair.
    for record in &ok {
        let entries = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::TransferId.eq(record.id))
            .all(&db)
            .await
            .expect("ledger rows");
        assert_eq!(entries.len(), 2, "transfer {}", record.id);
        let net: i64 = entries
            .iter()
            .map(|e| signed_ledger_amount(&e.direction, e.amount_minor))
            .sum();
        assert_eq!(net, 0, "transfer {} ledger unbalanced", record.id);
    }

    // 10. Completed transfer amounts are positive and match request size.
    for record in &ok {
        assert_eq!(record.amount_minor, TRANSFER_AMOUNT);
        let persisted = transfers::Entity::find_by_id(record.id)
            .one(&db)
            .await
            .expect("lookup")
            .expect("transfer row");
        assert_eq!(persisted.status, "completed");
        assert!(persisted.amount_minor > 0);
    }
}
