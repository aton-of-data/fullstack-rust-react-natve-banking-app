# Full Repository Security and Quality Remediation — Architecture

## Status

APPROVED — implementation authorized.

## Scope

Repository-wide audit and remediation across backend security, financial auditability, test isolation, Rust safety, mobile lint, CI gates, and observability defaults.

## Remediation matrix

| ID         | Layer       | Finding                                               | Severity | Evidence                                | Fix                                                                    | Test                             | Status |
| ---------- | ----------- | ----------------------------------------------------- | -------- | --------------------------------------- | ---------------------------------------------------------------------- | -------------------------------- | ------ |
| SEC-001    | API         | `/metrics` exposed without auth in production         | HIGH     | `adapters-http/src/lib.rs` public route | Bearer `METRICS_AUTH_TOKEN` middleware; disabled when unset in non-dev | HTTP contract (env=test open)    | FIXED  |
| SEC-002    | API         | `CORS_ORIGINS` ignored; `Any` origin allowed          | HIGH     | `config.rs` vs `cors_layer()`           | Wire `AppState.cors_origins` into `AllowOrigin::list`                  | Manual + integration             | FIXED  |
| SEC-003    | API         | Swagger UI public in all environments                 | MEDIUM   | `lib.rs` unconditional merge            | Gate `/api-docs` to development/test only                              | —                                | FIXED  |
| SEC-004    | API         | Login rate limit bypass via spoofed `X-Forwarded-For` | MEDIUM   | `rate_limit.rs`                         | `TRUST_PROXY_HEADERS` default false; use `ConnectInfo`                 | Existing 429 test                | FIXED  |
| FIN-001    | Persistence | Declined-transfer audit rolled back with txn          | LOW      | `postgres_transfer_executor.rs`         | Persist `TransferDeclined` after rollback                              | `transfer_partial_state_test.rs` | FIXED  |
| FIN-002    | Application | Feed publish failures silently dropped                | LOW      | `transfer.rs` `let _ = publish`         | Structured `warn!` on failure                                          | —                                | FIXED  |
| RUST-001   | HTTP        | Metrics init `expect` panics process                  | MEDIUM   | `metrics.rs`                            | `init_metrics() -> Result`                                             | Clippy clean                     | FIXED  |
| RUST-002   | HTTP        | SSE `feed_event` `expect` on serialize                | MEDIUM   | `feed.rs`                               | Return `Option`, log and skip                                          | `feed_http_api_test.rs`          | FIXED  |
| TEST-001   | Testkit     | Shared `TEST_DATABASE_URL` races (deadlock)           | HIGH     | Local `cargo test` failures             | Advisory lock spans truncate+seed; CI `-j 1` + nextest profile         | All integration tests            | FIXED  |
| MOBILE-001 | Mobile      | JSDoc `@param` lint failure                           | LOW      | `transferSubmissionSlice.ts`            | Add missing `@param state`                                             | `pnpm lint`                      | FIXED  |
| CI-001     | CI          | `cargo deny` absent                                   | MEDIUM   | `context.md` vs workflows               | `deny.toml` + CI step                                                  | `cargo deny check`               | FIXED  |
| CI-002     | CI          | Coverage/audit `continue-on-error: true`              | MEDIUM   | `quality.yml`, `security.yml`           | Remove continue-on-error on key gates                                  | CI                               | FIXED  |
| DOC-001    | Docs        | `context.md` path/stack drift                         | MEDIUM   | `backend/` vs `apps/api/`               | Deferred — document in limitations                                     | —                                | OPEN   |

## Decisions

- **Metrics**: Production/staging require `METRICS_AUTH_TOKEN` or return 404 on `/metrics`. Development/test remain open for local scraping.
- **CORS**: Explicit origin list from `CORS_ORIGINS`; empty list falls back to `http://localhost` only.
- **Test isolation**: `pg_advisory_lock` wraps truncate+seed; CI uses `TEST_DATABASE_URL` with `cargo nextest -j 1 --profile ci`.

## Acceptance criteria

- [x] Security hardening for CORS, metrics, Swagger, proxy headers
- [x] Declined transfer audit survives rollback
- [x] Integration tests pass with `TEST_DATABASE_URL` (serial CI profile)
- [x] Frontend lint/typecheck/test pass
- [ ] 90% mobile coverage (baseline thresholds at 60% — see limitations)
- [ ] k6 concurrency/idempotency in default PR CI (scheduled only)

## ADRs

No new ADRs required; changes align with ADR-003, ADR-004, ADR-009.
