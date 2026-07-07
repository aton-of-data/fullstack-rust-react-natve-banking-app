//! Atomic PostgreSQL transfer executor with double-entry ledger writes.

use std::collections::BTreeMap;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use ficus_application::ports::{TransferExecutor, TransferRecord};
use ficus_domain::audit::{AuditEventDraft, AuditEventType};
use ficus_domain::currency::Currency;
use ficus_domain::errors::DomainError;
use ficus_domain::ledger::{build_transfer_entries, LedgerDirection};
use ficus_domain::money::Money;
use ficus_domain::transfer::TransferStatus;
use rand::Rng;
use sea_orm::sea_query::LockType;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction,
    EntityTrait, QueryFilter, QuerySelect, Set, Statement, TransactionTrait,
};
use tracing::warn;
use uuid::Uuid;

use crate::entities::account_balances::{ActiveModel as BalanceActive, Entity as AccountBalance};
use crate::entities::accounts::{self, Entity as Account};
use crate::entities::audit_events::Entity as AuditEvent;
use crate::entities::idempotency_requests::{self, Entity as IdempotencyRequest};
use crate::entities::ledger_entries::{ActiveModel as LedgerActive, Entity as LedgerEntry};
use crate::entities::transfers::{self, ActiveModel as TransferActive, Entity as Transfer};
use crate::entities::users::{self, Entity as User};
use crate::error::{is_retryable_db_error, map_balance_constraint, map_db_err};
use crate::mapper::{audit_draft_to_active, transfer_status_to_db, transfer_to_record};

const MAX_SERIALIZATION_RETRIES: u32 = 5;
const BASE_RETRY_DELAY_MS: u64 = 25;
const MAX_RETRY_DELAY_MS: u64 = 250;

struct TransferExecutionRequest<'a> {
    sender_user_id: Uuid,
    recipient_username: &'a str,
    amount_minor: i64,
    currency_code: &'a str,
    description: Option<&'a str>,
    idempotency_key: &'a str,
    fingerprint: &'a str,
    request_id: &'a str,
    trace_id: &'a str,
}

/// Executes transfers inside PostgreSQL transactions with row-level locking.
pub struct PostgresTransferExecutor {
    db: DatabaseConnection,
}

impl PostgresTransferExecutor {
    /// Creates an executor backed by the given database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn execute_with_retry(
        &self,
        request: TransferExecutionRequest<'_>,
    ) -> Result<TransferRecord, DomainError> {
        let mut attempt = 0;
        loop {
            match self.try_execute_transfer(&request).await {
                Ok(record) => return Ok(record),
                Err(err) if is_retryable_domain(&err) && attempt < MAX_SERIALIZATION_RETRIES => {
                    attempt += 1;
                    warn!(
                        attempt,
                        max_attempts = MAX_SERIALIZATION_RETRIES,
                        "retrying transfer after serialization failure"
                    );
                    tokio::time::sleep(jitter_backoff(attempt)).await;
                }
                Err(err) => return Err(err),
            }
        }
    }

    async fn try_execute_transfer(
        &self,
        request: &TransferExecutionRequest<'_>,
    ) -> Result<TransferRecord, DomainError> {
        let currency = Currency::from_str(request.currency_code)?;
        let money = Money::transfer_amount(request.amount_minor, currency)?;

        let sender_user = User::find_by_id(request.sender_user_id)
            .one(&self.db)
            .await
            .map_err(map_db_err)?
            .ok_or(DomainError::UserNotFound)?;

        let recipient_user = User::find()
            .filter(users::Column::Username.eq(request.recipient_username))
            .one(&self.db)
            .await
            .map_err(map_db_err)?
            .ok_or(DomainError::RecipientNotFound)?;

        if request.sender_user_id == recipient_user.id {
            return Err(DomainError::SelfTransfer);
        }

        let sender_account = Account::find()
            .filter(accounts::Column::UserId.eq(request.sender_user_id))
            .one(&self.db)
            .await
            .map_err(map_db_err)?
            .ok_or(DomainError::AccountNotFound)?;

        let recipient_account = Account::find()
            .filter(accounts::Column::UserId.eq(recipient_user.id))
            .one(&self.db)
            .await
            .map_err(map_db_err)?
            .ok_or(DomainError::AccountNotFound)?;

        if sender_account.id == recipient_account.id {
            return Err(DomainError::SelfTransfer);
        }

        let txn = match self
            .db
            .begin_with_config(
                Some(sea_orm::IsolationLevel::ReadCommitted),
                Some(sea_orm::AccessMode::ReadWrite),
            )
            .await
        {
            Ok(txn) => txn,
            Err(err) if is_retryable_db_error(&err) => {
                return Err(DomainError::Validation(format!(
                    "serialization failure: {err}"
                )));
            }
            Err(err) => return Err(map_db_err(err)),
        };

        match self
            .execute_in_transaction(
                &txn,
                request,
                &sender_user,
                &recipient_user,
                &sender_account,
                &recipient_account,
                &money,
            )
            .await
        {
            Ok(record) => {
                txn.commit().await.map_err(map_txn_err)?;
                Ok(record)
            }
            Err(err) => {
                let _ = txn.rollback().await;
                Err(err)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn execute_in_transaction(
        &self,
        txn: &DatabaseTransaction,
        request: &TransferExecutionRequest<'_>,
        sender_user: &users::Model,
        recipient_user: &users::Model,
        sender_account: &accounts::Model,
        recipient_account: &accounts::Model,
        money: &Money,
    ) -> Result<TransferRecord, DomainError> {
        lock_idempotency_scope(txn, sender_user.id, request.idempotency_key).await?;

        if let Some(existing) = IdempotencyRequest::find()
            .filter(idempotency_requests::Column::SenderUserId.eq(sender_user.id))
            .filter(idempotency_requests::Column::IdempotencyKey.eq(request.idempotency_key))
            .one(txn)
            .await
            .map_err(map_db_err)?
        {
            if existing.fingerprint != request.fingerprint {
                return Err(DomainError::IdempotencyConflict);
            }
            return serde_json::from_str(&existing.response_body)
                .map_err(|_| DomainError::Validation("stored response corrupt".into()));
        }

        let transfer_id = Uuid::new_v4();
        let amount_minor = money.amount_minor();
        let currency_code = money.currency().code().to_string();

        append_audit(
            txn,
            AuditEventDraft {
                event_type: AuditEventType::TransferRequested,
                actor_user_id: Some(sender_user.id),
                transfer_id: None,
                request_id: request.request_id.to_string(),
                trace_id: request.trace_id.to_string(),
                metadata: transfer_metadata(
                    request.idempotency_key,
                    request.fingerprint,
                    recipient_user.username.as_str(),
                    amount_minor,
                    currency_code.as_str(),
                )
                .into_iter()
                .chain([("transfer_id".to_string(), transfer_id.to_string())])
                .collect(),
                occurred_at: Utc::now(),
            },
        )
        .await?;

        lock_balances_in_order(txn, sender_account.id, recipient_account.id).await?;

        let sender_balance = AccountBalance::find_by_id(sender_account.id)
            .one(txn)
            .await
            .map_err(map_db_err)?
            .ok_or(DomainError::AccountNotFound)?;

        if sender_balance.currency_code != currency_code {
            return Err(DomainError::UnsupportedCurrency(currency_code.clone()));
        }

        if sender_balance.balance_minor < amount_minor {
            append_audit(
                txn,
                AuditEventDraft {
                    event_type: AuditEventType::TransferDeclined,
                    actor_user_id: Some(sender_user.id),
                    transfer_id: None,
                    request_id: request.request_id.to_string(),
                    trace_id: request.trace_id.to_string(),
                    metadata: BTreeMap::from([
                        ("reason".to_string(), "insufficient_funds".to_string()),
                        ("transfer_id".to_string(), transfer_id.to_string()),
                    ]),
                    occurred_at: Utc::now(),
                },
            )
            .await?;
            return Err(DomainError::InsufficientFunds);
        }

        let recipient_balance = AccountBalance::find_by_id(recipient_account.id)
            .one(txn)
            .await
            .map_err(map_db_err)?
            .ok_or(DomainError::AccountNotFound)?;

        if recipient_balance.currency_code != currency_code {
            return Err(DomainError::UnsupportedCurrency(currency_code.clone()));
        }

        let transfer_model = TransferActive {
            id: Set(transfer_id),
            sender_account_id: Set(sender_account.id),
            recipient_account_id: Set(recipient_account.id),
            sender_user_id: Set(sender_user.id),
            recipient_user_id: Set(recipient_user.id),
            amount_minor: Set(amount_minor),
            currency_code: Set(currency_code.clone()),
            description: Set(request.description.map(str::to_string)),
            status: Set(transfer_status_to_db(TransferStatus::Completed).to_string()),
            created_at: Set(Utc::now().into()),
        };
        Transfer::insert(transfer_model)
            .exec(txn)
            .await
            .map_err(map_txn_err)?;

        let entries =
            build_transfer_entries(transfer_id, sender_account.id, recipient_account.id, money)?;

        for entry in entries {
            let direction = match entry.direction {
                LedgerDirection::Debit => "debit",
                LedgerDirection::Credit => "credit",
            };
            LedgerEntry::insert(LedgerActive {
                id: Set(Uuid::new_v4()),
                account_id: Set(entry.account_id),
                transfer_id: Set(entry.transfer_id),
                amount_minor: Set(entry.amount_minor),
                direction: Set(direction.to_string()),
                currency_code: Set(entry.currency.code().to_string()),
                created_at: Set(Utc::now().into()),
            })
            .exec(txn)
            .await
            .map_err(map_txn_err)?;
        }

        let new_sender_balance = sender_balance
            .balance_minor
            .checked_sub(amount_minor)
            .ok_or(DomainError::NegativeBalance)?;
        if new_sender_balance < 0 {
            return Err(DomainError::NegativeBalance);
        }

        let new_recipient_balance = recipient_balance
            .balance_minor
            .checked_add(amount_minor)
            .ok_or(DomainError::InvalidMoney("overflow".into()))?;

        AccountBalance::update(BalanceActive {
            account_id: Set(sender_account.id),
            balance_minor: Set(new_sender_balance),
            currency_code: Set(sender_balance.currency_code),
            updated_at: Set(Utc::now().into()),
        })
        .exec(txn)
        .await
        .map_err(|err| map_balance_constraint(&err).unwrap_or_else(|| map_txn_err(err)))?;

        AccountBalance::update(BalanceActive {
            account_id: Set(recipient_account.id),
            balance_minor: Set(new_recipient_balance),
            currency_code: Set(recipient_balance.currency_code),
            updated_at: Set(Utc::now().into()),
        })
        .exec(txn)
        .await
        .map_err(|err| map_balance_constraint(&err).unwrap_or_else(|| map_txn_err(err)))?;

        let record = transfer_to_record(
            transfers::Model {
                id: transfer_id,
                sender_account_id: sender_account.id,
                recipient_account_id: recipient_account.id,
                sender_user_id: sender_user.id,
                recipient_user_id: recipient_user.id,
                amount_minor,
                currency_code,
                description: request.description.map(str::to_string),
                status: transfer_status_to_db(TransferStatus::Completed).to_string(),
                created_at: Utc::now().into(),
            },
            sender_user.username.clone(),
            recipient_user.username.clone(),
        )?;

        let response_body = serde_json::to_string(&record)
            .map_err(|_| DomainError::Validation("serialization failed".into()))?;

        IdempotencyRequest::insert(idempotency_requests::ActiveModel {
            sender_user_id: Set(sender_user.id),
            idempotency_key: Set(request.idempotency_key.to_string()),
            fingerprint: Set(request.fingerprint.to_string()),
            response_body: Set(response_body),
            status_code: Set(200),
            created_at: Set(Utc::now().into()),
        })
        .exec(txn)
        .await
        .map_err(map_txn_err)?;

        append_audit(
            txn,
            AuditEventDraft {
                event_type: AuditEventType::TransferCompleted,
                actor_user_id: Some(sender_user.id),
                transfer_id: Some(transfer_id),
                request_id: request.request_id.to_string(),
                trace_id: request.trace_id.to_string(),
                metadata: BTreeMap::from([
                    ("amount_minor".to_string(), amount_minor.to_string()),
                    (
                        "recipient_username".to_string(),
                        recipient_user.username.clone(),
                    ),
                ]),
                occurred_at: Utc::now(),
            },
        )
        .await?;

        Ok(record)
    }
}

#[async_trait]
impl TransferExecutor for PostgresTransferExecutor {
    #[allow(clippy::too_many_arguments)]
    async fn execute_transfer(
        &self,
        sender_user_id: Uuid,
        recipient_username: &str,
        amount_minor: i64,
        currency_code: &str,
        description: Option<&str>,
        idempotency_key: &str,
        fingerprint: &str,
        request_id: &str,
        trace_id: &str,
    ) -> Result<TransferRecord, DomainError> {
        self.execute_with_retry(TransferExecutionRequest {
            sender_user_id,
            recipient_username,
            amount_minor,
            currency_code,
            description,
            idempotency_key,
            fingerprint,
            request_id,
            trace_id,
        })
        .await
    }
}

async fn lock_idempotency_scope(
    txn: &DatabaseTransaction,
    sender_user_id: Uuid,
    idempotency_key: &str,
) -> Result<(), DomainError> {
    let key = format!("{sender_user_id}:{idempotency_key}");
    let escaped = key.replace('\'', "''");
    let sql = format!("SELECT pg_advisory_xact_lock(hashtext('{escaped}'))");
    txn.execute(Statement::from_string(DatabaseBackend::Postgres, sql))
        .await
        .map_err(map_db_err)?;
    Ok(())
}

async fn lock_balances_in_order(
    txn: &DatabaseTransaction,
    first_account_id: Uuid,
    second_account_id: Uuid,
) -> Result<(), DomainError> {
    let (low, high) = if first_account_id < second_account_id {
        (first_account_id, second_account_id)
    } else {
        (second_account_id, first_account_id)
    };

    for account_id in [low, high] {
        AccountBalance::find_by_id(account_id)
            .lock(LockType::Update)
            .one(txn)
            .await
            .map_err(map_db_err)?
            .ok_or(DomainError::AccountNotFound)?;
    }

    Ok(())
}

async fn append_audit(
    txn: &DatabaseTransaction,
    draft: AuditEventDraft,
) -> Result<(), DomainError> {
    AuditEvent::insert(audit_draft_to_active(draft))
        .exec(txn)
        .await
        .map_err(map_txn_err)?;
    Ok(())
}

fn map_txn_err(err: sea_orm::DbErr) -> DomainError {
    if is_retryable_db_error(&err) {
        return DomainError::Validation(format!("serialization failure: {err}"));
    }
    if let Some(mapped) = map_balance_constraint(&err) {
        return mapped;
    }
    map_db_err(err)
}

fn transfer_metadata(
    idempotency_key: &str,
    fingerprint: &str,
    recipient_username: &str,
    amount_minor: i64,
    currency_code: &str,
) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("idempotency_key".to_string(), idempotency_key.to_string()),
        ("fingerprint".to_string(), fingerprint.to_string()),
        (
            "recipient_username".to_string(),
            recipient_username.to_string(),
        ),
        ("amount_minor".to_string(), amount_minor.to_string()),
        ("currency_code".to_string(), currency_code.to_string()),
    ])
}

fn jitter_backoff(attempt: u32) -> Duration {
    let mut rng = rand::thread_rng();
    let exp = BASE_RETRY_DELAY_MS.saturating_mul(1u64 << attempt.min(4));
    let capped = exp.min(MAX_RETRY_DELAY_MS);
    let jitter = rng.gen_range(0..=capped / 2);
    Duration::from_millis(capped + jitter)
}

fn is_retryable_domain(err: &DomainError) -> bool {
    matches!(
        err,
        DomainError::Validation(msg) if msg.contains("serialization") || msg.contains("deadlock")
    )
}

use std::str::FromStr;
