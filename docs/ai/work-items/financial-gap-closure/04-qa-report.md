# Financial Gap Closure — QA Report

## Status: PASS (local verification)

## Commands and exit codes

| Command                                                                 | Exit |
| ----------------------------------------------------------------------- | ---- |
| `TEST_DATABASE_URL=... cargo test -p ficus-testkit -- --test-threads=1` | 0    |
| `TEST_DATABASE_URL=... cargo test --workspace -- --test-threads=1`      | 0    |
| `cargo fmt --check`                                                     | 0    |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings`  | 0    |
| `pnpm --filter @ficus/mobile test`                                      | 0    |
| `pnpm --filter @ficus/mobile typecheck`                                 | 0    |

## Testkit summary (51 tests)

- Auth HTTP: 8/8 pass
- Feed HTTP: 4/4 pass
- Transfer HTTP: 14/14 pass
- Idempotency service: 7/7 pass
- Concurrency service + extended: 4/4 pass
- Partial state: 2/2 pass
- Money conservation: 3/3 pass
- Ledger reconciliation: 3/3 pass
- DB immutability: 4/4 pass
- Persistence retry unit: 2/2 pass

## Mobile summary (31 tests)

- 9 files, all pass

## Notes

- Integration tests require Postgres (`TEST_DATABASE_URL` or Docker testcontainers)
- Run testkit with `--test-threads=1` when sharing one database
- k6/90% coverage gates require local k6 / cargo-llvm-cov installation
