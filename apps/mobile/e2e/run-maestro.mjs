#!/usr/bin/env node
/**
 * Maestro mobile E2E runner for the Ficus Expo app.
 *
 * Runs flows **sequentially** (one process per YAML) so shared Expo Go /
 * SecureStore auth state cannot collide across flows.
 *
 * Usage:
 *   node apps/mobile/e2e/run-maestro.mjs
 *   node apps/mobile/e2e/run-maestro.mjs --platform=ios
 *   node apps/mobile/e2e/run-maestro.mjs --platform=android --record
 *
 * Exit codes:
 *   0 — all flows passed
 *   1 — one or more flows failed
 *   2 — skipped (maestro or Java missing)
 */

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, writeFileSync } from 'node:fs';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const FLOWS_DIR = join(__dirname, 'flows');
const REPORTS_DIR = join(__dirname, 'reports');
const SCREENSHOTS_DIR = join(__dirname, 'screenshots');
const CONFIG_PATH = join(__dirname, 'config.yaml');

/**
 * @typedef {'ios' | 'android' | undefined} Platform
 */

/**
 * @typedef {object} RunnerOptions
 * @property {Platform} platform Target platform filter for Maestro.
 * @property {boolean} record Capture screenshots under e2e/screenshots.
 */

/**
 * @typedef {object} FlowResult
 * @property {string} flow Flow file name.
 * @property {'PASS'|'FAIL'} status Outcome.
 * @property {number} exitCode Process exit code.
 * @property {string} stdout Captured stdout.
 * @property {string} stderr Captured stderr.
 * @property {number} durationMs Wall duration.
 */

/**
 * Parses CLI arguments.
 *
 * @param {string[]} argv Process argv.
 * @returns {RunnerOptions} Parsed options.
 */
function parseArgs(argv) {
  /** @type {RunnerOptions} */
  const opts = { platform: undefined, record: false };

  for (const arg of argv.slice(2)) {
    if (arg === '--record') {
      opts.record = true;
    } else if (arg.startsWith('--platform=')) {
      const value = arg.slice('--platform='.length);
      if (value === 'ios' || value === 'android') {
        opts.platform = value;
      } else {
        console.error(`Unknown platform: ${value}. Use ios or android.`);
        process.exit(1);
      }
    } else if (arg === '--help' || arg === '-h') {
      printHelp();
      process.exit(0);
    } else {
      console.error(`Unknown argument: ${arg}`);
      printHelp();
      process.exit(1);
    }
  }

  return opts;
}

/**
 * Prints CLI help.
 */
function printHelp() {
  console.log(`Ficus Maestro E2E runner

Usage:
  node apps/mobile/e2e/run-maestro.mjs [options]

Options:
  --platform=ios|android   Pass platform to maestro test
  --record                 Write screenshots under e2e/screenshots/
  -h, --help               Show this help

Environment:
  JAVA_HOME                JDK home (heuristic applied if unset)
`);
}

/**
 * Resolves a heuristic JAVA_HOME for Homebrew openjdk@17 when unset.
 *
 * @returns {string | undefined} JAVA_HOME path when found.
 */
function resolveJavaHome() {
  if (process.env.JAVA_HOME && existsSync(process.env.JAVA_HOME)) {
    return process.env.JAVA_HOME;
  }

  const candidates = [
    '/opt/homebrew/opt/openjdk@17/libexec/openjdk.jdk/Contents/Home',
    '/usr/local/opt/openjdk@17/libexec/openjdk.jdk/Contents/Home',
    '/opt/homebrew/Cellar/openjdk@17/17.0.19/libexec/openjdk.jdk/Contents/Home',
  ];

  const fromJavaHomeTool = spawnSync('/usr/libexec/java_home', ['-v', '17'], {
    encoding: 'utf8',
  });
  if (fromJavaHomeTool.status === 0) {
    const path = fromJavaHomeTool.stdout.trim();
    if (path && existsSync(path)) {
      return path;
    }
  }

  for (const candidate of candidates) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }

  return undefined;
}

/**
 * Returns whether a binary is on PATH.
 *
 * @param {string} binary Binary name.
 * @returns {boolean} True when resolvable.
 */
function hasBinary(binary) {
  const result = spawnSync('command', ['-v', binary], {
    encoding: 'utf8',
    shell: true,
  });
  return result.status === 0 && Boolean(result.stdout.trim());
}

/**
 * Lists Maestro flow YAML files.
 *
 * @returns {string[]} Absolute paths sorted by name.
 */
function listFlowFiles() {
  if (!existsSync(FLOWS_DIR)) {
    return [];
  }
  return readdirSync(FLOWS_DIR)
    .filter((name) => name.endsWith('.yaml') && name !== 'config.yaml' && !name.startsWith('docs-'))
    .sort()
    .map((name) => join(FLOWS_DIR, name));
}

/**
 * Writes JSON and Markdown report artifacts.
 *
 * @param {object} report Report payload.
 */
function writeReports(report) {
  mkdirSync(REPORTS_DIR, { recursive: true });
  const jsonPath = join(REPORTS_DIR, 'latest.json');
  const mdPath = join(REPORTS_DIR, 'latest.md');
  writeFileSync(jsonPath, `${JSON.stringify(report, null, 2)}\n`, 'utf8');

  const lines = [
    '# Maestro E2E Report',
    '',
    `- **Status:** ${report.status}`,
    `- **Started:** ${report.startedAt}`,
    `- **Finished:** ${report.finishedAt}`,
    `- **Platform:** ${report.platform ?? 'default'}`,
    `- **Record:** ${report.record ? 'yes' : 'no'}`,
    `- **Exit code:** ${report.exitCode}`,
    `- **Mode:** sequential (one flow per maestro process)`,
    '',
  ];

  if (report.reason) {
    lines.push(`- **Reason:** ${report.reason}`, '');
  }

  if (report.results?.length) {
    lines.push('## Flow results', '');
    for (const result of report.results) {
      lines.push(
        `- **${result.status}** \`${result.flow}\` (${result.durationMs}ms, exit ${result.exitCode})`,
      );
    }
    lines.push('');
  }

  if (report.stdout) {
    lines.push('## stdout', '', '```', report.stdout.trimEnd(), '```', '');
  }
  if (report.stderr) {
    lines.push('## stderr', '', '```', report.stderr.trimEnd(), '```', '');
  }

  writeFileSync(mdPath, `${lines.join('\n')}\n`, 'utf8');
  console.log(`Wrote ${jsonPath}`);
  console.log(`Wrote ${mdPath}`);
}

/**
 * Runs a single Maestro flow file.
 *
 * @param {string} flowPath Absolute flow path.
 * @param {RunnerOptions} opts Runner options.
 * @returns {FlowResult} Flow result.
 */
function runOneFlow(flowPath, opts) {
  /** @type {string[]} */
  const args = ['test', flowPath];
  if (existsSync(CONFIG_PATH)) {
    args.push('--config', CONFIG_PATH);
  }
  if (opts.platform) {
    args.push('--platform', opts.platform);
  }
  if (opts.record) {
    args.push('--test-output-dir', SCREENSHOTS_DIR);
  }

  const name = basename(flowPath);
  console.log(`\n=== Running ${name} ===`);
  const started = Date.now();
  const result = spawnSync('maestro', args, {
    encoding: 'utf8',
    env: process.env,
    cwd: join(__dirname, '..', '..', '..'),
  });
  const durationMs = Date.now() - started;
  const exitCode = result.status === null ? 1 : result.status;
  const status = exitCode === 0 ? 'PASS' : 'FAIL';
  console.log(`=== ${status} ${name} (${durationMs}ms) ===`);
  if (result.stdout) {
    console.log(result.stdout);
  }
  if (result.stderr) {
    console.error(result.stderr);
  }

  return {
    flow: name,
    status,
    exitCode,
    stdout: result.stdout ?? '',
    stderr: result.stderr ?? '',
    durationMs,
  };
}

/**
 * Main entry.
 */
function main() {
  const opts = parseArgs(process.argv);
  const startedAt = new Date().toISOString();
  const flows = listFlowFiles();

  const javaHome = resolveJavaHome();
  if (javaHome) {
    process.env.JAVA_HOME = javaHome;
  }

  const maestroMissing = !hasBinary('maestro');
  const javaMissing = !javaHome && !hasBinary('java');

  if (maestroMissing || javaMissing) {
    const reasons = [];
    if (maestroMissing) {
      reasons.push('maestro CLI not found on PATH');
    }
    if (javaMissing) {
      reasons.push('Java / JAVA_HOME not available (need openjdk@17 or equivalent)');
    }
    const reason = reasons.join('; ');
    const finishedAt = new Date().toISOString();
    writeReports({
      status: 'SKIP',
      reason,
      startedAt,
      finishedAt,
      platform: opts.platform,
      record: opts.record,
      exitCode: 2,
      results: [],
      stdout: '',
      stderr: reason,
    });
    console.error(`SKIP: ${reason}`);
    process.exit(2);
  }

  mkdirSync(REPORTS_DIR, { recursive: true });
  if (opts.record) {
    mkdirSync(SCREENSHOTS_DIR, { recursive: true });
  }

  console.log(`JAVA_HOME=${process.env.JAVA_HOME ?? '(unset)'}`);
  console.log(`Running ${flows.length} flows sequentially`);

  /** @type {FlowResult[]} */
  const results = [];
  for (const flow of flows) {
    results.push(runOneFlow(flow, opts));
  }

  const finishedAt = new Date().toISOString();
  const failed = results.filter((r) => r.status !== 'PASS');
  const exitCode = failed.length === 0 ? 0 : 1;
  const status = exitCode === 0 ? 'PASS' : 'FAIL';
  const stdout = results.map((r) => `--- ${r.flow} ---\n${r.stdout}`).join('\n');
  const stderr = results
    .filter((r) => r.stderr)
    .map((r) => `--- ${r.flow} ---\n${r.stderr}`)
    .join('\n');

  writeReports({
    status,
    startedAt,
    finishedAt,
    platform: opts.platform,
    record: opts.record,
    exitCode,
    results,
    command: 'maestro test <flow.yaml> (sequential)',
    stdout,
    stderr,
  });

  writeFileSync(join(REPORTS_DIR, 'run-ios-full.log'), `${stdout}\n${stderr}\n`, 'utf8');

  console.log(
    `\n${results.filter((r) => r.status === 'PASS').length}/${results.length} flows passed`,
  );
  process.exit(exitCode);
}

main();
