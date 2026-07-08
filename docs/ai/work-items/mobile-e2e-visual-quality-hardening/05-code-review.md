# Code Review: mobile-e2e-visual-quality-hardening

## Outcome

**APPROVED**

## Summary

Changes match architecture and ADRs: idempotency key preserve, global 401 logout, SSE buffering/reconnect, Maestro E2E with sequential runner, and curated documentation screenshots. Layer boundaries (Redux + RTK Query, no forbidden UI state) are respected.

## Architecture Alignment

- [x] Matches `01-architecture.md`
- [x] No unauthorized dependencies (Maestro is system CLI; `@types/react-test-renderer` is test typings only)
- [x] Layer boundaries respected (pure slices; no `useState` / Context / direct `fetch` for app state)

## Correctness

- `beginTransferAttempt` no-op when key exists; locked amount/description for in-flight attempts
- `baseQueryWithAuth` clears credentials and caches on non-login 401
- SSE incomplete-line buffer + `Last-Event-ID` + backoff
- Hermes `formatToParts` guard prevents RN money formatting crashes
- Maestro flows use `reset-to-login` and Expo Go `appId`

## Code Quality

Clear selectors/`testID`s; runner mirrors k6 SKIP pattern; docs-capture excluded from suite via `docs-` prefix.

## Documentation

- [x] JSDoc accurate on new exported symbols
- [x] Mermaid diagrams match implementation
- [x] Module READMEs / runbook / root README updated

## Tooling

- [x] Mobile lint / typecheck / tests pass (see `04-qa-report.md`)
- [x] Owned paths Prettier-clean
- [x] Proposed commits are Conventional Commits compliant

## Tests

- [x] Acceptance criteria covered (unit + Maestro evidence)
- [x] Edge cases: retry key, 401, insufficient funds, SSE parse
- [x] Money tests present

## Security

401 logout path reviewed; E2E uses documented seed users only; no secrets in tree.

## Performance

SSE reconnect backoff avoids tight spin; E2E runs sequential to avoid shared-session races (acceptable for MVP).

## Findings

| ID  | Severity | File | Comment              | Status |
| --- | -------- | ---- | -------------------- | ------ |
| —   | —        | —    | No blocking findings | —      |

## Proposed Commit Messages

```text
fix(money): guard formatToParts for Hermes Intl

feat(mobile): harden transfer idempotency, 401 logout, and SSE

test(mobile): cover idempotency, 401 logout, SSE, and confirmation UX

test(mobile): add Maestro E2E suite and curated flow screenshots

docs: document Maestro E2E, ADRs, and mobile hardening work item
```

## Review Sign-Off

> **Agent:** Code Reviewer Agent
>
> **Date:** 2026-07-08
