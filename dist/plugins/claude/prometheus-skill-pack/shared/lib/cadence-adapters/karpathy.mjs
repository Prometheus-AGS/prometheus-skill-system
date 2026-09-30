import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { canonicalSnapshot } from './kbd.mjs';
import { bindingAt, readJson, runJson, writeJson, writeAtomic } from './io.mjs';

/** Bridge a persisted finalized report; cadence completion never sets canonical KBD status. */
export function recordCadence(args, { defaultRecorder = null } = {}) {
  const project = path.resolve(args.project ?? process.cwd());
  if (!args.input) return { status: 'degraded', reason: 'finalized-report-required' };
  const input = path.resolve(args.input);
  const report = readJson(input);
  const iteration = report.iterations?.at(-1);
  if (!iteration || iteration.status !== 'finished' || !Number.isFinite(iteration.minutes?.elapsed)) {
    return { status: 'degraded', reason: 'report-is-not-finalized' };
  }
  const bytes = fs.readFileSync(input);
  const digest = createHash('sha256').update(bytes).digest('hex');
  const directory = path.join(project, '.prometheus', 'cadence-learning-reports');
  fs.mkdirSync(directory, { recursive: true });
  const artifact = path.join(directory, `${digest}.json`);
  const lock = fs.openSync(`${artifact}.lock`, 'wx');
  try {
    // Keep the exact bytes behind the digest, then persist the original observation for retries.
    if (!fs.existsSync(artifact)) writeAtomic(artifact, bytes);
    const durable = { report: artifact, reportSha256: digest };
    const binding = bindingAt(project);
    const configured = args.recorder ?? binding?.karpathyRecorder ?? defaultRecorder;
    if (typeof configured !== 'string') return { status: 'degraded', ...durable, reason: 'node-progress-recorder-unavailable' };
    const recorder = path.resolve(project, configured);
    if (!recorder.endsWith('.mjs') || !fs.existsSync(recorder)) return { status: 'degraded', ...durable, reason: 'node-progress-recorder-unavailable' };
    const eventPath = path.join(directory, `${digest}.event.json`);
    let event;
    if (fs.existsSync(eventPath)) event = readJson(eventPath);
    else {
      const canonical = canonicalSnapshot(project);
      const status = canonical.phases[canonical.phaseId].status;
      if (!['in_progress', 'complete', 'blocked', 'cancelled'].includes(status)) {
        return { status: 'degraded', ...durable, reason: 'canonical-phase-status-not-recordable' };
      }
      const summary = JSON.stringify({ cadenceOutcome: iteration.workOutcome, cadenceCounts: iteration.counts,
        minutes: iteration.minutes, reportSha256: digest, report: path.relative(project, artifact),
        note: 'Cadence report counts do not establish KBD completion; phase status is independently observed.' });
      event = {
        schemaVersion: 1, eventId: `cadence-${digest}`, observedAt: new Date().toISOString(),
        runId: canonical.runId, boundary: 'phase', status, phaseId: canonical.phaseId,
        changeId: null, taskId: null, taskClass: 'evidence', elapsedHours: iteration.minutes.elapsed / 60,
        touchedFiles: [], verification: [{ command: 'delivery-cadence persisted finalized report', exitCode: 0, summary }],
        commitSha: null, blocker: null, exactNextWork: null,
      };
      writeJson(eventPath, event);
    }
    try {
      const result = runJson(process.execPath, [recorder, '--project-root', project, '--input', eventPath], project);
      return { status: result.status ?? 'degraded', ...durable, recorder: result };
    } catch {
      return { status: 'degraded', ...durable, reason: 'recorder-unavailable' };
    }
  } catch {
    return { status: 'degraded', ...(fs.existsSync(artifact) ? { report: artifact, reportSha256: digest } : {}),
      reason: 'canonical-state-or-progress-recorder-unavailable' };
  } finally { fs.closeSync(lock); fs.rmSync(`${artifact}.lock`); }
}

export async function karpathyMain(args, options) {
  try { return recordCadence(args, options); }
  catch { return { status: 'degraded', reason: 'canonical-state-or-progress-recorder-unavailable' }; }
}
