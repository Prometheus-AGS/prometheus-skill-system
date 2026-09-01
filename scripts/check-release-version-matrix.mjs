#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const matrix = JSON.parse(
  fs.readFileSync(path.join(root, 'config/release-version-matrix.json'), 'utf8')
);

const failures = [];
for (const surface of matrix.ownedSurfaces) {
  const file = path.join(root, surface.path);
  const text = fs.existsSync(file) ? fs.readFileSync(file, 'utf8') : '';
  if (!new RegExp(surface.pattern).test(text)) {
    failures.push(`${surface.path}: missing ${surface.pattern}`);
  }
}

for (const generated of [
  '.claude-plugin/marketplace.json',
  '.agents/plugins/marketplace.json',
  'shared/harnesses/generated/release-manifest.json',
  'dist/plugins/claude/prometheus-skill-pack/.claude-plugin/plugin.json',
  'dist/plugins/codex/prometheus-skill-pack/.codex-plugin/plugin.json',
]) {
  const text = fs.readFileSync(path.join(root, generated), 'utf8');
  if (!text.includes(matrix.release)) failures.push(`${generated}: missing ${matrix.release}`);
}

if (failures.length) {
  process.stderr.write(`${failures.join('\n')}\n`);
  process.exit(1);
}
process.stdout.write(`release version matrix verified for ${matrix.release}\n`);
