// Issue #241. Each scenario drives the shipped CLI or evaluator in a throwaway
// project. The process-restriction scenario requires macOS and sandbox-exec.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash, randomUUID } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import test, { after } from 'node:test';
import { fileURLToPath } from 'node:url';
import { canonical, disposition, eventCounts, fixture, NAMESPACE, step, surface } from './lib/kbd-phase-closure-adapter.mjs';

const repo = fileURLToPath(new URL('../../', import.meta.url));
const mini = process.env.PROMETHEUS_241_MINI_ROOT ?? path.resolve(repo, '../prometheus-skills-mini');
const binary = process.env.PROMETHEUS_CLI_TEST_BINARY;
assert.ok(binary && fs.statSync(binary).isFile(), 'supply the candidate PROMETHEUS_CLI_TEST_BINARY');
const scratch = fs.mkdtempSync(path.join(process.env.PROMETHEUS_241_SCRATCH_PARENT ?? path.dirname(path.resolve(repo)), 'kbd-241-'));
const full = path.join(repo, 'dist/plugins/claude/prometheus-skill-pack/shared/scripts');
const small = path.join(mini, 'dist/plugins/codex/prometheus-skills-mini/scripts');
const issue = 'https://github.com/Prometheus-AGS/prometheus-skill-system/issues/241';
after(() => fs.rmSync(scratch, { recursive: true, force: true }));

const read = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
function proc(ctx, program, args) {
  const out = spawnSync(program, args, { cwd: ctx.project, env: ctx.env, encoding: 'utf8', timeout: 180000, maxBuffer: 32 << 20 });
  if (out.error) throw out.error;
  assert.notEqual(out.status, null, program + ' terminated by ' + out.signal);
  return { code: out.status, stdout: out.stdout, stderr: out.stderr };
}
function ok(out) {
  assert.equal(out.code, 0, out.stdout + out.stderr);
  return out.stdout;
}
function kbd(ctx, ...args) {
  return proc(ctx, binary, ['kbd', '--path', ctx.project, ...args]);
}
function helper(ctx, pack, name, args) {
  const isMini = pack === small;
  return proc(ctx, isMini ? process.execPath : 'bash', [path.join(pack, name + (isMini ? '.mjs' : '.sh')), ...args]);
}
async function setup(label) {
  const ctx = fixture(scratch, label, binary);
  ctx.surface = surface('full-source', repo, { sourceLayout: true });
  const result = await step(ctx, { op: 'adopt' });
  assert.equal(result.exit, 0, result.stderr + result.stdout);
  return ctx;
}
function makeDetails(ctx, exercise = true) {
  const d = disposition(ctx);
  const st = JSON.parse(ok(kbd(ctx, 'status', '--json')));
  // Fifteen observable pre-exit checks, with the real response and oracle for each.
  const checks = [
    ['project', d.projectId, st.projectId], ['run', d.runId, st.runId], ['phase', d.phaseId, ctx.phase],
    ['open', d.disposition, 'OPEN'], ['bound', d.contractBound, true], ['execution', d.dispatchAllowed, true],
    ['successor', d.successorAllowed, false], ['awaiting', d.awaitingSuccessor, false], ['terminal', d.closure, null],
    ['started', d.phaseStatus, 'in_progress'], ['predecessor', d.successor, null], ['format', d.schema, 'kbd.phase-disposition/v1'],
    ['revision', d.binding.bindingRevision, 1], ['scope', Object.keys(d.binding.scenarioDigests).sort(), ['CS-1', 'CS-2']],
    ['snapshot', hash(fs.readFileSync(path.join(ctx.phaseDir, 'adopted', d.binding.contractDigest + '.json'))), d.binding.contractDigest],
  ];
  const exercised = exercise ? checks.map(([name, actual, expected], index) => {
    assert.deepEqual(actual, expected, name);
    const file = 'observed-' + name + '.json';
    const bytes = JSON.stringify({ name, actual, expected });
    fs.writeFileSync(path.join(ctx.project, file), bytes);
    return { id: 'OP-' + (index + 1), verdict: 'PASS', evidence: [{ path: file, sha256: hash(bytes) }] };
  }) : [];
  const decision = { reference: 'issue-241-smaller-check', text: 'Close on the smaller check; withdraw the original scope to the issue.' };
  ok(kbd(ctx, 'decision', 'record', '--id', decision.reference, '--command-id', decision.reference, '--summary', decision.text));
  return { operatorDecision: decision, exercised,
    withdrawn: ['scenario', 'scope'].flatMap(kind => ['CS-1', 'CS-2'].map(id => ({ kind, id, destination: issue }))),
    budgetAccounting: { class: 'unmetered' }, limits: ['The original product scenarios were not exercised.'] };
}
function template(ctx, pack, details) {
  const file = path.join(ctx.base, 'details.json');
  fs.writeFileSync(file, JSON.stringify(details));
  const result = helper(ctx, pack, 'kbd-scope-authority-template', ['supersede-exit', '--phase', ctx.phase,
    '--details', file, '--approver', 'operator@kbd.test', '--reason', 'Operator chose an evidence-limited exit']);
  return JSON.parse(ok(result));
}
function sign(ctx, manifest, key = ctx.keys.operator) {
  const file = path.join(ctx.base, randomUUID() + '.json');
  fs.writeFileSync(file, canonical(manifest) + '\n');
  ok(proc(ctx, 'ssh-keygen', ['-q', '-Y', 'sign', '-f', key, '-n', NAMESPACE, file]));
  return { file, sig: file + '.sig' };
}
function evaluate(ctx, pack, signed) {
  return helper(ctx, pack, 'kbd-phase-close-eval', ['--phase', ctx.phase, '--superseded', signed.file, '--signature', signed.sig]);
}
function rawClose(ctx, signed) {
  return kbd(ctx, 'phase', 'close', '--id', ctx.phase, '--disposition', 'superseded', '--manifest', signed.file, '--signature', signed.sig);
}

test('A signed exit with fifteen observed checks closes the phase and admits its authorized successor', async () => {
  const ctx = await setup('superseded');
  const manifest = template(ctx, full, makeDetails(ctx));
  const signed = sign(ctx, manifest);
  ok(evaluate(ctx, full, signed));
  const d = disposition(ctx);
  assert.equal(d.disposition, 'CLOSED_SUPERSEDED');
  assert.equal(d.dispatchAllowed, false);
  assert.equal(d.successorAllowed, true);
  assert.deepEqual(d.closure.operatorExit, manifest);
  const packet = fs.readFileSync(path.join(ctx.phaseDir, 'closure-packet.md'), 'utf8');
  for (const item of manifest.exercised) assert.ok(packet.includes(item.id), item.id);
  for (const item of manifest.withdrawn) assert.ok(packet.includes(item.kind + ' ' + item.id), item.id);
  const before = eventCounts(ctx);
  assert.equal(JSON.parse(ok(rawClose(ctx, signed))).duplicate, true);
  assert.deepEqual(eventCounts(ctx), before);
  const alternate = sign(ctx, { ...manifest, nonce: randomUUID() });
  assert.notEqual(rawClose(ctx, alternate).code, 0);
  assert.deepEqual(eventCounts(ctx), before);
  const started = await step(ctx, { op: 'cli-successor', args: {
    successor: 'next-phase', authorization: { action: 'start-successor' } } });
  assert.equal(started.exit, 0, started.stdout + started.stderr);
  assert.equal(disposition(ctx).successor.phaseId, 'next-phase');
});

test('A signed manifest that omits a bound scenario is refused without a journal write', async () => {
  const ctx = await setup('missing-scenario');
  const manifest = template(ctx, full, makeDetails(ctx));
  manifest.withdrawn = manifest.withdrawn.filter(row => row.kind !== 'scenario' || row.id !== 'CS-2');
  const before = eventCounts(ctx);
  const out = rawClose(ctx, sign(ctx, manifest));
  assert.notEqual(out.code, 0);
  assert.match(out.stdout + out.stderr, /unaccounted bound scenario CS-2/);
  assert.deepEqual(eventCounts(ctx), before);
  assert.equal(disposition(ctx).disposition, 'OPEN');
});

test('Mini preserves withdrawals in status and the packet without certifying them', async () => {
  const ctx = await setup('no-certification');
  const m = template(ctx, small, makeDetails(ctx));
  ok(evaluate(ctx, small, sign(ctx, m)));
  const status = JSON.parse(ok(kbd(ctx, 'status', '--json')));
  const terminal = status.phaseClosure.phases[ctx.phase].terminal;
  assert.equal(terminal.disposition, 'superseded');
  assert.equal(terminal.scenarios.length, 0);
  assert.deepEqual(terminal.operatorExit.withdrawn, m.withdrawn);
  const packet = read(path.join(ctx.phaseDir, 'closure.json'));
  assert.deepEqual(packet.scenarios.map(row => [row.id, row.verdict]).sort(), [['CS-1', 'WITHDRAWN'], ['CS-2', 'WITHDRAWN']]);
});

function restricted(ctx, ...args) {
  const root = fs.realpathSync(path.join(ctx.project, '.prometheus'));
  const profile = '(version 1)(allow default)(deny file-write* (subpath ' + JSON.stringify(root) + '))';
  return proc(ctx, '/usr/bin/sandbox-exec', ['-p', profile, binary, 'kbd', '--path', ctx.project, ...args]);
}
const registryFile = ctx => path.join(ctx.env.PROMETHEUS_DATA_DIR, 'prometheus', 'kbd', 'registry.json');
test('Restricted read processes preserve the registry and a later unrestricted write succeeds', async () => {
  const ctx = await setup('reads');
  const file = registryFile(ctx);
  const snapshot = fs.readFileSync(file);
  for (const args of [['status', '--json'], ['replicas', '--json'], ['phase', 'disposition', '--id', ctx.phase, '--json'], ['audit', '--json']]) {
    ok(restricted(ctx, ...args));
    assert.deepEqual(fs.readFileSync(file), snapshot, args.join(' '));
  }
  ok(kbd(ctx, 'decision', 'record', '--command-id', 'read-recovery', '--id', 'read-recovery', '--summary', 'Write after restricted reads'));
});

test('A real lock denial names recovery and re-probing clears an old transient flag only', async () => {
  const ctx = await setup('reprobe');
  const refused = restricted(ctx, 'decision', 'record', '--command-id', 'probe-denied', '--id', 'probe-denied', '--summary', 'Restricted write');
  assert.notEqual(refused.code, 0);
  assert.match(refused.stdout + refused.stderr, /replicas --reprobe/);
  assert.ok((refused.stdout + refused.stderr).includes(ctx.project), refused.stderr);
  const file = registryFile(ctx);
  const registry = read(file);
  const row = registry.replicas[fs.realpathSync(ctx.project)];
  assert.equal(row.readOnly, false);
  // Import the exact stale classification persisted by an older runtime.
  row.readOnly = true;
  row.readOnlyReason = 'cross-process flock exclusion could not be proven: project volume: cannot create lock probe file: Operation not permitted (os error 1)';
  fs.writeFileSync(file, JSON.stringify(registry));
  ok(kbd(ctx, 'replicas', '--reprobe', '--replica', row.replicaId, '--json'));
  assert.equal(read(file).replicas[fs.realpathSync(ctx.project)].readOnly, false);
  ok(kbd(ctx, 'decision', 'record', '--command-id', 'probe-restored', '--id', 'probe-restored', '--summary', 'Writable after recovery'));
  const restored = read(file);
  const protectedRow = restored.replicas[fs.realpathSync(ctx.project)];
  protectedRow.kind = 'recovered';
  protectedRow.readOnly = true;
  protectedRow.readOnlyReason = 'recovered replicas are observation-only';
  fs.writeFileSync(file, JSON.stringify(restored));
  ok(kbd(ctx, 'replicas', '--reprobe', '--replica', row.replicaId, '--json'));
  assert.equal(read(file).replicas[fs.realpathSync(ctx.project)].readOnly, true);
});

test('An unknown owner task fails lint and both templates before signing', async () => {
  const ctx = await setup('owner-task');
  const file = path.join(ctx.phaseDir, 'adopted-contract.json');
  const candidate = read(path.join(repo, 'shared/references/conformance/kbd-contract-v2/golden/contract.json'));
  const bound = read(file);
  candidate.identity = bound.identity;
  for (const scenario of candidate.scenarioMatrix) scenario.ownerTask = 'T1';
  candidate.scenarioMatrix[0].ownerTask = 'UNREGISTERED';
  fs.writeFileSync(file, JSON.stringify(candidate));
  const rel = path.relative(ctx.project, file);
  for (const pack of [full, small]) {
    const lint = helper(ctx, pack, 'kbd-contract-adopt', [rel, '--lint-only']);
    assert.notEqual(lint.code, 0);
    assert.match(lint.stdout + lint.stderr, /owner-task/);
    const out = helper(ctx, pack, 'kbd-scope-authority-template', ['amend-contract', '--phase', ctx.phase, '--contract', rel,
      '--reason', 'Validate actual owner before signing', '--approver', 'operator@kbd.test']);
    assert.notEqual(out.code, 0);
    assert.equal(out.stdout.trim(), '', out.stdout);
    assert.ok(out.stderr.includes(candidate.scenarioMatrix[0].id), out.stderr);
    assert.match(out.stderr, /owner-task/);
    assert.doesNotMatch(out.stderr, /Sign with:/);
  }
});

test('A canonically bound phase with no registered tasks rejects adoption and both pre-sign templates', async () => {
  const ctx = await setup('empty-task-registry');
  const phase = 'empty-owner-phase';
  ok(kbd(ctx, 'phase', 'create', '--id', phase, '--title', 'Unregistered task owners', '--command-id', 'create-empty'));
  const status = JSON.parse(ok(kbd(ctx, 'status', '--json')));
  assert.equal(Object.keys(status.phases[phase].changes).length, 0);
  const dir = path.join(ctx.project, '.kbd-orchestrator/phases', phase);
  const file = path.join(dir, 'adopted-contract.json');
  const rel = path.relative(ctx.project, file);
  const contract = read(path.join(repo, 'shared/references/conformance/kbd-contract-v2/golden/contract.json'));
  contract.identity = { projectId: status.projectId, runId: status.runId, phaseId: phase };
  for (const scenario of contract.scenarioMatrix) scenario.ownerTask = 'T1';
  const snapshot = canonical(contract) + '\n';
  const digest = hash(snapshot);
  fs.mkdirSync(path.join(dir, 'adopted'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'adopted', digest + '.json'), snapshot);
  // A legacy or directly bound phase can predate all task registrations.
  ok(kbd(ctx, 'phase', 'bind-contract', '--id', phase, '--contract-path', rel, '--contract-digest', digest,
    '--plan-revision', String(status.planRevision), '--adoption-decision', 'original-empty-registry',
    ...contract.scenarioMatrix.flatMap(scenario => ['--scenario-digest', scenario.id + '=' + hash(canonical(scenario) + '\n')])));
  contract.budgets.timeMinutes += 1;
  fs.writeFileSync(file, JSON.stringify(contract));
  for (const pack of [full, small]) {
    const adopt = helper(ctx, pack, 'kbd-contract-adopt', [rel]);
    assert.notEqual(adopt.code, 0);
    assert.match(adopt.stdout + adopt.stderr, /owner-task/);
    const out = helper(ctx, pack, 'kbd-scope-authority-template', ['amend-contract', '--phase', phase,
      '--contract', rel, '--reason', 'Amend the bound phase budget', '--approver', 'operator@kbd.test']);
    assert.notEqual(out.code, 0);
    assert.equal(out.stdout.trim(), '');
    assert.match(out.stderr, /owner-task/);
    assert.doesNotMatch(out.stderr, /Sign with:/);
  }
  const view = JSON.parse(ok(kbd(ctx, 'phase', 'disposition', '--id', phase, '--json')));
  assert.equal(view.binding.contractDigest, digest);
  assert.equal(view.binding.bindingRevision, 1);
});

test('Both amendment templates accept the phase directory and refuse identical input stored outside it', async () => {
  const ctx = await setup('misplaced-amendment');
  const file = path.join(ctx.phaseDir, 'adopted-contract.json');
  const candidate = read(file);
  candidate.budgets.timeMinutes += 1;
  const valid = path.relative(ctx.project, file);
  const misplaced = 'misplaced-amendment.json';
  fs.writeFileSync(file, JSON.stringify(candidate));
  fs.copyFileSync(file, path.join(ctx.project, misplaced));
  for (const pack of [full, small]) {
    const render = contract => helper(ctx, pack, 'kbd-scope-authority-template', ['amend-contract',
      '--phase', ctx.phase, '--contract', contract, '--reason', 'Extend the operator-controlled budget',
      '--approver', 'operator@kbd.test']);
    assert.equal(JSON.parse(ok(render(valid))).contractPath, valid);
    const bad = render(misplaced);
    assert.notEqual(bad.code, 0);
    assert.equal(bad.stdout.trim(), '');
    assert.match(bad.stderr, /contract is not at/);
    assert.doesNotMatch(bad.stderr, /Sign with:/);
  }
});

test('An invalid signer or stale evidence cannot close; an explicit empty-evidence exit records CLOSED_UNMET', async () => {
  const ctx = await setup('unmet');
  const details = makeDetails(ctx);
  const m = template(ctx, full, details);
  const before = eventCounts(ctx);
  assert.notEqual(rawClose(ctx, sign(ctx, m, ctx.keys.intruder)).code, 0);
  fs.appendFileSync(path.join(ctx.project, details.exercised[0].evidence[0].path), 'changed');
  const stale = rawClose(ctx, sign(ctx, m));
  assert.notEqual(stale.code, 0);
  assert.match(stale.stdout + stale.stderr, /stale evidence/);
  assert.deepEqual(eventCounts(ctx), before);
  assert.equal(disposition(ctx).disposition, 'OPEN');
  details.exercised = [];
  const abandoned = sign(ctx, template(ctx, small, details));
  ok(evaluate(ctx, small, abandoned));
  const view = disposition(ctx);
  assert.equal(view.disposition, 'CLOSED_UNMET');
  assert.equal(view.dispatchAllowed, false);
  assert.equal(view.successorAllowed, true);
  assert.equal(view.closure.operatorExit.exercised.length, 0);
});
