# ADR-007: Redux Mobile State

## Status

Accepted

## Context

React Native apps often accumulate ad-hoc `useState` and direct `fetch` calls. Ficus governance requires predictable state ownership, testable reducers, and a single API layer for maintainability and agent-enforced rules.

## Decision

Mobile application state uses **Redux Toolkit exclusively**:

| Concern                | Owner                                    |
| ---------------------- | ---------------------------------------- |
| Session / auth         | `authSlice`                              |
| UI chrome              | `uiSlice`                                |
| Transfer form workflow | `transferFormSlice`                      |
| Server data            | RTK Query (`ficusApi`)                   |
| Side effects           | Listener middleware, RTK Query lifecycle |

**Prohibited** without ADR exception: `useState`, `useReducer`, Context, direct `fetch`/`axios` in UI layers.

Atomic Design folder structure under `apps/mobile/src/`.

## Alternatives Considered

- **React Query + Context** — violates governance; no slice purity guarantees
- **Zustand** — not aligned with enforced ESLint architecture rules
- **Local component state for forms** — rejected; transfer form is app workflow state

## Consequences

- Higher boilerplate for simple UI; offset by predictability
- ESLint enforces rules at CI
- SSE feed integration via RTK Query cache mutation pattern

## Migration Plan

N/A — greenfield mobile app.

## Rollback Plan

Any exception requires Accepted ADR and eslint rule update.

## Approval Status

- Architecture Agent: Accepted
- Human reviewer: Pending
