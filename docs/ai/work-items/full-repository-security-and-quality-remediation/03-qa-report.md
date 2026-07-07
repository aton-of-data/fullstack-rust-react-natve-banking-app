# Full Repository Security and Quality Remediation — QA Report (updated)

## Status

PASS

## Frontend

| Command | Exit code | Notes |
|---------|-----------|-------|
| `pnpm --filter @ficus/mobile test` | 0 | 66 tests (unit + component) |
| `pnpm --filter @ficus/mobile test:coverage` | 0 | ≥90% lines/statements; ≥86% functions |
| `pnpm lint` | 0 | (run in CI) |

### Mobile coverage (evidence)

- Lines/statements: **~98%**
- Functions: **~87%**
- Component tests: pages (`LoginPage`, `HomePage`, `TransferPage`) + organisms (`LoginForm`, `BalanceCard`, `FeedList`, `TransferConfirmation`, `RecipientSearch`, `TransferAmountForm`)

## Backend

| Command | Exit code | Notes |
|---------|-----------|-------|
| `cargo test -j 1 -- --test-threads=1` | 0 | With `TEST_DATABASE_URL` |
| `metrics_http_api_test` | 0 | 3 tests — CI auth contracts |

## CI updates

- `quality.yml`: API `llvm-cov` **90%** lines/functions; k6 `--all` on PR
- `ci.yml`: `TEST_DATABASE_URL`, `cargo deny`, nextest serial profile

## Remaining limitations

| Item | Status |
|------|--------|
| Local `cargo llvm-cov` 90% | CI-only unless `cargo-llvm-cov` installed |
| k6 post-load reconciliation | Scripts exist; full reconciliation in perf job optional |
| Docker testcontainers locally | Use `make up` or `TEST_DATABASE_URL` |
