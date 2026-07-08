//! Initial schema migration: users, accounts, balances, transfers, ledger,
//! idempotency, audit tables, and append-only / integrity constraints.
//!
//! This module is pure DDL. Do not add seed data or application funding logic
//! here. Financial correctness depends on the constraints and triggers defined
//! in `up` (for example append-only ledger/audit protection).

use sea_orm_migration::prelude::*;

/// First schema migration for the Ficus API database.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            DO $$
            BEGIN
                IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'ficus_app') THEN
                    CREATE ROLE ficus_app LOGIN PASSWORD 'ficus_app_dev';
                END IF;
            END
            $$;
            "#,
        )
        .await?;

        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Users::Username)
                            .string_len(32)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::PasswordHash)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Accounts::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Accounts::UserId).uuid())
                    .col(
                        ColumnDef::new(Accounts::IsSystem)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Accounts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_accounts_user_id")
                            .from(Accounts::Table, Accounts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_accounts_user_id")
                    .table(Accounts::Table)
                    .col(Accounts::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AccountBalances::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AccountBalances::AccountId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AccountBalances::BalanceMinor)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AccountBalances::CurrencyCode)
                            .string_len(3)
                            .not_null()
                            .default("USD"),
                    )
                    .col(
                        ColumnDef::new(AccountBalances::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_account_balances_account_id")
                            .from(AccountBalances::Table, AccountBalances::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Transfers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Transfers::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Transfers::SenderAccountId).uuid().not_null())
                    .col(
                        ColumnDef::new(Transfers::RecipientAccountId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Transfers::SenderUserId).uuid().not_null())
                    .col(ColumnDef::new(Transfers::RecipientUserId).uuid().not_null())
                    .col(
                        ColumnDef::new(Transfers::AmountMinor)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transfers::CurrencyCode)
                            .string_len(3)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Transfers::Description).text())
                    .col(ColumnDef::new(Transfers::Status).string_len(16).not_null())
                    .col(
                        ColumnDef::new(Transfers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transfers_sender_account_id")
                            .from(Transfers::Table, Transfers::SenderAccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transfers_recipient_account_id")
                            .from(Transfers::Table, Transfers::RecipientAccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transfers_sender_user_id")
                            .from(Transfers::Table, Transfers::SenderUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transfers_recipient_user_id")
                            .from(Transfers::Table, Transfers::RecipientUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_transfers_created_at_id")
                    .table(Transfers::Table)
                    .col(Transfers::CreatedAt)
                    .col(Transfers::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(LedgerEntries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LedgerEntries::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(LedgerEntries::AccountId).uuid().not_null())
                    .col(ColumnDef::new(LedgerEntries::TransferId).uuid().not_null())
                    .col(
                        ColumnDef::new(LedgerEntries::AmountMinor)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LedgerEntries::Direction)
                            .string_len(8)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LedgerEntries::CurrencyCode)
                            .string_len(3)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LedgerEntries::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ledger_entries_account_id")
                            .from(LedgerEntries::Table, LedgerEntries::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ledger_entries_transfer_id")
                            .from(LedgerEntries::Table, LedgerEntries::TransferId)
                            .to(Transfers::Table, Transfers::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ledger_entries_account_created_at")
                    .table(LedgerEntries::Table)
                    .col(LedgerEntries::AccountId)
                    .col(LedgerEntries::CreatedAt)
                    .col(LedgerEntries::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(IdempotencyRequests::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(IdempotencyRequests::SenderUserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IdempotencyRequests::IdempotencyKey)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IdempotencyRequests::Fingerprint)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IdempotencyRequests::ResponseBody)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IdempotencyRequests::StatusCode)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IdempotencyRequests::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(IdempotencyRequests::SenderUserId)
                            .col(IdempotencyRequests::IdempotencyKey),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_idempotency_sender_user_id")
                            .from(
                                IdempotencyRequests::Table,
                                IdempotencyRequests::SenderUserId,
                            )
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuditEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuditEvents::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AuditEvents::EventType)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(AuditEvents::ActorUserId).uuid())
                    .col(ColumnDef::new(AuditEvents::TransferId).uuid())
                    .col(
                        ColumnDef::new(AuditEvents::RequestId)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AuditEvents::TraceId)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AuditEvents::Metadata)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(AuditEvents::OccurredAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_events_actor_user_id")
                            .from(AuditEvents::Table, AuditEvents::ActorUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_events_transfer_id")
                            .from(AuditEvents::Table, AuditEvents::TransferId)
                            .to(Transfers::Table, Transfers::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_events_occurred_at")
                    .table(AuditEvents::Table)
                    .col(AuditEvents::OccurredAt)
                    .col(AuditEvents::Id)
                    .to_owned(),
            )
            .await?;

        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION ficus_prevent_ledger_mutation()
            RETURNS TRIGGER AS $$
            BEGIN
                RAISE EXCEPTION 'ledger_entries is append-only';
            END;
            $$ LANGUAGE plpgsql;

            CREATE OR REPLACE FUNCTION ficus_prevent_audit_mutation()
            RETURNS TRIGGER AS $$
            BEGIN
                RAISE EXCEPTION 'audit_events is append-only';
            END;
            $$ LANGUAGE plpgsql;

            DROP TRIGGER IF EXISTS ledger_entries_append_only ON ledger_entries;
            CREATE TRIGGER ledger_entries_append_only
                BEFORE UPDATE OR DELETE ON ledger_entries
                FOR EACH ROW
                EXECUTE FUNCTION ficus_prevent_ledger_mutation();

            DROP TRIGGER IF EXISTS audit_events_append_only ON audit_events;
            CREATE TRIGGER audit_events_append_only
                BEFORE UPDATE OR DELETE ON audit_events
                FOR EACH ROW
                EXECUTE FUNCTION ficus_prevent_audit_mutation();

            ALTER TABLE account_balances
                ADD CONSTRAINT chk_account_balances_non_negative
                CHECK (balance_minor >= 0);

            ALTER TABLE transfers
                ADD CONSTRAINT chk_transfers_amount_positive
                CHECK (amount_minor > 0);

            ALTER TABLE ledger_entries
                ADD CONSTRAINT chk_ledger_entries_amount_positive
                CHECK (amount_minor > 0);

            ALTER TABLE ledger_entries
                ADD CONSTRAINT chk_ledger_entries_direction
                CHECK (direction IN ('debit', 'credit'));
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            GRANT USAGE ON SCHEMA public TO ficus_app;
            GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO ficus_app;
            GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO ficus_app;
            ALTER DEFAULT PRIVILEGES IN SCHEMA public
                GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO ficus_app;
            ALTER DEFAULT PRIVILEGES IN SCHEMA public
                GRANT USAGE, SELECT ON SEQUENCES TO ficus_app;
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            DROP TRIGGER IF EXISTS ledger_entries_append_only ON ledger_entries;
            DROP TRIGGER IF EXISTS audit_events_append_only ON audit_events;
            DROP FUNCTION IF EXISTS ficus_prevent_ledger_mutation();
            DROP FUNCTION IF EXISTS ficus_prevent_audit_mutation();
            "#,
        )
        .await?;

        manager
            .drop_table(Table::drop().table(AuditEvents::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(IdempotencyRequests::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(LedgerEntries::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Transfers::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AccountBalances::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Username,
    PasswordHash,
    CreatedAt,
}

#[derive(Iden)]
enum Accounts {
    Table,
    Id,
    UserId,
    IsSystem,
    CreatedAt,
}

#[derive(Iden)]
enum AccountBalances {
    Table,
    AccountId,
    BalanceMinor,
    CurrencyCode,
    UpdatedAt,
}

#[derive(Iden)]
enum Transfers {
    Table,
    Id,
    SenderAccountId,
    RecipientAccountId,
    SenderUserId,
    RecipientUserId,
    AmountMinor,
    CurrencyCode,
    Description,
    Status,
    CreatedAt,
}

#[derive(Iden)]
enum LedgerEntries {
    Table,
    Id,
    AccountId,
    TransferId,
    AmountMinor,
    Direction,
    CurrencyCode,
    CreatedAt,
}

#[derive(Iden)]
enum IdempotencyRequests {
    Table,
    SenderUserId,
    IdempotencyKey,
    Fingerprint,
    ResponseBody,
    StatusCode,
    CreatedAt,
}

#[derive(Iden)]
enum AuditEvents {
    Table,
    Id,
    EventType,
    ActorUserId,
    TransferId,
    RequestId,
    TraceId,
    Metadata,
    OccurredAt,
}
