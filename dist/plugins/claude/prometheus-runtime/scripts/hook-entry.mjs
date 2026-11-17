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

// Hook dispatch is read-only with respect to installation. A source checkout
// or native cache may be newer than the operator-selected active generation.
function activationFailure() {
  if (activationPending) {
    fail('ACTIVATION_PENDING',
      `An unfinished activation at ${path.join(storeRoot, 'pointers/activation.pending.json')} blocks dispatch. ` +
      'Use the verified installer to explicitly recover or cancel the pending transaction, then restart the session. Hooks never recover or upgrade the installation automatically.', args.bundle);
  }
  if (pluginRoot) {
    const manifest = path.join(pluginRoot, 'shared/harnesses/generated/release-manifest.json');
    let release;
    try { release = JSON.parse(fs.readFileSync(manifest, 'utf8')); }
    catch {
      fail('PAYLOAD_INCOMPLETE', `Cannot read the release manifest at ${manifest}. Restore the intended plugin payload before explicitly installing it.`, args.bundle);
    }
    if (release.bundleId !== args.bundle) {
      fail('BUNDLE_IDENTITY_MISMATCH',
        `The session requested bundle ${args.bundle}, but ${pluginRoot} contains ${release.bundleId}. ` +
        'Restart the session using the intended installed plugin. No installation was changed.', args.bundle);
    }
  }
  fail('NOT_ACTIVATED',
    `Requested bundle ${args.bundle} is not available in the verified installation. ` +
    'Activate the intended payload explicitly with install-plugin-generation.js, or restart using the already installed plugin. Hooks never install or upgrade a payload.', args.bundle);
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
// Also protect children when an older compiled runtime is still installed.
process.env.PYTHONDONTWRITEBYTECODE = '1';
if (!/^[a-f0-9]{64}$/.test(args.bundle)) {
  fail('INVALID_BUNDLE', 'bundle id is not sha256', args.bundle);
}

const storeRoot =
  process.env.PROMETHEUS_PLUGIN_ROOT ||
  path.join(os.homedir(), '.prometheus/plugins/prometheus-skill-pack');
const activationPending = fs.existsSync(path.join(storeRoot, 'pointers/activation.pending.json'));

// Codex sets PLUGIN_ROOT where Claude Code sets CLAUDE_PLUGIN_ROOT. Both are
// exported onto the spawned process by the harness in either hook form.
const pluginRoot = process.env.CLAUDE_PLUGIN_ROOT || process.env.PLUGIN_ROOT || '';

// A pending transaction is never an invitation to run the installer.
if (activationPending) activationFailure();

function run(file, argv) {
  return spawnSync(file, argv, { stdio: 'inherit', shell: false });
}

// ---------------------------------------------------------------------------
// Fast path: the compiled dispatcher, once it is installed
// ---------------------------------------------------------------------------

const compiled = ['prometheus-hook', 'prometheus-hook.exe']
  .map(name => path.join(storeRoot, 'runtime/v1', name))
  .find(candidate => fs.existsSync(candidate));

if (
  !activationPending && compiled &&
  spawnSync(compiled, ['resolve', '--bundle', args.bundle], {
    stdio: 'ignore',
    shell: false,
  }).status === 0
) {
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
// Shell path: dispatch an explicitly installed bundle
// ---------------------------------------------------------------------------

// Cold-path scripts declare their dependency rather than dying on an
// interpreter error. Until the compiled dispatcher exists, resolution uses a shell script, so a host with no shell is told exactly
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
  !activationPending &&
  fs.existsSync(runner) &&
  spawnSync('bash', [runner, '--bundle', args.bundle, '--resolve-only'], {
    stdio: 'ignore',
    shell: false,
  }).status === 0;

if (!resolved) activationFailure();

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
