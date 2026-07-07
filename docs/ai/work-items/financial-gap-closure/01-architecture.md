# Financial Gap Closure — Architecture

## Status

APPROVED — implementation authorized.

## Scope

Close verified coverage and implementation gaps across backend HTTP contracts, financial concurrency, database immutability, mobile idempotency lifecycle, observability, performance testing, and documentation.

## Decisions

### HTTP integration tests

Use existing `TestAppBuilder::with_http_server()` against real Axum + PostgreSQL (testcontainers). No transfer-service mocks for HTTP contract tests.

### Mobile idempotency

Redux-owned `transferSubmission` slice holds idempotency key lifecycle. RTK Query `createTransfer` mutation receives key via argument; never regenerates on re-render or timeout.

See ADR: `docs/ai/adr/ADR-012-mobile-transfer-idempotency-lifecycle.md`.

### Serialization retry observability

Record `ficus_transfer_serialization_retry_total` in `PostgresTransferExecutor` retry loop. Unit-test retry classification functions in persistence crate.

### Reconciliation

`ficus_testkit::reconcile_all_accounts` validates projection vs ledger sum for all accounts. Used in integration tests and post-load scripts.

### Rate limit testing

`TestAppBuilder::with_login_rate_limit` configures deterministic low limits for HTTP 429 contract tests.

## Acceptance criteria

- HTTP tests cover auth, transfer, balance, ledger, feed, and SSE contracts.
- Concurrency tests prove exact success/failure counts, cross-account contention, inverse-direction locking, self-transfer rejection.
- Database append-only triggers validated by direct SQL tests.
- Mobile sends `Idempotency-Key`, reuses on retry, rotates on new transfer.
- k6 scenarios cover happy path, idempotency replay, concurrency, feed SSE, reconciliation.
- Documentation reflects implemented behavior with verified commands.

## Out of scope

- OTLP trace export wiring (dependency present; full integration deferred).
- Idempotency TTL/cleanup job (no `expires_at` column in schema).
