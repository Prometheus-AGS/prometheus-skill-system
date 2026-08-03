#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const target = path.join(root, 'hooks/hooks.json');
const payload = JSON.parse(fs.readFileSync(target, 'utf8'));
const command = 'bash ${CLAUDE_PLUGIN_ROOT:-$PLUGIN_ROOT}/shared/scripts/karpathy-hook-dispatch.sh';

payload.hooks.UserPromptSubmit = [
  {
    hooks: [
      {
        type: 'command',
        command: `${command} prompt claude-code`,
        timeout: 2100,
      },
    ],
  },
];
payload.hooks.Stop = [
  {
    hooks: [
      {
        type: 'command',
        command: `${command} stop claude-code`,
        timeout: 250,
      },
    ],
  },
];

fs.writeFileSync(target, `${JSON.stringify(payload, null, 2)}\n`);
console.log('Converged prompt and Stop hooks to the Karpathy learning dispatcher.');
