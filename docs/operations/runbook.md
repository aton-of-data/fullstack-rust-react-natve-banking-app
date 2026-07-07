# Operations Runbook

Procedures for running, debugging, and recovering the Ficus platform.

## Service Matrix

| Service         | Start             | Stop        | Logs                         |
| --------------- | ----------------- | ----------- | ---------------------------- |
| PostgreSQL      | `make up`         | `make down` | `docker logs ficus-postgres` |
| API (local)     | `make api-dev`    | Ctrl+C      | stdout                       |
| API (container) | `make up-full`    | `make down` | `docker logs ficus-api`      |
| Observability   | `make up-obs`     | `make down` | per-container `docker logs`  |
| Mobile          | `pnpm mobile:dev` | Ctrl+C      | Expo CLI                     |

## Common Procedures

### Fresh local environment

```bash
make down
docker volume rm ficusplatforms_postgres_data  # destructive
make up
make db-migrate
make db-seed
make api-dev
```

### Apply migrations

```bash
make db-migrate
```

Uses `DATABASE_MIGRATION_URL` (migrator role with DDL privileges). App runtime uses `DATABASE_URL` (app role, DML only).

### Re-seed users (idempotent)

```bash
make db-seed
```

Skips users that already exist. Does not reset balances on existing users.

### Verify API health

```bash
curl -s http://localhost:8080/health/live | jq .
curl -s http://localhost:8080/health/ready | jq .
```

### Login smoke test

```bash
TOKEN=$(curl -s -X POST http://localhost:8080/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"password123"}' | jq -r .access_token)

curl -s http://localhost:8080/v1/accounts/me/balance \
  -H "Authorization: Bearer $TOKEN" | jq .
```

## Incident Response

### API returns 503 on `/health/ready`

1. Check PostgreSQL: `docker ps`, `pg_isready -U ficus -d ficus`
2. Verify `DATABASE_URL` credentials match `infra/postgres/init` roles
3. Check connection pool exhaustion in API logs
4. Restart API after DB recovery

### Transfers failing with 500

1. Search logs for `retrying transfer after serialization failure`
2. If persistent, check for migration drift: `make db-migrate`
3. Run integration tests: `make api-test`
4. Check `balance_minor >= 0` constraint violations — indicates bug or data corruption

### Negative balance suspected

1. **Stop API** to prevent further damage
2. Query balances and ledger sums (see reconciliation query below)
3. Restore from backup if corruption confirmed
4. File work item; root-cause via `ledger_balance_reconciliation_test`

```sql
-- Per-account reconciliation
SELECT a.id,
       ab.balance_minor AS materialized,
       COALESCE(SUM(CASE WHEN le.direction = 'credit' THEN le.amount_minor
                         ELSE -le.amount_minor END), 0) AS ledger_sum
FROM accounts a
JOIN account_balances ab ON ab.account_id = a.id
LEFT JOIN ledger_entries le ON le.account_id = a.id
GROUP BY a.id, ab.balance_minor
HAVING ab.balance_minor != COALESCE(SUM(CASE WHEN le.direction = 'credit' THEN le.amount_minor
                                              ELSE -le.amount_minor END), 0);
```

### SSE feed not updating

1. Confirm transfer completes (REST `GET /v1/feed`)
2. Check `sse_connections_active` metric
3. Verify mobile token not expired
4. Test stream: `curl -N -H "Authorization: Bearer $TOKEN" http://localhost:8080/v1/feed/stream`

### JWT / auth issues

1. Confirm `JWT_SECRET` ≥ 32 characters and consistent across instances
2. Check `JWT_EXPIRY_SECS`
3. Verify clock skew on client devices

## Performance Testing

```bash
# Requires k6: https://k6.io/docs/get-started/installation/
pnpm test:performance
```

See `.ai/skills/performance-testing.md`.

## Security Scanning

```bash
pnpm security:scan
```

Runs gitleaks and trivy when installed; skips gracefully otherwise.

## Escalation

| Severity        | Action                                                   |
| --------------- | -------------------------------------------------------- |
| Data integrity  | Stop writes, preserve logs, Architecture Agent work item |
| Security breach | Rotate `JWT_SECRET`, force re-login, review audit_events |
| CI blocked      | See `.github/workflows/` and QA artifacts                |

## Related

- [Observability](./observability.md)
- [README](../../README.md)
