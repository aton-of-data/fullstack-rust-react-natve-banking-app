/**
 * k6 concurrent transfer load with unique idempotency keys.
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

const BASE_URL = __ENV.API_BASE_URL || 'http://localhost:8080';

export const options = {
  vus: Number(__ENV.K6_VUS || 10),
  duration: __ENV.K6_DURATION || '30s',
  thresholds: {
    http_req_failed: ['rate<0.1'],
    checks: ['rate>0.9'],
  },
};

export function setup() {
  const loginRes = http.post(
    `${BASE_URL}/v1/auth/login`,
    JSON.stringify({ username: 'alice', password: 'password123' }),
    { headers: { 'Content-Type': 'application/json' } },
  );
  return { token: JSON.parse(loginRes.body).access_token };
}

export default function (data) {
  const headers = {
    Authorization: `Bearer ${data.token}`,
    'Content-Type': 'application/json',
    'Idempotency-Key': uuidv4(),
  };

  const res = http.post(
    `${BASE_URL}/v1/transfers`,
    JSON.stringify({
      recipient_username: 'bob',
      amount_minor: '10',
      currency: 'USD',
      description: 'k6 concurrency',
    }),
    { headers, tags: { name: 'transfer_concurrent' } },
  );

  check(res, {
    'transfer resolved': (r) => r.status === 200 || r.status === 422,
  });
  sleep(0.2);
}
