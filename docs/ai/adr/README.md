# Architecture Decision Records

ADRs document significant technical decisions. Filename format:

```text
docs/ai/adr/NNNN-short-title.md
```

## When to Write an ADR

- State management strategy
- API contract shape
- Database schema or migrations
- Auth/authorization model
- Core crate boundaries
- New dependency or framework
- Cross-cutting infrastructure
- Repository rule exceptions

## Template

See `.ai/skills/architecture-decision-record.md`.

## Status Values

`Proposed` | `Accepted` | `Deprecated` | `Superseded by ADR-XXXX`

## Index

| ADR                                            | Title                        | Status   |
| ---------------------------------------------- | ---------------------------- | -------- |
| [0001](./0001-integer-money-representation.md) | Integer Money Representation | Proposed |
| [001](./001-monorepo-tooling.md)               | Monorepo Tooling             | Accepted |
| [002](./002-hexagonal-rust-backend.md)         | Hexagonal Rust Backend       | Accepted |
| [003](./003-jwt-authentication.md)             | JWT Authentication           | Accepted |
| [004](./004-transfer-idempotency.md)           | Transfer Idempotency         | Accepted |
| [005](./005-double-entry-ledger.md)            | Double-Entry Ledger          | Accepted |
| [006](./006-sse-global-feed.md)                | SSE Global Feed              | Accepted |
| [007](./007-redux-mobile-state.md)             | Redux Mobile State           | Accepted |
| [008](./008-postgresql-concurrency.md)         | PostgreSQL Concurrency       | Accepted |
| [009](./009-observability-stack.md)            | Observability Stack          | Accepted |
