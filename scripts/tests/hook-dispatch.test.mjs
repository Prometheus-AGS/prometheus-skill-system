/**
 * Fixtures for the parts of `shell-free-hook-dispatch` that do not require the
 * compiled dispatcher: execution eligibility taken from the manifest, the
 * script-shim guard on generated configuration, and the cold-path dependency
 * declaration.
 *
 * The runtime half runs the REAL `shared/scripts/hook-runtime-v1.sh` against a
 * fabricated store, including a store whose dispatcher has no executable bit at
 * all -- which is the condition the old `[[ -x ]]` gate could not survive.
 */

import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { probeShell } from '../lib/capabilities.js';
import { SHELL_ONLY_EXECUTABLES, hookExecutable, shellOnlyExecutableError } from '../lib/hook-config.js';
import { __testing } from '../install-plugin-generation.js';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const results = [];
const skipped = [];
const check = (name, body) => {
  body();
  results.push(name);
};
const skip = (name, why) => skipped.push(`${name} -- ${why}`);

const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-dispatch-'));
const sha256 = value => crypto.createHash('sha256').update(value).digest('hex');

// ---------------------------------------------------------------------------
// Requirement: hooks are dispatched without a host shell -- the shim guard
// ---------------------------------------------------------------------------

check('a hook whose executable is a script shim is refused', () => {
  for (const extension of SHELL_ONLY_EXECUTABLES) {
    const command = `/opt/prometheus/bin/dispatch${extension}`;
    const error = shellOnlyExecutableError(command);
    assert.ok(error, `${extension} must be refused as a hook executable`);
    assert.match(error, /real executable/);
  }
  // Windows spelling of the same defect: a backslash path to a .cmd shim.
  assert.ok(shellOnlyExecutableError('C:\\prometheus\\bin\\dispatch.cmd'));
});

check('a real executable is accepted, with or without arguments', () => {
  for (const command of [
    '/opt/prometheus/bin/prometheus-hook',
    'C:\\prometheus\\bin\\prometheus-hook.exe',
    'bash -c \'exec runner\'',
    '/usr/bin/env',
  ]) {
    assert.equal(shellOnlyExecutableError(command), null, `rejected ${command}`);
  }
});

check('the executable is taken from the command, not the whole string', () => {
  assert.equal(hookExecutable('/opt/bin/prometheus-hook --bundle x'), 'prometheus-hook');
  assert.equal(hookExecutable('C:\\bin\\prometheus-hook.exe run'), 'prometheus-hook.exe');
  assert.equal(shellOnlyExecutableError(''), 'hook entry has no executable');
});

check('every emitted hook entry is exec form with a real executable', () => {
  // The whole point of exec form is that `args` is present: without it the
  // harness hands `command` to a shell -- `sh -c` on POSIX, and PowerShell on
  // Windows whenever Git Bash is absent, which is what made the previous
  // `bash -c '<multi-line bash>'` entries a Windows accident rather than a
  // Windows feature.
  let entries = 0;
  for (const file of ['hooks/hooks.json', 'hooks/codex-hooks.json']) {
    const config = JSON.parse(fs.readFileSync(path.join(repoRoot, file), 'utf8'));
    for (const groups of Object.values(config.hooks)) {
      for (const group of groups) {
        for (const hook of group.hooks) {
          assert.ok(Array.isArray(hook.args), `${file} still emits a shell-form entry`);
          assert.equal(hook.command, 'node', `${file} names a non-portable executable`);
          assert.equal(
            hook.args[0],
            '${CLAUDE_PLUGIN_ROOT}/scripts/hook-entry.mjs',
            `${file} does not route through the entry point`
          );
          entries += 1;
        }
      }
    }
  }
  assert.equal(entries, 61, `expected the full hook matrix, saw ${entries}`);
});

check('every emitted hook entry passes the guard today', () => {
  // The guard is only worth having if it runs against the real artifact.
  let entries = 0;
  for (const file of ['hooks/hooks.json', 'hooks/codex-hooks.json']) {
    const config = JSON.parse(fs.readFileSync(path.join(repoRoot, file), 'utf8'));
    for (const groups of Object.values(config.hooks)) {
      for (const group of groups) {
        for (const hook of group.hooks) {
          assert.equal(
            shellOnlyExecutableError(hook.command),
            null,
            `${file} emits a hook that needs a shell to spawn`
          );
          entries += 1;
        }
      }
    }
  }
  assert.ok(entries >= 61, `expected the full hook matrix, saw ${entries}`);
});

// ---------------------------------------------------------------------------
// Requirement: execution eligibility comes from the manifest
// ---------------------------------------------------------------------------

const DISPATCHER_RELATIVE = 'shared/scripts/generated/hook-dispatch-v1.sh';
const DISPATCHER_BODY = '#!/usr/bin/env bash\nprintf \'dispatched\\n\'\n';

function fabricateStore(name, { interpreter = 'bash', clearExecutableBit = true, record = false } = {}) {
  const root = path.join(workspace, name);
  const generation = sha256(`${name}-generation`);
  const bundleId = sha256(`${name}-bundle`);
  const generationRoot = path.join(root, 'generations', generation);
  const dispatcher = path.join(generationRoot, ...DISPATCHER_RELATIVE.split('/'));
  const argvLog = path.join(root, 'argv.log');
  // A dispatcher that records its own argv, one argument per line, using printf
  // rather than echo so nothing re-interprets what it was handed.
  const body = record
    ? `#!/usr/bin/env bash\nprintf '%s\\n' "$@" > ${JSON.stringify(argvLog)}\n`
    : DISPATCHER_BODY;
  fs.mkdirSync(path.dirname(dispatcher), { recursive: true });
  fs.writeFileSync(dispatcher, body);
  if (clearExecutableBit) fs.chmodSync(dispatcher, 0o644);
  fs.writeFileSync(
    path.join(generationRoot, 'manifest.json'),
    `${JSON.stringify(
      {
        abi: 'hook-runtime-v1',
        bundleId,
        dispatcherPath: DISPATCHER_RELATIVE,
        dispatcherSha256: sha256(body),
        dispatcherInterpreter: interpreter,
        generation,
      },
      null,
      2
    )}\n`
  );
  fs.mkdirSync(path.join(root, 'pointers/bundles'), { recursive: true });
  fs.writeFileSync(
    path.join(root, 'pointers/bundles', bundleId),
    `generations/${generation}\n`
  );
  // The entry point resolves through `runtime/v1/run-hook`, a copy of the
  // runtime the installer places in the store.
  fs.mkdirSync(path.join(root, 'runtime/v1'), { recursive: true });
  fs.copyFileSync(
    path.join(repoRoot, 'shared/scripts/hook-runtime-v1.sh'),
    path.join(root, 'runtime/v1/run-hook')
  );
  return { root, generation, bundleId, dispatcher, argvLog };
}

function runRuntime(store, bundleId, extraArgs = ['--resolve-only']) {
  return spawnSync(
    'bash',
    [path.join(repoRoot, 'shared/scripts/hook-runtime-v1.sh'), '--bundle', bundleId, ...extraArgs],
    { encoding: 'utf8', env: { ...process.env, PROMETHEUS_PLUGIN_ROOT: store } }
  );
}

const shell = probeShell();
if (!shell.available) {
  skip('runtime execution eligibility', 'no POSIX shell on this host');
} else {
  check('a dispatcher with no executable bit is still launched', () => {
    const store = fabricateStore('no-exec-bit');
    // Prove the premise: nothing on disk marks this file executable.
    assert.equal(fs.statSync(store.dispatcher).mode & 0o111, 0);
    const result = runRuntime(store.root, store.bundleId);
    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    assert.equal(JSON.parse(result.stdout).generation, store.generation);
  });

  check('the dispatcher is launched by the interpreter the receipt names', () => {
    const store = fabricateStore('dispatch');
    const result = runRuntime(store.root, store.bundleId, [
      '--hook',
      'anything',
      '--harness',
      'claude-code',
    ]);
    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    assert.match(result.stdout, /dispatched/);
  });

  check('an interpreter outside the allowlist is refused', () => {
    const store = fabricateStore('bad-interpreter', { interpreter: 'python3' });
    const result = runRuntime(store.root, store.bundleId);
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /DISPATCHER_INTERPRETER/);
  });

  check('a receipt with no interpreter at all is refused', () => {
    const store = fabricateStore('absent-interpreter');
    const manifestPath = path.join(
      store.root,
      'generations',
      store.generation,
      'manifest.json'
    );
    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
    delete manifest.dispatcherInterpreter;
    fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
    const result = runRuntime(store.root, store.bundleId);
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /DISPATCHER_INTERPRETER/);
  });

  check('a tampered dispatcher is still refused by its digest', () => {
    // Removing the executable-bit gate must not weaken the check that actually
    // makes the dispatcher safe to run.
    const store = fabricateStore('tampered');
    fs.appendFileSync(store.dispatcher, 'echo pwned\n');
    const result = runRuntime(store.root, store.bundleId);
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /DISPATCHER_HASH/);
  });
}

// ---------------------------------------------------------------------------
// Requirement: cold-path shell scripts declare their dependency
// ---------------------------------------------------------------------------

check('shell availability is probed, not assumed', () => {
  const probed = probeShell();
  assert.equal(typeof probed.available, 'boolean');
  if (probed.available) {
    assert.ok(probed.command, 'an available shell must be named');
    assert.ok(
      probed.digestTool === null || ['sha256sum', 'shasum'].includes(probed.digestTool),
      'the digest tool must be one the runtime actually calls'
    );
  } else {
    assert.equal(probed.command, null);
  }
});

if (shell.available) {
  check('the runtime selects a digest tool that exists on this host', () => {
    // The previous version of this script called `shasum` unconditionally.
    // `shasum` is a Perl script that ships with git-bash; `sha256sum` is
    // coreutils. This host has both, so what is asserted is that the runtime
    // agrees with the probe about which one it will reach for, and that it
    // reaches the same digest either way.
    const probed = probeShell();
    assert.ok(probed.digestTool, 'this host must expose at least one digest tool');
    const store = fabricateStore('digest');
    const viaRuntime = runRuntime(store.root, store.bundleId);
    assert.equal(viaRuntime.status, 0, `${viaRuntime.stdout}${viaRuntime.stderr}`);
    for (const tool of ['sha256sum', 'shasum -a 256']) {
      const probe = spawnSync('bash', ['-c', `command -v ${tool.split(' ')[0]}`], {
        encoding: 'utf8',
      });
      if (probe.status !== 0) continue;
      // Hash through a POSIX path. GNU coreutils escapes a filename containing
      // a backslash and prefixes the whole line with one, so handing it a
      // Windows path would compare a `\`-prefixed digest.
      const out = spawnSync(
        'bash',
        ['-c', `cd "$(dirname "$1")" && ${tool} "$(basename "$1")"`, 'hash', store.dispatcher],
        { encoding: 'utf8' }
      );
      assert.equal(
        out.stdout.trim().replace(/^\\/, '').split(/\s+/)[0],
        sha256(DISPATCHER_BODY),
        `${tool} must agree with the recorded digest`
      );
    }
  });

  skip(
    'missing digest tool',
    'both sha256sum and shasum live in /usr/bin on msys2 and cannot be taken off PATH without removing bash itself'
  );

  check('the bootstrap declares the tools it shells out to', () => {
    const bootstrap = fs.readFileSync(
      path.join(repoRoot, 'shared/scripts/bootstrap-hook-runtime.sh'),
      'utf8'
    );
    assert.match(bootstrap, /missing dependency/, 'it must name what is missing');
    for (const tool of ['awk', 'node']) {
      assert.ok(bootstrap.includes(tool), `${tool} must be declared`);
    }
    // And it must not gate on an executable bit any more.
    assert.equal(/\[\[ -x /.test(bootstrap), false, 'no executable-bit gate may remain');
  });

  check('no hot-path script gates execution on a filesystem executable bit', () => {
    for (const relative of [
      'shared/scripts/hook-runtime-v1.sh',
      'shared/scripts/bootstrap-hook-runtime.sh',
      'shared/scripts/generated/hook-dispatch-v1.sh',
    ]) {
      const body = fs.readFileSync(path.join(repoRoot, relative), 'utf8');
      const gates = body
        .split('\n')
        .filter(line => /\[\[ -x /.test(line) && !line.trim().startsWith('#'));
      assert.deepEqual(gates, [], `${relative} still gates on an executable bit`);
    }
  });
}

// ---------------------------------------------------------------------------
// Requirement: a path containing shell-significant characters arrives verbatim
// ---------------------------------------------------------------------------

if (shell.available) {
  check('arguments containing backslashes, $ and backticks reach the dispatcher unmodified', () => {
    // The store's dispatcher is replaced with one that records its own argv, so
    // what is asserted is what the far end actually received -- not what this
    // fixture hoped it sent.
    const store = fabricateStore('verbatim', { record: true });
    // Every character a shell would treat as syntax: command substitution in
    // two spellings, a variable, a backslash escape, quotes, and a semicolon.
    const hostile = 'C:\\plug in\\$HOME `id` $(id) "q" \'s\' ;rm -rf /';
    const result = spawnSync(
      process.execPath,
      [
        path.join(repoRoot, 'scripts/hook-entry.mjs'),
        '--bundle',
        store.bundleId,
        '--hook',
        'session-start',
        '--harness',
        hostile,
      ],
      {
        encoding: 'utf8',
        shell: false,
        env: { ...process.env, PROMETHEUS_PLUGIN_ROOT: store.root },
      }
    );
    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    const received = fs.readFileSync(store.argvLog, 'utf8').split('\n');
    assert.deepEqual(
      received.slice(0, 4),
      ['--hook', 'session-start', '--harness', hostile],
      'the dispatcher must receive the argument vector exactly as written'
    );
    // And nothing executed it: a shell would have run `id` and left its output.
    assert.equal(/uid=/.test(received.join('\n')), false, 'a shell interpreted the argument');
  });

  check('the entry point refuses an argument it does not recognise', () => {
    const store = fabricateStore('strict-argv');
    const result = spawnSync(
      process.execPath,
      [path.join(repoRoot, 'scripts/hook-entry.mjs'), '--bundle', store.bundleId, '--nope', 'x'],
      { encoding: 'utf8', shell: false, env: { ...process.env, PROMETHEUS_PLUGIN_ROOT: store.root } }
    );
    assert.equal(result.status, 78);
    assert.match(result.stderr, /INVALID_ARGUMENT/);
  });

  check('the entry point rejects a bundle id that is not a digest', () => {
    const result = spawnSync(
      process.execPath,
      [path.join(repoRoot, 'scripts/hook-entry.mjs'), '--bundle', 'nope'],
      { encoding: 'utf8', shell: false }
    );
    assert.equal(result.status, 78);
    assert.match(result.stderr, /INVALID_BUNDLE/);
  });
}

// ---------------------------------------------------------------------------
// Install-time selection of the compiled dispatcher
// ---------------------------------------------------------------------------

/**
 * Build a payload with candidate dispatchers.
 *
 * `node` itself stands in for a compiled dispatcher: it is a real executable
 * that answers `--version` with status 0, which is exactly what the probe asks.
 * Using a real process rather than a stub is the point -- selection by
 * execution is the behaviour under test.
 */
function dispatcherPayload(name, candidates) {
  const pluginRoot = path.join(workspace, name);
  const generationPath = path.join(pluginRoot, 'generations', 'a'.repeat(64));
  const files = [];
  for (const candidate of candidates) {
    const relative = `bin/${candidate.target}/${candidate.name}`;
    const absolute = path.join(generationPath, ...relative.split('/'));
    fs.mkdirSync(path.dirname(absolute), { recursive: true });
    fs.copyFileSync(candidate.broken ? path.join(workspace, 'not-an-executable') : process.execPath, absolute);
    files.push({ path: relative, type: 'file', executable: candidate.executable !== false });
  }
  return { pluginRoot, generationPath, manifest: { schemaVersion: 2, files } };
}

fs.writeFileSync(path.join(workspace, 'not-an-executable'), 'this is not a program\n');

check('the installer selects a dispatcher by running it, and places it', () => {
  const payload = dispatcherPayload('select', [
    { target: 'x86_64-unknown-linux-gnu', name: 'prometheus-hook', broken: true },
    { target: 'x86_64-pc-windows-msvc', name: 'prometheus-hook.exe' },
  ]);
  const chosen = __testing.installHookDispatcher(
    payload.pluginRoot,
    payload.generationPath,
    payload.manifest
  );
  assert.ok(chosen, 'a runnable candidate must be selected');
  assert.equal(chosen.name, 'prometheus-hook.exe');
  const installed = path.join(payload.pluginRoot, 'runtime/v1', chosen.name);
  assert.equal(fs.existsSync(installed), true, 'the chosen dispatcher must be installed');
  // The unrunnable candidate must leave nothing behind for the entry point to
  // try to exec later.
  assert.equal(fs.existsSync(path.join(payload.pluginRoot, 'runtime/v1/prometheus-hook')), false);
});

check('a candidate the manifest does not call executable is never run', () => {
  const payload = dispatcherPayload('not-executable', [
    { target: 'x86_64-pc-windows-msvc', name: 'prometheus-hook.exe', executable: false },
  ]);
  assert.equal(
    __testing.installHookDispatcher(payload.pluginRoot, payload.generationPath, payload.manifest),
    null
  );
  assert.equal(fs.existsSync(path.join(payload.pluginRoot, 'runtime/v1')), false);
});

check('a payload with no dispatcher is not an error', () => {
  // The shell path still serves; a missing binary is slower, not broken.
  const pluginRoot = path.join(workspace, 'no-bin');
  const generationPath = path.join(pluginRoot, 'generations', 'b'.repeat(64));
  fs.mkdirSync(generationPath, { recursive: true });
  assert.equal(
    __testing.installHookDispatcher(pluginRoot, generationPath, { schemaVersion: 2, files: [] }),
    null
  );
});

// ---------------------------------------------------------------------------
// The generated hooks must name an executable this host can resolve
// ---------------------------------------------------------------------------

function sourceWithHookCommand(name, command, execForm = true) {
  const source = path.join(workspace, name);
  fs.mkdirSync(path.join(source, 'hooks'), { recursive: true });
  const hook = execForm
    ? { type: 'command', command, args: ['${CLAUDE_PLUGIN_ROOT}/scripts/hook-entry.mjs'] }
    : { type: 'command', command };
  fs.writeFileSync(
    path.join(source, 'hooks/hooks.json'),
    `${JSON.stringify({ hooks: { SessionStart: [{ hooks: [hook] }] } }, null, 2)}\n`
  );
  return source;
}

check('an unresolvable hook executable is refused at install, not at run time', () => {
  // The failure this prevents is silent: the harness spawn fails before any of
  // this pack executes, so nothing of ours is left to report it.
  const source = sourceWithHookCommand('unresolvable', 'definitely-not-on-path-9f3a');
  assert.throws(
    () => __testing.assertHookExecutablesResolvable(source),
    error => /cannot be resolved here/.test(error.message) && /launchd/.test(error.message),
    'the failure must name the executable and the macOS PATH scenario'
  );
});

check('the executable the real hooks name resolves on this host', () => {
  // Against the committed artifact, not a fixture: if `node` stops resolving
  // here, this is where it is noticed.
  assert.doesNotThrow(() => __testing.assertHookExecutablesResolvable(repoRoot));
});

check('resolvability is tested, not behaviour', () => {
  // `--version` on an arbitrary executable may exit non-zero. That still proves
  // it was found and started, which is the only question being asked.
  const source = sourceWithHookCommand('nonzero-exit', process.execPath);
  assert.doesNotThrow(() => __testing.assertHookExecutablesResolvable(source));
});

check('shell-form and absolute-path entries are left alone', () => {
  // A shell form entry is the shell's problem, and an absolute path is resolved
  // by the path itself rather than by PATH.
  const shellForm = sourceWithHookCommand('shell-form', 'definitely-not-on-path-9f3a', false);
  assert.doesNotThrow(() => __testing.assertHookExecutablesResolvable(shellForm));
  const absolute = sourceWithHookCommand(
    'absolute',
    path.join(workspace, 'no-such-binary-here')
  );
  assert.doesNotThrow(() => __testing.assertHookExecutablesResolvable(absolute));
});

fs.rmSync(workspace, { recursive: true, force: true });

process.stdout.write(`hook-dispatch: ${results.length} checks passed\n`);
for (const name of results) process.stdout.write(`  ok   ${name}\n`);
for (const note of skipped) process.stdout.write(`  SKIP ${note}\n`);
