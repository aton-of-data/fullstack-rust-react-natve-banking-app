//! Test database seeding helpers.

use chrono::Utc;
use ficus_adapters_persistence::entities::{
    account_balances, accounts, ledger_entries, transfers, users,
};
use ficus_application::ports::PasswordHasher;
use ficus_infrastructure::auth::Argon2PasswordHasher;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionTrait};
use uuid::Uuid;

use crate::TestUser;

/// Well-known system account used for initial funding.
pub const SYSTEM_ACCOUNT_ID: &str = "00000000-0000-0000-0000-000000000001";

struct SeedSpec {
    username: &'static str,
    password: &'static str,
    balance_minor: i64,
}

const SEED_SPECS: &[SeedSpec] = &[
    SeedSpec {
        username: "alice",
        password: "password123",
        balance_minor: 100_000,
    },
    SeedSpec {
        username: "bob",
        password: "password123",
        balance_minor: 50_000,
    },
    SeedSpec {
        username: "charlie",
        password: "password123",
        balance_minor: 25_000,
    },
];

/// Seeded users available to integration tests.
#[derive(Debug, Clone)]
pub struct TestUsers {
    pub alice: TestUser,
    pub bob: TestUser,
    pub charlie: TestUser,
    pub system_account_id: Uuid,
}

/// Seeds alice, bob, and charlie with funded balances.
pub async fn seed_test_users(db: &DatabaseConnection) -> Result<TestUsers, sea_orm::DbErr> {
    let hasher = Argon2PasswordHasher;
    let system_id =
        Uuid::parse_str(SYSTEM_ACCOUNT_ID).map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
    let txn = db.begin().await?;

    ensure_system_account(&txn, system_id).await?;

    let mut seeded = Vec::new();
    for spec in SEED_SPECS {
        let user_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();
        let password_hash = hasher
            .hash(spec.password)
            .await
            .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

        users::ActiveModel {
            id: Set(user_id),
            username: Set(spec.username.to_string()),
            password_hash: Set(password_hash),
            created_at: Set(Utc::now().into()),
        }
        .insert(&txn)
        .await?;

        accounts::ActiveModel {
            id: Set(account_id),
            user_id: Set(Some(user_id)),
            is_system: Set(false),
            created_at: Set(Utc::now().into()),
        }
        .insert(&txn)
        .await?;

        account_balances::ActiveModel {
            account_id: Set(account_id),
            balance_minor: Set(0),
            currency_code: Set("USD".into()),
            updated_at: Set(Utc::now().into()),
        }
        .insert(&txn)
        .await?;

        if spec.balance_minor > 0 {
            fund_from_system(
                &txn,
                system_id,
                account_id,
                user_id,
                user_id,
                spec.balance_minor,
            )
            .await?;
        }

        seeded.push(TestUser {
            id: user_id,
            username: spec.username.to_string(),
            account_id,
            initial_balance_minor: spec.balance_minor,
        });
    }

    txn.commit().await?;

    Ok(TestUsers {
        alice: seeded[0].clone(),
        bob: seeded[1].clone(),
        charlie: seeded[2].clone(),
        system_account_id: system_id,
    })
}

/// Sets a user account balance directly (for deterministic concurrency tests).
pub async fn set_account_balance(
    db: &DatabaseConnection,
    account_id: Uuid,
    balance_minor: i64,
) -> Result<(), sea_orm::DbErr> {
    let row = account_balances::Entity::find_by_id(account_id)
        .one(db)
        .await?
        .expect("account balance row");
    let mut active: account_balances::ActiveModel = row.into();
    active.balance_minor = Set(balance_minor);
    active.updated_at = Set(Utc::now().into());
    active.update(db).await?;
    Ok(())
}

async fn ensure_system_account(
    txn: &sea_orm::DatabaseTransaction,
    system_id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    if accounts::Entity::find_by_id(system_id)
        .one(txn)
        .await?
        .is_some()
    {
        return Ok(());
    }

    accounts::ActiveModel {
        id: Set(system_id),
        user_id: Set(None),
        is_system: Set(true),
        created_at: Set(Utc::now().into()),
    }
    .insert(txn)
    .await?;

    account_balances::ActiveModel {
        account_id: Set(system_id),
        balance_minor: Set(10_000_000_000),
        currency_code: Set("USD".into()),
        updated_at: Set(Utc::now().into()),
    }
    .insert(txn)
    .await?;

    Ok(())
}

async fn fund_from_system(
    txn: &sea_orm::DatabaseTransaction,
    system_id: Uuid,
    recipient_account_id: Uuid,
    sender_user_id: Uuid,
    recipient_user_id: Uuid,
    amount_minor: i64,
) -> Result<(), sea_orm::DbErr> {
    let transfer_id = Uuid::new_v4();

    transfers::ActiveModel {
        id: Set(transfer_id),
        sender_account_id: Set(system_id),
        recipient_account_id: Set(recipient_account_id),
        sender_user_id: Set(sender_user_id),
        recipient_user_id: Set(recipient_user_id),
        amount_minor: Set(amount_minor),
        currency_code: Set("USD".into()),
        description: Set(Some("Initial system funding".into())),
        status: Set("completed".into()),
        created_at: Set(Utc::now().into()),
    }
    .insert(txn)
    .await?;

    ledger_entries::ActiveModel {
        id: Set(Uuid::new_v4()),
        account_id: Set(system_id),
        transfer_id: Set(transfer_id),
        amount_minor: Set(amount_minor),
        direction: Set("debit".into()),
        currency_code: Set("USD".into()),
        created_at: Set(Utc::now().into()),
    }
    .insert(txn)
    .await?;

    ledger_entries::ActiveModel {
        id: Set(Uuid::new_v4()),
        account_id: Set(recipient_account_id),
        transfer_id: Set(transfer_id),
        amount_minor: Set(amount_minor),
        direction: Set("credit".into()),
        currency_code: Set("USD".into()),
        created_at: Set(Utc::now().into()),
    }
    .insert(txn)
    .await?;

    let recipient_balance = account_balances::Entity::find_by_id(recipient_account_id)
        .one(txn)
        .await?
        .expect("recipient balance");
    let mut active: account_balances::ActiveModel = recipient_balance.into();
    active.balance_minor = Set(amount_minor);
    active.updated_at = Set(Utc::now().into());
    active.update(txn).await?;

    let system_balance = account_balances::Entity::find_by_id(system_id)
        .one(txn)
        .await?
        .expect("system balance");
    let mut system_active: account_balances::ActiveModel = system_balance.into();
    system_active.balance_minor = Set(system_active.balance_minor.as_ref() - amount_minor);
    system_active.updated_at = Set(Utc::now().into());
    system_active.update(txn).await?;

    Ok(())
}
