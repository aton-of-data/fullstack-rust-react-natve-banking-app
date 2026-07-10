/**
 * k6 load test: login → transfer → poll feed.
 *
 * Authenticates seeded users once in `setup()` so iterations exercise transfer,
 * balance, and feed paths without tripping login rate limits (default 10/min/IP).
 *
 * Environment (set by run-k6.mjs):
 *   API_BASE_URL — default http://localhost:8080
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import {
  randomIntBetween,
  uuidv4,
} from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

const BASE_URL = __ENV.API_BASE_URL || 'http://localhost:8080';

const USERS = [
  { username: 'alice', password: 'password123' },
  { username: 'bob', password: 'password123' },
  { username: 'charlie', password: 'password123' },
];

export const options = {
  thresholds: {
    http_req_failed: ['rate<0.05'],
    http_req_duration: ['p(95)<2000'],
    checks: ['rate>0.95'],
  },
};

/**
 * Picks a sender/recipient pair that are different users.
 * @param {Array<{ user: object, token: string }>} sessions
 * @returns {{ sender: { user: object, token: string }, recipient: object }}
 */
function pickTransferPair(sessions) {
  const senderIdx = randomIntBetween(0, sessions.length - 1);
  let recipientIdx = randomIntBetween(0, sessions.length - 1);
  while (recipientIdx === senderIdx) {
    recipientIdx = randomIntBetween(0, sessions.length - 1);
  }
  return {
    sender: sessions[senderIdx],
    recipient: sessions[recipientIdx].user,
  };
}

/**
 * Authenticates and returns bearer token.
 * @param {object} user
 * @returns {string|null}
 */
function login(user) {
  const res = http.post(
    `${BASE_URL}/v1/auth/login`,
    JSON.stringify({ username: user.username, password: user.password }),
    {
      headers: { 'Content-Type': 'application/json' },
      tags: { name: 'login' },
    },
  );

  check(res, {
    'login status 200': (r) => r.status === 200,
    'login has token': (r) => {
      try {
        return JSON.parse(r.body).access_token !== undefined;
      } catch {
        return false;
      }
    },
  });

  if (res.status !== 200) {
    return null;
  }

  return JSON.parse(res.body).access_token;
}

export default function (data) {
  const { sessions } = data;
  const { sender, recipient } = pickTransferPair(sessions);
  const token = sender.token;

  const authHeaders = {
    Authorization: `Bearer ${token}`,
    'Content-Type': 'application/json',
    'Idempotency-Key': uuidv4(),
  };

  const amountMinor = String(randomIntBetween(1, 50));

  const transferRes = http.post(
    `${BASE_URL}/v1/transfers`,
    JSON.stringify({
      recipient_username: recipient.username,
      amount_minor: amountMinor,
      currency: 'USD',
      description: 'k6 load test',
    }),
    { headers: authHeaders, tags: { name: 'transfer' } },
  );

  check(transferRes, {
    'transfer accepted': (r) => r.status === 200 || r.status === 422,
  });

  const balanceRes = http.get(`${BASE_URL}/v1/accounts/me/balance`, {
    headers: { Authorization: `Bearer ${token}` },
    tags: { name: 'balance' },
  });

  check(balanceRes, {
    'balance status 200': (r) => r.status === 200,
  });

  const feedRes = http.get(`${BASE_URL}/v1/feed`, {
    headers: { Authorization: `Bearer ${token}` },
    tags: { name: 'feed' },
  });

  check(feedRes, {
    'feed status 200': (r) => r.status === 200,
    'feed has items array': (r) => {
      try {
        return Array.isArray(JSON.parse(r.body).items);
      } catch {
        return false;
      }
    },
  });

  sleep(randomIntBetween(1, 3));
}

export function setup() {
  const res = http.get(`${BASE_URL}/health/ready`);
  check(res, { 'API ready': (r) => r.status === 200 });

  const sessions = USERS.map((user) => {
    const token = login(user);
    if (!token) {
      throw new Error(`setup login failed for ${user.username}`);
    }
    return { user, token };
  });

  return { sessions };
}
