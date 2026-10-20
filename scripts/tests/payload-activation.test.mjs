/** Real explicit installer plus every packaged hook, in isolated homes. */
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
// PAYLOAD_ACTIVATION_SOURCE points the check at another payload (for example a released
// plugin cache) to prove it reports a known-broken one.
// Both distributed payloads ship hooks, so both must activate; the override checks one payload.
const payloadSources = process.env.PAYLOAD_ACTIVATION_SOURCE
  ? [process.env.PAYLOAD_ACTIVATION_SOURCE]
  : ['claude', 'codex'].map(client => path.join(root, 'dist/plugins', client, 'prometheus-skill-pack'));
const payloadSource = payloadSources[0];
const PER_HOOK_TIMEOUT_MS = Number(process.env.PAYLOAD_ACTIVATION_HOOK_TIMEOUT_MS ?? 90_000);
const FAILURE_MARKERS = [/ERR_MODULE_NOT_FOUND/, /Cannot find module/, /HOOK_RUNTIME_ERROR/, /NOT_ACTIVATED/];

function makeSandbox(label, source = payloadSource) {
  const base = fs.mkdtempSync(path.join(os.tmpdir(), `payload-activation-${label}-`));
  const payload = path.join(base, 'cache', 'prometheus-skill-pack');
  const home = path.join(base, 'home');
  const project = path.join(base, 'project');
  fs.mkdirSync(path.dirname(payload), { recursive: true });
  fs.mkdirSync(home, { recursive: true });
  fs.mkdirSync(project, { recursive: true });
  fs.cpSync(source, payload, { recursive: true, verbatimSymlinks: true });
  return { base, payload, home, project };
}

function declaredHooks(payload) {
  const manifest = JSON.parse(fs.readFileSync(path.join(payload, 'hooks/hooks.json'), 'utf8'));
  const hooks = [];
  for (const [event, entries] of Object.entries(manifest.hooks)) {
    for (const entry of entries) {
      for (const hook of entry.hooks ?? []) {
        if (hook.type !== 'command') continue;
        // Claude declares exec form (command + args); Codex declares one shell command string.
        const shellForm = !Array.isArray(hook.args);
        const substitute = text => text.replaceAll('${CLAUDE_PLUGIN_ROOT}', payload);
        const args = (hook.args ?? []).map(substitute);
        const command = shellForm ? substitute(hook.command) : hook.command;
        const words = shellForm ? command.split(/\s+/) : args;
        const name = words.includes('--hook') ? words[words.indexOf('--hook') + 1] : hook.command;
        hooks.push({ event, name, command, args, shellForm, timeoutMs: Number.isFinite(hook.timeout) ? hook.timeout * 1000 : null });
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
    const run = spawnSync(hook.command, hook.shellForm ? [] : hook.args, {
      shell: hook.shellForm,
      cwd: sandbox.project,
      input: payload,
      encoding: 'utf8',
      // The harness kills a hook at its declared timeout, so every warm hook must finish inside it.
      timeout: hook.timeoutMs ?? PER_HOOK_TIMEOUT_MS,
      env: isolatedEnv(sandbox),
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

function isolatedEnv(box) {
  return { ...process.env, HOME: box.home, CODEX_HOME: path.join(box.home, '.codex'),
    CLAUDE_CONFIG_DIR: path.join(box.home, '.claude'), CLAUDE_PLUGIN_ROOT: box.payload,
    PROMETHEUS_PLUGIN_ROOT: path.join(box.home, '.prometheus/plugins/prometheus-skill-pack'),
    XDG_CONFIG_HOME: path.join(box.home, '.config'), XDG_DATA_HOME: path.join(box.home, '.local/share'),
    PYTHONDONTWRITEBYTECODE: '0' };
}
function install(box) {
  return spawnSync(process.execPath, [path.join(box.payload, 'scripts/install-plugin-generation.js'),
    '--source-root', box.payload, '--home', box.home, '--targets', 'all'],
    { env: isolatedEnv(box), cwd: box.project, encoding: 'utf8', timeout: 180_000 });
}

const failures = results => results.filter(r => r.timedOut || r.status !== 0 || r.marker);

// 1. Each shipped payload activates from nothing and runs every hook it declares.
for (const source of payloadSources) {
  const sandbox = makeSandbox(path.basename(path.dirname(source)) || 'shipped', source);
  try {
    const installed = install(sandbox);
    assert.equal(installed.status, 0, installed.stderr);
    const results = activate(sandbox);
    assert(results.length >= 30, `expected the full hook set from ${source}, ran ${results.length}`);
    const bad = failures(results);
    assert.deepEqual(
      bad.map(r => `${r.event}/${r.name}: ${r.timedOut ? 'timed out' : `exit ${r.status}`} ${r.marker ?? ''}`),
      [],
      `hooks failed to activate from the shipped payload ${source}:\n${bad.map(r => r.output.slice(0, 400)).join('\n---\n')}`
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
    const result = install(sandbox);
    assert.notEqual(result.status, 0, 'incomplete payload must not explicitly install');
    assert.match(result.stderr, /capabilities.js/);
  } finally {
    fs.rmSync(sandbox.base, { recursive: true, force: true });
  }
}

console.log('PASS: both shipped payloads explicitly install and run all hooks; incomplete input is rejected');
