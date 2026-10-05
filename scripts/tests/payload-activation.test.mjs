/**
 * Activation closure for the distributed Claude plugin payload.
 *
 * The distribution test proves the payload's files exist and that its relative JS
 * imports resolve. It cannot prove the payload ACTIVATES: that needs the payload to
 * run the way a user's machine runs it. This test copies the built payload to a temp
 * directory (as a plugin cache would hold it), points HOME at an empty directory (so
 * no generation is registered and `hook-entry.mjs` must take the bootstrap path),
 * and executes every hook the shipped hooks.json declares, in order.
 *
 * It is the check that would have caught the 1.10.0/1.11.0 payloads, which shipped
 * `install-plugin-generation.js` without five of its `scripts/lib` imports: every Stop
 * hook died with ERR_MODULE_NOT_FOUND on a machine whose generation was older than the
 * payload. The second case proves the detector detects, by removing one of those
 * modules and requiring the failure to be reported.
 *
 * Local only (no hosted CI). Needs bash and node, like the hooks themselves.
 */
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
// PAYLOAD_ACTIVATION_SOURCE points the check at another payload (for example a released
// plugin cache) to prove it reports a known-broken one.
const payloadSource =
  process.env.PAYLOAD_ACTIVATION_SOURCE ?? path.join(root, 'dist/plugins/claude/prometheus-skill-pack');
const PER_HOOK_TIMEOUT_MS = Number(process.env.PAYLOAD_ACTIVATION_HOOK_TIMEOUT_MS ?? 90_000);
const FAILURE_MARKERS = [/ERR_MODULE_NOT_FOUND/, /Cannot find module/, /HOOK_RUNTIME_ERROR/, /NOT_ACTIVATED/];

function makeSandbox(label) {
  const base = fs.mkdtempSync(path.join(os.tmpdir(), `payload-activation-${label}-`));
  const payload = path.join(base, 'cache', 'prometheus-skill-pack');
  const home = path.join(base, 'home');
  const project = path.join(base, 'project');
  fs.mkdirSync(path.dirname(payload), { recursive: true });
  fs.mkdirSync(home, { recursive: true });
  fs.mkdirSync(project, { recursive: true });
  fs.cpSync(payloadSource, payload, { recursive: true, verbatimSymlinks: true });
  return { base, payload, home, project };
}

function declaredHooks(payload) {
  const manifest = JSON.parse(fs.readFileSync(path.join(payload, 'hooks/hooks.json'), 'utf8'));
  const hooks = [];
  for (const [event, entries] of Object.entries(manifest.hooks)) {
    for (const entry of entries) {
      for (const hook of entry.hooks ?? []) {
        if (hook.type !== 'command') continue;
        const args = (hook.args ?? []).map(arg => arg.replaceAll('${CLAUDE_PLUGIN_ROOT}', payload));
        const name = args.includes('--hook') ? args[args.indexOf('--hook') + 1] : hook.command;
        hooks.push({ event, name, command: hook.command, args });
      }
    }
  }
  return hooks;
}

/** Run every declared hook in order against one sandbox; return one result per hook. */
function activate(sandbox) {
  const results = [];
  for (const hook of declaredHooks(sandbox.payload)) {
    const payload = JSON.stringify({
      session_id: 'payload-activation',
      cwd: sandbox.project,
      hook_event_name: hook.event,
      stop_hook_active: false,
      source: 'startup',
      prompt: 'hello',
      tool_name: 'Bash',
      tool_input: { command: 'true' },
    });
    const run = spawnSync(hook.command, hook.args, {
      cwd: sandbox.project,
      input: payload,
      encoding: 'utf8',
      timeout: PER_HOOK_TIMEOUT_MS,
      env: { ...process.env, HOME: sandbox.home, CLAUDE_PLUGIN_ROOT: sandbox.payload },
    });
    const output = `${run.stdout ?? ''}\n${run.stderr ?? ''}`;
    results.push({
      event: hook.event,
      name: hook.name,
      status: run.status,
      timedOut: run.error?.code === 'ETIMEDOUT',
      marker: FAILURE_MARKERS.find(marker => marker.test(output))?.source ?? null,
      output,
    });
  }
  return results;
}

const failures = results => results.filter(r => r.timedOut || r.status !== 0 || r.marker);

// 1. The shipped payload activates from nothing and runs every hook it declares.
{
  const sandbox = makeSandbox('shipped');
  try {
    const results = activate(sandbox);
    assert(results.length >= 30, `expected the full hook set, ran ${results.length}`);
    const bad = failures(results);
    assert.deepEqual(
      bad.map(r => `${r.event}/${r.name}: ${r.timedOut ? 'timed out' : `exit ${r.status}`} ${r.marker ?? ''}`),
      [],
      `hooks failed to activate from the shipped payload:\n${bad.map(r => r.output.slice(0, 400)).join('\n---\n')}`
    );
  } finally {
    fs.rmSync(sandbox.base, { recursive: true, force: true });
  }
}

// 2. The detector detects: a payload missing a module its installer imports is reported.
{
  const sandbox = makeSandbox('mutated');
  try {
    fs.rmSync(path.join(sandbox.payload, 'scripts/lib/capabilities.js'));
    const [first] = activate(sandbox).slice(0, 1);
    assert.notEqual(first.status, 0, 'a payload missing scripts/lib/capabilities.js must not activate');
    assert(first.marker, `the failure must be recognisable, got: ${first.output.slice(0, 300)}`);
  } finally {
    fs.rmSync(sandbox.base, { recursive: true, force: true });
  }
}

console.log('PASS: the shipped payload activates from a clean HOME and an incomplete one is detected');
