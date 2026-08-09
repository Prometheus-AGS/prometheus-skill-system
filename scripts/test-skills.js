#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');

const allTests = [
  [
    'strict skill validation',
    'node',
    ['scripts/validate-skills.js', '--strict', '--exclude-submodules'],
  ],
  ['progress-signal validation', 'node', ['scripts/validate-progress-signals.js']],
  ['Codex artifact consistency', 'node', ['scripts/build-codex-plugin.js', '--check']],
  ['skill collision matrix', 'node', ['scripts/skill-matrix.js', '--ci']],
  ['KBD control-plane fixtures', 'bash', ['scripts/test-kbd-control-plane.sh']],
  ['kbd-init packaging', 'bash', ['shared/scripts/tests/test-kbd-init-packaging.sh']],
  ['kbd-next-phase packaging', 'bash', ['shared/scripts/tests/test-kbd-next-phase-packaging.sh']],
  [
    'cross-tool payload packaging',
    'bash',
    ['shared/scripts/tests/test-cross-tool-skill-payloads.sh'],
  ],
  ['PK health hook fixtures', 'bash', ['shared/scripts/tests/test-pk-health.sh']],
  ['Karpathy hook fixtures', 'bash', ['shared/scripts/tests/test-karpathy-dispatch.sh']],
  ['Atomic plugin generation', 'bash', ['shared/scripts/tests/test-plugin-generation.sh']],
  ['verified updater policy', 'bash', ['scripts/tests/update-skill-pack.test.sh']],
  [
    'native plugin refresh policy',
    'bash',
    ['scripts/tests/refresh-native-plugin-installs.test.sh'],
  ],
  ['installer entrypoint policy', 'bash', ['scripts/tests/test-installer-entrypoints.sh']],
  ['learning basic flow', 'bash', ['tests/learn/integration-basic-flow.sh']],
  ['learning full loop', 'bash', ['tests/learn/integration-full-loop.sh']],
  ['learning KB adapter', 'bash', ['tests/learn/integration-kb.sh']],
  ['learning meta/harness parity', 'bash', ['tests/learn/integration-meta.sh']],
];

const skippedKbdTests = new Set([
  'KBD control-plane fixtures',
  'kbd-init packaging',
  'kbd-next-phase packaging',
]);
const tests =
  process.env.PROMETHEUS_SKIP_KBD === '1'
    ? allTests.filter(([name]) => !skippedKbdTests.has(name))
    : allTests;

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
