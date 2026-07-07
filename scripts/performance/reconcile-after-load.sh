#!/usr/bin/env bash
# Post-load reconciliation via API health and documented SQL guidance.
set -euo pipefail

API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"

echo "== Ficus post-load reconciliation =="
echo "API: ${API_BASE_URL}"

ready_code="$(curl -s -o /dev/null -w '%{http_code}' "${API_BASE_URL}/health/ready")"
echo "health/ready HTTP ${ready_code}"

metrics="$(curl -s "${API_BASE_URL}/metrics" || true)"
if [[ -n "${metrics}" ]]; then
  echo "--- transfer metrics snapshot ---"
  echo "${metrics}" | rg 'ficus_transfers_total|ficus_transfer_idempotency|ficus_http_requests_total' || true
fi

cat <<'EOF'

Manual DB reconciliation (requires psql access):
  SELECT ab.account_id,
         ab.balance_minor AS projected,
         COALESCE(SUM(CASE WHEN le.direction = 'credit' THEN le.amount_minor ELSE -le.amount_minor END), 0) AS ledger_sum
  FROM account_balances ab
  LEFT JOIN ledger_entries le ON le.account_id = ab.account_id
  GROUP BY ab.account_id, ab.balance_minor
  HAVING ab.balance_minor <> COALESCE(SUM(CASE WHEN le.direction = 'credit' THEN le.amount_minor ELSE -le.amount_minor END), 0);

Integration test helper:
  cargo nextest run -p ficus-testkit ledger_balance_reconciliation_test

EOF
