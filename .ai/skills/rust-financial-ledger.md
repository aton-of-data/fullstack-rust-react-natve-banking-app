# Skill: Rust Financial Ledger

## When to Use

Implementing or reviewing money movement, balances, ledger entries, reconciliation, or transfer invariants in the Rust API.

## Core Rules

1. **Integer minor units only** — `Money` newtype in `domain`; never `f64` (ADR-0001)
2. **Double-entry** — every completed transfer = debit sender + credit recipient (ADR-005)
3. **Atomic writes** — transfer, ledger, balances, idempotency in one DB transaction
4. **Append-only ledger** — never UPDATE/DELETE `ledger_entries`
5. **Reconciliation** — sum(signed ledger) == `account_balances.balance_minor`

## Implementation Checklist

| Step                     | Location                                         |
| ------------------------ | ------------------------------------------------ |
| Validate amount/currency | `domain::money`, `domain::currency`              |
| Build entry pair         | `domain::ledger::build_transfer_entries`         |
| Execute atomically       | `adapters-persistence::PostgresTransferExecutor` |
| Map errors               | `DomainError` → HTTP status in adapter           |
| Publish feed event       | `application::FeedService` after commit          |

## Concurrency

- `SELECT … FOR UPDATE` on balance rows (ADR-008)
- Retry serialization failures (max 5, jittered backoff)
- `CHECK (balance_minor >= 0)` as DB backstop

## Idempotency

- Require `Idempotency-Key` header
- Fingerprint body fields; 409 on mismatch (ADR-004)
- Store response in same transaction

## Required Tests

```bash
cd apps/api && cargo nextest run -p ficus-testkit
```

| Test file                               | Proves                                   |
| --------------------------------------- | ---------------------------------------- |
| `transfer_concurrency_test.rs`          | 100 parallel debits, no negative balance |
| `transfer_idempotency_test.rs`          | Duplicate key safe                       |
| `money_conservation_test.rs`            | Total money conserved                    |
| `ledger_balance_reconciliation_test.rs` | Ledger sums match balances               |

## Documentation

- Architecture: `docs/architecture/financial-ledger.md`
- ADRs: 0001, 004, 005, 008

## Red Flags

- Balance update without ledger entries
- Ledger write outside transaction
- Float parsing in transfer path
- ORM entities imported in `domain`
