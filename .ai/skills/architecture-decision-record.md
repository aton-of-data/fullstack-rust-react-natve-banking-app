# Skill: Architecture Decision Record (ADR)

## When to Create an ADR

Create under `docs/ai/adr/` when changing:

- State management strategy
- API contract shape
- Database schema or migration strategy
- Auth or authorization model
- Core crate boundaries
- New dependency or framework
- Cross-cutting infrastructure (WebSocket, caching, queues)
- Repository rule or exception

## Filename

```text
docs/ai/adr/NNNN-short-title.md
```

`NNNN` = zero-padded sequence (0001, 0002, ...).

## Template

```markdown
# ADR-NNNN: Title

## Status
Proposed | Accepted | Deprecated | Superseded by ADR-XXXX

## Context
What is the issue? What forces are at play?

## Decision
What we decided.

## Alternatives Considered
- Option A — pros/cons
- Option B — pros/cons

## Consequences
Positive and negative outcomes.

## Migration Plan
How to adopt; steps for existing code/data.

## Rollback Plan
How to revert if decision fails.

## Approval Status
- Architecture Agent: ...
- Human reviewer (if required): ...
```

## Linking

- Reference ADR IDs in `01-architecture.md`
- Update ADR status when implemented

## Rules

- Exceptions to lint prohibitions (useState, fetch, etc.) require Accepted ADR
- Do not merge conflicting ADRs — supersede explicitly
