/**
 * k6 idempotency replay scenario.
 *
 * Environment (set by run-k6.mjs):
 *   API_BASE_URL — default http://localhost:8080
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

const BASE_URL = __ENV.API_BASE_URL || 'http://localhost:8080';

export const options = {
  vus: 1,
  iterations: 1,
  thresholds: {
    checks: ['rate>0.99'],
  },
};

export default function () {
  const loginRes = http.post(
    `${BASE_URL}/v1/auth/login`,
    JSON.stringify({ username: 'alice', password: 'password123' }),
    { headers: { 'Content-Type': 'application/json' } },
  );
  const token = JSON.parse(loginRes.body).access_token;
  const idempotencyKey = uuidv4();
  const body = JSON.stringify({
    recipient_username: 'bob',
    amount_minor: '25',
    currency: 'USD',
    description: 'k6 idempotency replay',
  });
  const headers = {
    Authorization: `Bearer ${token}`,
    'Content-Type': 'application/json',
    'Idempotency-Key': idempotencyKey,
  };

  const first = http.post(`${BASE_URL}/v1/transfers`, body, { headers });
  const replay = http.post(`${BASE_URL}/v1/transfers`, body, { headers });

  check(first, { 'first transfer 200': (r) => r.status === 200 });
  check(replay, { 'replay transfer 200': (r) => r.status === 200 });

  const firstId = JSON.parse(first.body).transfer_id;
  const replayId = JSON.parse(replay.body).transfer_id;
  check(null, {
    'same transfer id on replay': () => firstId === replayId,
  });

  sleep(1);
}
