// IDR-001..003: change identity diagnosis, signed repair and task continuation.
import fs from 'node:fs';
import path from 'node:path';
import { World, PHASE, sha256, writeFile } from './kbd-194-196-world.mjs';

const ORCH = (ROOT) => path.join(ROOT, 'skills/process/kbd-process-orchestrator');
const KBD_APPLY = (ROOT) => path.join(ORCH(ROOT), 'skills/kbd-apply/kbd-apply.sh');
const SHARED_TITLE = '1.1 Shared task';
const TARGET_TASK = 'target-t1';

/** Resolution mapping derived from the real inspection output (digests are never invented). */
function resolution(world, { omitTask = false, omitReceipt = false, badDigest = false } = {}) {
  const inspect = JSON.parse(world.kbd(['change', 'inspect-identity', '--phase', PHASE]).stdout);
  const sources = Object.fromEntries(inspect.changes.map((c) => [c.key, badDigest ? 'f'.repeat(64) : c.sha256]));
  const taskMappings = [{ sourceKey: 'alias-a', sourceTaskKey: '1', targetTaskKey: TARGET_TASK }];
  if (!omitTask) taskMappings.push({ sourceKey: 'target', sourceTaskKey: '1', targetTaskKey: TARGET_TASK });
  const receiptMappings = omitReceipt ? [] : [{ sourceKey: 'change:alias-a', targetSubject: 'target' }, { sourceKey: 'task:alias-a/1', targetSubject: `target/${TARGET_TASK}` }];
  return { inspect, mapping: { sources, survivorKey: 'target', targetKey: 'target', taskMappings, receiptMappings } };
}

function repair(world, mapping, { expectedRevision, commandId, reason = 'operator confirmed damaged identity', env } = {}) {
  const file = path.join(world.base, `resolution-${commandId}.json`);
  writeFile(file, mapping);
  return world.kbd(['change', 'repair-identity', '--command-id', commandId, '--phase', PHASE,
    '--expected-revision', String(expectedRevision), '--resolution-file', file, '--reason', reason], env ? { env } : undefined);
}

const stable = (state) => JSON.stringify({ phases: state.phases, obligations: state.boundaryObligations, repairs: state.changeIdentityRepairs });

async function idr001({ prometheus, seeder, ROOT }) {
  const w = new World('idr001', { prometheus, ROOT }).healthy();
  const before = w.state();
  const original = before.phases[PHASE].changes.C1;

  const conflict = w.kbd(['change', 'register', '--command-id', 'dup-conflict', '--phase', PHASE, '--id', 'C1', '--title', 'Conflicting title']);
  const afterConflict = w.state();
  w.ok('conflicting registration is refused', conflict.exit !== 0 && /change C1 already exists/.test(conflict.stdout + conflict.stderr), conflict.stderr);
  w.ok('original change record is byte-for-byte preserved', JSON.stringify(afterConflict.phases[PHASE].changes.C1) === JSON.stringify(original), afterConflict.phases[PHASE].changes.C1);
  w.ok('no revision is consumed by the refused registration', afterConflict.revision === before.revision, `${before.revision} -> ${afterConflict.revision}`);

  const retry = w.kbd(['change', 'register', '--command-id', 'fx-change', '--phase', PHASE, '--id', 'C1', '--title', 'First change']);
  w.ok('equivalent retry is idempotent and creates no duplicate', retry.exit === 0 && w.state().revision === before.revision
    && Object.keys(w.state().phases[PHASE].changes).length === 1, retry.stderr);
  const reuse = w.kbd(['change', 'register', '--command-id', 'fx-change', '--phase', PHASE, '--id', 'C1', '--title', 'Different content']);
  w.ok('a command id reused for different content is refused', reuse.exit !== 0 && /different change registration content|different change identity/i.test(reuse.stdout + reuse.stderr), reuse.stderr);

  // Diagnostics for a task whose qualified key is absent versus an ambiguous title.
  w.kbd(['change', 'register', '--command-id', 'fx-change2', '--phase', PHASE, '--id', 'C2', '--title', 'Second change']);
  w.kbd(['task', 'register', '--command-id', 'fx-task2', '--phase', PHASE, '--change', 'C2', '--id', 'T1', '--title', 'Shared title']);
  const missing = w.kbd(['guard', 'evaluate', '--boundary', 'task', '--edge', 'before', '--subject', 'C1/T9', '--json']);
  w.ok('missing qualified task key is reported as a missing key, not a missing receipt',
    /missing qualified task key C1\/T9/.test(missing.stdout + missing.stderr), (missing.stdout + missing.stderr).slice(0, 400));
  const ambiguous = w.kbd(['guard', 'evaluate', '--boundary', 'task', '--edge', 'before', '--subject', 'Shared title', '--json']);
  w.ok('an ambiguous title names the multiple matches', /has 2 matches/.test(ambiguous.stdout + ambiguous.stderr), (ambiguous.stdout + ambiguous.stderr).slice(0, 400));

  // Duplicate legacy input is refused with every competing identity named and the bytes untouched.
  const legacy = new World('idr001-legacy', { prometheus, ROOT });
  const phaseDir = path.join(legacy.project, '.kbd-orchestrator/phases/legacy-dup');
  const progress = `${JSON.stringify({ phase: 'legacy-dup', status: 'in_progress', changes: [
    { id: 'dup-id', title: 'first', status: 'complete', tasks_done: 1, tasks_total: 1 },
    { id: 'dup-id', title: 'second', status: 'pending', tasks_done: 0, tasks_total: 1 }] }, null, 2)}\n`;
  writeFile(path.join(phaseDir, 'progress.json'), progress);
  const importDup = legacy.kbd(['migrate', '--apply']);
  w.ok('duplicate legacy rows are refused naming the repeated ID and both rows',
    importDup.exit !== 0 && /duplicate change ID "dup-id" at rows\/keys "0" and "1"/.test(importDup.stdout + importDup.stderr), (importDup.stdout + importDup.stderr).slice(0, 500));
  w.ok('legacy input bytes are unchanged', fs.readFileSync(path.join(phaseDir, 'progress.json'), 'utf8') === progress, 'sha256 compare');

  // Reconcile names a duplicated progress ID and a canonical key/embedded-ID mismatch.
  const dmg = new World('idr001-dmg', { prometheus, ROOT }).damaged(seeder, 'equal');
  const dmgDir = path.join(dmg.project, '.kbd-orchestrator/phases', PHASE);
  writeFile(path.join(dmgDir, 'progress.json'), { phase: PHASE, changes: [{ id: 'target', tasks_done: 0, tasks_total: 1 }, { id: 'target', tasks_done: 0, tasks_total: 1 }] });
  const dupRows = dmg.run('bash', [KBD_APPLY(ROOT), 'reconcile', PHASE, '--json'], { env: { ...dmg.env, KBD_ORCHESTRATOR_ROOT: ORCH(ROOT) } });
  w.ok('reconcile names the repeated progress ID', /duplicate progress change ID \\?"target\\?"/.test(dupRows.stdout + dupRows.stderr), (dupRows.stdout + dupRows.stderr).slice(0, 500));
  writeFile(path.join(dmgDir, 'progress.json'), { phase: PHASE, changes: [{ id: 'target', tasks_done: 0, tasks_total: 1 }] });
  const canon = dmg.run('bash', [KBD_APPLY(ROOT), 'reconcile', PHASE, '--json'], { env: { ...dmg.env, KBD_ORCHESTRATOR_ROOT: ORCH(ROOT) } });
  w.ok('reconcile names the canonical key/embedded-ID mismatch and duplicate identity',
    /change key \\?"alias-a\\?" differs from embedded ID \\?"target\\?"/.test(canon.stdout + canon.stderr) && /duplicate canonical change ID/.test(canon.stdout + canon.stderr), (canon.stdout + canon.stderr).slice(0, 600));
  const inspect = JSON.parse(dmg.kbd(['change', 'inspect-identity', '--phase', PHASE]).stdout);
  w.ok('inspect-identity reports both competing keys without changing state', inspect.issues.length >= 2 && inspect.changes.length === 2 && inspect.changes.every((c) => /^[0-9a-f]{64}$/.test(c.sha256)), inspect.issues);

  const outcome = w.result('conflicting registration, duplicate legacy import, reconcile and inspect diagnostics');
  outcome.commands = [...w.log, ...legacy.log, ...dmg.log];
  for (const x of [w, legacy, dmg]) x.cleanup();
  return outcome;
}

async function idr002({ prometheus, seeder, ROOT }) {
  const w = new World('idr002', { prometheus, ROOT }).damaged(seeder, 'equal');
  const start = w.state();
  const startJournal = w.journalKinds();
  const refuse = (name, mapping, extra, expect) => {
    const before = w.state();
    const r = repair(w, mapping, { commandId: `neg-${name}`, expectedRevision: before.revision, ...extra });
    const after = w.state();
    w.ok(`${name}: refused`, r.exit !== 0 && expect.test(r.stdout + r.stderr), (r.stdout + r.stderr).slice(0, 300));
    w.ok(`${name}: journal and state unchanged`, stable(before) === stable(after) && before.revision === after.revision, `${before.revision}->${after.revision}`);
  };
  refuse('incomplete-task-mapping', resolution(w, { omitTask: true }).mapping, {}, /every source task requires an explicit mapping/);
  refuse('unmapped-receipt', resolution(w, { omitReceipt: true }).mapping, {}, /outstanding source receipt requires an explicit mapping/);
  refuse('stale-source-digest', resolution(w, { badDigest: true }).mapping, {}, /source content digest changed/);
  const stale = repair(w, resolution(w).mapping, { commandId: 'neg-stale-revision', expectedRevision: Math.max(0, start.revision - 1) });
  w.ok('stale expected revision is refused', stale.exit !== 0 && /expected revision \d+, current revision is \d+/.test(stale.stdout + stale.stderr), (stale.stdout + stale.stderr).slice(0, 300));
  w.ok('stale revision left the journal unchanged', JSON.stringify(w.journalKinds()) === JSON.stringify(startJournal), w.journalKinds());

  const other = path.join(w.base, 'other-key.json');
  const unenrolled = { ...w.env, PROMETHEUS_DEVICE_KEY_FILE: other };
  const keygen = w.run('node', ['-e', `const c=require('crypto'),f=require('fs');const p=c.generateKeyPairSync('ed25519');const pub=p.publicKey.export({format:'jwk'});const pr=p.privateKey.export({format:'jwk'});f.writeFileSync(${JSON.stringify(other)},JSON.stringify({schemaVersion:'1',keyId:'ed25519:'+c.createHash('sha256').update(Buffer.from(pub.x,'base64url')).digest('hex'),privateKey:Buffer.from(pr.d,'base64url').toString('base64')}),{mode:0o600})`]);
  w.ok('the unenrolled signer key was generated', keygen.exit === 0 && fs.existsSync(other), keygen.stderr);
  const noAuthority = repair(w, resolution(w).mapping, { commandId: 'neg-unenrolled', expectedRevision: w.state().revision, env: unenrolled });
  w.ok('a signer that is not the journal operator is refused', noAuthority.exit !== 0 && /is not the journal's active enrolled operator/.test(noAuthority.stdout + noAuthority.stderr), (noAuthority.stdout + noAuthority.stderr).slice(0, 300));
  w.ok('unauthorized signer left the journal unchanged', JSON.stringify(w.journalKinds()) === JSON.stringify(startJournal), w.journalKinds());

  const conflictWorld = new World('idr002-conflict', { prometheus, ROOT }).damaged(seeder, 'conflict');
  const cres = resolution(conflictWorld);
  const conflict = repair(conflictWorld, cres.mapping, { commandId: 'neg-conflict', expectedRevision: conflictWorld.state().revision });
  w.ok('conflicting task outcomes are refused without fabricating completion', conflict.exit !== 0 && /conflicting outcomes\/content mapped to task/.test(conflict.stdout + conflict.stderr), (conflict.stdout + conflict.stderr).slice(0, 300));
  w.ok('conflicting fixture was not mutated', !('change_identity_repaired' in conflictWorld.journalKinds()), conflictWorld.journalKinds());

  // The authorized repair.
  const pre = w.state();
  const { mapping } = resolution(w);
  const good = repair(w, mapping, { commandId: 'repair-identity-1', expectedRevision: pre.revision });
  const post = w.state();
  const keys = Object.keys(post.phases[PHASE].changes);
  const tasks = Object.keys(post.phases[PHASE].changes.target?.tasks ?? {});
  w.ok('authorized repair succeeds with one new revision', good.exit === 0 && post.revision === pre.revision + 1, `${pre.revision}->${post.revision} ${good.stderr}`);
  w.ok('effective identity is unique', keys.length === 1 && keys[0] === 'target' && post.phases[PHASE].changes.target.id === 'target', keys);
  w.ok('the merged task is the single mapped task, not fabricated complete', tasks.length === 1 && tasks[0] === TARGET_TASK
    && post.phases[PHASE].changes.target.tasks[TARGET_TASK].status === 'in_progress', post.phases[PHASE].changes.target?.tasks);
  const repairRecord = Object.values(post.changeIdentityRepairs ?? {})[0];
  w.ok('original change and receipt snapshots are retained in the signed record', repairRecord
    && Object.keys(repairRecord.originalChanges).sort().join() === 'alias-a,target' && Object.keys(repairRecord.originalObligations).sort().join() === 'change:alias-a,task:alias-a/1', repairRecord && Object.keys(repairRecord.originalChanges));
  w.ok('receipt lineage moved to the effective task and was not synthesized', `task:target/${TARGET_TASK}` in post.boundaryObligations && 'change:target' in post.boundaryObligations && !('task:alias-a/1' in post.boundaryObligations) && !('change:alias-a' in post.boundaryObligations)
    && post.boundaryObligations[`task:target/${TARGET_TASK}`].receiptId === pre.boundaryObligations['task:alias-a/1'].receiptId
    && post.boundaryObligations['change:target'].receiptId === pre.boundaryObligations['change:alias-a'].receiptId, Object.keys(post.boundaryObligations));
  w.ok('exactly one signed repair event was appended', (w.journalKinds().change_identity_repaired ?? 0) === 1, w.journalKinds());
  const retry = repair(w, mapping, { commandId: 'repair-identity-1', expectedRevision: pre.revision });
  w.ok('identical retry is idempotent', retry.exit === 0 && w.state().revision === post.revision && (w.journalKinds().change_identity_repaired ?? 0) === 1, retry.stderr);
  const different = repair(w, mapping, { commandId: 'repair-identity-1', expectedRevision: pre.revision, reason: 'a different reason for the same command id' });
  w.ok('the same command id with different content is refused', different.exit !== 0 && /already used for different identity-repair content/.test(different.stdout + different.stderr), (different.stdout + different.stderr).slice(0, 300));
  const replayed = w.coldReplay();
  w.ok('cold replay of the signed journal converges to the same effective state', stable(replayed) === stable(post), 'checkpoints removed, state recomputed');

  const outcome = w.result('inspect, refuse five invalid repairs, repair the damaged canonical identity, retry and cold-replay', {
    limitations: ['Non-operator actor kind cannot be selected from the public CLI (it signs repair as operator); authority is exercised through an unenrolled signer.',
      'Original #194 journal and 1.11.3 binary are unavailable; this proves only the seeded damaged-identity family.'] });
  outcome.commands = [...w.log, ...conflictWorld.log];
  w.cleanup(); conflictWorld.cleanup();
  return outcome;
}

function backend(world, change) {
  const dir = path.join(world.project, '.kbd-orchestrator/changes', change);
  writeFile(path.join(dir, 'tasks.json'), { changeId: change, schemaVersion: '1', tasks: [{ id: '1', title: SHARED_TITLE, done: false, doneAt: null, doneBy: null }] });
  writeFile(path.join(dir, 'spec.md'), '# spec\n');
}

async function idr003({ prometheus, seeder, ROOT }) {
  const env = (w) => ({ ...w.env, KBD_ORCHESTRATOR_ROOT: ORCH(ROOT) });
  const apply = (w, verb) => w.run('bash', [KBD_APPLY(ROOT), verb, 'alias-a', '1', '1', '1', SHARED_TITLE], { env: env(w) });

  // Positive: a started task survives repair and is completed exactly once through the retired key.
  const w = new World('idr003', { prometheus, ROOT }).damaged(seeder, 'equal');
  const pre = w.state();
  const { mapping } = resolution(w);
  const rep = repair(w, mapping, { commandId: 'repair-for-continuation', expectedRevision: pre.revision });
  w.ok('precondition: repair succeeded', rep.exit === 0, rep.stderr);
  backend(w, 'alias-a');
  const repaired = w.state();
  const done = apply(w, 'end-task');
  const after = w.state();
  const changeKeys = Object.keys(after.phases?.[PHASE]?.changes ?? {});
  const taskKeys = Object.keys(after.phases?.[PHASE]?.changes?.target?.tasks ?? {});
  w.ok('end-task through the retired key exits 0 with no failing lifecycle hook', done.exit === 0 && /Completed task 1 out of 1/.test(done.stdout) && !/hook .* failed/.test(done.stdout + done.stderr), (done.stdout + done.stderr).slice(-500));
  w.ok('the registered task key was reused, no duplicate change or task exists', changeKeys.join() === 'target' && taskKeys.join() === TARGET_TASK, { changeKeys, taskKeys });
  w.ok('the task is complete in the canonical ledger', after.phases[PHASE].changes.target?.tasks?.[TARGET_TASK]?.status === 'complete', after.phases[PHASE].changes.target?.tasks);
  w.ok('the start receipts were consumed by the single completion', ['change:target', `task:target/${TARGET_TASK}`].every((k) => !(k in (after.boundaryObligations ?? {})) && k in (repaired.boundaryObligations ?? {})), Object.keys(after.boundaryObligations ?? {}));
  const again = apply(w, 'end-task');
  w.ok('a second completion is refused rather than duplicating work', again.exit !== 0 && /has no matching start receipt/.test(again.stdout + again.stderr)
    && stable(after) === stable(w.state()), (again.stdout + again.stderr).slice(-300));
  writeFile(path.join(w.project, '.kbd-orchestrator/phases', PHASE, 'progress.json'), { phase: PHASE, changes: [{ id: 'alias-a', tasks_done: 1, tasks_total: 1 }] });
  const reconcile = w.run('bash', [KBD_APPLY(ROOT), 'reconcile', PHASE, '--json'], { env: env(w) });
  let report; try { report = JSON.parse(reconcile.stdout); } catch { report = null; }
  w.ok('reconcile reads the retired alias row through its surviving change: no error, no ledger-missing drift, no drift keyed by the alias', report && report.errors.length === 0
    && report.drift.every((d) => d.change !== 'alias-a' && d.kind !== 'ledger-missing'), reconcile.stdout.slice(0, 500) + reconcile.stderr.slice(0, 300));
  w.ok('reconcile created no new change or task', Object.keys(w.state().phases[PHASE].changes).join() === 'target', Object.keys(w.state().phases[PHASE].changes));

  // Negative: no start receipt ever existed; completion must stay blocked and no evidence may be synthesized.
  const n = new World('idr003-nostart', { prometheus, ROOT }).damaged(seeder, 'equal-nostart');
  const nres = resolution(n, { omitReceipt: true });
  const nrep = repair(n, nres.mapping, { commandId: 'repair-nostart', expectedRevision: n.state().revision });
  w.ok('precondition: repair of the no-receipt fixture succeeded', nrep.exit === 0, nrep.stderr);
  backend(n, 'alias-a');
  writeFile(path.join(n.project, '.kbd-orchestrator/phases', PHASE, 'progress.json'), { phase: PHASE, changes: [{ id: 'alias-a', tasks_done: 0, tasks_total: 1 }] });
  const blocked = apply(n, 'end-task');
  const nafter = n.state();
  w.ok('completion without a start receipt is blocked and explicit', blocked.exit !== 0 && /has no matching start receipt/.test(blocked.stdout + blocked.stderr), (blocked.stdout + blocked.stderr).slice(-400));
  w.ok('the task is not completed and no receipt was manufactured', nafter.phases[PHASE].changes.target.tasks[TARGET_TASK].status !== 'complete'
    && Object.keys(nafter.boundaryObligations ?? {}).length === 0, nafter.phases[PHASE].changes.target.tasks);

  const outcome = w.result('repair a damaged identity, complete the started task through the retired key, reconcile, and block a task with no start receipt');
  outcome.commands = [...w.log, ...n.log];
  w.cleanup(); n.cleanup();
  return outcome;
}

export const idrScenarios = [['IDR-001', idr001], ['IDR-002', idr002], ['IDR-003', idr003]];
