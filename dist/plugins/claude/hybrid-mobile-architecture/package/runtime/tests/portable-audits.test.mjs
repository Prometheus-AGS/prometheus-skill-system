import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readdirSync, readFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { copyPathExact } from './support.mjs';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const run = (script, args = []) => spawnSync(process.execPath, [script, ...args], { encoding: 'utf8', shell: false });
const temporary = t => { const path = mkdtempSync(join(tmpdir(), 'portable-audit ü-')); t.after(() => rmSync(path, { recursive: true, force: true })); return path; };

test('architecture audit rejects empty and unknown modes and detects Flutter FFI layer leakage', t => {
  const work = temporary(t), script = join(root, 'scripts/audit.mjs');
  assert.equal(run(script, ['typo', work]).status, 2); assert.equal(run(script, ['all', work]).status, 1);
  writeFileSync(join(work, 'pubspec.yaml'), 'dependencies:\n  flutter_riverpod: any\n  riverpod_annotation: any\n');
  for (const name of ['chat', 'notes', 'memory', 'startup']) for (const layer of ['data', 'domain', 'presentation/screens']) mkdirSync(join(work, 'lib/features', name, layer), { recursive: true });
  let result = run(script, ['flutter', work]); assert.equal(result.status, 0, result.stderr); assert.match(result.stdout, /Static audit is not build\/run proof/);
  writeFileSync(join(work, 'lib/features/chat/presentation/screens/chat.dart'), "import '../../../../bridge/rust_bridge_provider.dart';\n");
  result = run(script, ['flutter', work]); assert.equal(result.status, 1); assert.match(result.stderr, /imports FFI facade directly/);
});

test('refiner keeps typed replay data and enforces approval and verification transitions', t => {
  const work = temporary(t); copyPathExact(join(root, 'scripts'), join(work, 'scripts')); mkdirSync(join(work, 'skills/example'), { recursive: true });
  writeFileSync(join(work, 'skills/example/SKILL.md'), '---\nname: example\ndescription: Example architecture verification guidance.\n---\n');
  const script = join(work, 'scripts/refiner-loop.mjs');
  let result = run(script, ['--list']); assert.equal(result.status, 0); assert.equal(existsSync(join(work, '.prometheus')), false);
  result = run(script, ['--skill', 'example', '--evidence', 'Observed failure', '--replay', 'echo unsafe']); assert.equal(result.status, 1); assert.match(result.stderr, /--replay-argv/);
  result = run(script, ['--skill', 'example', '--evidence', 'Observed failure', '--replay-argv', JSON.stringify([process.execPath, '-e', 'process.exit(0)'])]); assert.equal(result.status, 0, result.stderr);
  const directory = join(work, '.prometheus/refiner'), file = join(directory, readdirSync(directory)[0]), ticket = JSON.parse(readFileSync(file));
  assert.equal(ticket.status, 'triaged'); assert.equal(ticket.replayed, false); assert.ok(Array.isArray(ticket.replayArgv));
  result = run(script, ['--ship', ticket.id]); assert.equal(result.status, 1); assert.match(result.stderr, /successful verification/);
  result = run(script, ['--approve', ticket.id]); assert.equal(result.status, 0); assert.equal(JSON.parse(readFileSync(file)).status, 'approved');
  result = run(script, ['--reject', ticket.id, '--reason', 'Independent review rejected diagnosis']); assert.equal(result.status, 0); assert.equal(JSON.parse(readFileSync(file)).status, 'rejected');
});

test('refiner cannot verify or ship a ticket without replay evidence, including formerly verified tickets', t => {
  const work = temporary(t); copyPathExact(join(root, 'scripts'), join(work, 'scripts')); mkdirSync(join(work, 'skills/example'), { recursive: true });
  writeFileSync(join(work, 'skills/example/SKILL.md'), '---\nname: example\ndescription: Example architecture verification guidance.\n---\n');
  const script = join(work, 'scripts/refiner-loop.mjs');
  let result = run(script, ['--skill', 'example', '--evidence', 'Observed failure without replay']); assert.equal(result.status, 0, result.stderr);
  const directory = join(work, '.prometheus/refiner'), file = join(directory, readdirSync(directory)[0]), ticket = JSON.parse(readFileSync(file));
  result = run(script, ['--approve', ticket.id]); assert.equal(result.status, 0, result.stderr);
  result = run(script, ['--verify', ticket.id]); assert.equal(result.status, 1); assert.match(result.stderr, /requires a recorded typed replayArgv/);
  const unverified = JSON.parse(readFileSync(file)); assert.equal(unverified.status, 'approved'); assert.equal(unverified.replayed, false); assert.equal(unverified.verifiedAt, '');
  result = run(script, ['--ship', ticket.id]); assert.equal(result.status, 1); assert.match(result.stderr, /recorded typed replay/);
  writeFileSync(file, JSON.stringify({ ...unverified, status: 'verified', verifiedAt: new Date().toISOString() }));
  result = run(script, ['--ship', ticket.id]); assert.equal(result.status, 1); assert.match(result.stderr, /recorded typed replay/);
  result = run(script, ['--verify', ticket.id]); assert.equal(result.status, 1); assert.equal(JSON.parse(readFileSync(file)).status, 'approved');
});
