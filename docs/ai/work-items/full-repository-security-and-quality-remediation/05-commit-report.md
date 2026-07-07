# Full Repository Security and Quality Remediation — Commit Report

## Status

NOT COMMITTED — awaiting explicit user authorization per governance rules.

## Proposed commit message

```
fix(security): harden API surface and stabilize integration test isolation

Wire CORS_ORIGINS, protect metrics in non-dev, gate Swagger, and fix
proxy-aware rate limiting. Persist declined-transfer audits after rollback,
remove production expect paths, and serialize shared-DB integration tests.
```

## commitlint

Not run — no commit created.

## Files changed (summary)

- `apps/api/crates/adapters-http/**` — security, CORS, metrics, SSE
- `apps/api/crates/adapters-persistence/**` — declined audit
- `apps/api/crates/application/**` — feed publish logging
- `apps/api/crates/infrastructure/**` — config
- `apps/api/crates/testkit/**` — isolation lock
- `apps/api/deny.toml`, `apps/api/.config/nextest.toml`
- `apps/mobile/**` — JSDoc, vitest thresholds
- `.github/workflows/**`, `Makefile`, `.env.example`
- `docs/ai/work-items/full-repository-security-and-quality-remediation/**`
