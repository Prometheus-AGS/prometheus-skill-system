#!/usr/bin/env node

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const siteRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(siteRoot, '..');
const outputPath = path.join(siteRoot, 'static/openapi/sovereign-sync-v2.openapi.json');
const targetDir =
  process.env.CARGO_TARGET_DIR ?? path.join(os.tmpdir(), 'prometheus-docs-cargo-target');
const generated = spawnSync(
  'cargo',
  [
    'run',
    '--quiet',
    '--manifest-path',
    path.join(repoRoot, 'substrate/sovereign-sync/Cargo.toml'),
    '--',
    '--mode',
    'openapi',
  ],
  {
    cwd: repoRoot,
    encoding: 'utf8',
    env: {
      ...process.env,
      CARGO_TARGET_DIR: targetDir,
      RUSTUP_TOOLCHAIN: process.env.RUSTUP_TOOLCHAIN ?? 'stable',
    },
  }
);

if (generated.status !== 0) {
  process.stderr.write(generated.stderr);
  process.exit(generated.status ?? 1);
}

let document;
try {
  document = JSON.parse(generated.stdout);
} catch (error) {
  console.error(`Sovereign OpenAPI generator returned invalid JSON: ${error.message}`);
  process.exit(1);
}
const rendered = `${JSON.stringify(document, null, 2)}\n`;

if (process.argv.includes('--check')) {
  const existing = fs.existsSync(outputPath) ? fs.readFileSync(outputPath, 'utf8') : '';
  if (existing !== rendered) {
    console.error(
      'Sovereign Sync OpenAPI drifted. Run npm --prefix site run generate:sovereign-openapi.'
    );
    process.exit(1);
  }
  console.log('Sovereign Sync OpenAPI matches Rust routes and types.');
} else {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, rendered);
  console.log(`Wrote ${path.relative(siteRoot, outputPath)}`);
}
