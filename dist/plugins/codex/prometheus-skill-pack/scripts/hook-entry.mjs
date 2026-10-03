#!/usr/bin/env node
/**
 * Exec-form hook entry point.
 *
 * WHY THIS FILE EXISTS RATHER THAN A COMPILED BINARY IN `hooks.json`
 *
 * Exec form -- a `command` plus an `args` array -- is spawned directly with no
 * shell on any platform, which is the whole point: a substituted plugin root
 * containing backslashes, `$`, or backticks arrives here verbatim because
 * nothing tokenizes it. Shell form, by contrast, is handed to `sh -c` on POSIX
 * and to POWERSHELL on Windows whenever Git Bash is absent, so the pack's
 * previous `bash -c '<multi-line bash>'` entries worked on Windows only where
 * Git Bash happened to be installed.
 *
 * Exec form needs `command` to name one executable, and `hooks.json` is one
 * file shared by every host. The harness substitutes only `CLAUDE_PROJECT_DIR`,
 * `CLAUDE_PLUGIN_ROOT`, and `CLAUDE_PLUGIN_DATA` -- there is no platform
 * placeholder -- so no single path can name the right per-target binary on six
 * targets. And a binary cannot bootstrap itself: before the first install there
 * is nothing at any stable path to run.
 *
 * `node` resolves both problems. It is a real executable on every supported
 * host, it is already a hard dependency of the bootstrap path, and it is
 * present before anything is installed. The compiled dispatcher still owns the
 * hot path -- this file execs it as soon as it exists -- it simply is not the
 * thing `hooks.json` has to name.
 *
 * EVERY child here is spawned with an explicit argument vector and `shell:
 * false`. Nothing on this path reconstructs a command string.
 */

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

/** EX_CONFIG: the hook is misconfigured, not the tool call that triggered it. */
const NOT_ACTIVATED = 78;

function fail(code, message, bundle) {
  process.stderr.write(
    `${JSON.stringify({ status: 'HOOK_RUNTIME_ERROR', code, message, bundle })}\n`
  );
  process.exit(NOT_ACTIVATED);
}

/**
 * Read `--flag value` pairs.
 *
 * Deliberately strict: exec form delivers a clean argument vector, so anything
 * unexpected here means the configuration is wrong rather than that a shell
 * mangled it, and guessing would hide that.
 */
function parseArgs(argv) {
  const args = { bundle: '', hook: '', harness: '' };
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    if (flag === '--bundle') args.bundle = argv[++index] ?? '';
    else if (flag === '--hook') args.hook = argv[++index] ?? '';
    else if (flag === '--harness') args.harness = argv[++index] ?? '';
    else fail('INVALID_ARGUMENT', `unknown argument: ${flag}`, args.bundle);
  }
  return args;
}

const args = parseArgs(process.argv.slice(2));
if (!/^[a-f0-9]{64}$/.test(args.bundle)) {
  fail('INVALID_BUNDLE', 'bundle id is not sha256', args.bundle);
}

const storeRoot =
  process.env.PROMETHEUS_PLUGIN_ROOT ||
  path.join(os.homedir(), '.prometheus/plugins/prometheus-skill-pack');

// Codex sets PLUGIN_ROOT where Claude Code sets CLAUDE_PLUGIN_ROOT. Both are
// exported onto the spawned process by the harness in either hook form.
const pluginRoot = process.env.CLAUDE_PLUGIN_ROOT || process.env.PLUGIN_ROOT || '';

function run(file, argv) {
  return spawnSync(file, argv, { stdio: 'inherit', shell: false });
}

// ---------------------------------------------------------------------------
// Fast path: the compiled dispatcher, once it is installed
// ---------------------------------------------------------------------------

const compiled = ['prometheus-hook', 'prometheus-hook.exe']
  .map(name => path.join(storeRoot, 'runtime/v1', name))
  .find(candidate => fs.existsSync(candidate));

if (compiled) {
  const result = run(compiled, [
    'run',
    '--bundle',
    args.bundle,
    '--hook',
    args.hook,
    '--harness',
    args.harness,
  ]);
  if (result.error) {
    fail('DISPATCHER_SPAWN', `cannot launch the dispatcher: ${result.error.message}`, args.bundle);
  }
  process.exit(result.status ?? NOT_ACTIVATED);
}

// ---------------------------------------------------------------------------
// Shell path: used until the dispatcher is installed, and to bootstrap it
// ---------------------------------------------------------------------------

// Cold-path scripts declare their dependency rather than dying on an
// interpreter error. Until the compiled dispatcher exists, resolution and
// bootstrap are both shell scripts, so a host with no shell is told exactly
// what is missing.
if (spawnSync('bash', ['-c', 'exit 0'], { shell: false }).error) {
  fail(
    'MISSING_SHELL',
    'no POSIX shell is available; install Git Bash or the compiled dispatcher',
    args.bundle
  );
}

const runner = path.join(storeRoot, 'runtime/v1/run-hook');
const resolved =
  fs.existsSync(runner) &&
  spawnSync('bash', [runner, '--bundle', args.bundle, '--resolve-only'], {
    stdio: 'ignore',
    shell: false,
  }).status === 0;

if (!resolved) {
  const bootstrap = path.join(pluginRoot, 'shared/scripts/bootstrap-hook-runtime.sh');
  if (!pluginRoot || !fs.existsSync(bootstrap)) {
    fail('NOT_ACTIVATED', 'no activated bundle and no bootstrap payload', args.bundle);
  }
  const install = run('bash', [
    bootstrap,
    '--source-root',
    pluginRoot,
    '--expected-bundle',
    args.bundle,
  ]);
  if (install.status !== 0) process.exit(install.status ?? NOT_ACTIVATED);
}

if (!args.hook) fail('MISSING_HOOK', 'hook id is required', args.bundle);
if (!args.harness) fail('MISSING_HARNESS', 'harness is required', args.bundle);

const dispatched = run('bash', [
  runner,
  '--bundle',
  args.bundle,
  '--hook',
  args.hook,
  '--harness',
  args.harness,
]);
process.exit(dispatched.status ?? NOT_ACTIVATED);
