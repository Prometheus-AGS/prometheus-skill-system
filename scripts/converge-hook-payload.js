#!/usr/bin/env node
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
execFileSync(process.execPath, [path.join(root, 'scripts/generate-harness-adapters.js')], {
  cwd: root,
  stdio: 'inherit',
});
console.log('Converged all hook payloads through the canonical bundle-pinned generator.');
