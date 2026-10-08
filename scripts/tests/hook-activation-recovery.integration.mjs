#!/usr/bin/env node
// Real installer/bootstrap processes and signed filesystem state. Local only.
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawn, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const evidence = path.resolve(process.env.ISSUE160_EVIDENCE || '/tmp/issue160-evidence/activation');
fs.mkdirSync(evidence, { recursive: true });
const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'issue160-activation-'));
const home = path.join(scratch, 'home');
const store = path.join(home, '.prometheus/plugins/prometheus-skill-pack');
fs.mkdirSync(home);
const env = {
  PATH: process.env.PATH, HOME: home, CODEX_HOME: path.join(home, '.codex'),
  XDG_CONFIG_HOME: path.join(home, '.config'), XDG_DATA_HOME: path.join(home, '.local/share'),
  PROMETHEUS_PLUGIN_ROOT: store, GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: os.devNull,
};
const logs = [];
const hash = data => crypto.createHash('sha256').update(data).digest('hex');
const canonical = value => Array.isArray(value) ? value.map(canonical) :
  value && typeof value === 'object' ? Object.fromEntries(Object.keys(value).sort().map(k => [k, canonical(value[k])])) : value;
const json = value => `${JSON.stringify(canonical(value), null, 2)}\n`;
const read = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const old = path.join(scratch, 'payload-current');
const next = path.join(scratch, 'payload-upgrade');
fs.cpSync(path.join(root, 'dist/plugins/claude/prometheus-skill-pack'), old, { recursive: true, verbatimSymlinks: true });
fs.cpSync(old, next, { recursive: true, verbatimSymlinks: true });
// A coherent next-release input; the real installer still builds/signs/verifies it.
const currentVersion = read(path.join(old, 'skill-system.json')).releaseVersion;
const parts = currentVersion.split('.').map(Number); parts[2] += 1;
const nextVersion = parts.join('.');
for (const [file, field] of [['skill-system.json', 'releaseVersion'], ['package.json', 'version'],
  ['.claude-plugin/plugin.json', 'version'], ['config/prometheus-exec-component.json', 'release']]) {
  const value = read(path.join(next, file)); value[field] = nextVersion;
  fs.writeFileSync(path.join(next, file), json(value));
}
const releaseFile = 'shared/harnesses/generated/release-manifest.json';
const release = read(path.join(next, releaseFile)), priorBundle = release.bundleId;
delete release.bundleId; release.sourceVersion = nextVersion;
release.bundleId = hash(json(release));
fs.writeFileSync(path.join(next, releaseFile), json(release));
// Hook declarations bind to the new release; runtime source remains identical.
for (const file of ['hooks/hooks.json', 'shared/harnesses/generated/claude-hooks.json',
  'shared/harnesses/generated/kimi-hooks.json']) {
  const full = path.join(next, file);
  if (fs.existsSync(full)) fs.writeFileSync(full, fs.readFileSync(full, 'utf8').replaceAll(priorBundle, release.bundleId));
}

function argv(source, extra = []) {
  return [path.join(source, 'scripts/install-plugin-generation.js'), '--source-root', source,
    '--home', home, '--plugin-root', store, '--targets', 'claude,codex', ...extra];
}
function record(label, command, args, result) {
  const index = String(logs.length).padStart(2, '0');
  fs.writeFileSync(path.join(evidence, `${index}-${label}.stdout`), result.stdout || '');
  fs.writeFileSync(path.join(evidence, `${index}-${label}.stderr`), result.stderr || '');
  logs.push({ label, command, args, status: result.status, signal: result.signal, error: result.error?.message });
  fs.writeFileSync(path.join(evidence, 'commands.json'), json(logs));
}
function run(label, source, extra = [], status = 0) {
  const args = argv(source, extra);
  const result = spawnSync(process.execPath, args, { cwd: scratch, env, encoding: 'utf8', timeout: 180_000, maxBuffer: 8e6 });
  record(label, process.execPath, args, result);
  assert.equal(result.status, status, `${label}: ${result.stderr || result.error}`);
  return result;
}
function snapshot() {
  const entries = {};
  function walk(file) {
    if (!fs.existsSync(file) && !fs.lstatSync(file, { throwIfNoEntry: false })) return;
    const stat = fs.lstatSync(file), key = path.relative(home, file);
    if (stat.isSymbolicLink()) entries[key] = ['link', fs.readlinkSync(file)];
    else if (stat.isDirectory()) for (const name of fs.readdirSync(file).sort()) walk(path.join(file, name));
    else entries[key] = [stat.mode & 0o777, hash(fs.readFileSync(file))];
  }
  for (const file of ['pointers', 'runtime', 'receipts', 'current', 'previous']) walk(path.join(store, file));
  for (const file of ['.claude/skills', '.codex/skills']) walk(path.join(home, file));
  return entries;
}
function consistent() {
  for (const name of ['current', 'previous']) {
    const pointer = path.join(store, 'pointers', name);
    if (fs.existsSync(pointer)) assert.equal(fs.realpathSync(path.join(store, name)),
      fs.realpathSync(path.join(store, fs.readFileSync(pointer, 'utf8').trim())));
  }
  assert(!fs.existsSync(path.join(store, 'pointers/activation.pending.json')));
}
async function crashUpgrade() {
  const journal = path.join(store, 'pointers/activation.pending.json');
  let killed = false, savedJournal;
  const child = spawn(process.execPath, argv(next), { cwd: scratch, env, stdio: ['ignore', 'pipe', 'pipe'] });
  let stdout = '', stderr = '';
  child.stdout.on('data', b => { stdout += b; }); child.stderr.on('data', b => { stderr += b; });
  const poll = setInterval(() => {
    if (fs.existsSync(journal) && !killed) {
      killed = true; child.kill('SIGSTOP');
      savedJournal = fs.readFileSync(journal);
      child.kill('SIGKILL');
    }
  }, 1);
  const timer = setTimeout(() => child.kill('SIGKILL'), 180_000);
  const [status, signal] = await new Promise((resolve, reject) => {
    child.on('error', reject); child.on('exit', (code, signal) => resolve([code, signal]));
  });
  clearInterval(poll); clearTimeout(timer);
  record('interrupted-upgrade', process.execPath, argv(next), { status, signal, stdout, stderr });
  assert(killed && fs.existsSync(journal), 'must interrupt real activation after journal publication');
  assert.equal(signal, 'SIGKILL');
  return savedJournal;
}

try {
  run('fresh-install', old); consistent();
  const baseline = snapshot();
  run('equal-version-install', old); consistent();
  assert.deepEqual(snapshot(), baseline);
  const savedJournal = await crashUpgrade();
  assert.match(run('pending-verification-rejected', next, ['--verify'], 1).stderr, /activation is pending/);
  // An invalid candidate blocks replay before any shared activation state changes.
  const pending = read(path.join(store, 'pointers/activation.pending.json'));
  const candidate = path.join(store, 'generations', pending.body.generation);
  const contamination = path.join(candidate, 'unmanifested.txt');
  fs.writeFileSync(contamination, 'integration contamination');
  const interruptedState = snapshot();
  run('unverified-recovery-rejected', next, [], 1);
  assert.deepEqual(snapshot(), interruptedState);
  fs.unlinkSync(contamination);
  const escaped = path.join(scratch, 'escaped-generation');
  fs.renameSync(candidate, escaped); fs.symlinkSync(escaped, candidate);
  assert.match(run('escaped-recovery-rejected', next, [], 1).stderr, /escapes generations/);
  assert.deepEqual(snapshot(), interruptedState);
  fs.unlinkSync(candidate); fs.renameSync(escaped, candidate);
  run('recover-upgrade', next); consistent();
  run('verify-upgrade', next, ['--verify']);
  // Model interruption immediately before journal removal using the unchanged
  // signed record captured from the real killed installer. Bundle already resolves.
  fs.writeFileSync(path.join(store, 'pointers/activation.pending.json'), savedJournal);
  const bootstrapArgs = [path.join(next, 'shared/scripts/bootstrap-hook-runtime.sh'),
    '--source-root', next, '--expected-bundle', release.bundleId];
  const recovered = spawnSync('bash', bootstrapArgs, { cwd: scratch, env, encoding: 'utf8', timeout: 180_000 });
  record('warm-bootstrap-recovers-journal', 'bash', bootstrapArgs, recovered);
  assert.equal(recovered.status, 0, recovered.stderr); consistent();
  fs.writeFileSync(path.join(store, 'pointers/activation.pending.json'), savedJournal);
  const entryArgs = [path.join(next, 'scripts/hook-entry.mjs'), '--bundle', release.bundleId,
    '--hook', 'not-a-hook', '--harness', 'claude-code'];
  const entryRecovery = spawnSync(process.execPath, entryArgs, { cwd: scratch,
    env: { ...env, CLAUDE_PLUGIN_ROOT: next }, encoding: 'utf8', timeout: 180_000 });
  record('warm-hook-entry-recovers-journal', process.execPath, entryArgs, entryRecovery);
  assert.equal(entryRecovery.status, 64, entryRecovery.stderr); consistent();
  const unexpectedBinary = path.join(store, 'runtime/v1/prometheus-hook');
  fs.writeFileSync(unexpectedBinary, 'unmanifested executable');
  assert.match(run('unsigned-compiled-runtime-rejected', next, ['--verify'], 1).stderr, /compiled hook runtime/);
  fs.unlinkSync(unexpectedBinary);
  const upgraded = snapshot();
  assert.match(run('direct-downgrade-rejected', old, [], 1).stderr, /DOWNGRADE_REFUSED/);
  assert.deepEqual(snapshot(), upgraded);
  assert.match(run('bootstrap-downgrade-rejected', old, ['--bootstrap'], 1).stderr, /DOWNGRADE_REFUSED/);
  assert.deepEqual(snapshot(), upgraded);
  const runner = path.join(store, 'runtime/v1/run-hook');
  const oldBundle = read(path.join(old, releaseFile)).bundleId;
  const resolved = spawnSync('bash', [runner, '--bundle', oldBundle, '--resolve-only'], { env, encoding: 'utf8' });
  record('retained-bundle-resolves', 'bash', [runner, '--bundle', oldBundle, '--resolve-only'], resolved);
  assert.equal(resolved.status, 0); assert.deepEqual(snapshot(), upgraded);
  const oldPointer = path.join(store, 'pointers/bundles', oldBundle);
  const oldLink = path.join(store, 'bundles', oldBundle);
  const pointerBytes = fs.readFileSync(oldPointer), linkValue = fs.readlinkSync(oldLink);
  fs.unlinkSync(oldPointer); fs.unlinkSync(oldLink);
  const unresolvedState = snapshot();
  const staleArgs = [path.join(old, 'shared/scripts/bootstrap-hook-runtime.sh'),
    '--source-root', old, '--expected-bundle', oldBundle];
  const stale = spawnSync('bash', staleArgs, { cwd: scratch, env, encoding: 'utf8', timeout: 180_000 });
  record('unresolved-stale-shell-bootstrap-rejected', 'bash', staleArgs, stale);
  assert.notEqual(stale.status, 0); assert.match(stale.stderr, /DOWNGRADE_REFUSED/);
  assert.deepEqual(snapshot(), unresolvedState);
  fs.writeFileSync(oldPointer, pointerBytes); fs.symlinkSync(linkValue, oldLink);
  run('explicit-rollback', next, ['--rollback']); consistent();
  assert.equal(read(path.join(store, 'current/manifest.json')).sourceVersion, currentVersion);
  run('upgrade-again', next); consistent();
  const wanted = fs.readFileSync(path.join(store, 'pointers/current'), 'utf8').trim();
  const previous = fs.readFileSync(path.join(store, 'pointers/previous'), 'utf8').trim();
  fs.unlinkSync(path.join(store, 'current')); fs.symlinkSync(previous, path.join(store, 'current'));
  assert.match(run('wrong-valid-link-rejected', next, ['--verify'], 1).stderr, /pointer\/link mismatch/);
  run('repair-wrong-link', next); consistent();
  assert.equal(fs.readFileSync(path.join(store, 'pointers/current'), 'utf8').trim(), wanted);
  fs.unlinkSync(path.join(store, 'current'));
  assert.match(run('missing-link-rejected', next, ['--verify'], 1).stderr, /pointer\/link mismatch/);
  run('repair-missing-link', next); consistent();
  // The original signed journal is valid but an edited target cannot be replayed.
  const altered = JSON.parse(savedJournal); altered.body.generation = '0'.repeat(64);
  fs.writeFileSync(path.join(store, 'pointers/activation.pending.json'), json(altered));
  const beforeTamper = snapshot(); run('tampered-journal-rejected', next, [], 1);
  assert.deepEqual(snapshot(), beforeTamper);
  fs.unlinkSync(path.join(store, 'pointers/activation.pending.json'));
  // Concurrent actual shell bootstraps acquire the same store lock.
  const concurrentHome = path.join(scratch, 'concurrent-home'); fs.mkdirSync(concurrentHome);
  const concurrentStore = path.join(concurrentHome, '.prometheus/plugins/prometheus-skill-pack');
  const args = [path.join(next, 'shared/scripts/bootstrap-hook-runtime.sh'), '--source-root', next, '--expected-bundle', release.bundleId];
  const concurrentResults = await Promise.allSettled([0, 1].map(index => new Promise((resolve, reject) => {
    const child = spawn('bash', args, { cwd: scratch, env: { ...env, HOME: concurrentHome,
      CODEX_HOME: path.join(concurrentHome, '.codex'), PROMETHEUS_PLUGIN_ROOT: concurrentStore }, stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '', stderr = ''; child.stdout.on('data', b => { stdout += b; }); child.stderr.on('data', b => { stderr += b; });
    const timer = setTimeout(() => child.kill('SIGKILL'), 360_000);
    child.on('error', reject); child.on('exit', (status, signal) => {
      clearTimeout(timer); record(`concurrent-bootstrap-${index}`, 'bash', args, { status, signal, stdout, stderr });
      if (status !== 0) reject(new Error(stderr)); else resolve();
    });
  })));
  for (const result of concurrentResults) if (result.status === 'rejected') throw result.reason;
  fs.writeFileSync(path.join(evidence, 'result.json'), json({ status: 'PASS', commands: logs.length, currentVersion, nextVersion }));
  console.log(`PASS: activation recovery (${logs.length} production processes)`);
} catch (error) {
  fs.writeFileSync(path.join(evidence, 'result.json'), json({ status: 'FAIL', error: error.stack, scratch }));
  throw error;
} finally {
  fs.rmSync(scratch, { recursive: true, force: true });
}
