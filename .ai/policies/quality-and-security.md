# Quality and Security Policy

Canonical reference: `context.md`

## Verification Minimums

Every change includes proportionate evidence:

| Category | When required |
|----------|---------------|
| Type checking | Always (TS, Rust) |
| **ESLint (`npm run lint`)** | Always (frontend) — includes JSDoc rules |
| **Prettier (`npm run format:check`)** | Always (frontend) |
| Formatting (Rust) | `cargo fmt --check` |
| Linting (Rust) | `cargo clippy -D warnings` |
| **JSDoc / rustdoc** | All new/changed exports |
| Unit tests | Always for logic changes |
| Integration tests | API, persistence, feature workflows |
| Contract tests | Public API shape changes |
| Regression tests | Bug fixes |
| Accessibility | User-facing mobile UI |
| Security review | Auth, secrets, validation, logging, dependencies |
| Performance review | Lists, selectors, API caching, DB access, N+1 risk |

## QA Agent Obligations

- Validate every acceptance criterion independently
- Run commands; capture exit codes and relevant output
- Test failure paths, invalid input, auth boundaries, race conditions
- Mobile: loading/error/empty states, accessibility basics
- Backend: migration behavior, error contracts, idempotency

## Outcomes

QA reports only: `PASS`, `FAIL`, `BLOCKED`.

Never mark PASS without executed checks.

## Security Review Triggers

Mandatory security review when changing:

- Authentication or authorization
- Password or token handling
- Money transfer logic
- Input validation boundaries
- Logging (PII/secrets)
- Dependencies (especially crypto, HTTP, ORM)
- CORS, rate limiting, headers

Use skill: `.ai/skills/security-review.md`

## Prohibited Bypasses

```text
TODO: test later
temporary direct fetch
temporary useState
temporary Context provider
temporary ORM model in domain
temporary skipped migration
temporary lint disable without documented justification
```

## CI Enforcement (Target)

When CI is added (GitHub Actions), enforce:

**Frontend (when scaffold exists):**

- `npm run typecheck`
- `npm run lint` (ESLint + eslint-plugin-jsdoc)
- `npm run format:check` (Prettier)
- `npm test`

**Git:**

- commitlint on commit-msg hook
- Conventional Commits required

**Backend:**

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo deny check` / `cargo audit` when configured

**Governance:**

- PR template referencing work item ID
- Block merge without QA PASS and Review APPROVED artifacts (manual until automated)

## Evidence Format

Reports must include:

```text
Command: <exact command>
Exit code: <0 or non-zero>
Summary: <what was verified>
```

Never claim "tests pass" without command output or CI link.
