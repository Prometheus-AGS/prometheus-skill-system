// Full #241 production-boundary coverage. Uses an installed generation, a real
// signed runtime journal and real hooks, with all writable state in scratch.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';
import { disposition, fixture, step, surface } from './lib/kbd-phase-closure-adapter.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const json = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const rows = file => fs.existsSync(file)
  ? fs.readFileSync(file, 'utf8').trim().split('\n').filter(Boolean).map(line => JSON.parse(line)) : [];

function execute(ctx, command, args, expected = 0) {
  const run = spawnSync(command, args, {
    cwd: ctx.project, env: ctx.env, encoding: 'utf8', timeout: 300_000, maxBuffer: 32 * 1024 * 1024,
  });
  assert.equal(run.status, expected, `${command} ${args.join(' ')}\n${run.error ?? ''}\n${run.stdout}\n${run.stderr}`);
  return run.stdout;
}

function kbd(ctx, ...args) {
  return execute(ctx, ctx.prometheusBin, ['kbd', '--path', ctx.project, ...args]);
}

function checked(out) {
  if (out) assert.equal(out.exit, 0, out.stdout + out.stderr);
}

test('Full installed reflection and phase hook effects, retry, nested attribution and projection drift', async t => {
  assert.ok(process.env.PROMETHEUS_CLI_TEST_BINARY, 'PROMETHEUS_CLI_TEST_BINARY is required');
  const binary = fs.realpathSync(process.env.PROMETHEUS_CLI_TEST_BINARY);
  assert.equal(path.basename(binary), 'prometheus');
  const scratch = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'kbd-after-241-')));
  t.after(() => fs.rmSync(scratch, { recursive: true, force: true }));

  const ctx = fixture(scratch, 'consumer', binary);
  // Use the actual candidate and runtime tools in a private PATH. Optional
  // host model clients and machine-global learning services stay out of the fixture.
  const bin = path.join(scratch, 'bin');
  fs.mkdirSync(bin);
  for (const [name, target] of Object.entries({
    prometheus: binary, node: process.execPath,
    jq: spawnSync('which', ['jq'], { encoding: 'utf8' }).stdout.trim(),
    python3: spawnSync('which', ['python3'], { encoding: 'utf8' }).stdout.trim(),
  })) {
    assert.ok(target && fs.existsSync(target), `missing integration prerequisite ${name}`);
    fs.symlinkSync(target, path.join(bin, name));
  }
  ctx.env.PATH = [bin, '/usr/bin', '/bin', '/usr/sbin', '/sbin'].join(path.delimiter);
  const payload = path.join(root, 'dist/plugins/codex/prometheus-skill-pack');
  const home = ctx.env.HOME;
  const store = path.join(home, '.prometheus/plugins/prometheus-skill-pack');
  Object.assign(ctx.env, {
    PROMETHEUS_PLUGIN_ROOT: store,
    KBD_MEMORY_MCP_URL: 'http://127.0.0.1:1',
    SURREAL_MEMORY_URL: 'http://127.0.0.1:1',
    PK_BIN: path.join(scratch, 'pk-unavailable'),
    PROMETHEUS_LEARNING_LOG_DIR: path.join(scratch, 'lessons'),
    PROMETHEUS_LEARNING_QUEUE: path.join(scratch, 'queue'),
    PROMETHEUS_LEARNING_INDEX_DIR: path.join(scratch, 'lesson-index'),
    PROMETHEUS_LEARNING_CORTEX: '0',
    PROMETHEUS_LEARNING_PK: '0',
  });
  execute(ctx, process.execPath, [path.join(payload, 'scripts/install-plugin-generation.js'),
    '--source-root', payload, '--home', home, '--plugin-root', store, '--targets', 'agents', '--domains', 'all']);
  const pkg = fs.realpathSync(path.join(store, 'current'));
  ctx.surface = surface('installed Full', pkg, { sourceLayout: false });
  ctx.env.KPM_MEMORY_ENQUEUE = path.join(pkg, 'shared/scripts/enqueue-memory-operation.py');
  const skills = path.join(home, '.agents/skills');
  const orch = path.join(skills, 'kbd-process-orchestrator');
  const finalize = path.join(skills, 'kbd-reflect/scripts/finalize.sh');
  const handoffs = path.join(orch, 'shared/lib/stage-gate.sh');
  const log = path.join(ctx.phaseDir, 'hooks.log.jsonl');
  const progressFile = path.join(ctx.phaseDir, 'progress.json');
  const lessons = path.join(scratch, 'lessons/lessons.jsonl');
  const phase = ctx.phase;

  await t.test('exact registered task repairs its OpenSpec checkbox and reports a stale 0/0 projection', () => {
    const tasks = path.join(ctx.project, 'openspec/changes/C1/tasks.md');
    fs.mkdirSync(path.dirname(tasks), { recursive: true });
    fs.writeFileSync(tasks, '- [ ] Closure task\n- [ ] Unassigned obligation\n');
    for (const state of ['in-progress', 'complete']) {
      kbd(ctx, 'task', 'transition', '--phase', phase, '--change', 'C1', '--id', 'T1',
        '--command-id', 'fixture-task-' + state, '--status', state);
    }
    const saved = fs.readFileSync(progressFile);
    fs.writeFileSync(progressFile, JSON.stringify({ ...json(progressFile), changes: [] }));
    const inspect = repair => JSON.parse(execute(ctx, process.execPath,
      [path.join(skills, 'kbd-apply/reconcile-command.mjs'), phase, '--json', ...(repair ? ['--repair'] : [])], 1));
    const before = inspect(false);
    assert.deepEqual(before.taskCounts.runtime, { completed: 1, total: 1 });
    assert.deepEqual(before.taskCounts.projection, { completed: 0, total: 0 });
    assert.deepEqual(before.taskCounts.backend, { completed: 0, total: 2 });
    assert.ok(before.drift.some(row => row.kind === 'ledger-ahead'));
    assert.ok(before.drift.some(row => row.kind === 'unmappable'));
    // Resolve the unrelated row explicitly into separate follow-up scope;
    // repair then has an unambiguous one-to-one change.
    fs.writeFileSync(tasks, '- [ ] Closure task\n');
    const after = inspect(true);
    assert.equal(fs.readFileSync(tasks, 'utf8'), '- [x] Closure task\n');
    assert.deepEqual(after.taskCounts.backend, { completed: 1, total: 1 });
    assert.ok(after.drift.some(row => row.kind === 'projection'));
    fs.writeFileSync(progressFile, saved);
  });

  await t.test('rejected reflection produces no writeback and an accepted reflection persists a lesson and closes', async () => {
    checked(await step(ctx, { op: 'adopt' }));
    await step(ctx, { op: 'evidence', args: { statuses: { 'CS-1': 'PASS', 'CS-2': 'PASS' } } });
    fs.writeFileSync(path.join(ctx.phaseDir, 'reflection.md'),
      '# Reflection\n\n## Delta\nOne task projection lagged.\n\n## Root Cause\nThe projection missed an update.\n\n## Corrective Actions\nReconcile the task identities.\n\n## Lessons Learned\n- Verify the 241 consumer task projection after completion.\n');
    execute(ctx, 'bash', ['-c', '. "$1"; kbd_stage_handoff_write execute "Implemented the boundary" "$2" reflection.md',
      'handoff', handoffs, ctx.phaseDir]);
    const saved = fs.readFileSync(progressFile);
    fs.writeFileSync(progressFile, JSON.stringify({ ...json(progressFile), reflect_gate: 'rejected' }));
    execute(ctx, 'bash', [finalize, 'reflect', phase], 2);
    assert.ok(!rows(log).some(row => row.kind === 'reflect' && row.edge === 'after'));
    assert.ok(!rows(lessons).some(row => row.text?.includes('241 consumer')));
    fs.writeFileSync(progressFile, saved);
    // A directory at the output path makes the real method-context write fail.
    // Closure is durable, but the caller must see the pending delivery.
    const methodIndex = path.join(ctx.phaseDir, 'method-context.json');
    fs.rmSync(methodIndex, { force: true });
    fs.mkdirSync(methodIndex);
    execute(ctx, 'bash', [finalize, 'reflect', phase], 2);
    assert.equal(disposition(ctx).disposition, 'CLOSED');
    const failed = rows(log).filter(row => row.hookId === 'method-context-phase-after' && row.status !== 0);
    assert.equal(failed.length, 1);
    fs.rmdirSync(methodIndex);
    execute(ctx, 'bash', [finalize, 'reflect', phase]);
    assert.equal(disposition(ctx).disposition, 'CLOSED');
    assert.ok(rows(lessons).some(row => row.text?.includes('Verify the 241 consumer task projection')));
    for (const id of ['memory-reflection-writeback', 'karpathy-progress-phase-boundary', 'method-context-phase-after']) {
      const events = rows(log).filter(row => row.hookId === id && row.edge === 'after' && row.status === 0);
      assert.equal(events.length, 1, id);
      assert.equal(events[0].status, 0, JSON.stringify(events[0]));
      assert.equal(events[0].name, phase);
    }
    assert.ok(fs.existsSync(path.join(ctx.phaseDir, 'method-context.json')));
    assert.match(fs.readFileSync(path.join(ctx.project, '.prometheus/session-log.md'), 'utf8'), /closure-phase/);
  });

  await t.test('retry delivers a failed action once and attributes it to the predecessor after successor activation', async () => {
    checked(await step(ctx, { op: 'cli-successor', args: { successor: 'successor', authorization: { action: 'start-successor' } } }));
    const script = path.join(scratch, 'delivery.mjs');
    const output = path.join(scratch, 'delivered.jsonl');
    const ready = path.join(scratch, 'ready');
    fs.writeFileSync(script, `import fs from 'node:fs';
const [ready, output] = process.argv.slice(2);
if (!fs.existsSync(ready)) process.exit(3);
fs.appendFileSync(output, JSON.stringify({id:process.env.KBD_HOOK_PHASE_ID, dir:process.env.KBD_HOOK_PHASE_DIR}) + '\\n');
`);
    fs.writeFileSync(path.join(ctx.project, '.kbd-orchestrator/hooks-config.json'), JSON.stringify({ hooks: [{
      id: 'observer', event: 'phase:after', action: { command: process.execPath, args: [script, ready, output], on_failure: 'error' },
    }] }));
    execute(ctx, 'bash', [finalize, 'phase-after', phase], 3);
    fs.writeFileSync(ready, '');
    execute(ctx, 'bash', [finalize, 'phase-after', phase]);
    execute(ctx, 'bash', [finalize, 'phase-after', phase]);
    assert.deepEqual(rows(output), [{ id: phase, dir: ctx.phaseDir }]);
    assert.deepEqual(rows(log).filter(row => row.hookId === 'observer').map(row => row.status), [3, 0]);
    assert.equal(rows(log).filter(row => row.hookId === 'karpathy-progress-phase-boundary' && row.status === 0).length, 1);
    const nextLog = path.join(ctx.project, '.kbd-orchestrator/phases/successor/hooks.log.jsonl');
    assert.ok(!rows(nextLog).some(row => row.name === phase));
  });

  await t.test('child exit delivers the legacy completion to the nested phase before resuming its parent', () => {
    const child = 'successor::child';
    const childDir = path.join(ctx.project, '.kbd-orchestrator/phases/successor/children/child');
    kbd(ctx, 'phase', 'create', '--command-id', 'child/create', '--id', child, '--slug', 'child', '--parent', 'successor', '--title', 'Child');
    kbd(ctx, 'phase', 'activate', '--command-id', 'child/activate', '--id', child, '--ancestor', 'successor');
    kbd(ctx, 'guard', 'evaluate', '--boundary', 'phase', '--edge', 'before', '--subject', child, '--json', '--repair-projections');
    kbd(ctx, 'phase', 'transition', '--command-id', 'child/start', '--id', child, '--status', 'in-progress');
    fs.writeFileSync(path.join(childDir, 'reflection.md'), '# Child reflection\n');
    ctx.env.KBD_ORCHESTRATOR_ROOT = orch;
    const grandchild = child + '::leaf';
    const grandchildDir = path.join(childDir, 'children/leaf');
    kbd(ctx, 'phase', 'create', '--command-id', 'leaf/create', '--id', grandchild, '--slug', 'leaf', '--parent', child, '--title', 'Leaf');
    kbd(ctx, 'phase', 'activate', '--command-id', 'leaf/activate', '--id', grandchild, '--ancestor', 'successor', '--ancestor', child);
    kbd(ctx, 'guard', 'evaluate', '--boundary', 'phase', '--edge', 'before', '--subject', grandchild, '--json', '--repair-projections');
    kbd(ctx, 'phase', 'transition', '--command-id', 'leaf/start', '--id', grandchild, '--status', 'in-progress');
    fs.writeFileSync(path.join(grandchildDir, 'reflection.md'), '# Grandchild reflection\n');
    execute(ctx, 'bash', [path.join(orch, 'skills/kbd-child-exit/kbd-child-exit.sh')]);
    assert.deepEqual(JSON.parse(kbd(ctx, 'status', '--json')).activePath.phasePath, ['successor', child]);
    assert.equal(disposition(ctx, grandchild).phaseStatus, 'complete');
    execute(ctx, 'bash', [path.join(orch, 'skills/kbd-child-exit/kbd-child-exit.sh')]);
    assert.equal(JSON.parse(kbd(ctx, 'status', '--json')).activePath.phaseId, 'successor');
    assert.equal(disposition(ctx, child).phaseStatus, 'complete');
    const childLog = rows(path.join(childDir, 'hooks.log.jsonl'));
    assert.ok(childLog.some(row => row.kind === 'phase' && row.edge === 'after' && row.name === child && row.status === 0));
  });
});
