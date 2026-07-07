# Backend Policy — Rust API

Canonical reference: `context.md`

## Default Stack

| Concern | Choice |
|---------|--------|
| Runtime | Tokio |
| HTTP | Axum |
| Middleware | Tower, tower-http |
| Serialization | Serde |
| Validation | validator or typed boundary validation |
| Database | PostgreSQL |
| ORM | SeaORM (or repo-approved equivalent) |
| Migrations | ORM-native tooling |
| Observability | tracing, tracing-subscriber, OpenTelemetry as needed |
| OpenAPI | utoipa or equivalent |
| Errors | thiserror (domain/library) |
| Testing | cargo test, nextest, rstest, testcontainers, wiremock, proptest |

Verify crate versions from official docs before adding. Pin via `Cargo.lock`.

## Workspace Layout

```text
backend/
  Cargo.toml
  crates/
    domain/                 # business rules, money invariants
    application/            # use cases, ports, orchestration
    contracts/              # request/response/OpenAPI types
    adapters-http/          # Axum handlers, HTTP mapping
    adapters-persistence/   # ORM, repos, migrations
    infrastructure/         # config, telemetry, external clients
    testkit/                # shared test fixtures
```

## Dependency Direction

```text
domain ← application ← adapters & infrastructure ← composition root (binary)
```

| Crate | May depend on | Must not depend on |
|-------|---------------|-------------------|
| domain | std, pure libs | HTTP, ORM, Axum, env |
| application | domain | framework details in domain rules |
| contracts | serde types | ORM entities |
| adapters-http | application, contracts | business logic in handlers |
| adapters-persistence | application, domain (via mapping) | leaking ORM to domain |
| infrastructure | application ports | domain rule violations |

## API Design

- Thin handlers: validate → map to command → invoke use case → map response
- Typed errors with consistent HTTP mapping
- Version public APIs
- Request IDs, structured logs, timeouts
- Health and readiness endpoints
- Graceful shutdown

## Money & Transfers (Product Critical)

- Integer minor units in domain and DB
- Idempotency keys on transfer endpoints
- Database transactions with explicit isolation for concurrent debits
- Append-only transaction/ledger records for audit
- No partial state on failure (rollback or compensating design)
- Document invariants in domain and test concurrently

Required tests (minimum):

- 100 concurrent transfers from one funded account
- Retry/idempotency duplicate request test

## Quality Gates

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo nextest run          # when configured
cargo deny check           # when configured
cargo audit                # when configured
```

If a gate is unavailable, document why in work item and add CI when possible.

## Security

- Validate all inputs at system boundaries
- Password hashing (argon2 or bcrypt) — ADR for choice
- Auth tokens/sessions — ADR required
- No secrets in code or logs
- SQL via ORM/query builder; no string-concatenated SQL
