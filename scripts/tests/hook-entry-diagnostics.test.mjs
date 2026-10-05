/**
 * `hook-entry.mjs` must say what is wrong, and what to do, when a plugin payload
 * cannot activate. Before this, an incomplete payload surfaced as a raw Node stack
 * trace ("Failed with non-blocking status code: node:internal/modules/esm/resolve…")
 * on every Stop hook of every turn, with no hint that the plugin install was the
 * problem.
 *
 * Each case runs the real hook entry from a copy of the built payload with an empty
 * HOME, so activation takes the bootstrap path. Local only.
 */
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const payloadSource = path.join(root, 'dist/plugins/claude/prometheus-skill-pack');

function sandbox() {
  const base = fs.mkdtempSync(path.join(os.tmpdir(), 'hook-entry-diag-'));
  const payload = path.join(base, 'cache', 'prometheus-skill-pack');
  fs.mkdirSync(path.dirname(payload), { recursive: true });
  fs.mkdirSync(path.join(base, 'home'), { recursive: true });
  fs.cpSync(payloadSource, payload, { recursive: true, verbatimSymlinks: true });
  return { base, payload, home: path.join(base, 'home') };
}

function stopHook(payload) {
  const manifest = JSON.parse(fs.readFileSync(path.join(payload, 'hooks/hooks.json'), 'utf8'));
  const hook = manifest.hooks.Stop[0].hooks[0];
  return hook.args.map(arg => arg.replaceAll('${CLAUDE_PLUGIN_ROOT}', payload));
}

function runHook(box, { env = {}, pluginRoot = box.payload } = {}) {
  const run = spawnSync('node', stopHook(box.payload), {
    cwd: box.base,
    input: JSON.stringify({ session_id: 'diag', cwd: box.base, hook_event_name: 'Stop' }),
    encoding: 'utf8',
    timeout: 120_000,
    env: {
      ...process.env,
      HOME: box.home,
      ...(pluginRoot ? { CLAUDE_PLUGIN_ROOT: pluginRoot } : { CLAUDE_PLUGIN_ROOT: '' }),
      ...env,
    },
  });
  const jsonLines = (run.stderr ?? '')
    .split('\n')
    .filter(line => line.startsWith('{'))
    .map(line => {
      try { return JSON.parse(line); } catch { return null; }
    })
    .filter(Boolean);
  return { run, report: jsonLines.find(entry => entry.status === 'HOOK_RUNTIME_ERROR') };
}

const withBox = fn => {
  const box = sandbox();
  try { return fn(box); } finally { fs.rmSync(box.base, { recursive: true, force: true }); }
};

// A healthy payload stays silent and succeeds.
withBox(box => {
  const { run } = runHook(box);
  assert.equal(run.status, 0, run.stderr);
  assert.equal((run.stderr ?? '').trim(), '');
});

// A payload missing a module its installer imports is named, with a remedy and no stack.
withBox(box => {
  fs.rmSync(path.join(box.payload, 'scripts/lib/capabilities.js'));
  const { run, report } = runHook(box);
  assert.notEqual(run.status, 0);
  assert(report, `expected a structured HOOK_RUNTIME_ERROR report, got:\n${run.stderr.slice(0, 500)}`);
  assert.equal(report.code, 'PAYLOAD_INCOMPLETE');
  assert(report.message.includes(box.payload), 'the message names the plugin root');
  assert(report.message.includes('capabilities.js'), 'the message names the missing module');
  assert(/update or reinstall/i.test(report.message), 'the message states the remedy');
  assert(!/\n\s+at /.test(run.stderr), `no raw stack trace by default:\n${run.stderr.slice(0, 500)}`);
});

// Debug mode keeps the raw child output for diagnosis.
withBox(box => {
  fs.rmSync(path.join(box.payload, 'scripts/lib/capabilities.js'));
  const { run } = runHook(box, { env: { PROMETHEUS_HOOK_DEBUG: '1' } });
  assert(/ERR_MODULE_NOT_FOUND/.test(run.stderr), 'PROMETHEUS_HOOK_DEBUG=1 relays the raw bootstrap error');
});

// Any other bootstrap failure is reported as BOOTSTRAP_FAILED with its first line.
withBox(box => {
  fs.writeFileSync(
    path.join(box.payload, 'scripts/install-plugin-generation.js'),
    "throw new Error('synthetic bootstrap failure');\n"
  );
  const { run, report } = runHook(box);
  assert.notEqual(run.status, 0);
  assert(report, `expected a structured report, got:\n${run.stderr.slice(0, 500)}`);
  assert.equal(report.code, 'BOOTSTRAP_FAILED');
  assert(report.message.includes('synthetic bootstrap failure'));
  assert(!/\n\s+at /.test(run.stderr), 'no raw stack trace by default');
});

// No plugin root at all is still NOT_ACTIVATED, now saying so plainly.
withBox(box => {
  const { run, report } = runHook(box, { pluginRoot: '' });
  assert.notEqual(run.status, 0);
  assert.equal(report?.code, 'NOT_ACTIVATED');
  assert(/plugin root/i.test(report.message));
});

console.log('PASS: hook-entry reports why a payload cannot activate, and what to do');
