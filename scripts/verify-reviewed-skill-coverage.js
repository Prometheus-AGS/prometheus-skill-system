#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

function parseArgs(argv) {
  const args = { pluginRoot: null, home: os.homedir(), trustStore: null };
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--plugin-root') args.pluginRoot = argv[++index];
    else if (value === '--home') args.home = argv[++index];
    else if (value === '--trust-store') args.trustStore = argv[++index];
    else throw new Error(`unknown argument: ${value}`);
  }
  if (!args.pluginRoot) throw new Error('missing --plugin-root');
  args.pluginRoot = path.resolve(args.pluginRoot);
  args.home = path.resolve(args.home);
  args.trustStore = path.resolve(args.trustStore ?? path.join(args.pluginRoot, 'trust/allowed-signers.json'));
  return args;
}

function verifyInstalledGeneration(args) {
  const sourceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  const installer = path.join(sourceRoot, 'scripts', 'install-plugin-generation.js');
  const result = spawnSync(process.execPath, [installer, '--reviewed-coverage', '--source-root', sourceRoot, '--plugin-root', args.pluginRoot, '--home', args.home, '--trust-store', args.trustStore], {
    encoding: 'utf8',
    shell: false,
    maxBuffer: 64 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(result.stderr.trim() || 'installed generation verification failed');
  return result.stdout.trim();
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  process.stdout.write(`${verifyInstalledGeneration(args)}\n`);
}

try {
  main();
} catch (error) {
  process.stderr.write(`verify-reviewed-skill-coverage: ${error.message}\n`);
  process.exitCode = 1;
}
