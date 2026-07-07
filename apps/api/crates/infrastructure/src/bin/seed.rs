//! Seeds development users with auditable system-funded balances.
//!
//! Credentials (local dev only):
//! - alice / password123 — $1,000.00
//! - bob / password123 — $500.00
//! - charlie / password123 — $250.00

use chrono::Utc;
use ficus_adapters_persistence::entities::{
    account_balances, accounts, ledger_entries, transfers, users,
};
use ficus_application::ports::PasswordHasher;
use ficus_infrastructure::auth::Argon2PasswordHasher;
use ficus_infrastructure::AppConfig;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use uuid::Uuid;

const SYSTEM_ACCOUNT_ID: &str = "00000000-0000-0000-0000-000000000001";

struct SeedUser {
    username: &'static str,
    password: &'static str,
    balance_minor: i64,
}

const SEED_USERS: &[SeedUser] = &[
    SeedUser {
        username: "alice",
        password: "password123",
        balance_minor: 100_000,
    },
    SeedUser {
        username: "bob",
        password: "password123",
        balance_minor: 50_000,
    },
    SeedUser {
        username: "charlie",
        password: "password123",
        balance_minor: 25_000,
    },
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env().map_err(|e| format!("config error: {e}"))?;
    let db = sea_orm::Database::connect(&config.migration_database_url).await?;
    seed(&db).await?;
    println!("seed complete");
    Ok(())
}

async fn seed(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let hasher = Argon2PasswordHasher;
    let system_id = Uuid::parse_str(SYSTEM_ACCOUNT_ID)?;

    let txn = db.begin().await?;

    ensure_system_account(&txn, system_id).await?;

    for user in SEED_USERS {
        let existing = users::Entity::find()
            .filter(users::Column::Username.eq(user.username))
            .one(&txn)
            .await?;
        if existing.is_some() {
            println!("user {} already exists, skipping", user.username);
            continue;
        }

        let user_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();
        let password_hash = hasher.hash(user.password).await?;

        users::ActiveModel {
            id: Set(user_id),
            username: Set(user.username.to_string()),
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

        if user.balance_minor > 0 {
            fund_from_system(
                &txn,
                system_id,
                account_id,
                user_id,
                user_id,
                user.username,
                user.username,
                user.balance_minor,
            )
            .await?;
        }

        println!(
            "seeded user {} with balance {}",
            user.username, user.balance_minor
        );
    }

    txn.commit().await?;
    Ok(())
}

async fn ensure_system_account(
    txn: &sea_orm::DatabaseTransaction,
    system_id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    let existing = accounts::Entity::find_by_id(system_id).one(txn).await?;
    if existing.is_some() {
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

#[allow(clippy::too_many_arguments)]
async fn fund_from_system(
    txn: &sea_orm::DatabaseTransaction,
    system_id: Uuid,
    recipient_account_id: Uuid,
    sender_user_id: Uuid,
    recipient_user_id: Uuid,
    sender_username: &str,
    recipient_username: &str,
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
    let new_system = system_active.balance_minor.as_ref() - amount_minor;
    system_active.balance_minor = Set(new_system);
    system_active.updated_at = Set(Utc::now().into());
    system_active.update(txn).await?;

    println!(
        "funded {} from system: {} minor units",
        recipient_username, amount_minor
    );
    let _ = sender_username;
    Ok(())
}
