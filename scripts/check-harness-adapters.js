#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const read = relative => JSON.parse(fs.readFileSync(path.join(root, relative), 'utf8'));
const contract = read('shared/harnesses/hook-contract.json');
const release = read('shared/harnesses/generated/release-manifest.json');
const plugin = read('dist/plugins/codex/prometheus-skill-pack/.codex-plugin/plugin.json');
const manifests = {
  'claude-code': read('hooks/hooks.json'),
  codex: read('hooks/codex-hooks.json'),
};
const dispatcher = fs.readFileSync(
  path.join(root, 'shared/scripts/generated/hook-dispatch-v1.sh'),
  'utf8'
);
const failures = [];

function sha256(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

const contractHooks = contract.events.flatMap(group => group.hooks);
const ids = new Set(contractHooks.map(hook => hook.id));
if (ids.size !== contractHooks.length) failures.push('hook contract contains duplicate ids');
if (plugin.hooks !== undefined) {
  failures.push('Codex native package must omit the rejected hooks field');
}

const releaseDispatcher = release.runtimeFiles.find(
  entry => entry.path === 'shared/scripts/generated/hook-dispatch-v1.sh'
);
if (!releaseDispatcher || releaseDispatcher.sha256 !== sha256(dispatcher)) {
  failures.push('release manifest does not bind the generated dispatcher bytes');
}
if (!/^[a-f0-9]{64}$/.test(release.bundleId ?? '')) {
  failures.push('release manifest has no valid bundle identity');
}

for (const [harness, manifest] of Object.entries(manifests)) {
  const emitted = Object.values(manifest.hooks ?? {}).flatMap(groups =>
    groups.flatMap(group => group.hooks ?? [])
  );
  if (emitted.length !== contractHooks.length) {
    failures.push(`${harness}: expected ${contractHooks.length} hooks, found ${emitted.length}`);
  }
  for (const hook of emitted) {
    const command = hook.command ?? '';
    if (!command.includes('/runtime/v1/run-hook')) {
      failures.push(`${harness}: hook bypasses runtime/v1`);
    }
    if (!command.includes(`'${release.bundleId}'`)) {
      failures.push(`${harness}: hook does not embed release bundle ${release.bundleId}`);
    }
    if (!command.includes(`'${harness}'`)) {
      failures.push(`${harness}: hook carries a different harness identity`);
    }
    if (!command.includes('bootstrap-hook-runtime.sh')) {
      failures.push(`${harness}: hook omits the guarded acquisition path`);
    }
    const cacheScriptReferences = command.match(/\$plugin_root\/shared\/scripts\//g) ?? [];
    if (cacheScriptReferences.length !== 1) {
      failures.push(`${harness}: hook has a non-canonical cache execution dependency`);
    }
    if (command.includes('/stable/') || command.includes('/current/')) {
      failures.push(`${harness}: hook resolves through mutable runtime state`);
    }
  }
}

for (const hook of contractHooks) {
  if (!dispatcher.includes(`  '${hook.id}')`)) {
    failures.push(`dispatcher omits contract hook: ${hook.id}`);
  }
  for (const [harness, manifest] of Object.entries(manifests)) {
    const serialized = JSON.stringify(manifest);
    if (!serialized.includes(`'${hook.id}' '${harness}'`)) {
      failures.push(`${harness}: manifest omits contract hook: ${hook.id}`);
    }
  }
}

for (const harness of ['claude-code', 'codex', 'opencode', 'kimi']) {
  const events = read('shared/harnesses/capabilities.json').harnesses[harness]?.events ?? {};
  for (const required of ['sessionStart', 'preCompact', 'postCompact', 'stop']) {
    if (!(required in events)) failures.push(`${harness}: capability omits ${required}`);
  }
}

for (const executable of [
  'shared/scripts/hook-runtime-v1.sh',
  'shared/scripts/bootstrap-hook-runtime.sh',
  'shared/scripts/generated/hook-dispatch-v1.sh',
]) {
  const mode = fs.statSync(path.join(root, executable)).mode;
  if ((mode & 0o111) === 0) failures.push(`${executable} is not executable`);
}

if (failures.length) {
  console.error(failures.join('\n'));
  process.exit(1);
}

console.log(
  `Harness runtime parity: ${contractHooks.length} hooks × 2 manifests → bundle ${release.bundleId}`
);
