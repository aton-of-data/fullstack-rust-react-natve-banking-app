#!/usr/bin/env node
/**
 * k6 performance test runner for Ficus API.
 *
 * Usage:
 *   node scripts/performance/run-k6.mjs
 *   node scripts/performance/run-k6.mjs --duration 1m --vus 10
 *   node scripts/performance/run-k6.mjs --all --duration 30s --vus 5
 *   node scripts/performance/run-k6.mjs --concurrency-suite
 *   node scripts/performance/run-k6.mjs --script transfer-concurrency.js --concurrency 500
 *   API_BASE_URL=http://localhost:8080 node scripts/performance/run-k6.mjs --script idempotency-replay.js
 *
 * Requires k6: https://k6.io/docs/get-started/installation/
 */

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = join(__dirname, '..', '..');

const DEFAULT_SCRIPTS = [
  'login-transfer-feed.js',
  'idempotency-replay.js',
  'transfer-concurrency.js',
];

const SCRIPT_OWNED_OPTIONS = new Set(['transfer-concurrency.js']);
const CONCURRENCY_TIERS = [100, 500, 1000];

function parseArgs(argv) {
  const opts = {
    duration: process.env.K6_DURATION ?? '30s',
    vus: process.env.K6_VUS ?? '5',
    baseUrl: process.env.API_BASE_URL ?? 'http://localhost:8080',
    concurrency: process.env.K6_CONCURRENCY ?? null,
    ci: false,
    all: false,
    concurrencySuite: false,
    scripts: [DEFAULT_SCRIPTS[0]],
  };

  for (let i = 2; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--duration' && argv[i + 1]) {
      opts.duration = argv[++i];
    } else if (arg === '--vus' && argv[i + 1]) {
      opts.vus = argv[++i];
    } else if (arg === '--base-url' && argv[i + 1]) {
      opts.baseUrl = argv[++i];
    } else if (arg === '--script' && argv[i + 1]) {
      opts.scripts = [argv[++i]];
    } else if (arg === '--concurrency' && argv[i + 1]) {
      opts.concurrency = argv[++i];
      opts.scripts = ['transfer-concurrency.js'];
    } else if (arg === '--concurrency-suite') {
      opts.concurrencySuite = true;
      opts.scripts = ['transfer-concurrency.js'];
    } else if (arg === '--all') {
      opts.all = true;
      opts.scripts = [...DEFAULT_SCRIPTS];
    } else if (arg === '--ci') {
      opts.ci = true;
    } else if (arg === '--help' || arg === '-h') {
      printHelp();
      process.exit(0);
    }
  }

  if (opts.all) {
    opts.scripts = [...DEFAULT_SCRIPTS];
  }

  return opts;
}

function printHelp() {
  console.log(`Ficus k6 runner

Usage:
  node scripts/performance/run-k6.mjs [options]

Options:
  --duration <time>       k6 duration for load scripts (default: 30s)
  --vus <n>               virtual users for load scripts (default: 5)
  --base-url <url>        API base URL (default: http://localhost:8080)
  --script <file>         Run a single script from scripts/performance/
  --concurrency <n>       Run transfer-concurrency.js with N parallel transfers
  --concurrency-suite     Run transfer concurrency at 100, 500, and 1000
  --all                   Run login, idempotency, and concurrency scripts
  --ci                    Exit 0 with skip message if k6 missing (for CI smoke)
  -h, --help              Show this help

Environment:
  API_BASE_URL, K6_DURATION, K6_VUS, K6_CONCURRENCY

Concurrency suite prerequisite:
  TRANSFER_RATE_LIMIT_PER_MIN=10000 LOGIN_RATE_LIMIT_PER_MIN=100 make api-dev
`);
}

function hasK6() {
  const result = spawnSync('k6', ['version'], { encoding: 'utf8' });
  return result.status === 0;
}

/**
 * @param {string} baseUrl
 * @param {string} username
 * @param {string} password
 * @returns {Promise<string>}
 */
async function login(baseUrl, username, password) {
  const res = await fetch(`${baseUrl}/v1/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password }),
  });
  if (!res.ok) {
    throw new Error(`login failed for ${username}: HTTP ${res.status}`);
  }
  const body = await res.json();
  return body.access_token;
}

/**
 * Probes transfer rate limits using charlie so alice's burst quota stays intact.
 * @param {string} baseUrl
 * @param {number} requiredBurst
 * @returns {Promise<void>}
 */
async function assertTransferRateLimitHeadroom(baseUrl, requiredBurst) {
  const token = await login(baseUrl, 'charlie', 'password123');
  const probeCount = Math.min(Math.max(requiredBurst, 35), 40);
  let rateLimited = 0;

  for (let i = 0; i < probeCount; i++) {
    const res = await fetch(`${baseUrl}/v1/transfers`, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${token}`,
        'Content-Type': 'application/json',
        'Idempotency-Key': `00000000-0000-4000-8000-${String(i).padStart(12, '0')}`,
      },
      body: JSON.stringify({
        recipient_username: 'bob',
        amount_minor: '1',
        currency: 'USD',
        description: 'transfer rate-limit probe',
      }),
    });
    if (res.status === 429) {
      rateLimited += 1;
    }
  }

  if (rateLimited > 0) {
    printRateLimitFailure(requiredBurst, rateLimited, probeCount);
    process.exit(1);
  }
}

/**
 * @param {number} requiredBurst
 * @param {number} rateLimited
 * @param {number} probeCount
 */
function printRateLimitFailure(requiredBurst, rateLimited, probeCount) {
  console.error(`
✗ API transfer rate limit is too low for ${requiredBurst} concurrent transfers.
  Probe: ${rateLimited}/${probeCount} transfer requests returned HTTP 429.

The running API process must be restarted so apps/api/.env is loaded:

  # apps/api/.env (local dev defaults)
  TRANSFER_RATE_LIMIT_PER_MIN=10000
  LOGIN_RATE_LIMIT_PER_MIN=100

  make api-dev

One-shot alternative:
  TRANSFER_RATE_LIMIT_PER_MIN=10000 LOGIN_RATE_LIMIT_PER_MIN=100 make api-dev
`);
}

/**
 * Resets seeded balances before concurrency tiers mutate alice's account.
 * @returns {void}
 */
function reseedDatabase() {
  console.log('Reseeding database for consistent concurrency balances...');
  const result = spawnSync('make', ['db-seed'], {
    cwd: REPO_ROOT,
    stdio: 'inherit',
    env: {
      ...process.env,
      SEED_RESET_BALANCES: '1',
    },
  });
  if (result.status !== 0) {
    console.error('make db-seed failed — run manually before concurrency tests');
    process.exit(result.status ?? 1);
  }
}

function printConcurrencyPrerequisite() {
  console.log(`
╔══════════════════════════════════════════════════════════════════════╗
║  Transfer concurrency suite                                          ║
║                                                                      ║
║  Requires API restarted with apps/api/.env rate limits:              ║
║    TRANSFER_RATE_LIMIT_PER_MIN=10000                                 ║
║    LOGIN_RATE_LIMIT_PER_MIN=100                                      ║
║                                                                      ║
║  Tiers: ${CONCURRENCY_TIERS.join(', ')} parallel transfers from alice → bob                       ║
╚══════════════════════════════════════════════════════════════════════╝
`);
}

function runScript(scriptName, opts) {
  const scriptPath = join(__dirname, scriptName);
  if (!existsSync(scriptPath)) {
    console.error(`k6 script not found: ${scriptPath}`);
    return 1;
  }

  const ownsOptions = SCRIPT_OWNED_OPTIONS.has(scriptName);
  const concurrency = opts.concurrency;

  if (ownsOptions) {
    console.log(
      `Running k6 script ${scriptName}: ${concurrency ?? CONCURRENCY_TIERS[0]} parallel transfers against ${opts.baseUrl}`,
    );
  } else {
    console.log(
      `Running k6 script ${scriptName}: ${opts.vus} VUs for ${opts.duration} against ${opts.baseUrl}`,
    );
  }

  const args = ['run', '-e', `API_BASE_URL=${opts.baseUrl}`];

  if (ownsOptions) {
    args.push('-e', `K6_CONCURRENCY=${concurrency ?? CONCURRENCY_TIERS[0]}`);
  } else {
    args.push('--vus', String(opts.vus), '--duration', opts.duration);
  }

  args.push(scriptPath);

  const reportsDir = join(REPO_ROOT, 'reports');
  mkdirSync(reportsDir, { recursive: true });

  const result = spawnSync('k6', args, {
    stdio: 'inherit',
    cwd: REPO_ROOT,
  });

  return result.status ?? 1;
}

async function main() {
  const opts = parseArgs(process.argv);

  if (!hasK6()) {
    const msg =
      'k6 not installed — skipping performance test. Install: https://k6.io/docs/get-started/installation/';
    if (opts.ci) {
      console.warn(`[skip] ${msg}`);
      process.exit(0);
    }
    console.error(msg);
    process.exit(1);
  }

  if (opts.concurrencySuite) {
    printConcurrencyPrerequisite();
    reseedDatabase();
    await assertTransferRateLimitHeadroom(
      opts.baseUrl,
      CONCURRENCY_TIERS[CONCURRENCY_TIERS.length - 1],
    );
    for (const tier of CONCURRENCY_TIERS) {
      console.log(`\n── Tier: ${tier} concurrent transfers ──\n`);
      const code = runScript('transfer-concurrency.js', {
        ...opts,
        concurrency: String(tier),
      });
      if (code !== 0) {
        process.exit(code);
      }
    }
    console.log('\n✓ Concurrency suite passed (100, 500, 1000)\n');
    process.exit(0);
  }

  for (const script of opts.scripts) {
    if (script === 'transfer-concurrency.js' && !opts.concurrency) {
      opts.concurrency = opts.ci ? '10' : String(CONCURRENCY_TIERS[0]);
      if (!opts.ci) {
        printConcurrencyPrerequisite();
      }
    }

    if (script === 'transfer-concurrency.js' && !opts.ci) {
      if (!opts.concurrencySuite) {
        reseedDatabase();
      }
      await assertTransferRateLimitHeadroom(
        opts.baseUrl,
        Number(opts.concurrency ?? CONCURRENCY_TIERS[0]),
      );
    }

    const code = runScript(script, opts);
    if (code !== 0) {
      process.exit(code);
    }
  }

  process.exit(0);
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
