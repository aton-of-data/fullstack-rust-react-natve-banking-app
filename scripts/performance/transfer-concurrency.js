/**
 * k6 burst test: N concurrent transfers from one funded account.
 *
 * Mirrors `transfer_concurrency_test.rs` / `http_hundred_concurrent_transfers_*`
 * at the HTTP layer. Each virtual user fires exactly one transfer in parallel
 * with a unique idempotency key, then `teardown()` verifies money invariants.
 *
 * Tiers: 100, 500, 1000 (via `K6_CONCURRENCY` or `--concurrency-suite`).
 *
 * Prerequisites:
 *   - API running with seeds (`make db-seed && make api-dev`)
 *   - API restarted with apps/api/.env rate limits (TRANSFER_RATE_LIMIT_PER_MIN=10000)
 *
 * Environment:
 *   API_BASE_URL     — default http://localhost:8080
 *   K6_CONCURRENCY   — parallel transfers (default 100)
 */

import http from 'k6/http';
import { check } from 'k6';
import { Counter } from 'k6/metrics';
import { textSummary } from 'https://jslib.k6.io/k6-summary/0.1.0/index.js';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

const BASE_URL = __ENV.API_BASE_URL || 'http://localhost:8080';
const CONCURRENCY = Number(__ENV.K6_CONCURRENCY || 100);
const SENDER = { username: 'alice', password: 'password123' };
const RECIPIENT = { username: 'bob', password: 'password123' };

const transfersCompleted = new Counter('ficus_transfers_completed');
const transfersDeclined = new Counter('ficus_transfers_declined');
const transfersRateLimited = new Counter('ficus_transfers_rate_limited');
const transfersErrored = new Counter('ficus_transfers_errored');

export const options = {
  scenarios: {
    concurrent_burst: {
      executor: 'per-vu-iterations',
      vus: CONCURRENCY,
      iterations: 1,
      maxDuration: '2m',
    },
  },
  thresholds: {
    checks: ['rate>0.99'],
    ficus_transfers_rate_limited: ['count==0'],
    ficus_transfers_errored: ['count==0'],
  },
};

/**
 * @param {string} username
 * @param {string} password
 * @returns {string}
 */
function login(username, password) {
  const res = http.post(
    `${BASE_URL}/v1/auth/login`,
    JSON.stringify({ username, password }),
    { headers: { 'Content-Type': 'application/json' }, tags: { name: 'login' } },
  );
  if (res.status !== 200) {
    throw new Error(`login failed for ${username}: HTTP ${res.status}`);
  }
  return JSON.parse(res.body).access_token;
}

/**
 * @param {string} token
 * @returns {number}
 */
function fetchBalanceMinor(token) {
  const res = http.get(`${BASE_URL}/v1/accounts/me/balance`, {
    headers: { Authorization: `Bearer ${token}` },
    tags: { name: 'balance' },
  });
  if (res.status !== 200) {
    throw new Error(`balance fetch failed: HTTP ${res.status}`);
  }
  return Number(JSON.parse(res.body).balance_minor);
}

/**
 * Chooses a transfer amount so not every concurrent request can succeed.
 * @param {number} senderBalanceMinor
 * @param {number} concurrency
 * @returns {number}
 */
function chooseTransferAmountMinor(senderBalanceMinor, concurrency) {
  return Math.floor(senderBalanceMinor / concurrency) + 1;
}

export function setup() {
  const ready = http.get(`${BASE_URL}/health/ready`, { tags: { name: 'health' } });
  check(ready, { 'API ready': (r) => r.status === 200 });

  const senderToken = login(SENDER.username, SENDER.password);
  const recipientToken = login(RECIPIENT.username, RECIPIENT.password);

  const initialSenderBalance = fetchBalanceMinor(senderToken);
  const initialRecipientBalance = fetchBalanceMinor(recipientToken);
  const transferAmountMinor = chooseTransferAmountMinor(
    initialSenderBalance,
    CONCURRENCY,
  );
  const maxFundable = Math.floor(initialSenderBalance / transferAmountMinor);
  const expectedCompleted = Math.min(CONCURRENCY, maxFundable);
  const expectedDeclined = CONCURRENCY - expectedCompleted;

  return {
    runId: uuidv4(),
    senderToken,
    recipientToken,
    initialSenderBalance,
    initialRecipientBalance,
    transferAmountMinor,
    maxFundable,
    expectedCompleted,
    expectedDeclined,
    concurrency: CONCURRENCY,
  };
}

export default function (data) {
  const idempotencyKey = `${data.runId}-${String(__VU).padStart(4, '0')}`;
  const headers = {
    Authorization: `Bearer ${data.senderToken}`,
    'Content-Type': 'application/json',
    'Idempotency-Key': idempotencyKey,
  };

  const res = http.post(
    `${BASE_URL}/v1/transfers`,
    JSON.stringify({
      recipient_username: RECIPIENT.username,
      amount_minor: String(data.transferAmountMinor),
      currency: 'USD',
      description: `k6 concurrency burst (${data.concurrency})`,
    }),
    { headers, tags: { name: 'transfer_burst' } },
  );

  if (res.status === 200) {
    const body = JSON.parse(res.body);
    if (body.status === 'COMPLETED') {
      transfersCompleted.add(1);
    } else if (body.status === 'DECLINED') {
      transfersDeclined.add(1);
    } else {
      transfersErrored.add(1);
    }
    check(res, {
      'transfer resolved (200)': () => true,
      'completed or declined status': () =>
        body.status === 'COMPLETED' || body.status === 'DECLINED',
    });
    return;
  }

  if (res.status === 422) {
    transfersDeclined.add(1);
    check(res, { 'transfer declined (422)': () => true });
    return;
  }

  if (res.status === 429) {
    transfersRateLimited.add(1);
    check(res, { 'not rate limited': () => false });
    return;
  }

  if (res.status === 409) {
    transfersErrored.add(1);
    check(res, { 'idempotency key not reused across runs': () => false });
    return;
  }

  transfersErrored.add(1);
  check(res, {
    'transfer unexpected status': () => false,
  });
}

/**
 * @param {object} data
 * @param {object} summary
 * @returns {Record<string, string>}
 */
export function handleSummary(summary) {
  const completed = summary.metrics.ficus_transfers_completed?.values.count ?? 0;
  const declined = summary.metrics.ficus_transfers_declined?.values.count ?? 0;
  const rateLimited = summary.metrics.ficus_transfers_rate_limited?.values.count ?? 0;
  const errored = summary.metrics.ficus_transfers_errored?.values.count ?? 0;
  const checksPass = summary.metrics.checks?.values.rate === 1;

  const reportLines = [
    '',
    '══════════════════════════════════════════════════════════════════════',
    `  Ficus transfer concurrency — ${CONCURRENCY} parallel requests`,
    '══════════════════════════════════════════════════════════════════════',
    '',
    '  Outcomes',
    `    completed (200 COMPLETED) : ${completed}`,
    `    declined (422 / DECLINED) : ${declined}`,
    `    rate limited (429)        : ${rateLimited}`,
    `    unexpected errors         : ${errored}`,
    '',
    '  Invariants (see teardown checks)',
    '    • sender balance never negative',
    '    • money conserved between sender and recipient',
    '    • only fundable transfers complete',
    '    • every request resolves (no orphan partial state)',
    '    • unique idempotency key per concurrent request',
    '',
  ];

  if (rateLimited > 0) {
    reportLines.push(
      '  Rate limit failure',
      '    Restart the API so apps/api/.env is loaded, then rerun:',
      '      TRANSFER_RATE_LIMIT_PER_MIN=10000 LOGIN_RATE_LIMIT_PER_MIN=100 make api-dev',
      '',
    );
  }

  reportLines.push(
    `  Result: ${checksPass && rateLimited === 0 && errored === 0 ? 'PASS' : 'FAIL'}`,
    '══════════════════════════════════════════════════════════════════════',
    '',
    textSummary(summary, { indent: '  ', enableColors: true }),
  );

  const report = reportLines.join('\n');

  return {
    stdout: report,
    [`reports/transfer-concurrency-${CONCURRENCY}.json`]: JSON.stringify(summary, null, 2),
  };
}

export function teardown(data) {
  const finalSenderBalance = fetchBalanceMinor(data.senderToken);
  const finalRecipientBalance = fetchBalanceMinor(data.recipientToken);

  const actualCompleted = Math.round(
    (data.initialSenderBalance - finalSenderBalance) / data.transferAmountMinor,
  );
  const expectedSender =
    data.initialSenderBalance - data.expectedCompleted * data.transferAmountMinor;
  const expectedRecipient =
    data.initialRecipientBalance + data.expectedCompleted * data.transferAmountMinor;
  const conservedPair =
    data.initialSenderBalance + data.initialRecipientBalance ===
    finalSenderBalance + finalRecipientBalance;

  check(null, {
    'sender balance is not negative': () => finalSenderBalance >= 0,
    'exactly expected transfers completed': () =>
      actualCompleted === data.expectedCompleted,
    'sender balance matches completed debits': () =>
      finalSenderBalance === expectedSender,
    'recipient balance matches completed credits': () =>
      finalRecipientBalance === expectedRecipient,
    'pairwise money conserved': () => conservedPair,
    'no partial debit remainder': () =>
      (data.initialSenderBalance - finalSenderBalance) %
        data.transferAmountMinor ===
      0,
  });
}
