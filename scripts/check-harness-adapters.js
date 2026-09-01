#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { probeFilesystemCapabilities } from './lib/capabilities.js';
import { shellOnlyExecutableError } from './lib/hook-config.js';
import { readIngestOracle } from './lib/payload-manifest.js';

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
  const harnessHooks = contract.events
    .filter(group => group.harnesses === undefined || group.harnesses.includes(harness))
    .flatMap(group => group.hooks);
  const emitted = Object.values(manifest.hooks ?? {}).flatMap(groups =>
    groups.flatMap(group => group.hooks ?? [])
  );
  if (emitted.length !== harnessHooks.length) {
    failures.push(`${harness}: expected ${harnessHooks.length} hooks, found ${emitted.length}`);
  }
  // These assertions moved from the command STRING to the argument VECTOR when
  // hooks became exec form. The properties are unchanged and are now checked
  // more exactly: an argument either equals the bundle id or it does not, where
  // the old substring match on a quoted string could be satisfied by an
  // accident of quoting.
  for (const hook of emitted) {
    if (hook.type !== 'command' || !Array.isArray(hook.args)) {
      failures.push(`${harness}: hook is not exec form, so a shell would parse it`);
      continue;
    }
    if (shellOnlyExecutableError(hook.command)) {
      failures.push(`${harness}: hook executable cannot be spawned without a shell`);
    }
    const entry = hook.args[0] ?? '';
    if (!entry.startsWith('${CLAUDE_PLUGIN_ROOT}/')) {
      failures.push(`${harness}: hook entry point is not resolved from the plugin root`);
    }
    if (!entry.endsWith('/scripts/hook-entry.mjs')) {
      failures.push(`${harness}: hook bypasses the guarded entry point`);
    }
    const flag = name => {
      const index = hook.args.indexOf(name);
      return index < 0 ? null : (hook.args[index + 1] ?? null);
    };
    if (flag('--bundle') !== release.bundleId) {
      failures.push(`${harness}: hook does not embed release bundle ${release.bundleId}`);
    }
    if (flag('--harness') !== harness) {
      failures.push(`${harness}: hook carries a different harness identity`);
    }
    if (!flag('--hook')) {
      failures.push(`${harness}: hook does not name a hook id`);
    }
    // The entry point is the ONE cache-resident script a hook may reach; it
    // performs the guarded acquisition itself. Anything else under the plugin
    // root would be a second, unguarded execution dependency.
    const cacheScripts = hook.args.filter(value => value.includes('${CLAUDE_PLUGIN_ROOT}'));
    if (cacheScripts.length !== 1) {
      failures.push(`${harness}: hook has a non-canonical cache execution dependency`);
    }
    if (hook.args.some(value => value.includes('/stable/') || value.includes('/current/'))) {
      failures.push(`${harness}: hook resolves through mutable runtime state`);
    }
  }
}

for (const hook of contractHooks) {
  if (!dispatcher.includes(`  '${hook.id}')`)) {
    failures.push(`dispatcher omits contract hook: ${hook.id}`);
  }
  for (const [harness, manifest] of Object.entries(manifests)) {
    const group = contract.events.find(candidate =>
      candidate.hooks.some(entry => entry.id === hook.id)
    );
    const expected = group.harnesses === undefined || group.harnesses.includes(harness);
    // Exec form again: the hook id and harness are ARGUMENTS now, so matching a
    // quoted `'id' 'harness'` pair in the serialized JSON no longer applies.
    // Matching the argument vector is also exact rather than substring-based.
    const emittedIds = new Set(
      Object.values(manifest.hooks ?? {})
        .flatMap(groups => groups.flatMap(entry => entry.hooks ?? []))
        .filter(entry => Array.isArray(entry.args))
        .filter(entry => entry.args[entry.args.indexOf('--harness') + 1] === harness)
        .map(entry => entry.args[entry.args.indexOf('--hook') + 1])
    );
    if (expected && !emittedIds.has(hook.id)) {
      failures.push(`${harness}: manifest omits contract hook: ${hook.id}`);
    } else if (!expected && emittedIds.has(hook.id)) {
      failures.push(`${harness}: manifest emitted filtered hook: ${hook.id}`);
    }
  }
}

for (const harness of ['claude-code', 'codex', 'opencode', 'kimi']) {
  const events = read('shared/harnesses/capabilities.json').harnesses[harness]?.events ?? {};
  for (const required of ['sessionStart', 'preCompact', 'postCompact', 'stop']) {
    if (!(required in events)) failures.push(`${harness}: capability omits ${required}`);
  }
}

// Executability is asserted against an authority the host can express.
//
// `stat().mode & 0o111` is zero for EVERY file on a volume that cannot record a
// permission bit, so this check used to fail on Windows for three files that
// are correctly marked 100755 in git. Where the filesystem can answer, it is
// still the one asked; where it cannot, git's index is.
const hostCapabilities = probeFilesystemCapabilities(root);
const oracle = readIngestOracle(root);
// The guarded acquisition path moved into the entry point when hooks became
// exec form. Assert it is still there rather than assuming it.
const hookEntry = fs.readFileSync(path.join(root, 'scripts/hook-entry.mjs'), 'utf8');
for (const required of ['runtime/v1/run-hook', 'bootstrap-hook-runtime.sh', 'runtime/v1']) {
  if (!hookEntry.includes(required)) {
    failures.push(`hook entry point omits ${required}`);
  }
}
if (/shell:\s*true/.test(hookEntry)) {
  failures.push('hook entry point spawns through a shell');
}

for (const executable of [
  'shared/scripts/hook-runtime-v1.sh',
  'shared/scripts/bootstrap-hook-runtime.sh',
  'shared/scripts/generated/hook-dispatch-v1.sh',
]) {
  if (hostCapabilities.executableBit) {
    if ((fs.statSync(path.join(root, executable)).mode & 0o111) === 0) {
      failures.push(`${executable} is not executable`);
    }
    continue;
  }
  const recorded = oracle?.get(executable);
  if (!recorded)
    failures.push(`${executable} is untracked, so its executable bit cannot be verified`);
  else if (!recorded.executable) failures.push(`${executable} is not recorded as executable`);
}

if (failures.length) {
  console.error(failures.join('\n'));
  process.exit(1);
}

console.log(
  `Harness runtime parity: ${Object.entries(manifests)
    .map(([harness, manifest]) => {
      const count = Object.values(manifest.hooks ?? {}).flatMap(groups =>
        groups.flatMap(group => group.hooks ?? [])
      ).length;
      return `${harness}=${count}`;
    })
    .join(', ')} → bundle ${release.bundleId}`
);
