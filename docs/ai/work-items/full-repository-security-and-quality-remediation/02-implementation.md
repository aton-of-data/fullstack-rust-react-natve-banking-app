# Full Repository Security and Quality Remediation — Implementation

## Status

COMPLETE

## Changes

### Backend (`apps/api`)

| Area                                 | Files                                                                |
| ------------------------------------ | -------------------------------------------------------------------- |
| CORS + Swagger gating + metrics auth | `adapters-http/src/lib.rs`, `state.rs`, `middleware/metrics_auth.rs` |
| Rate-limit proxy trust               | `middleware/rate_limit.rs`, `infrastructure/src/config.rs`           |
| Metrics init safety                  | `metrics.rs`                                                         |
| SSE serialize safety                 | `handlers/feed.rs`                                                   |
| Declined audit persistence           | `executor/postgres_transfer_executor.rs`                             |
| Feed publish logging                 | `application/src/transfer.rs`                                        |
| Test isolation                       | `testkit/src/lib.rs`                                                 |
| Cargo deny policy                    | `deny.toml`                                                          |
| Nextest serial profile               | `.config/nextest.toml`                                               |

### Mobile

| Area                          | Files                        |
| ----------------------------- | ---------------------------- |
| JSDoc lint fix                | `transferSubmissionSlice.ts` |
| Coverage thresholds (interim) | `vitest.config.ts`           |

### CI / ops

| Area                                            | Files                            |
| ----------------------------------------------- | -------------------------------- |
| `TEST_DATABASE_URL`, nextest `-j 1`, cargo deny | `.github/workflows/ci.yml`       |
| Coverage gate enforcement                       | `.github/workflows/quality.yml`  |
| Audit enforcement                               | `.github/workflows/security.yml` |
| Env documentation                               | `.env.example`                   |
| Makefile serial tests                           | `Makefile`                       |

## Verification commands (local)

```bash
# Frontend
pnpm lint          # exit 0
pnpm format:check  # exit 0
pnpm typecheck     # exit 0
pnpm test          # exit 0

# Backend (requires Postgres; Docker or TEST_DATABASE_URL)
export TEST_DATABASE_URL=postgres://$USER@localhost:5432/ficus_test
createdb ficus_test  # once
cd apps/api && cargo test -j 1 --workspace -- --test-threads=1  # exit 0
cargo fmt --check   # exit 0
cargo clippy --workspace --all-targets --all-features -- -D warnings  # exit 0
```
