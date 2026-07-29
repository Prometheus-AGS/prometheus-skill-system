#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const manifestPath = path.join(root, 'shared/harnesses/capabilities.json');
const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
const output = path.join(root, 'shared/harnesses/generated');
fs.mkdirSync(output, { recursive: true });

const adapter = '${PROMETHEUS_SKILL_PACK_ROOT}/shared/scripts/kbd-harness-adapter.sh';
const artifacts = {
  'claude-hooks.json': {
    schemaVersion: manifest.schemaVersion,
    harness: 'claude-code',
    hooks: {
      SessionStart: [{ command: `bash ${adapter} session_start claude-code`, timeout: 1000 }],
      PreCompact: [{ command: `bash ${adapter} pre_compact claude-code`, timeout: 1000 }],
      UserPromptSubmit: [{ command: `bash ${adapter} prompt claude-code`, timeout: 1000 }],
      Stop: [{ command: `bash ${adapter} stop claude-code`, timeout: 250 }],
    },
  },
  'codex-hooks.toml': [
    '# Generated from shared/harnesses/capabilities.json.',
    '[hooks.kbd_control]',
    `command = ["bash", "${adapter}"]`,
    'events = ["session_start", "pre_compact", "post_compact", "user_prompt_submit", "stop", "turn_cancelled", "post_tool"]',
    'timeout_ms = 250',
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
          {
            command: `bash ${adapter} ${normalized.replace(/[A-Z]/g, c => `_${c.toLowerCase()}`)} kimi`,
            timeoutMs: normalized === 'preMutation' ? 250 : 1000,
          },
        ])
    ),
  },
  'opencode-kbd-control.json': {
    schemaVersion: manifest.schemaVersion,
    harness: 'opencode',
    adapter,
    events: manifest.harnesses.opencode.events,
    timeoutMs: 250,
  },
};

for (const [name, value] of Object.entries(artifacts)) {
  const content = typeof value === 'string' ? value : `${JSON.stringify(value, null, 2)}\n`;
  fs.writeFileSync(path.join(output, name), content);
}

console.log(`Generated ${Object.keys(artifacts).length} harness adapters from ${manifestPath}`);
