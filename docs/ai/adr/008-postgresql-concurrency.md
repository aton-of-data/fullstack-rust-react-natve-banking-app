# ADR-008: PostgreSQL Concurrency

## Status

Accepted

## Context

The product mandates a test proving 100 concurrent transfers from one funded account do not overdraft or corrupt balances. Application-level read-modify-write without locking is insufficient under parallel requests.

## Decision

Use **pessimistic row-level locking** in PostgreSQL:

1. `BEGIN` transaction per transfer attempt
2. `SELECT … FOR UPDATE` on sender and recipient `account_balances` rows (ordered by account ID)
3. Validate balance, write transfer + ledger + balance updates atomically
4. `CHECK (balance_minor >= 0)` constraint on `account_balances`
5. Retry up to 5 times on serialization/deadlock with jittered backoff

## Alternatives Considered

- **Optimistic locking (version column)** — higher conflict rate under 100-way contention
- **Serializable isolation** — broader contention, more aborts
- **Application mutex** — does not work across multiple API instances

## Consequences

- Latency increases under same-account hot spots
- Correctness proven by `transfer_concurrency_test.rs`
- Executor complexity in `PostgresTransferExecutor`

## Migration Plan

Constraint and indexes included in initial migration.

## Rollback Plan

Switching models requires new ADR and re-running concurrency test suite.

## Approval Status

- Architecture Agent: Accepted
- Human reviewer: Pending
