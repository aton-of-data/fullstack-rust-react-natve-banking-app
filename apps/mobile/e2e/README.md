# Mobile Maestro E2E

Device-level end-to-end flows for the Ficus Expo app (display name **Ficus**).

Maestro prefers `id:` selectors that map to React Native `testID`s. Tab navigation uses accessibility labels (`Send tab` / `Home tab`) because Expo Go may not expose tab `testID`s reliably.

## Prerequisites

| Requirement                  | Notes                                                                                                                                 |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| **Java 17**                  | Maestro needs a JDK. On macOS Homebrew: `brew install openjdk@17`. Set `JAVA_HOME` if unset (the runner script tries Homebrew paths). |
| **Maestro CLI**              | [Install Maestro](https://maestro.mobile.dev/getting-started/installing-maestro).                                                     |
| **iOS Simulator or Android** | Booted before running.                                                                                                                |
| **API + DB**                 | API on `http://localhost:8080`. `make db-migrate` + `make db-seed` (`alice` / `bob` / `charlie`, `password123`).                      |
| **Expo app open**            | `pnpm mobile:dev`, then open the project in **Expo Go** (or a development build) on the simulator.                                    |

### JAVA_HOME (Homebrew)

```bash
export JAVA_HOME="$(/usr/libexec/java_home -v 17 2>/dev/null || true)"
export JAVA_HOME="${JAVA_HOME:-/opt/homebrew/opt/openjdk@17/libexec/openjdk.jdk/Contents/Home}"
```

`node apps/mobile/e2e/run-maestro.mjs` applies a similar heuristic when `JAVA_HOME` is unset.

## Expo Go vs development build

| Target                | `appId`             |
| --------------------- | ------------------- |
| iOS Expo Go (default) | `host.exp.Exponent` |
| Android Expo Go       | `host.exp.exponent` |
| Dev / release build   | `com.ficus.mobile`  |

```bash
MAESTRO_APP_ID=com.ficus.mobile pnpm mobile:e2e
```

**Expo Go:** Maestro drives the Expo Go shell. Load Ficus in Expo Go before flows run. Optionally:

```yaml
- openLink: exp://127.0.0.1:8081
```

## Seed users

| Username | Password    |
| -------- | ----------- |
| alice    | password123 |
| bob      | password123 |
| charlie  | password123 |

Re-seed if balances drift: `make db-seed` (see [operations runbook](../../../../docs/operations/runbook.md)).

## Layout

| Path                          | Purpose                                            |
| ----------------------------- | -------------------------------------------------- |
| `config.yaml`                 | Shared Maestro config (`appId`)                    |
| `helpers/reset-to-login.yaml` | Escape transfer wizard + logout when needed        |
| `flows/01`–`06`               | Suite flows (runner executes these sequentially)   |
| `flows/docs-capture.yaml`     | Optional curated screenshots (excluded from suite) |
| `run-maestro.mjs`             | Sequential runner + `reports/latest.{json,md}`     |
| `screenshots/`                | Curated flow docs PNGs (`01`–`06`) + `.gitkeep`    |
| `reports/`                    | Runner output + docs-capture evidence              |

## Flows

| File                                | Coverage                                            |
| ----------------------------------- | --------------------------------------------------- |
| `flows/01-launch-login-screen.yaml` | Login title, username, password, Sign In            |
| `flows/02-invalid-login.yaml`       | Wrong password → Invalid username or password       |
| `flows/03-successful-login.yaml`    | alice → Available Balance / `home-balance`          |
| `flows/04-logout.yaml`              | Login then logout → login screen                    |
| `flows/05-transfer-happy-path.yaml` | alice → Send → bob → 1.00 → confirm → Transfer sent |
| `flows/06-insufficient-funds.yaml`  | charlie → large amount → Insufficient funds         |
| `flows/docs-capture.yaml`           | Screenshot pass only (not in `pnpm mobile:e2e`)     |

## Commands

From the repo root:

```bash
pnpm mobile:e2e              # flows 01–06 sequentially
pnpm mobile:e2e:ios          # --platform=ios
pnpm mobile:e2e:android      # --platform=android
pnpm mobile:e2e:record       # --record (extra Maestro output under screenshots/)
```

```bash
node apps/mobile/e2e/run-maestro.mjs --platform=ios
maestro test apps/mobile/e2e/flows/docs-capture.yaml --platform ios  # docs PNGs
```

**Sequential only:** the runner starts one Maestro process per YAML so shared Expo Go auth cannot collide across flows.

## Documentation screenshots

Curated assets (commit these):

| File                                    | Screen                    |
| --------------------------------------- | ------------------------- |
| `screenshots/01-login-screen.png`       | Login                     |
| `screenshots/02-home-balance-feed.png`  | Home balance + feed       |
| `screenshots/03-recipient-search.png`   | Transfer recipient search |
| `screenshots/04-transfer-confirm.png`   | Confirm transfer          |
| `screenshots/05-transfer-success.png`   | Transfer sent             |
| `screenshots/06-insufficient-funds.png` | Insufficient funds error  |

Evidence: `reports/docs-capture.md`.

## Reports

| Artifact              | Meaning                                       |
| --------------------- | --------------------------------------------- |
| `reports/latest.json` | Last suite run (generated; usually untracked) |
| `reports/latest.md`   | Human-readable suite summary                  |
| Exit `0`              | All suite flows passed                        |
| Exit `1`              | One or more flows failed                      |
| Exit `2`              | SKIP (Maestro or Java missing)                |

## Troubleshooting

- **Tabs:** Prefer `tapOn: "Send tab"` / `"Home tab"` (accessibility labels), not `"Send"`.
- **Decimal pad / hideKeyboard:** Often fails on iOS; dismiss by tapping `"Send Money"` or `"Find recipient"`.
- **Already logged in:** `helpers/reset-to-login.yaml` no-ops when `login-screen` is visible; otherwise Home → Logout (Back out of transfer first).
- **iOS Save Password alert:** Flows tap `"Agora Não"` / `"Not Now"` when present.
- **Insufficient funds assert:** Match `"Insufficient funds.*"` and `id: transfer-error`.
- **API / seeds:** Confirm `extra.apiUrl` points at a running API; re-seed after many happy-path transfers.

## Related

- [ADR-013 Maestro](../../../../docs/ai/adr/013-maestro-mobile-e2e.md)
- [Operations runbook — Mobile E2E](../../../../docs/operations/runbook.md#mobile-e2e-maestro)
- Work item: `docs/ai/work-items/mobile-e2e-visual-quality-hardening/`
