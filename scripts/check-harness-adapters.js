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
  if (!content.includes('kbd-harness-adapter.sh')) {
    failures.push(`${harness}: adapter does not invoke the canonical guard`);
  }
  if (config.writerRole && !config.nativeMutationGuard) {
    failures.push(`${harness}: writerRole requires nativeMutationGuard`);
  }
  for (const required of ['sessionStart', 'preCompact', 'postCompact', 'stop', 'preMutation']) {
    if (!(required in config.events)) {
      failures.push(`${harness}: missing normalized ${required} capability`);
    }
  }
}

const claudePayload = JSON.parse(fs.readFileSync(path.join(root, 'hooks/hooks.json'), 'utf8'));
for (const event of ['UserPromptSubmit', 'Stop']) {
  const serialized = JSON.stringify(claudePayload.hooks[event] ?? []);
  if (!serialized.includes('kbd-harness-adapter.sh')) {
    failures.push(`claude-code: ${event} does not use the bounded adapter`);
  }
  for (const forbidden of [
    'memory-writeback',
    'pk-focus-on-prompt',
    'evaluate-session',
    'propose-skill-update',
    'write-session-summary',
    'kbd-close',
  ]) {
    if (serialized.includes(forbidden)) {
      failures.push(`claude-code: ${event} still runs synchronous ${forbidden}`);
    }
  }
}

if (failures.length) {
  console.error(failures.join('\n'));
  process.exit(1);
}
console.log(`Harness adapter parity: ${Object.keys(expected).length}/4`);
