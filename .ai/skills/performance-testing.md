# Skill: Performance Testing

## When to Use

Validating API throughput/latency under load, before releases, or when changing transfer/feed/auth hot paths.

## Tooling

- **k6** — load generator
- **Scripts:** `scripts/performance/login-transfer-feed.js`
- **Runner:** `scripts/performance/run-k6.mjs`

## Prerequisites

```bash
# macOS
brew install k6

# Linux — see https://k6.io/docs/get-started/installation/
```

API must be running with seeded users:

```bash
make up && make db-migrate && make db-seed && make api-dev
```

## Running Tests

```bash
# Default: 10 VUs, 1 minute
pnpm test:performance

# Custom
node scripts/performance/run-k6.mjs --vus 20 --duration 2m --base-url http://localhost:8080
```

Environment variables: `API_BASE_URL`, `K6_VUS`, `K6_DURATION`

## Scenario Coverage

The k6 script exercises:

1. `POST /v1/auth/login`
2. `POST /v1/transfers` with unique `Idempotency-Key`
3. `GET /v1/accounts/me/balance`
4. `GET /v1/feed`

## Thresholds (default in script)

| Metric                  | Threshold  |
| ----------------------- | ---------- |
| `http_req_failed`       | < 5%       |
| `http_req_duration` p95 | < 2000ms   |
| `checks`                | > 95% pass |

Adjust in `login-transfer-feed.js` `options.thresholds` for stricter gates.

## Interpreting Results

| Symptom               | Likely cause                            |
| --------------------- | --------------------------------------- |
| High 422 on transfers | Insufficient seeded balances under load |
| p95 latency spike     | Row lock contention on hot accounts     |
| Login failures        | Rate limit (`LOGIN_RATE_LIMIT_PER_MIN`) |
| Connection errors     | API not ready; check `/health/ready`    |

## CI Integration

`.github/workflows/quality.yml` runs a short k6 smoke on `workflow_dispatch` only.

Local wrapper skips gracefully when k6 missing (`--ci` flag).

## Evidence Template

```markdown
### Performance: login-transfer-feed

- **Command:** `node scripts/performance/run-k6.mjs --vus 10 --duration 1m`
- **API_BASE_URL:** http://localhost:8080
- **http_req_duration p95:** XXX ms
- **http_req_failed:** X.XX%
- **checks:** XX.X% pass
```

## Related

- `docs/operations/runbook.md`
- ADR-008 (concurrency under load)
