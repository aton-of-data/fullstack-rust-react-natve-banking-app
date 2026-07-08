# Implementation Handoff: mobile-e2e-visual-quality-hardening

## Summary

Hardened the mobile transfer and session lifecycle for safer retries and clearer UX, added automation `testID`s, and introduced a Maestro E2E suite with runner/scripts and ADRs. Idempotency keys no longer rotate on Back→Review; unauthenticated API responses force logout via RTK base query; SSE reconnects with line buffering and `Last-Event-ID`; money formatting stays integer-safe for Hermes; transfer confirmation exposes a dedicated success state.

## Files Changed

| File                                                                      | Change                                                                                      |
| ------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| `apps/mobile/src/features/transfer-submission/transferSubmissionSlice.ts` | `beginTransferAttempt` no-op when key exists; locked amount/description; cancel/reset paths |
| `apps/mobile/src/features/transfer-submission/selectors.ts`               | Active-attempt / locked payload selectors                                                   |
| `apps/mobile/src/features/transfer-submission/*.test.ts`                  | Key preserve, retry, flow integration coverage                                              |
| `apps/mobile/src/services/baseApi.ts`                                     | `baseQueryWithAuth`: 401 → `clearCredentials` + form/submission reset + `resetApiState`     |
| `apps/mobile/src/services/baseApi.test.ts`                                | 401 logout coverage                                                                         |
| `apps/mobile/src/services/sse.ts`                                         | Incomplete-line buffer; reconnect; `Last-Event-ID`; dedupe                                  |
| `apps/mobile/src/services/sse.test.ts`                                    | Reconnect / header tests                                                                    |
| `apps/mobile/src/shared/lib/money.ts` (+ package money)                   | Integer-safe validation / Hermes-friendly formatting                                        |
| `apps/mobile/src/shared/ui/organisms/TransferConfirmation.tsx`            | Success UI (`Transfer sent`), retry/conflict guidance, lock-aware actions, `testID`s        |
| `apps/mobile/src/shared/ui/organisms/TransferAmountForm.tsx`              | Lock edits while attempt active; cancel attempt; `testID`s                                  |
| `apps/mobile/src/shared/ui/organisms/RecipientSearch.tsx`                 | Self-filter; search `testID`s                                                               |
| `apps/mobile/src/shared/ui/organisms/LoginForm.tsx`                       | Login screen `testID`s; stable error copy                                                   |
| `apps/mobile/src/shared/ui/molecules/BalanceDisplay.tsx`                  | `home-balance` / amount `testID`s                                                           |
| `apps/mobile/src/shared/ui/molecules/FeedItemCard.tsx`                    | a11y labels + feed item `testID`s                                                           |
| `apps/mobile/src/shared/ui/organisms/FeedList.tsx`                        | `feed-list` `testID`                                                                        |
| `apps/mobile/src/shared/ui/atoms/Button.tsx` / `ErrorBanner.tsx`          | `testID` passthrough                                                                        |
| `apps/mobile/src/pages/HomePage.tsx`                                      | Logout clears submission/form/API cache; `home-logout`                                      |
| `apps/mobile/src/app/navigation.tsx`                                      | Tab a11y labels (`Home tab` / `Send tab`) for Maestro                                       |
| `apps/mobile/e2e/**`                                                      | Maestro flows, README, `run-maestro.mjs`, curated screenshots                               |
| `package.json`                                                            | `mobile:e2e`, `:ios`, `:android`, `:record` scripts                                         |
| `docs/ai/adr/ADR-012-mobile-transfer-idempotency-lifecycle.md`            | Consequences amend (key preserve, begin no-op, 401 via baseQuery)                           |
| `docs/ai/adr/013-maestro-mobile-e2e.md`                                   | Accepted: Maestro for Expo E2E                                                              |
| `docs/ai/adr/README.md`                                                   | Index ADR-012 and ADR-013                                                                   |
| `docs/operations/runbook.md` / `README.md`                                | Mobile E2E operator docs                                                                    |

## Design Decisions Applied

- ADR-012 amended: preserve idempotency key across Back→Review; `beginTransferAttempt` only allocates when `idempotencyKey === null`.
- ADR-013: Maestro YAML + Expo Go `appId` (`host.exp.Exponent`) over Detox for Expo MVP.
- Global 401 handling in RTK `baseQueryWithAuth` (not only transfer error mappers).
- SSE reconnect uses exponential backoff while the cache subscription is alive; incomplete SSE lines buffered across XHR progress chunks.
- E2E selectors prefer Maestro `id:` ↔ React Native `testID`, with text / a11y assertions as backups.
- Runner mirrors k6 pattern: SKIP + exit `2` when Maestro/Java missing; write `reports/latest.{json,md}`; sequential per flow.

## Deviations from Architecture

| Deviation                              | Rationale                                                                                              | Architecture notified |
| -------------------------------------- | ------------------------------------------------------------------------------------------------------ | --------------------- |
| No `tabBarTestID` on BottomTab options | Not in typed `BottomTabNavigationOptions`; Expo Go already exposes `Send tab` / `Home tab` a11y labels | Documented in 02/03   |

## Tests Added or Updated

| Test                          | File                               | Covers                            |
| ----------------------------- | ---------------------------------- | --------------------------------- |
| Key preserve / no-op begin    | `transferSubmissionSlice.test.ts`  | AC-8, AC-9                        |
| Transfer flow integration     | `transferFlow.integration.test.ts` | Retry same key                    |
| 401 base query logout         | `baseApi.test.ts`                  | AC-11                             |
| SSE Last-Event-ID / stream    | `sse.test.ts`                      | AC-12                             |
| Confirmation success / errors | `TransferConfirmation.test.tsx`    | AC-7, AC-10                       |
| Money validation              | `money.test.ts`                    | AC-6                              |
| Maestro flows (device)        | `apps/mobile/e2e/flows/*.yaml`     | AC-1–3, AC-7, AC-10, AC-13, AC-14 |

## Commands Executed

| Command                                 | Exit Code | Summary                               |
| --------------------------------------- | --------- | ------------------------------------- |
| `pnpm --filter @ficus/mobile lint`      | 0         | ESLint pass                           |
| `pnpm --filter @ficus/mobile typecheck` | 0         | After removing invalid `tabBarTestID` |
| `pnpm --filter @ficus/mobile test`      | 0         | 79 tests passed                       |
| `pnpm --filter @ficus/money test`       | 0         | 4 tests passed                        |
| Full suite E2E                          | —         | Recorded in `04-qa-report.md`         |

## Known Limitations

- Maestro requires booted simulator, Java 17, Maestro CLI, running API + seed, and Expo project already open in Expo Go (unless using a development build / `openLink`).
- Transfer happy-path E2E spends alice balance; re-seed if flows fail on funds.
- Android Emulator E2E may SKIP locally when no emulator is available (document in QA).
- Idempotency keys remain in-memory only (SecureStore persistence still rejected per ADR-012).
- Tab bars are selected via accessibility labels (`Send tab` / `Home tab`).

## QA Entry Criteria

- [x] All acceptance criteria from `01-architecture.md` testable (unit + Maestro when env present)
- [x] Lint/typecheck/test commands documented above with exit codes
- [x] No known blocking defects in shipped hardening paths
- [x] Security-sensitive paths flagged: auth 401 logout, transfer idempotency, test credentials only for seed users

## Ready for QA

> **Implementation complete:** Yes
>
> **Date:** 2026-07-08
>
> **Notes:** Documenter filled `03-documentation.md`; curated screenshots under `e2e/screenshots/`; QA records full suite exit codes in `04-qa-report.md`.
