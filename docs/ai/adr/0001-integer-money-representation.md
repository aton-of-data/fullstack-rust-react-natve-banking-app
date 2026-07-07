# ADR-0001: Integer Money Representation

## Status

Proposed

## Context

The Ficus app handles real money transfers. Floating-point arithmetic is unsafe for currency. The product brief requires deliberate choices about balance representation, concurrency, and auditability.

## Decision

Represent all monetary amounts as **signed integer minor units** (e.g., cents) using a dedicated `Money` newtype in the Rust `domain` crate. Persist as `BIGINT` in PostgreSQL. Never use `f64`/`Decimal` in hot transfer paths unless an ADR supersedes this.

Display formatting (dollars.cents) happens only at UI/API presentation boundaries.

## Alternatives Considered

- **Decimal/Numeric in DB** — accurate but adds dependency and mapping complexity; defer unless rounding rules require it
- **Float** — rejected; rounding errors unacceptable
- **String amounts** — rejected; parsing/validation overhead without benefit

## Consequences

- All arithmetic is exact in domain logic
- API must document amounts as integer minor units
- Frontend must format for display without floating math on raw values

## Migration Plan

N/A — greenfield. Initial schema uses integer columns from day one.

## Rollback Plan

If Decimal adopted later, migration ADR required with backfill strategy.

## Approval Status

- Architecture Agent: Proposed (governance bootstrap)
- Human reviewer: Pending
