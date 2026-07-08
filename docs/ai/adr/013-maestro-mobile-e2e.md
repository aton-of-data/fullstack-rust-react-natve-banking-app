# ADR-013: Maestro for Expo Mobile E2E

## Status

Accepted

## Context

The mobile client needs device-level proof of login, logout, transfer happy path, and insufficient-funds UX. Unit and RTK Query tests cover reducers and API wiring but cannot exercise Expo Go / simulator gesture and navigation stacks.

Candidates:

| Option  | Pros                                                           | Cons                                                        |
| ------- | -------------------------------------------------------------- | ----------------------------------------------------------- |
| Detox   | Mature RN E2E                                                  | Heavy native build; awkward with Expo Go / managed workflow |
| Appium  | Cross-platform                                                 | High ops cost for a small monorepo interview app            |
| Maestro | YAML flows, fast setup, works against Expo Go `id:` / `testID` | Less deep RN instrumentation than Detox                     |

Governance requires tracked E2E evidence under the delivery pipeline without forcing a new mobile build toolchain for MVP.

## Decision

Adopt **Maestro** for mobile E2E:

- Flows live under `apps/mobile/e2e/flows/`
- Default `appId` for simulator Expo Go is `host.exp.Exponent` (override to `com.ficus.mobile` for development builds)
- React Native `testID`s are the primary selectors (`id:` in Maestro YAML)
- Runner script `apps/mobile/e2e/run-maestro.mjs` sets `JAVA_HOME` heuristics, runs flows **sequentially** (one Maestro process per YAML), writes `reports/latest.{json,md}`, and exits `2` with `SKIP` when Maestro or Java is missing
- Suite flows are `flows/01`–`06`; optional `flows/docs-capture.yaml` is excluded from the runner (`docs-` prefix)
- Root scripts: `mobile:e2e`, `mobile:e2e:ios`, `mobile:e2e:android`, `mobile:e2e:record`
- Tab selection in Expo Go uses accessibility labels (`Send tab` / `Home tab`)

## Consequences

- Local and CI can skip gracefully when the simulator toolchain is absent
- Operators must open the Expo project in Expo Go (or use a linked URL) before Expo Go flows
- Seed users (`alice` / `bob` / `charlie`) must be present; balances can drift after transfer flows
- Sequential execution avoids shared Expo Go / SecureStore session races across flows
- ADR-aligned alternative to Detox; no new Detox/Appium dependency in package.json

## Alternatives considered

- **Detox + Expo prebuild** — rejected for MVP cost vs Maestro YAML
- **Manual QA only** — rejected; lacks repeatable artifacts for QA reports
- **Persist Maestro in package.json as a dependency** — rejected; CLI is installed system-wide like k6
