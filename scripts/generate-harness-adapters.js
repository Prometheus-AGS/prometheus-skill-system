#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const manifestPath = path.join(root, 'shared/harnesses/capabilities.json');
const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
const output = path.join(root, 'shared/harnesses/generated');
fs.mkdirSync(output, { recursive: true });

const controlAdapter = '${PROMETHEUS_SKILL_PACK_ROOT}/shared/scripts/kbd-harness-adapter.sh';
const learningDispatcher = '${PROMETHEUS_SKILL_PACK_ROOT}/shared/scripts/karpathy-hook-dispatch.sh';
const artifacts = {
  'claude-hooks.json': {
    schemaVersion: manifest.schemaVersion,
    harness: 'claude-code',
    hooks: {
      SessionStart: [{ command: `bash ${controlAdapter} session_start claude-code`, timeout: 1000 }],
      PreCompact: [{ command: `bash ${controlAdapter} pre_compact claude-code`, timeout: 1000 }],
      UserPromptSubmit: [{ command: `bash ${learningDispatcher} prompt claude-code`, timeout: 2100 }],
      Stop: [{ command: `bash ${learningDispatcher} stop claude-code`, timeout: 250 }],
      SubagentStop: [{ matcher: 'executor', command: `bash ${learningDispatcher} executor_complete claude-code`, timeout: 250 }],
    },
  },
  'codex-hooks.toml': [
    '# Generated from shared/harnesses/capabilities.json.',
    '[hooks.kbd_control]',
    `command = ["bash", "${controlAdapter}", "auto", "codex"]`,
    'events = ["session_start", "pre_compact", "post_compact", "turn_cancelled"]',
    'timeout_ms = 250',
    '',
    '[hooks.karpathy_learning]',
    `command = ["bash", "${learningDispatcher}", "auto", "codex"]`,
    'events = ["user_prompt_submit", "stop"]',
    'timeout_ms = 2100',
    '',
  ].join('\n'),
  'kimi-hooks.json': {
    schemaVersion: manifest.schemaVersion,
    harness: 'kimi',
    hooks: Object.fromEntries(
      Object.entries(manifest.harnesses.kimi.events)
        .filter(([, native]) => native)
        .map(([normalized, native]) => [
          native,
          (() => {
            const event = normalized.replace(/[A-Z]/g, c => `_${c.toLowerCase()}`);
            const learning = normalized === 'prompt' || normalized === 'stop';
            return {
              command: `bash ${learning ? learningDispatcher : controlAdapter} ${event} kimi`,
              timeoutMs: normalized === 'stop' ? 250 : (normalized === 'prompt' ? 2100 : 1000),
            };
          })(),
        ])
    ),
  },
  'opencode-kbd-control.json': {
    schemaVersion: manifest.schemaVersion,
    harness: 'opencode',
    controlAdapter,
    learningDispatcher,
    events: manifest.harnesses.opencode.events,
    timeoutMs: 250,
  },
};

for (const [name, value] of Object.entries(artifacts)) {
  const content = typeof value === 'string' ? value : `${JSON.stringify(value, null, 2)}\n`;
  fs.writeFileSync(path.join(output, name), content);
}

console.log(`Generated ${Object.keys(artifacts).length} harness adapters from ${manifestPath}`);
