import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');

test('deployment helper boots as ESM before validating its operation', () => {
  const result = spawnSync(process.execPath, [join(root, 'scripts/ci-deployment.mjs')], {
    encoding: 'utf8',
    shell: false,
  });

  assert.equal(result.status, 1);
  assert.match(result.stderr, /Expected record-digests/);
  assert.doesNotMatch(result.stderr, /Dynamic require/);
});
