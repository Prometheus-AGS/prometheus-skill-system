#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

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

function modeString(mode) {
  return (mode & 0o7777).toString(8).padStart(4, '0');
}

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
        if (normalized !== hook.target || normalized.startsWith('../') || path.isAbsolute(normalized)) {
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
      result.push({ path: relative, sha256: sha256(bytes), mode: modeString(stat.mode) });
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
  for (const relative of [
    'scripts/install-plugin-generation.js',
    'shared/harnesses/hook-contract.json',
  ]) {
    const absolute = path.join(root, ...relative.split('/'));
    const stat = fs.statSync(absolute);
    runtimeFiles.push({
      path: relative,
      sha256: sha256(fs.readFileSync(absolute)),
      mode: modeString(stat.mode),
    });
  }
  runtimeFiles.push({
    path: 'shared/scripts/generated/hook-dispatch-v1.sh',
    sha256: sha256(dispatcher),
    mode: '0755',
  });
  runtimeFiles.sort((left, right) => left.path.localeCompare(right.path));
  return {
    schemaVersion: 1,
    sourceVersion,
    contractSchemaVersion: contract.schemaVersion,
    dispatcherAbi: contract.dispatcherAbi,
    contractSha256: sha256(canonicalJson(contract)),
    runtimeFiles,
  };
}

function hookCommand(bundleId, hookId, harness) {
  const runner = '$HOME/.prometheus/plugins/prometheus-skill-pack/runtime/v1/run-hook';
  const body = `runner="${runner}"
if [[ ! -x "$runner" ]] || ! "$runner" --bundle "$1" --resolve-only >/dev/null 2>&1; then
  plugin_root="\${CLAUDE_PLUGIN_ROOT:-\${PLUGIN_ROOT:-}}"
  bootstrap="$plugin_root/shared/scripts/bootstrap-hook-runtime.sh"
  if [[ -z "$plugin_root" || ! -x "$bootstrap" ]]; then
    printf '{"status":"NOT_ACTIVATED","bundle":"%s"}\\n' "$1" >&2
    exit 78
  fi
  bash "$bootstrap" --source-root "$plugin_root" --expected-bundle "$1" || exit
fi
exec "$runner" --bundle "$1" --hook "$2" --harness "$3"`;
  return `bash -c ${shellQuote(body)} prometheus-hook ${shellQuote(bundleId)} ${shellQuote(hookId)} ${shellQuote(harness)}`;
}

function renderHooks(bundleId, harness) {
  const hooks = {};
  for (const group of contract.events) {
    if (group.harnesses !== undefined && !group.harnesses.includes(harness)) continue;
    const emitted = {
      hooks: group.hooks.map(hook => {
        const value = {
          type: 'command',
          command: hookCommand(bundleId, hook.id, harness),
        };
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
  if (check) {
    if (!fs.existsSync(absolute) || fs.readFileSync(absolute, 'utf8') !== next) {
      failures.push(`generated artifact is stale: ${relative}`);
      return;
    }
    if ((fs.statSync(absolute).mode & 0o7777) !== mode) {
      failures.push(`generated artifact mode is stale: ${relative}`);
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
            normalized === 'prompt' || normalized === 'stop'
              ? learningDispatcher
              : controlAdapter
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
