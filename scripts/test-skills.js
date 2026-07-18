#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');

const tests = [
  [
    'strict skill validation',
    'node',
    ['scripts/validate-skills.js', '--strict', '--exclude-submodules'],
  ],
  ['progress-signal validation', 'node', ['scripts/validate-progress-signals.js']],
  ['Codex artifact consistency', 'node', ['scripts/build-codex-plugin.js', '--check']],
  ['skill collision matrix', 'node', ['scripts/skill-matrix.js', '--ci']],
  ['kbd-next-phase packaging', 'bash', ['shared/scripts/tests/test-kbd-next-phase-packaging.sh']],
  [
    'cross-tool payload packaging',
    'bash',
    ['shared/scripts/tests/test-cross-tool-skill-payloads.sh'],
  ],
  ['PK health hook fixtures', 'bash', ['shared/scripts/tests/test-pk-health.sh']],
  ['Karpathy hook fixtures', 'bash', ['shared/scripts/tests/test-karpathy-hooks.sh']],
  ['learning basic flow', 'bash', ['tests/learn/integration-basic-flow.sh']],
  ['learning full loop', 'bash', ['tests/learn/integration-full-loop.sh']],
  ['learning KB adapter', 'bash', ['tests/learn/integration-kb.sh']],
  ['learning meta/harness parity', 'bash', ['tests/learn/integration-meta.sh']],
];

let passed = 0;
for (const [name, command, args] of tests) {
  console.log(`\n▶ ${name}`);
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    env: process.env,
    encoding: 'utf8',
    stdio: 'inherit',
  });
  if (result.error) {
    console.error(`✗ ${name}: ${result.error.message}`);
    process.exit(1);
  }
  if (result.status !== 0) {
    console.error(`✗ ${name}: exited ${result.status}`);
    process.exit(result.status ?? 1);
  }
  passed += 1;
  console.log(`✓ ${name}`);
}

console.log(`\n✓ ${passed}/${tests.length} deterministic test suites passed`);
