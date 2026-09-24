// Real process integration: catches skipped runtime postconditions and fabricated apply counts.
// Requires the actual prometheus executable on PATH (or PROMETHEUS_CLI_TEST_BINARY)
// and a local OpenSpec installation (or OPENSPEC_TEST_BIN_DIR). No CLI substitutes.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash, generateKeyPairSync } from 'node:crypto';
import { spawnSync } from 'node:child_process';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const scratch = path.join(repo, '.scratch', 'kbd-phase-integration');
const prometheus = process.env.PROMETHEUS_CLI_TEST_BINARY || 'prometheus';
const openspecBins = process.env.OPENSPEC_TEST_BIN_DIR || path.resolve(repo, '..', '..', '..', 'prometheus-skills-mini', 'node_modules', '.bin');
const sourceOrchestrator = path.join(repo, 'skills', 'process', 'kbd-process-orchestrator');
const shellQuote = value => `'${value.replaceAll("'", "'\"'\"'")}'`;
const metadata = { name: 'fixture', activePhase: 'old-phase', active_phase: 'old-alias', specBackend: 'openspec', custom: { retained: [1, 'two'] } };

function write(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, typeof content === 'string' ? content : `${JSON.stringify(content, null, 2)}\n`);
}

function read(file) { return JSON.parse(fs.readFileSync(file, 'utf8')); }
function success(result, context) {
  assert.equal(result.error, undefined, `${context}: ${result.error?.message}`);
  assert.equal(result.status, 0, `${context}\n${result.stdout}\n${result.stderr}`);
  return result;
}

function snapshot(root) {
  const entries = {};
  function visit(dir) {
    if (!fs.existsSync(dir)) return;
    for (const entry of fs.readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      const file = path.join(dir, entry.name);
      const key = path.relative(root, file);
      if (entry.isDirectory()) { entries[key] = 'directory'; visit(file); }
      else entries[key] = createHash('sha256').update(fs.readFileSync(file)).digest('hex');
    }
  }
  visit(root);
  return entries;
}

function fixture(t, mode) {
  fs.mkdirSync(scratch, { recursive: true });
  const base = fs.mkdtempSync(path.join(scratch, `${mode}-`));
  t.after(() => fs.rmSync(base, { recursive: true, force: true }));
  const root = path.join(base, 'project');
  const data = path.join(base, 'runtime-data');
  const orchestrator = path.join(base, 'orchestrator');
  // CLI project discovery walks ancestors: establish the fixture boundary first.
  fs.mkdirSync(path.join(root, '.kbd-orchestrator'), { recursive: true });
  fs.mkdirSync(data, { recursive: true });
  write(path.join(orchestrator, 'hooks', 'hooks.json'), { hooks: [] });
  fs.cpSync(path.join(sourceOrchestrator, 'shared', 'lib'), path.join(orchestrator, 'shared', 'lib'), { recursive: true });

  // A private, fresh real signer; canonical CLI calls cannot consult the host keychain.
  const { privateKey, publicKey } = generateKeyPairSync('ed25519');
  const privateJwk = privateKey.export({ format: 'jwk' });
  const publicJwk = publicKey.export({ format: 'jwk' });
  const keyFile = path.join(base, 'device-key.json');
  fs.writeFileSync(keyFile, JSON.stringify({
    schemaVersion: '1',
    keyId: `ed25519:${createHash('sha256').update(Buffer.from(publicJwk.x, 'base64url')).digest('hex')}`,
    privateKey: Buffer.from(privateJwk.d, 'base64url').toString('base64'),
  }), { mode: 0o600 });
  const executableDir = path.isAbsolute(prometheus) ? [path.dirname(prometheus)] : [];
  const env = {
    ...process.env,
    PATH: [...executableDir, openspecBins, process.env.PATH || ''].join(path.delimiter),
    PROMETHEUS_DATA_DIR: data,
    PROMETHEUS_DEVICE_KEY_FILE: keyFile,
    PROMETHEUS_KBD_CONTROL_PLANE: '0',
    PROMETHEUS_CONTROL_ENDPOINT: 'http://127.0.0.1:1',
    PROMETHEUS_HARNESS: 'kbd-helper-integration',
    KBD_ORCHESTRATOR_ROOT: orchestrator,
    OPENSPEC_TELEMETRY: '0', DO_NOT_TRACK: '1',
  };
  const run = (program, args) => spawnSync(program, args, { cwd: root, env, encoding: 'utf8', shell: false, timeout: 60000, maxBuffer: 8 * 1024 * 1024 });
  const cli = (...args) => run(prometheus, ['kbd', '--path', root, ...args]);
  const helper = (name, ...args) => run('/bin/bash', [path.join(sourceOrchestrator, 'skills', name, `${name}.sh`), ...args]);
  const pj = path.join(root, '.kbd-orchestrator', 'project.json');
  const wp = path.join(root, '.kbd-orchestrator', 'current-waypoint.json');
  if (mode === 'runtime') {
    const initial = JSON.parse(success(cli('status', '--json'), 'register isolated fixture').stdout);
    assert.ok(initial.runtimePath.startsWith(data + path.sep), 'runtime storage must remain inside fixture');
    const registry = JSON.parse(success(cli('projects', '--json'), 'read fixture registration').stdout);
    const registeredPaths = Object.keys(registry.replicas);
    assert.equal(registeredPaths.length, 1, 'private registry must contain only this fixture');
    assert.equal(fs.realpathSync(registeredPaths[0]), fs.realpathSync(root), 'CLI project discovery must stop at the fixture');
    const fixtureProjectId = read(path.join(root, '.prometheus', 'project.json')).projectId;
    assert.equal(registry.replicas[registeredPaths[0]].projectId, fixtureProjectId);
    success(cli('phase', 'create', '--command-id', 'fixture-create', '--id', 'old-phase', '--title', 'Old phase'), 'initialize real runtime');
    success(cli('phase', 'activate', '--command-id', 'fixture-activate', '--id', 'old-phase', '--exact-next-work', '/kbd-assess old-phase'), 'activate fixture phase');
    const state = JSON.parse(success(cli('status', '--json'), 'confirm initialized fixture identity').stdout);
    assert.equal(state.projectId, fixtureProjectId, 'canonical state must belong to fixture project');
    assert.equal(read(wp).generatedBy, 'kbd-runtime');
  } else {
    write(wp, { schemaVersion: '5', phase: 'old-phase', status: 'assessment_ready', sourceTool: 'integration-test', revision: 1 });
    write(path.join(root, '.kbd-orchestrator', 'phases', 'old-phase', 'progress.json'), { phase: 'old-phase', changes: [] });
  }
  write(pj, metadata);
  return { base, root, data, env, run, cli, helper, pj, wp, mode };
}

function configureHook(f, { exitCode = 0, event = 'phase:before' } = {}) {
  const observer = path.join(f.base, 'observe.mjs');
  write(observer, `import fs from 'node:fs';
import path from 'node:path';
const read = p => JSON.parse(fs.readFileSync(p, 'utf8'));
const project = read(path.join('.kbd-orchestrator', 'project.json'));
const waypoint = read(path.join('.kbd-orchestrator', 'current-waypoint.json'));
const record = { project, waypoint, kind: process.env.KBD_HOOK_KIND, edge: process.env.KBD_HOOK_EDGE, name: process.env.KBD_HOOK_NAME };
fs.appendFileSync('hook-observations.jsonl', JSON.stringify(record) + '\\n');
process.exit(Number(process.argv[2]));
`);
  write(path.join(f.root, '.kbd-orchestrator', 'hooks-config.json'), { hooks: [{
    id: 'integration-observer', event,
    action: { command: [process.execPath, observer, String(exitCode)].map(shellQuote).join(' '), timeout: 15, on_failure: 'warn' },
  }] });
}

function observations(f) {
  const file = path.join(f.root, 'hook-observations.jsonl');
  return fs.existsSync(file) ? fs.readFileSync(file, 'utf8').trim().split('\n').filter(Boolean).map(JSON.parse) : [];
}

function assertCreated(f) {
  assert.equal(read(f.pj).activePhase, 'new-phase');
  assert.equal(read(f.wp).phase, 'new-phase');
  assert.match(fs.readFileSync(path.join(f.root, '.kbd-orchestrator', 'phases', 'new-phase', 'goals.md'), 'utf8'), /Verify integration boundary/);
  assert.ok(fs.existsSync(path.join(f.root, '.kbd-orchestrator', 'phases', 'new-phase', 'progress.json')));
  if (f.mode === 'runtime') {
    const state = JSON.parse(success(f.cli('status', '--json'), 'read canonical phase').stdout);
    assert.equal(state.activePath.phaseId, 'new-phase');
  }
  const seen = observations(f);
  assert.equal(seen.length, 1, 'real phase:before hook must run exactly once');
  assert.equal(seen[0].project.activePhase, 'new-phase', 'metadata must precede hook');
  assert.equal(seen[0].waypoint.phase, 'new-phase', 'activation must precede hook');
  assert.equal(seen[0].kind, 'phase');
  assert.equal(seen[0].edge, 'before');
  assert.equal(seen[0].name, 'new-phase');
}

test('runtime: rejected duplicate canonical phase creates no local goals or postconditions', t => {
  const f = fixture(t, 'runtime');
  configureHook(f);
  success(f.cli('phase', 'create', '--command-id', 'preexisting-new-phase', '--id', 'new-phase', '--title', 'Existing canonical phase'), 'register duplicate fixture phase');
  const phaseDir = path.join(f.root, '.kbd-orchestrator', 'phases', 'new-phase');
  assert.ok(fs.existsSync(phaseDir), 'canonical create must generate the fixture projection');
  fs.rmSync(phaseDir, { recursive: true });
  const projectBefore = fs.readFileSync(f.pj);
  const result = f.helper('kbd-new-phase', 'new-phase', 'Verify integration boundary');
  assert.notEqual(result.status, 0, 'actual CLI must reject duplicate phase registration');
  assert.doesNotMatch(result.stdout, /Completed kbd-new-phase/);
  assert.equal(fs.existsSync(path.join(phaseDir, 'goals.md')), false, 'rejected canonical create must not leave local goals');
  assert.deepEqual(fs.readFileSync(f.pj), projectBefore, 'rejected canonical create must preserve project metadata');
  assert.deepEqual(observations(f), [], 'rejected canonical create must not invoke hooks');
});

for (const mode of ['legacy', 'runtime']) {
  test(`${mode}: phase postconditions preserve metadata and invoke real hook exactly once`, t => {
    const f = fixture(t, mode);
    configureHook(f);
    const result = success(f.helper('kbd-new-phase', 'new-phase', 'Verify integration boundary'), 'create phase');
    assert.match(result.stdout, /Completed kbd-new-phase/);
    assertCreated(f);
    const project = read(f.pj);
    assert.deepEqual(project.custom, metadata.custom);
    assert.equal(project.specBackend, metadata.specBackend);
    assert.equal(project.name, metadata.name);
    assert.equal(Object.hasOwn(project, 'active_phase'), false);
    assert.ok(Number.isFinite(Date.parse(project.updatedAt)));
    assert.deepEqual(observations(f)[0].project, project);
  });

  for (const invalid of ['{broken', '[]', 'null', '42', '"scalar"']) {
    test(`${mode}: invalid project ${invalid} refuses all phase and canonical writes`, t => {
      const f = fixture(t, mode);
      configureHook(f);
      write(f.pj, invalid);
      const before = snapshot(f.base);
      const result = f.helper('kbd-new-phase', 'new-phase', 'Verify integration boundary');
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, /project.*JSON object|malformed project/);
      assert.doesNotMatch(result.stdout, /Completed kbd-new-phase/);
      assert.deepEqual(snapshot(f.base), before);
    });
  }

  test(`${mode}: missing metadata bootstraps only the documented minimal fields`, t => {
    const f = fixture(t, mode);
    fs.rmSync(f.pj);
    configureHook(f);
    success(f.helper('kbd-new-phase', 'new-phase', 'Verify integration boundary'), 'bootstrap project metadata');
    assertCreated(f);
    const project = read(f.pj);
    assert.deepEqual(Object.keys(project).sort(), ['activePhase', 'bootstrappedBy', 'focus_project_path', 'name', 'updatedAt'].sort());
    assert.equal(project.name, path.basename(f.root));
    assert.equal(project.focus_project_path, f.root);
    assert.equal(project.bootstrappedBy, 'kbd-new-phase');
    assert.ok(Number.isFinite(Date.parse(project.updatedAt)));
  });

  test(`${mode}: real failing hook is reported and keeps the activated phase`, t => {
    const f = fixture(t, mode);
    configureHook(f, { exitCode: 7 });
    const result = success(f.helper('kbd-new-phase', 'new-phase', 'Verify integration boundary'), 'create phase despite hook failure');
    assert.match(result.stderr, /hook.*fail/i);
    assert.match(result.stdout, /Completed kbd-new-phase/);
    assertCreated(f);
  });

  for (const boundary of ['begin-task', 'end-task']) {
    test(`${mode}: ${boundary} preserves pending tasks when real OpenSpec cannot load its schema`, t => {
      const f = fixture(t, mode);
      configureHook(f, { event: '*:*' });
      const changeDir = path.join(f.root, 'openspec', 'changes', 'unavailable-backend');
      write(path.join(changeDir, '.openspec.yaml'), 'schema: nonexistent-kbd-integration-schema\n');
      write(path.join(changeDir, 'tasks.md'), '## 1. Pending\n\n- [ ] 1.1 Preserve pending work\n');
      success(f.run('openspec', ['--version']), 'failure case must use the real available OpenSpec CLI');
      const before = snapshot(f.base);
      const result = f.helper('kbd-apply', boundary, 'unavailable-backend', '1', '1', '1', 'Pending work');
      assert.notEqual(result.status, 0);
      assert.doesNotMatch(result.stdout, /Completed (?:task|change)/);
      assert.deepEqual(snapshot(f.base), before, 'progress read failure must precede checkbox, canonical state, and hooks');
    });

    test(`${mode}: ${boundary} refuses missing backend data before mutation or completion`, t => {
      const f = fixture(t, mode);
      configureHook(f, { event: '*:*' });
      const before = snapshot(f.base);
      const result = f.helper('kbd-apply', boundary, 'missing-change', '1', '1', '1', 'Pending work');
      assert.notEqual(result.status, 0);
      assert.doesNotMatch(result.stdout, /Completed (?:task|change)/);
      assert.deepEqual(snapshot(f.base), before, 'backend failure must not register tasks, mutate projections, or invoke hooks');
    });
  }
}

test('actual OpenSpec apply retains remaining work and reports completion only after the last task', t => {
  const f = fixture(t, 'legacy');
  const change = 'real-apply';
  const changeDir = path.join(f.root, 'openspec', 'changes', change);
  write(path.join(f.root, 'openspec', 'config.yaml'), 'schema: spec-driven\n');
  write(path.join(changeDir, '.openspec.yaml'), 'schema: spec-driven\n');
  write(path.join(changeDir, 'proposal.md'), '# Why\n\nVerify actual apply boundaries.\n\n## What Changes\n\nExercise two tasks.\n');
  write(path.join(changeDir, 'design.md'), '# Design\n\nUse real CLI and local files.\n');
  write(path.join(changeDir, 'specs', 'fixture', 'spec.md'), '## ADDED Requirements\n\n### Requirement: Observable work\nWork SHALL be observable.\n\n#### Scenario: Completion\n- **WHEN** work completes\n- **THEN** its checkbox is checked\n');
  const tasks = path.join(changeDir, 'tasks.md');
  write(tasks, '## 1. Integration\n\n- [ ] 1.1 First real task\n- [ ] 1.2 Second real task\n');
  write(path.join(f.root, '.kbd-orchestrator', 'phases', 'old-phase', 'progress.json'), {
    phase: 'old-phase', changes: [{ id: change, title: 'Real apply', status: 'PENDING', tasks_done: 0, tasks_total: 2 }],
  });
  configureHook(f, { event: '*:*' });
  const openSpec = (...args) => f.run('openspec', args);
  success(openSpec('--version'), 'real local OpenSpec must be available');
  const progress = () => JSON.parse(success(openSpec('instructions', 'apply', '--change', change, '--json'), 'read actual OpenSpec progress').stdout).progress;
  assert.equal(progress().remaining, 2);
  for (const index of [1, 2]) {
    success(f.helper('kbd-apply', 'begin-task', change, String(index), String(index), '2', `Real task ${index}`), 'begin actual task');
    const result = success(f.helper('kbd-apply', 'end-task', change, String(index), String(index), '2', `Real task ${index}`), 'end actual task');
    assert.match(result.stdout, /Completed task/);
    if (index === 1) assert.doesNotMatch(result.stdout, /Completed change/);
    else assert.match(result.stdout, /Completed change/);
    assert.equal(progress().remaining, 2 - index);
  }
  assert.equal((fs.readFileSync(tasks, 'utf8').match(/- \[x\]/g) || []).length, 2);
  const seen = observations(f);
  assert.equal(seen.filter(e => e.kind === 'task' && e.edge === 'before').length, 2);
  assert.equal(seen.filter(e => e.kind === 'task' && e.edge === 'after').length, 2);
  assert.equal(seen.filter(e => e.kind === 'change' && e.edge === 'after').length, 1);
});
