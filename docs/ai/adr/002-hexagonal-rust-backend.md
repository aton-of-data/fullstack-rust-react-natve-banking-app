# ADR-002: Hexagonal Rust Backend

## Status

Accepted

## Context

The API must enforce money invariants, remain testable without HTTP/DB, and support swapping persistence or transport layers. A flat Axum + SeaORM structure would couple business rules to frameworks.

## Decision

Structure the API as a **hexagonal (ports and adapters)** Cargo workspace:

```
domain → application ← adapters-http
                      ← adapters-persistence
infrastructure (composition root)
```

- **domain** — pure types and rules (`Money`, `Transfer`, ledger math)
- **application** — use cases and port traits
- **adapters-*** — framework-specific implementations
- **infrastructure** — wiring, config, binaries

## Alternatives Considered

- **Anemic handlers + SeaORM everywhere** — fast to start, untestable domain
- **Single crate** — simpler build, poor boundary enforcement
- **Microservices** — overkill for product scope

## Consequences

- More crates to navigate; clippy/test run across workspace
- Clear test pyramid: domain unit tests, application with fakes, integration via testkit
- New features touch application ports before adapters

## Migration Plan

N/A — greenfield.

## Rollback Plan

Consolidating crates possible but loses boundary compiler enforcement; would need new ADR.

## Approval Status

- Architecture Agent: Accepted
- Human reviewer: Pending
