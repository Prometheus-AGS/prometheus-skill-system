#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);
let format = 'text';
for (let index = 0; index < args.length; index += 1) {
  if (args[index] === '--format') format = args[++index];
  else if (args[index] === '--help') {
    console.log('Usage: npm run lint -- [--format text|json]');
    process.exit(0);
  } else {
    console.error(`Unknown lint argument: ${args[index]}`);
    process.exit(2);
  }
}
if (!['text', 'json'].includes(format)) {
  console.error(`Unsupported lint format: ${format}`);
  process.exit(2);
}

const result = spawnSync(
  process.execPath,
  ['scripts/validate-skills.js', '--strict', '--exclude-submodules'],
  {
    cwd: repoRoot,
    env: process.env,
    encoding: 'utf8',
    stdio: format === 'text' ? 'inherit' : 'pipe',
  }
);

if (result.error) {
  console.error(`Skill lint failed to start: ${result.error.message}`);
  process.exit(1);
}
if (format === 'json') {
  console.log(
    JSON.stringify({
      schemaVersion: 1,
      ok: result.status === 0,
      exitCode: result.status ?? 1,
      validator: 'scripts/validate-skills.js --strict --exclude-submodules',
      stdout: result.stdout,
      stderr: result.stderr,
    })
  );
}
process.exit(result.status ?? 1);
