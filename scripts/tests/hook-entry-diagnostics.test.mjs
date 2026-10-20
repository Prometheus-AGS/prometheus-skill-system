// Real hook-entry processes must diagnose missing installation without mutations.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const base = fs.mkdtempSync(path.join(os.tmpdir(), 'hook entry diagnostics '));
const home = path.join(base, 'home'), payload = path.join(base, 'payload');
fs.mkdirSync(home); fs.cpSync(path.join(root, 'dist/plugins/claude/prometheus-skill-pack'), payload, { recursive: true });
const release = JSON.parse(fs.readFileSync(path.join(payload, 'shared/harnesses/generated/release-manifest.json')));
const store = path.join(home, '.prometheus/plugins/prometheus-skill-pack');
function hook(bundle = release.bundleId, pluginRoot = payload) {
  const result = spawnSync(process.execPath, [path.join(payload, 'scripts/hook-entry.mjs'),
    '--bundle', bundle, '--hook', 'learning-stop', '--harness', 'claude-code'], {
    cwd: base, encoding: 'utf8', input: '{}', timeout: 10000,
    env: { PATH: process.env.PATH, HOME: home, CODEX_HOME: path.join(home, '.codex'),
      PROMETHEUS_PLUGIN_ROOT: store, CLAUDE_PLUGIN_ROOT: pluginRoot },
  });
  assert.equal(result.status, 78, result.stderr);
  assert(!fs.existsSync(store), 'a failed hook must not create or mutate an installation');
  return JSON.parse(result.stderr.trim());
}
try {
  assert.equal(hook().code, 'NOT_ACTIVATED');
  assert.equal(hook('0'.repeat(64)).code, 'BUNDLE_IDENTITY_MISMATCH');
  assert.equal(hook(release.bundleId, '').code, 'NOT_ACTIVATED');
  fs.rmSync(path.join(payload, 'shared/harnesses/generated/release-manifest.json'));
  assert.equal(hook().code, 'PAYLOAD_INCOMPLETE');
  console.log('PASS: real hook diagnostics refuse activation without creating a store');
} finally { fs.rmSync(base, { recursive: true, force: true }); }
