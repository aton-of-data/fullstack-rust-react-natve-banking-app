#!/usr/bin/env node
/**
 * k6 performance test runner for Ficus API.
 *
 * Usage:
 *   node scripts/performance/run-k6.mjs
 *   node scripts/performance/run-k6.mjs --duration 1m --vus 10
 *   API_BASE_URL=http://localhost:8080 node scripts/performance/run-k6.mjs
 *
 * Requires k6: https://k6.io/docs/get-started/installation/
 */

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const scriptPath = join(__dirname, 'login-transfer-feed.js');

function parseArgs(argv) {
  const opts = {
    duration: process.env.K6_DURATION ?? '1m',
    vus: process.env.K6_VUS ?? '10',
    baseUrl: process.env.API_BASE_URL ?? 'http://localhost:8080',
    ci: false,
  };

  for (let i = 2; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--duration' && argv[i + 1]) {
      opts.duration = argv[++i];
    } else if (arg === '--vus' && argv[i + 1]) {
      opts.vus = argv[++i];
    } else if (arg === '--base-url' && argv[i + 1]) {
      opts.baseUrl = argv[++i];
    } else if (arg === '--ci') {
      opts.ci = true;
    } else if (arg === '--help' || arg === '-h') {
      printHelp();
      process.exit(0);
    }
  }

  return opts;
}

function printHelp() {
  console.log(`Ficus k6 runner

Usage:
  node scripts/performance/run-k6.mjs [options]

Options:
  --duration <time>   k6 duration (default: 1m)
  --vus <n>           virtual users (default: 10)
  --base-url <url>    API base URL (default: http://localhost:8080)
  --ci                Exit 0 with skip message if k6 missing (for CI smoke)
  -h, --help          Show this help

Environment:
  API_BASE_URL, K6_DURATION, K6_VUS
`);
}

function hasK6() {
  const result = spawnSync('k6', ['version'], { encoding: 'utf8' });
  return result.status === 0;
}

function main() {
  const opts = parseArgs(process.argv);

  if (!existsSync(scriptPath)) {
    console.error(`k6 script not found: ${scriptPath}`);
    process.exit(1);
  }

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

  console.log(`Running k6: ${opts.vus} VUs for ${opts.duration} against ${opts.baseUrl}`);

  const result = spawnSync(
    'k6',
    [
      'run',
      '--vus',
      String(opts.vus),
      '--duration',
      opts.duration,
      '-e',
      `API_BASE_URL=${opts.baseUrl}`,
      scriptPath,
    ],
    { stdio: 'inherit' },
  );

  process.exit(result.status ?? 1);
}

main();
