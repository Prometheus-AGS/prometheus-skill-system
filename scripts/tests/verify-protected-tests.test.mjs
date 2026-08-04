import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const verifier = resolve('scripts/verify-protected-tests.mjs');
const fixture = mkdtempSync(join(tmpdir(), 'protected-tests-'));

function run(command, args, options = {}) {
  return execFileSync(command, args, { cwd: fixture, encoding: 'utf8', ...options });
}

run('git', ['init', '-q']);
run('git', ['config', 'user.name', 'Fixture']);
run('git', ['config', 'user.email', 'fixture@example.test']);
mkdirSync(join(fixture, 'tests/features'), { recursive: true });
writeFileSync(join(fixture, 'tests/features/login.feature'), 'Feature: login\n');
writeFileSync(join(fixture, 'app.txt'), 'base\n');
run('git', ['add', '.']);
run('git', ['commit', '-qm', 'base']);
const base = run('git', ['rev-parse', 'HEAD']).trim();

run('bash', ['-c', 'printf "  Scenario: bash mutation\\n" >> tests/features/login.feature']);
run('git', ['add', '.']);
run('git', ['commit', '-qm', 'bash mutation']);
const bashCandidate = run('git', ['rev-parse', 'HEAD']).trim();
let result = spawnSync(
  'node',
  [verifier, '--base', base, '--candidate', bashCandidate],
  { cwd: fixture, encoding: 'utf8' },
);
if (result.status === 0) throw new Error('Bash mutation was not detected');

const key = join(fixture, 'approval-key');
run('ssh-keygen', ['-q', '-t', 'ed25519', '-N', '', '-f', key]);
const approver = 'fixture-approver';
const manifest = run('node', [
  verifier,
  '--base',
  base,
  '--candidate',
  bashCandidate,
  '--template',
  '--approver',
  approver,
  '--reason',
  'Fixture approval for an intentional protected test change.',
  '--timestamp',
  '2026-08-03T00:00:00.000Z',
]);
const manifestPath = join(fixture, 'approval.json');
writeFileSync(manifestPath, manifest);
run('ssh-keygen', ['-Y', 'sign', '-f', key, '-n', 'prometheus-test-change', manifestPath]);
const publicKey = readFileSync(`${key}.pub`, 'utf8').trim();
const allowedSigners = join(fixture, 'allowed-signers');
writeFileSync(
  allowedSigners,
  `${approver} namespaces="prometheus-test-change" ${publicKey}\n`,
);
run('node', [
  verifier,
  '--base',
  base,
  '--candidate',
  bashCandidate,
  '--approval',
  manifestPath,
  '--allowed-signers',
  allowedSigners,
]);

run('python3', [
  '-c',
  "from pathlib import Path; p=Path('tests/features/login.feature'); p.write_text(p.read_text()+'  Scenario: python mutation\\n')",
]);
run('git', ['add', '.']);
run('git', ['commit', '-qm', 'python mutation']);
const pythonCandidate = run('git', ['rev-parse', 'HEAD']).trim();
result = spawnSync(
  'node',
  [verifier, '--base', bashCandidate, '--candidate', pythonCandidate],
  { cwd: fixture, encoding: 'utf8' },
);
if (result.status === 0) throw new Error('Python mutation was not detected');

run('chmod', ['+x', 'tests/features/login.feature']);
run('git', ['add', '.']);
run('git', ['commit', '-qm', 'mode mutation']);
const modeCandidate = run('git', ['rev-parse', 'HEAD']).trim();
result = spawnSync(
  'node',
  [verifier, '--base', pythonCandidate, '--candidate', modeCandidate],
  { cwd: fixture, encoding: 'utf8' },
);
if (result.status === 0) throw new Error('Mode mutation was not detected');

mkdirSync(join(fixture, 'tests/features/drafts'), { recursive: true });
run('git', ['mv', 'tests/features/login.feature', 'tests/features/drafts/login.feature']);
run('git', ['commit', '-qm', 'rename protected test into drafts']);
const renameCandidate = run('git', ['rev-parse', 'HEAD']).trim();
result = spawnSync(
  'node',
  [verifier, '--base', modeCandidate, '--candidate', renameCandidate],
  { cwd: fixture, encoding: 'utf8' },
);
if (result.status === 0) throw new Error('Protected rename was not detected');

mkdirSync(join(fixture, 'tests/support'), { recursive: true });
writeFileSync(join(fixture, 'tests/support/world.ts'), 'export const world = {};\n');
run('git', ['add', '.']);
run('git', ['commit', '-qm', 'establish protected deletion base']);
const deletionBase = run('git', ['rev-parse', 'HEAD']).trim();
run('git', ['rm', 'tests/support/world.ts']);
run('git', ['commit', '-qm', 'delete protected support']);
const deletionCandidate = run('git', ['rev-parse', 'HEAD']).trim();
result = spawnSync(
  'node',
  [verifier, '--base', deletionBase, '--candidate', deletionCandidate],
  { cwd: fixture, encoding: 'utf8' },
);
if (result.status === 0) throw new Error('Protected deletion was not detected');

console.log(
  'protected-test verifier: Bash/Python unrestricted; content, mode, rename, and deletion certification enforced',
);
