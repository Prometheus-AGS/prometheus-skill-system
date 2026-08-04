import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const tool = resolve('scripts/local-review-receipt.mjs');
const fixture = mkdtempSync(join(tmpdir(), 'local-review-'));

function run(command, args) {
  return execFileSync(command, args, { cwd: fixture, encoding: 'utf8' });
}

run('git', ['init', '-q']);
run('git', ['config', 'user.name', 'Fixture']);
run('git', ['config', 'user.email', 'fixture@example.test']);
writeFileSync(join(fixture, 'README.md'), '# Base\n');
run('git', ['add', '.']);
run('git', ['commit', '-qm', 'base']);
const base = run('git', ['rev-parse', 'HEAD']).trim();
writeFileSync(join(fixture, 'README.md'), '# Documentation-only change\n');
run('git', ['add', '.']);
run('git', ['commit', '-qm', 'single docs file']);
const candidate = run('git', ['rev-parse', 'HEAD']).trim();

run('node', [
  tool,
  'pending',
  '--base',
  base,
  '--candidate',
  candidate,
  '--out',
  'pending.json',
]);
let result = spawnSync(
  'node',
  [tool, 'verify', '--base', base, '--candidate', candidate, '--receipt', 'pending.json'],
  { cwd: fixture, encoding: 'utf8' },
);
if (result.status === 0) throw new Error('pending review incorrectly passed certification');

writeFileSync(join(fixture, 'findings.json'), '{"findings":[]}\n');
run('node', [
  tool,
  'complete',
  '--base',
  base,
  '--candidate',
  candidate,
  '--findings',
  'findings.json',
  '--reviewer',
  'fixture-judge',
  '--model',
  'fixture-model',
  '--timestamp',
  '2026-08-03T00:00:00.000Z',
  '--out',
  'completed.json',
]);
run('node', [
  tool,
  'verify',
  '--base',
  base,
  '--candidate',
  candidate,
  '--receipt',
  'completed.json',
]);

writeFileSync(join(fixture, 'source.js'), 'export const value = 1;\n');
run('git', ['add', '.']);
run('git', ['commit', '-qm', 'new unreviewed source']);
const nextCandidate = run('git', ['rev-parse', 'HEAD']).trim();
result = spawnSync(
  'node',
  [tool, 'verify', '--base', base, '--candidate', nextCandidate, '--receipt', 'completed.json'],
  { cwd: fixture, encoding: 'utf8' },
);
if (result.status === 0) throw new Error('stale review receipt covered a later commit');

const key = join(fixture, 'waiver-key');
run('ssh-keygen', ['-q', '-t', 'ed25519', '-N', '', '-f', key]);
const approver = 'fixture-approver';
run('node', [
  tool,
  'waiver-template',
  '--base',
  base,
  '--candidate',
  nextCandidate,
  '--reason',
  'Fixture waiver for local certification without an available judge.',
  '--approver',
  approver,
  '--timestamp',
  '2026-08-03T00:00:00.000Z',
  '--out',
  'waiver.json',
]);
run('ssh-keygen', ['-Y', 'sign', '-f', key, '-n', 'prometheus-review-waiver', 'waiver.json']);
const allowed = join(fixture, 'allowed-signers');
writeFileSync(
  allowed,
  `${approver} namespaces="prometheus-review-waiver" ${readFileSync(`${key}.pub`, 'utf8').trim()}\n`,
);
run('node', [
  tool,
  'verify',
  '--base',
  base,
  '--candidate',
  nextCandidate,
  '--waiver',
  'waiver.json',
  '--allowed-signers',
  allowed,
]);

console.log('local review receipts: cumulative, no small/docs skip, pending fails, signed waiver passes');
