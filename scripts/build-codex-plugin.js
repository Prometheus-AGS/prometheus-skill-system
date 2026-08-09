#!/usr/bin/env node

import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');
const check = process.argv.includes('--check');

execFileSync(process.execPath, [
  path.join(root, 'scripts/generate-skill-system-distribution.js'),
  ...(check ? ['--check'] : []),
], { cwd: root, stdio: 'inherit' });

const contract = JSON.parse(fs.readFileSync(path.join(root, 'skill-system.json')));
const manifest = JSON.parse(fs.readFileSync(path.join(root, contract.outputs.codexPackage, '.codex-plugin/plugin.json')));
const marketplace = JSON.parse(fs.readFileSync(path.join(root, contract.outputs.codexMarketplace)));

if (manifest.version !== contract.releaseVersion) throw new Error('Codex package version differs from the distribution contract');
if (manifest.skills !== './skills') throw new Error('Codex package must expose the flattened skills directory');
if (manifest.hooks !== undefined) throw new Error('Codex package contains the rejected hooks field');
if (!manifest.interface?.defaultPrompt) throw new Error('Codex package has no default prompt');
for (const plugin of marketplace.plugins) {
  if (!plugin.policy?.installation || !plugin.policy?.authentication || !plugin.category) {
    throw new Error(`Codex marketplace policy is incomplete: ${plugin.name}`);
  }
}

console.log('✓ Generated Codex package and marketplace are current and valid.');
