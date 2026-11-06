// A finite owned runner that a real harness session may invoke at the boundary.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

export function makeBoundaryRunner(w, env, ctx, operation) {
  const manifest = path.join(ctx.project, 'boundary-manifest.json');
  const receipt = path.join(ctx.project, 'boundary-receipt.json');
  fs.writeFileSync(manifest, JSON.stringify({ ROOT: env.ROOT, prometheus: env.prometheus, seeder: env.seeder, evidenceDir: env.evidenceDir, ctx, operation, receipt }));
  const module = pathToFileURL(fileURLToPath(import.meta.url)).href;
  fs.writeFileSync(path.join(ctx.project, 'boundary-runner.mjs'),
    `import { runBoundary } from ${JSON.stringify(module)};\nawait runBoundary(${JSON.stringify(manifest)});\n`);
  return { command: 'node boundary-runner.mjs', receipt, manifest };
}

export function qualifyRepair(result) {
  const required = ['the registered task key was reused, no duplicate change or task exists',
    'the task is complete in the canonical ledger', 'the start receipts were consumed by the single completion'];
  const cleanup = result.cleanupChildren ?? (result.cleanup ? [result.cleanup] : []);
  if (!cleanup.length || cleanup.some(c => c.status !== 'REMOVED')) return 'BLOCKED';
  if (required.some(name => !result.assertions?.some(a => a.name === name))) return 'NOT_EXERCISED';
  return result.assertions.some(a => !a.ok) ? 'FAIL' : 'PASS';
}

export async function runBoundary(manifestFile) {
  const { ROOT, prometheus, seeder, evidenceDir, ctx, operation, receipt } = JSON.parse(fs.readFileSync(manifestFile, 'utf8'));
  const { createHash } = await import('node:crypto');
  const { spawnSync } = await import('node:child_process');
  const sha = bytes => createHash('sha256').update(bytes).digest('hex');
  const identity = { manifestSha256: sha(fs.readFileSync(manifestFile)), binarySha256: sha(fs.readFileSync(prometheus)), operation };
  if (fs.existsSync(receipt)) throw new Error('boundary receipt already exists: reconcile before retry');
  let result;
  if (operation === 'repair') {
    const { idrScenarios } = await import('./kbd-194-196-idr.mjs');
    const run = idrScenarios.find(([id]) => id === 'IDR-003')[1];
    result = await run({ ROOT, prometheus, seeder, evidenceDir });
    result.verdict = qualifyRepair(result);
    const wrong = await run({ ROOT, prometheus, seeder, evidenceDir, expectedTaskStatus: 'wrong-value' });
    const missing = await run({ ROOT, prometheus, seeder, evidenceDir, omitTaskStateAssertion: true });
    const wrongVerdict = qualifyRepair(wrong), missingVerdict = qualifyRepair(missing);
    const control = result.assertions.find(a => a.name === 'completion without a start receipt is blocked and explicit');
    const stateControl = result.assertions.find(a => a.name === 'the task is not completed and no receipt was manufactured');
    result.negativeControlExercised = control?.ok === true && stateControl?.ok === true && wrongVerdict === 'FAIL' && missingVerdict === 'NOT_EXERCISED';
    result.negativeControl = { control, stateControl, wrong: { expectedTaskStatus: 'wrong-value', verdict: wrongVerdict, result: wrong },
      missing: { omittedAssertion: 'the task is complete in the canonical ledger', verdict: missingVerdict, result: missing } };
    result.cleanupChildren = [...(result.cleanupChildren ?? []), ...(wrong.cleanupChildren ?? []), ...(missing.cleanupChildren ?? [])];
    if (result.cleanupChildren.some(c => c.status !== 'REMOVED')) result.verdict = 'BLOCKED';
  } else if (operation === 'closure') {
    const commands = [], assertions = [];
    const run = (exe, args) => {
      const r = spawnSync(exe, args, { cwd: ctx.project, env: ctx.env, encoding: 'utf8', timeout: 180_000, maxBuffer: 16 * 1024 * 1024 });
      const value = { argv: [exe, ...args], exit: r.status ?? 127, stdout: r.stdout ?? '', stderr: `${r.stderr ?? ''}${r.error ?? ''}` };
      commands.push(value); return value;
    };
    const ok = (name, value, observed) => assertions.push({ name, ok: Boolean(value), observed });
    const pref = path.join(ctx.project, 'src/pref.js');
    const saved = run(process.execPath, [pref, 'set', 'accepted-value']);
    const first = run(process.execPath, [pref, 'get']);
    ok('separate processes save and recover the expected preference', saved.exit === 0 && first.exit === 0 && first.stdout === 'accepted-value', first);
    run(process.execPath, [pref, 'set', 'wrong-value']);
    const wrong = run(process.execPath, [pref, 'get']);
    const negativeControlExercised = wrong.exit === 0 && wrong.stdout !== 'accepted-value' && wrong.stdout === 'wrong-value';
    ok('the same readback oracle rejects an actual wrong persisted value', negativeControlExercised, wrong);
    run(process.execPath, [pref, 'set', 'accepted-value']);
    const restored = run(process.execPath, [pref, 'get']);
    ok('the authorized value was restored and read back', restored.exit === 0 && restored.stdout === 'accepted-value', restored);
    const kbd = args => run(prometheus, ['kbd', '--path', ctx.project, ...args]);
    const eventCounts = () => {
      const counts = {}, projects = path.join(ctx.env.PROMETHEUS_DATA_DIR, 'prometheus/kbd/projects');
      for (const project of fs.readdirSync(projects)) {
        const replicas = path.join(projects, project, 'replicas');
        if (!fs.existsSync(replicas)) continue;
        for (const replica of fs.readdirSync(replicas)) {
          const file = path.join(replicas, replica, 'events.jsonl');
          if (!fs.existsSync(file)) continue;
          for (const line of fs.readFileSync(file, 'utf8').split('\n').filter(Boolean)) {
            const type = JSON.parse(line).kind.type; counts[type] = (counts[type] ?? 0) + 1;
          }
        }
      }
      return counts;
    };
    const before = JSON.parse(kbd(['status', '--json']).stdout);
    const beforeEvents = eventCounts();
    const disposition = JSON.parse(kbd(['phase', 'disposition', '--id', ctx.phase, '--json']).stdout);
    const evidence = { phase: ctx.phase, projectId: before.projectId, runId: before.runId, contractDigest: disposition.binding.contractDigest,
      sourceDigests: Object.fromEntries(['src.txt', 'src/pref.js'].map(f => [f, sha(fs.readFileSync(path.join(ctx.project, f)))])),
      budgetConsumption: { source: 'owned boundary runner; measured elapsed/process invocations; token telemetry UNKNOWN',
        values: { timeMinutes: { value: 1, class: 'estimate' }, toolCalls: { value: commands.length + 5, class: 'estimate' }, repairAttempts: { value: 0, class: 'measured' },
        // One straight-line pass: no iteration exists that could fail to make progress.
        noProgress: { value: 0, class: 'measured' } } },
      scenarios: [{ id: 'CS-1', status: assertions.every(a => a.ok) ? 'PASS' : 'FAIL', attempted: 'save, fresh-process readback, wrong-value challenge and restoration',
        observed: { expected: 'accepted-value', first: first.stdout, wrong: wrong.stdout, restored: restored.stdout }, evidence: 'boundary-receipt.json' }] };
    fs.writeFileSync(path.join(ctx.phaseDir, 'boundary-results.json'), JSON.stringify(evidence, null, 2) + '\n');
    // Retain the observations before passing them to the canonical writer.
    fs.writeFileSync(receipt, JSON.stringify({ identity, assertions, commands, evidence, pendingClosure: true }, null, 2) + '\n');
    const evaluator = path.join(ROOT, 'shared/scripts/kbd-phase-close-eval.sh');
    const close = run('bash', [evaluator, '--phase', ctx.phase]);
    const closed = JSON.parse(kbd(['status', '--json']).stdout);
    const retry = run('bash', [evaluator, '--phase', ctx.phase]);
    const after = JSON.parse(kbd(['status', '--json']).stdout);
    const terminal = JSON.parse(kbd(['phase', 'disposition', '--id', ctx.phase, '--json']).stdout);
    const afterEvents = eventCounts();
    const closedOnce = close.exit === 0 && retry.exit === 0 && terminal.disposition === 'CLOSED' && after.revision === closed.revision;
    const successorCreated = Object.keys(after.phases).length !== Object.keys(before.phases).length;
    ok('canonical closure is idempotent with no successor', closedOnce && !successorCreated, { first: close.exit, retry: retry.exit, terminal: terminal.disposition });
    ok('closure adds only its typed disposition, no phase/task/publication event', Object.entries(afterEvents).every(([type, count]) =>
      count === (beforeEvents[type] ?? 0) + (type === 'phase_disposition_recorded' ? 1 : 0)), { beforeEvents, afterEvents });
    result = { assertions, commands, negativeControlExercised, negativeControl: wrong, closedOnce, successorCreated, closure: terminal,
      verdict: assertions.every(a => a.ok) && closedOnce && !successorCreated ? 'PASS' : 'FAIL',
      eventCounts: { before: beforeEvents, after: afterEvents },
      claims: { implementation: 'complete in canonical task state', measuredAcceptance: evidence.scenarios[0].status,
        runtimeValue: first.stdout === 'accepted-value' && restored.stdout === 'accepted-value', certification: 'NOT_RUN',
        releaseEligibility: 'NOT_ESTABLISHED', releaseAuthority: 'NOT_GRANTED' } };
  } else throw new Error(`unsupported boundary operation ${operation}`);
  fs.writeFileSync(receipt, JSON.stringify({ identity, ...result }, null, 2) + '\n');
  if (!result.assertions?.length || result.assertions.some(a => !a.ok) || !result.negativeControlExercised) process.exitCode = 1;
  process.stdout.write(`Boundary ${operation}: ${result.assertions.filter(a => a.ok).length}/${result.assertions.length} observed assertions; receipt ${receipt}\n`);
}
