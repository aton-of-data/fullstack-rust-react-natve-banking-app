#!/usr/bin/env node
/**
 * Security scan wrapper — runs gitleaks and trivy when available.
 *
 * Usage:
 *   node scripts/security-scan.mjs
 *   node scripts/security-scan.mjs --ci
 *
 * Gracefully skips tools that are not installed locally.
 */

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');

function parseArgs(argv) {
  return { ci: argv.includes('--ci') };
}

function toolAvailable(cmd, args = ['--version']) {
  const result = spawnSync(cmd, args, { encoding: 'utf8' });
  return result.status === 0;
}

/**
 * @param {string} name
 * @param {string[]} cmd
 * @returns {{ skipped: boolean, ok: boolean }}
 */
function runTool(name, cmd) {
  const [bin, ...args] = cmd;

  if (!toolAvailable(bin, ['--version']) && !toolAvailable(bin, ['version'])) {
    console.warn(`[skip] ${name}: ${bin} not installed`);
    return { skipped: true, ok: true };
  }

  console.log(`[run] ${name}: ${cmd.join(' ')}`);
  const result = spawnSync(bin, args, {
    cwd: REPO_ROOT,
    stdio: 'inherit',
    encoding: 'utf8',
  });

  const ok = result.status === 0;
  if (!ok) {
    console.error(`[fail] ${name} exited with code ${result.status}`);
  } else {
    console.log(`[pass] ${name}`);
  }

  return { skipped: false, ok };
}

function main() {
  const opts = parseArgs(process.argv);
  const results = [];

  results.push(
    runTool('gitleaks', ['gitleaks', 'detect', '--source', REPO_ROOT, '--no-banner', '--redact']),
  );

  const dockerfile = `${REPO_ROOT}/infra/docker/Dockerfile.api`;
  if (existsSync(dockerfile) && toolAvailable('trivy', ['--version'])) {
    results.push(
      runTool('trivy-config', ['trivy', 'config', '--severity', 'HIGH,CRITICAL', REPO_ROOT]),
    );
  } else if (!toolAvailable('trivy', ['--version'])) {
    console.warn('[skip] trivy: not installed');
    results.push({ skipped: true, ok: true });
  }

  const failed = results.filter((r) => !r.skipped && !r.ok);
  const allSkipped = results.every((r) => r.skipped);

  if (failed.length > 0) {
    process.exit(1);
  }

  if (allSkipped) {
    const msg = 'No security tools installed (gitleaks, trivy). Install for local scans.';
    if (opts.ci) {
      console.warn(`[skip] ${msg}`);
      process.exit(0);
    }
    console.warn(msg);
    process.exit(0);
  }

  console.log('Security scan complete.');
  process.exit(0);
}

main();
