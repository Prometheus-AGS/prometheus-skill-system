#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { probeFilesystemCapabilities } from './lib/capabilities.js';
import { shellOnlyExecutableError } from './lib/hook-config.js';
import { readIngestOracle } from './lib/payload-manifest.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const check = process.argv.includes('--check');
const contractPath = path.join(root, 'shared/harnesses/hook-contract.json');
const capabilitiesPath = path.join(root, 'shared/harnesses/capabilities.json');
const contract = JSON.parse(fs.readFileSync(contractPath, 'utf8'));
const capabilities = JSON.parse(fs.readFileSync(capabilitiesPath, 'utf8'));
const sourceVersion = String(
  JSON.parse(fs.readFileSync(path.join(root, 'skill-system.json'), 'utf8')).releaseVersion
);
const failures = [];

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map(key => [key, canonical(value[key])])
    );
  }
  return value;
}

function canonicalJson(value) {
  return `${JSON.stringify(canonical(value), null, 2)}\n`;
}

function sha256(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

// Bundle identity must not depend on the host that generated it.
//
// This file used to record a full permission mode per runtime file. That folded
// the authoring host's umask into `bundleId`, and made the identity flatly
// unreproducible on Windows, where libuv reports 0o444 or 0o666 for every file
// and nothing else. Schema 2 records a normalized executable bit taken from
// git's index -- the only authority that is the same on every host -- and falls
// back to the filesystem only where the filesystem can actually answer.
const hostCapabilities = probeFilesystemCapabilities(root);
const oracle = readIngestOracle(root);

function executableOf(relative) {
  const recorded = oracle?.get(relative);
  if (recorded) return recorded.type === 'file' && recorded.executable;
  if (hostCapabilities.executableBit) {
    const stat = fs.statSync(path.join(root, ...relative.split('/')), { throwIfNoEntry: false });
    return Boolean(stat) && (stat.mode & 0o100) !== 0;
  }
  failures.push(
    `no portable executable-bit authority for ${relative}: this volume cannot observe an ` +
      'executable bit and the file is not tracked by git'
  );
  return false;
}

// The one interpreter the dispatcher may be launched with. The runtime
// allowlists the same value, so widening this needs both to change.
const DISPATCHER_INTERPRETER = 'bash';

// Path of the exec-form entry point, relative to the plugin root.
const HOOK_ENTRY = 'scripts/hook-entry.mjs';

function shellQuote(value) {
  return `'${String(value).replaceAll("'", `'"'"'`)}'`;
}

function validateContract() {
  if (contract.schemaVersion !== 'hook-contract-v1') failures.push('unsupported hook contract');
  if (contract.dispatcherAbi !== 'hook-runtime-v1') failures.push('unsupported dispatcher ABI');
  const ids = new Set();
  const supportedHarnesses = new Set(contract.harnesses ?? []);
  for (const group of contract.events ?? []) {
    if (!group.event || !Array.isArray(group.hooks) || group.hooks.length === 0) {
      failures.push('hook contract contains an empty or unnamed event group');
      continue;
    }
    if (group.harnesses !== undefined) {
      if (
        !Array.isArray(group.harnesses) ||
        group.harnesses.length === 0 ||
        new Set(group.harnesses).size !== group.harnesses.length ||
        group.harnesses.some(harness => !supportedHarnesses.has(harness))
      ) {
        failures.push(`${group.event}: harness filter is empty, duplicated, or unsupported`);
      }
    }
    for (const hook of group.hooks) {
      if (!/^[a-z0-9-]+$/.test(hook.id ?? '')) failures.push(`unsafe hook id: ${hook.id}`);
      if (ids.has(hook.id)) failures.push(`duplicate hook id: ${hook.id}`);
      ids.add(hook.id);
      if (Boolean(hook.target) === Boolean(hook.externalTarget)) {
        failures.push(`${hook.id}: exactly one target kind is required`);
      }
      if (hook.target) {
        const normalized = path.posix.normalize(hook.target);
        if (
          normalized !== hook.target ||
          normalized.startsWith('../') ||
          path.isAbsolute(normalized)
        ) {
          failures.push(`${hook.id}: target escapes the bundle: ${hook.target}`);
        } else if (!fs.existsSync(path.join(root, ...normalized.split('/')))) {
          failures.push(`${hook.id}: target is missing: ${hook.target}`);
        }
      }
      if (hook.externalTarget && hook.externalTarget !== '$HOME/.local/bin/kbd-open') {
        failures.push(`${hook.id}: external target is not allowlisted`);
      }
    }
  }
}

function renderArgs(args = []) {
  return args
    .map(value => {
      if (value === '{harness}') return '"$HARNESS"';
      if (value === '{evolution}') return '"$EVOLUTION"';
      return shellQuote(value);
    })
    .join(' ');
}

function renderInvocation(hook) {
  const args = renderArgs(hook.args);
  const command = hook.target
    ? `run_bundle_script ${shellQuote(hook.target)}${args ? ` ${args}` : ''}`
    : `bash "$HOME/.local/bin/kbd-open"${args ? ` ${args}` : ''}`;
  const redirected = hook.stderrToStdout ? `${command} 2>&1` : command;
  return hook.ignoreFailure ? `${redirected} || true` : redirected;
}

function renderDispatcher() {
  const cases = [];
  for (const group of contract.events) {
    for (const hook of group.hooks) {
      cases.push(`  ${shellQuote(hook.id)})\n    ${renderInvocation(hook)}\n    ;;`);
    }
  }
  return `#!/usr/bin/env bash
set -euo pipefail

HOOK_ID=""
HARNESS=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --hook) HOOK_ID="\${2:-}"; shift 2 ;;
    --harness) HARNESS="\${2:-}"; shift 2 ;;
    *) printf 'hook-dispatch-v1: unknown argument: %s\\n' "$1" >&2; exit 64 ;;
  esac
done

case "$HARNESS" in
  claude-code|codex) ;;
  *) printf 'hook-dispatch-v1: unsupported harness: %s\\n' "$HARNESS" >&2; exit 64 ;;
esac

BUNDLE_ROOT="$(cd "$(dirname "\${BASH_SOURCE[0]}")/../../.." && pwd -P)"
EVOLUTION="\${EVOLUTION_NAME:-default}"

run_bundle_script() {
  local relative="$1"
  shift
  case "$relative" in
    /*|../*|*/../*) printf 'hook-dispatch-v1: unsafe target: %s\\n' "$relative" >&2; return 65 ;;
  esac
  local target="$BUNDLE_ROOT/$relative"
  [[ -f "$target" ]] || { printf 'hook-dispatch-v1: missing target: %s\\n' "$relative" >&2; return 66; }
  bash "$target" "$@"
}

case "$HOOK_ID" in
${cases.join('\n')}
  *) printf 'hook-dispatch-v1: unknown hook id: %s\\n' "$HOOK_ID" >&2; exit 64 ;;
esac
`;
}

function collectFiles(directory, relativeRoot, result) {
  if (!fs.existsSync(directory)) return;
  for (const name of fs.readdirSync(directory).sort()) {
    if (['tests', 'fixtures', 'generated'].includes(name)) continue;
    const absolute = path.join(directory, name);
    const relative = path.posix.join(relativeRoot, name);
    const stat = fs.lstatSync(absolute);
    if (stat.isDirectory()) collectFiles(absolute, relative, result);
    else if (stat.isFile()) {
      const bytes = fs.readFileSync(absolute);
      result.push({ path: relative, sha256: sha256(bytes), executable: executableOf(relative) });
    }
  }
}

function releaseIdentity(dispatcher) {
  const runtimeFiles = [];
  collectFiles(path.join(root, 'shared/scripts'), 'shared/scripts', runtimeFiles);
  collectFiles(
    path.join(root, 'skills/process/iterative-evolver/scripts'),
    'skills/process/iterative-evolver/scripts',
    runtimeFiles
  );
  // scripts/lib/*.js are load-bearing for the installer -- it will not start
  // without them -- so they belong in the identity that pins the runtime.
  for (const relative of [
    'scripts/hook-entry.mjs',
    'scripts/install-plugin-generation.js',
    'scripts/lib/capabilities.js',
    'scripts/lib/jcs.js',
    'scripts/lib/key-protection.js',
    'scripts/lib/payload-manifest.js',
    'scripts/lib/skill-frontmatter.js',
    'scripts/lib/skill-system.js',
    'shared/harnesses/hook-contract.json',
  ]) {
    const absolute = path.join(root, ...relative.split('/'));
    runtimeFiles.push({
      path: relative,
      sha256: sha256(fs.readFileSync(absolute)),
      executable: executableOf(relative),
    });
  }
  runtimeFiles.push({
    path: 'shared/scripts/generated/hook-dispatch-v1.sh',
    sha256: sha256(dispatcher),
    executable: true,
  });
  runtimeFiles.sort((left, right) => left.path.localeCompare(right.path));
  return {
    schemaVersion: 2,
    sourceVersion,
    contractSchemaVersion: contract.schemaVersion,
    dispatcherAbi: contract.dispatcherAbi,
    // Naming the interpreter in the identity is what lets the runtime stop
    // depending on a filesystem executable bit. A shebang is a request the
    // kernel may decline -- on a volume with no permission bits it always
    // does -- whereas this is a signed statement about how the dispatcher is
    // launched, and the runtime allowlists it before use.
    dispatcherInterpreter: DISPATCHER_INTERPRETER,
    contractSha256: sha256(canonicalJson(contract)),
    runtimeFiles,
  };
}

/**
 * Exec-form invocation for one hook.
 *
 * `command` plus `args` is spawned directly with no shell on any platform, and
 * the harness substitutes path placeholders into both as plain strings. That
 * removes the entire class of quoting defects the previous shell form carried:
 * a plugin root containing backslashes, `$`, or backticks now reaches the
 * entry point verbatim because nothing tokenizes it.
 *
 * It also fixes a Windows defect the shell form could not. Shell form is handed
 * to `sh -c` on POSIX and to POWERSHELL on Windows whenever Git Bash is absent,
 * so 31 entries of `bash -c '<multi-line bash>'` worked there only by accident
 * of Git Bash being installed.
 *
 * `command` is `node` rather than the compiled dispatcher because `hooks.json`
 * is ONE file shared by every host, the harness exposes no platform
 * placeholder, and a binary cannot bootstrap itself. `node` is a real
 * executable everywhere and is already a hard dependency of bootstrap; the
 * compiled dispatcher still owns the hot path, `hook-entry.mjs` execs it as
 * soon as it exists.
 */
function hookInvocation(bundleId, hookId, harness) {
  return {
    command: 'node',
    args: [
      `\${CLAUDE_PLUGIN_ROOT}/${HOOK_ENTRY}`,
      '--bundle',
      bundleId,
      '--hook',
      hookId,
      '--harness',
      harness,
    ],
  };
}

function renderHooks(bundleId, harness) {
  const hooks = {};
  for (const group of contract.events) {
    if (group.harnesses !== undefined && !group.harnesses.includes(harness)) continue;
    const emitted = {
      hooks: group.hooks.map(hook => {
        const { command, args } = hookInvocation(bundleId, hook.id, harness);
        // Asserted on every emitted entry, not just the ones that look
        // suspicious, so it cannot silently stop being true.
        const shellOnly = shellOnlyExecutableError(command);
        if (shellOnly) failures.push(`${harness}/${hook.id}: ${shellOnly}`);
        const value = { type: 'command', command, args };
        if (hook.timeout !== undefined) value.timeout = hook.timeout;
        return value;
      }),
    };
    if (group.matcher !== undefined) emitted.matcher = group.matcher;
    if (group.description !== undefined) emitted.description = group.description;
    if (group.groupId !== undefined) emitted.id = group.groupId;
    (hooks[group.event] ??= []).push(emitted);
  }
  return { hooks };
}

function emit(relative, content, mode = 0o644) {
  const absolute = path.join(root, ...relative.split('/'));
  const next = typeof content === 'string' ? content : `${JSON.stringify(content, null, 2)}\n`;
  const wantExecutable = (mode & 0o111) !== 0;
  if (check) {
    if (!fs.existsSync(absolute) || fs.readFileSync(absolute, 'utf8') !== next) {
      failures.push(`generated artifact is stale: ${relative}`);
      return;
    }
    // On a volume that records permission bits, assert the exact mode as before.
    // On one that does not, assert the executable INTENT against git's index --
    // the same authority the release identity uses -- instead of asserting a
    // mode the filesystem is structurally incapable of reporting.
    if (hostCapabilities.posixModes) {
      if ((fs.statSync(absolute).mode & 0o7777) !== mode) {
        failures.push(`generated artifact mode is stale: ${relative}`);
      }
    } else {
      const recorded = oracle?.get(relative);
      if (!recorded) {
        failures.push(
          `generated artifact is untracked, so its mode cannot be verified: ${relative}`
        );
      } else if (recorded.executable !== wantExecutable) {
        failures.push(
          `generated artifact executable bit is stale: ${relative} ` +
            `(recorded ${recorded.executable}, expected ${wantExecutable})`
        );
      }
    }
    return;
  }
  fs.mkdirSync(path.dirname(absolute), { recursive: true });
  fs.writeFileSync(absolute, next, { mode });
  fs.chmodSync(absolute, mode);
}

validateContract();
if (failures.length) {
  console.error(failures.join('\n'));
  process.exit(1);
}

const dispatcher = renderDispatcher();
const identity = releaseIdentity(dispatcher);
const bundleId = sha256(canonicalJson(identity));
const releaseManifest = { ...identity, bundleId };
const claudeHooks = renderHooks(bundleId, 'claude-code');
const codexHooks = renderHooks(bundleId, 'codex');

emit('shared/scripts/generated/hook-dispatch-v1.sh', dispatcher, 0o755);
emit('shared/harnesses/generated/release-manifest.json', releaseManifest);
emit('hooks/hooks.json', claudeHooks);
emit('hooks/codex-hooks.json', codexHooks);
emit('shared/harnesses/generated/claude-hooks.json', claudeHooks);

const controlAdapter = '${PROMETHEUS_SKILL_PACK_ROOT}/shared/scripts/kbd-harness-adapter.sh';
const learningDispatcher =
  '$HOME/.prometheus/plugins/prometheus-skill-pack/stable/karpathy-hook-dispatch.sh';
emit('shared/harnesses/generated/kimi-hooks.json', {
  schemaVersion: capabilities.schemaVersion,
  harness: 'kimi',
  hooks: Object.fromEntries(
    Object.entries(capabilities.harnesses.kimi.events)
      .filter(([, native]) => native)
      .map(([normalized, native]) => [
        native,
        {
          command: `bash ${
            normalized === 'prompt' || normalized === 'stop' ? learningDispatcher : controlAdapter
          } ${normalized.replace(/[A-Z]/g, character => `_${character.toLowerCase()}`)} kimi`,
          ...(normalized === 'prompt' || normalized === 'stop' ? {} : { timeoutMs: 1000 }),
        },
      ])
  ),
});
emit('shared/harnesses/generated/opencode-kbd-control.json', {
  schemaVersion: capabilities.schemaVersion,
  harness: 'opencode',
  controlAdapter,
  learningDispatcher,
  events: capabilities.harnesses.opencode.events,
  controlTimeoutMs: 250,
});

if (failures.length) {
  console.error(failures.join('\n'));
  process.exit(1);
}

console.log(
  `${check ? 'Verified' : 'Generated'} bundle ${bundleId} with ${contract.events.reduce(
    (total, group) => total + group.hooks.length,
    0
  )} hooks for Claude Code and Codex.`
);
