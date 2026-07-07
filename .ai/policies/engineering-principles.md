# Engineering Principles

Canonical reference: `context.md`

## Core Principles

1. **Minimize scope** — Smallest correct diff. No unrelated changes.
2. **Match conventions** — Read surrounding code before writing. Reuse existing abstractions.
3. **Explicit over implicit** — Contracts, types, and boundaries must be clear.
4. **Evidence over claims** — Never report success without executed verification.
5. **Reversible changes** — Prefer small commits and clear rollback paths.

## Architectural Ownership

| Layer | Owns |
|-------|------|
| `domain` (Rust) | Business rules, invariants, money math |
| `application` | Use cases, orchestration, ports |
| `features/` (RN) | Feature slices, RTK Query endpoints, selectors |
| `shared/ui/` | Presentational atoms, molecules, organisms |
| `pages/` | Route composition only |

## Money Domain Rules

- Represent money as integer minor units (e.g., cents) in domain and persistence.
- Balance changes must be atomic and auditable.
- Transfers must be idempotent (idempotency keys required).
- Concurrent transfers must not produce negative balances or lost funds.
- Document consistency model and isolation level in ADR.

## Dependency Policy

New dependencies require:

1. Documented rationale in work item or ADR
2. Version pinning (`Cargo.lock`, lockfile)
3. Security review for network/crypto/auth crates
4. License compatibility check (`cargo deny` when available)

## Comments and Documentation

- Code should be mostly self-explanatory.
- Comments explain non-obvious business logic, invariants, and failure modes.
- Work item artifacts document decisions across the pipeline.

## Prohibited Shortcuts

- Skipping pipeline stages
- Self-approving QA or review
- "Temporary" violations without ADR
- Claiming tests passed without running them
