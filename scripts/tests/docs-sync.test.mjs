#!/usr/bin/env node

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

const root = path.resolve(import.meta.dirname, '..', '..');
const outputs = [
  'docs/generated/runtime-reference.md',
  'site/docs/operations/generated-reference.md',
  'site/static/openapi/prometheus-exec.openapi.json',
];

execFileSync('node', ['scripts/docs-sync.mjs'], { cwd: root, stdio: 'inherit' });
const first = new Map(outputs.map(file => [file, fs.readFileSync(path.join(root, file), 'utf8')]));
execFileSync('node', ['scripts/docs-sync.mjs'], { cwd: root, stdio: 'inherit' });
for (const file of outputs) {
  const second = fs.readFileSync(path.join(root, file), 'utf8');
  if (first.get(file) !== second) throw new Error(`docs:sync is not idempotent: ${file}`);
  if (file.endsWith('.md') && !second.includes('BEGIN PROMETHEUS DOCS SYNC: runtime-reference')) {
    throw new Error(`managed block marker is missing: ${file}`);
  }
}

const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-docs-sync-'));
try {
  const output = path.join(temporary, 'github-output');
  execFileSync('node', ['scripts/docs-sync.mjs', '--changed-between', 'HEAD', 'HEAD'], {
    cwd: root,
    env: { ...process.env, GITHUB_OUTPUT: output },
    stdio: 'inherit',
  });
  if (fs.readFileSync(output, 'utf8').trim() !== 'relevant=false') {
    throw new Error('unchanged source inputs were not classified as irrelevant');
  }
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}

console.log('docs:sync idempotence and input-selection fixtures passed');
