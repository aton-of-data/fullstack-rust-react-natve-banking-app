# ADR-005: Double-Entry Ledger

## Status

Accepted

## Context

Money apps require auditability: every balance change must be explainable from historical records. A balance-only table cannot reconstruct how funds moved.

## Decision

Implement **double-entry bookkeeping** with an append-only `ledger_entries` table:

- Each completed transfer creates a **debit** on sender account and **credit** on recipient account
- `account_balances` is a materialized snapshot updated in the same transaction
- System funding (seed) uses the same ledger pattern from the system account
- Domain function `build_transfer_entries` enforces entry pairing

Extends [ADR-0001](./0001-integer-money-representation.md) integer money rules.

## Alternatives Considered

- **Balance-only updates** — no audit trail; rejected
- **Single-entry log** — insufficient for reconciliation
- **Full chart of accounts** — overkill for P2P transfers

## Consequences

- Writes touch 4+ rows per transfer (transfer, 2 ledger rows, 2 balance updates, idempotency, audit)
- Reconciliation tests required on every schema change
- Storage grows linearly with transfers (acceptable for scope)

## Migration Plan

Initial migration creates `ledger_entries` with FK to `transfers`.

## Rollback Plan

Would require backfill strategy to drop ledger; not planned.

## Approval Status

- Architecture Agent: Accepted
- Human reviewer: Pending
