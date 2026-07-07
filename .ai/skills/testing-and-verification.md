# Skill: Testing and Verification

## Principles

- Run commands; record exit codes and summaries
- Never claim PASS without evidence
- Proportionate coverage — money logic demands thorough tests

## Verification

Run and record:

```bash
npm run typecheck
npm run lint          # ESLint + JSDoc rules
npm run format:check  # Prettier
npm test
```

## Backend Commands

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test -p <crate> -- <test_name>
cargo nextest run        # if configured
```

## Evidence Template

```markdown
### Verification: <name>

- **Command:** `cargo test transfer_concurrent`
- **Exit code:** 0
- **Summary:** 100 concurrent transfers; balance never negative; conservation holds
```

## QA Checklist by Change Type

| Change         | Minimum tests                                 |
| -------------- | --------------------------------------------- |
| Transfer/money | Concurrent + idempotency + conservation       |
| Auth           | Invalid creds, missing token, expired session |
| Feed           | New item appears without refresh              |
| API contract   | Contract test or OpenAPI snapshot             |
| Migration      | Up/down or fresh install                      |

## Failure Reporting

```markdown
### Failure: <AC-id>

- **Expected:** ...
- **Actual:** ...
- **Reproduce:** `command` or steps
```

## Regression

Every bug fix includes a test that fails without the fix.
