// Reconciliation is an inspection operation unless repair is explicitly requested.
import { existsSync } from 'node:fs';
import path from 'node:path';
import { resolveRuntimeTaskId } from './task-identity.mjs';
import { readJson, complete, cancelled, resolvePhase, readBackendTasks } from './reconcile-artifacts.mjs';

export function parseReconcileArgs(args) {
  let phase;
  let repair = false;
  let json = false;
  for (const arg of args) {
    if (arg === '--repair') repair = true;
    else if (arg === '--json') json = true;
    else if (arg.startsWith('-')) throw new Error(`unknown flag ${arg}`);
    else if (phase) throw new Error('only one phase may be given');
    else phase = arg;
  }
  return { phase, repair, json };
}

const errorRow = (kind, message, change = null) => ({ kind, change, message });
const result = (phase, drift, errors) => ({ phase, clean: !drift.length && !errors.length, drifted: drift.length, drift, errors });

function phaseCounts(phase, progress) {
  const changes = phase ? Object.values(phase.changes ?? {}) : progress.changes;
  const done = (change) => complete(change.implementationStatus ?? change.implementation_status ?? change.status);
  const baseline = phase?.legacyCompletionBaseline;
  if (!baseline) return { completed: changes.filter(done).length, total: changes.length };
  const imported = baseline.importedChangeStatuses ?? {};
  const added = changes.filter((change) => !Object.hasOwn(imported, change.id));
  const total = baseline.total + added.length;
  const completed = baseline.completed - Object.values(imported).filter(complete).length +
    changes.filter((change) => Object.hasOwn(imported, change.id) && done(change)).length + added.filter(done).length;
  return { completed: Math.max(0, Math.min(total, completed)), total };
}

export async function scanReconcile(root, requested, { readCanonical, resolveIdentity = resolveRuntimeTaskId } = {}) {
  const drift = [];
  const errors = [];
  const context = { root, artifacts: {}, repairs: [] };
  let phase = requested ?? '';
  try {
    const waypoint = readJson(path.join(root, '.kbd-orchestrator', 'current-waypoint.json'));
    context.authoritative = waypoint.generatedBy === 'kbd-runtime';
    const state = context.authoritative ? await readCanonical(root) : null;
    if (context.authoritative && (!state?.phases || !state.activePath)) throw new Error('canonical runtime state is unavailable or invalid');
    Object.assign(context, resolvePhase(root, requested, waypoint, state), { state });
    phase = context.phase;
    const progressFile = path.join(context.directory, 'progress.json');
    const progress = readJson(progressFile);
    if (!Array.isArray(progress.changes)) throw new Error('invalid phase progress changes');
    const rows = new Map();
    for (const row of progress.changes) {
      if (!row?.id || rows.has(row.id)) throw new Error('invalid or duplicate progress change ID');
      rows.set(row.id, row);
    }
    const projectFile = path.join(root, '.kbd-orchestrator', 'project.json');
    const pinned = existsSync(projectFile) ? readJson(projectFile).specBackend : undefined;
    const changes = state ? Object.keys(state.phases[phase].changes ?? {}) : [...rows.keys()];
    // Projection-only changes must also be inspected: silently dropping them would hide drift.
    for (const id of rows.keys()) if (!changes.includes(id)) changes.push(id);
    Object.assign(context, { progressFile, progress });
    for (const change of changes) {
      const ledger = state?.phases[phase].changes?.[change];
      const projection = rows.get(change);
      try {
        const artifact = readBackendTasks(root, change, pinned);
        context.artifacts[change] = artifact;
        const { tasks, archived } = artifact;
        const total = tasks.length;
        const bcomp = tasks.filter((task) => task.done).length;
        const add = (task, kind, title = task.title) => drift.push({ change, task: task.id, sequence: task.sequence, total, kind, title });
        if (state) {
          const registered = ledger?.tasks ?? {};
          const mapped = new Set();
          for (const task of tasks) {
            let id;
            try {
              id = resolveIdentity(registered, task.id, task.sequence, task.title).id;
              if (mapped.has(id)) throw new Error('multiple backend tasks map to the same canonical task');
              mapped.add(id);
            } catch {
              add(task, 'unmappable');
              continue;
            }
            const status = registered[id]?.status;
            const isCancelled = cancelled(status) || cancelled(ledger?.status) || cancelled(ledger?.implementationStatus) || cancelled(state.phases[phase].status);
            if (task.done && !complete(status)) {
              const kind = isCancelled ? 'cancelled' : archived ? 'archived' : status ? 'ledger-pending' : 'ledger-missing';
              add(task, kind);
              if (kind === 'ledger-pending' || kind === 'ledger-missing') context.repairs.push({ change, ...task, total, runtimeId: id, status });
            } else if (!task.done && complete(status)) add(task, 'ledger-ahead');
          }
          for (const [id, task] of Object.entries(registered)) {
            if (!mapped.has(id) && !cancelled(task.status)) add({ id, sequence: task.sequence ?? 0, title: task.title ?? id }, 'ledger-only');
          }
          const expectedDone = Object.values(registered).filter((task) => complete(task.status) || cancelled(task.status)).length;
          const expectedTotal = Object.keys(registered).length;
          if (!projection || projection.tasks_done !== expectedDone || projection.tasks_total !== expectedTotal) {
            add({ id: '-', sequence: 0 }, 'projection', `progress.json counts must match ledger ${expectedDone}/${expectedTotal}`);
          }
        } else if (projection.tasks_done !== bcomp || projection.tasks_total !== total) {
          add({ id: '-', sequence: 0 }, archived ? 'archived' : 'count', `progress.json counts must match backend ${bcomp}/${total}`);
        }
      } catch (error) { errors.push(errorRow('artifact', error.message, change)); }
    }
    const counts = phaseCounts(state?.phases[phase], progress);
    context.phaseCounts = counts;
    const pairs = [
      [progress.changes_completed, progress.changes_total],
      [progress.implementation_completed, progress.implementation_total],
      [progress.completion?.implementation?.completed, progress.completion?.implementation?.total],
    ];
    if (pairs.some(([done, total]) => (done !== undefined || total !== undefined) && (done !== counts.completed || total !== counts.total))) {
      drift.push({ change: '-', task: '-', sequence: 0, total: counts.total,
        kind: state ? 'projection' : 'phase-count', title: `phase implementation counts must match change ledger ${counts.completed}/${counts.total}` });
    }
  } catch (error) { errors.push(errorRow('scan', error.message)); }
  return { report: result(phase, drift, errors), context };
}

export async function reconcile(root, options, adapters) {
  let scanned = await scanReconcile(root, options.phase, adapters);
  if (!options.repair) return scanned.report;
  const { context } = scanned;
  if (scanned.report.errors.length) return scanned.report;
  if (context.phase !== context.active) {
    scanned.report.errors.push(errorRow('repair', 'repair requires the selected phase to be active'));
    scanned.report.clean = false;
    return scanned.report;
  }
  let repairError;
  try {
    if (context.authoritative) {
      // Multiple backend rows mapping to one identity are never independently repairable.
      const ambiguousChanges = new Set(scanned.report.drift.filter((row) => row.kind === 'unmappable').map((row) => row.change));
      for (const task of context.repairs) {
        if (ambiguousChanges.has(task.change)) continue;
        await adapters.repairTask(root, context.phase, task);
      }
    } else {
      const counts = scanned.report.drift.filter((row) => ['count', 'phase-count'].includes(row.kind));
      if (counts.length) await adapters.repairCounts(context, counts);
    }
  } catch (error) { repairError = errorRow('repair', error.message); }
  scanned = await scanReconcile(root, context.phase, adapters);
  if (repairError) { scanned.report.errors.push(repairError); scanned.report.clean = false; }
  return scanned.report;
}

export function printReconcile(report, json, stdout = process.stdout, stderr = process.stderr) {
  if (json) stdout.write(`${JSON.stringify(report, null, 2)}\n`);
  else {
    for (const row of report.drift) stdout.write(`DRIFT ${row.change} task ${row.task} (${row.kind}): ${row.title}\n`);
    for (const row of report.errors) stderr.write(`reconcile: ${row.kind}: ${row.message}\n`);
    stdout.write(`reconcile: ${report.errors.length ? 'incomplete' : report.clean ? 'clean' : `${report.drifted} drifted item(s)`} — ${report.phase}\n`);
  }
  return report.errors.length ? 2 : report.drifted ? 1 : 0;
}

export async function runReconcile(args, adapters, root = process.cwd()) {
  let report;
  try { report = await reconcile(root, parseReconcileArgs(args), adapters); }
  catch (error) { report = result('', [], [errorRow('input', error.message)]); }
  return printReconcile(report, args.includes('--json'));
}
