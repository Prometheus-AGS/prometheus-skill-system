#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const manifest = JSON.parse(
  fs.readFileSync(path.join(root, 'shared/harnesses/capabilities.json'), 'utf8')
);
const generated = path.join(root, 'shared/harnesses/generated');
const expected = {
  'claude-code': 'claude-hooks.json',
  codex: 'codex-hooks.toml',
  opencode: 'opencode-kbd-control.json',
  kimi: 'kimi-hooks.json',
};
const failures = [];

for (const [harness, file] of Object.entries(expected)) {
  const config = manifest.harnesses[harness];
  const target = path.join(generated, file);
  if (!config || !fs.existsSync(target)) {
    failures.push(`${harness}: missing manifest entry or generated adapter`);
    continue;
  }
  const content = fs.readFileSync(target, 'utf8');
  if (!content.includes('karpathy-hook-dispatch.sh')) {
    failures.push(`${harness}: adapter does not invoke the learning dispatcher`);
  }
  if (
    !content.includes(
      '$HOME/.prometheus/plugins/prometheus-skill-pack/stable/karpathy-hook-dispatch.sh'
    )
  ) {
    failures.push(`${harness}: learning dispatcher does not resolve through the active generation`);
  }
  if (!content.includes('kbd-harness-adapter.sh')) {
    failures.push(`${harness}: adapter does not retain bounded reanchor/interrupt control`);
  }
  // The pre-mutation fence was removed deliberately: it gated the operator's
  // own shell on KBD lifecycle state, which blocks ordinary work such as
  // editing a submodule or a project this one depends on. Adapters now observe
  // lifecycle events only; they never intercept a tool call.
  for (const required of ['sessionStart', 'preCompact', 'postCompact', 'stop']) {
    if (!(required in config.events)) {
      failures.push(`${harness}: missing normalized ${required} capability`);
    }
  }
}

const claudePayload = JSON.parse(fs.readFileSync(path.join(root, 'hooks/hooks.json'), 'utf8'));
for (const event of ['UserPromptSubmit', 'Stop']) {
  const serialized = JSON.stringify(claudePayload.hooks[event] ?? []);
  if (!serialized.includes('karpathy-hook-dispatch.sh')) {
    failures.push(`claude-code: ${event} does not use the learning dispatcher`);
  }
  if (serialized.includes('kbd-harness-adapter.sh')) {
    failures.push(`claude-code: ${event} still routes learning through KBD control`);
  }
  if (
    !serialized.includes(
      '$HOME/.prometheus/plugins/prometheus-skill-pack/stable/karpathy-hook-dispatch.sh'
    )
  ) {
    failures.push(`claude-code: ${event} bypasses the stable active-generation dispatcher`);
  }
  if (serialized.includes('"timeout"')) {
    failures.push(`claude-code: ${event} still uses a latency deadline as a learning bound`);
  }
  for (const forbidden of [
    'memory-writeback',
    'pk-focus-on-prompt',
    'propose-skill-update',
    'write-session-summary',
    'kbd-close',
  ]) {
    if (serialized.includes(forbidden)) {
      failures.push(`claude-code: ${event} still runs synchronous ${forbidden}`);
    }
  }
}

const executorHooks = JSON.stringify(claudePayload.hooks.SubagentStop ?? []);
if (
  !executorHooks.includes('karpathy-hook-dispatch.sh') ||
  !executorHooks.includes('executor_complete claude-code')
) {
  failures.push('claude-code: executor completion does not enqueue a learning job');
}
if (
  !executorHooks.includes(
    '$HOME/.prometheus/plugins/prometheus-skill-pack/stable/karpathy-hook-dispatch.sh'
  )
) {
  failures.push('claude-code: executor completion bypasses the active generation');
}

if (
  fs
    .readFileSync(path.join(root, 'shared/scripts/kbd-harness-adapter.sh'), 'utf8')
    .includes('deferred-hooks')
) {
  failures.push('KBD adapter still contains the obsolete deferred observational queue');
}

if (failures.length) {
  console.error(failures.join('\n'));
  process.exit(1);
}
console.log(`Harness adapter parity: ${Object.keys(expected).length}/4`);
