import path from 'node:path';
import { cacheHome, findProject, deadline, acquireLock, receipt, writeJson } from './state.mjs';
import { cachedRuntime, selectedRuntime, selectVersion, ensureRuntime, nodeProcess } from './runtime.mjs';
import { updateFootprint, backupArtifacts, auditAuthored } from './backup.mjs';

export async function refresh({ project: requested = process.cwd(), timeoutMs = 120000 } = {}) {
  const home = cacheHome();
  const project = findProject(requested);
  const base = { operation: 'refresh', project: project || path.resolve(requested) };
  const finish = (status, details = {}, code = 0) => ({ code, ...receipt(home, { ...base, status, ...details }) });
  if (process.env.PROMETHEUS_OPENSPEC_DISABLE === '1') return finish('disabled', { policy: 'operator-disabled' });
  if (!project) return finish('skipped', { reason: 'No ancestor with OpenSpec and canonical or legacy KBD identity' });
  let release;
  let selection;
  let snapshot;
  const remaining = deadline(timeoutMs);
  try {
    release = acquireLock(home, 'refresh', project);
    selection = await selectVersion(home, remaining);
    const runtime = await ensureRuntime(home, selection, remaining);
    const footprint = await updateFootprint(runtime, project);
    remaining();
    snapshot = backupArtifacts(home, project, runtime.version, footprint, remaining);
    const result = await nodeProcess(runtime.entry, ['update', project, '--force'], { cwd: project, remaining });
    const changed = auditAuthored(snapshot, remaining);
    const details = { ...selection, backup: snapshot.backup, output: result.output, authoredPathsChanged: changed };
    if (changed.length) return finish('migration-required', { ...details, reason: 'Authored state changed during refresh; inspect concurrent writes or upstream compatibility. No authored files were restored or overwritten by the runner.' }, 2);
    if (result.code !== 0) return finish('failed', { ...details, reason: `openspec update exited ${result.code}` }, result.code);
    if (/No configured tools found/.test(result.output)) return finish('migration-required', { ...details, reason: 'Run openspec init to select tools before refreshing generated artifacts' }, 2);
    if (footprint.legacyProject) return finish('migration-required', { ...details, reason: 'Legacy openspec/project.md requires an authored migration; generated tools were refreshed but project migration is not certified' }, 2);
    writeJson(path.join(home, 'selected.json'), { version: runtime.version, selectedAt: new Date().toISOString(), ...selection });
    return finish(selection.latestVerified ? 'refreshed' : 'refreshed-latest-unverified', {
      ...details, compatibility: 'Generated integration artifacts refreshed; authored specification semantics not migrated or validated',
    });
  } catch (error) {
    return finish(error.status || 'failed', {
      ...selection, backup: snapshot?.backup || null, reason: error.message,
      output: error.output || '', authoredAudit: snapshot ? 'not-completed' : 'not-started',
    }, error.status === 'pending' || error.status === 'contended' ? 75 : error.status === 'migration-required' ? 2 : 1);
  } finally { release?.(); }
}

export async function run({ project: requested = process.cwd(), args = [], timeoutMs = 120000 } = {}) {
  const home = cacheHome();
  const project = findProject(requested, false) || path.resolve(requested);
  const base = { operation: 'run', project };
  const finish = (status, details, code) => ({ code, ...receipt(home, { ...base, status, ...details }) });
  if (process.env.PROMETHEUS_OPENSPEC_DISABLE === '1') return finish('disabled', { policy: 'operator-disabled' }, 2);
  let release;
  let runtime;
  const remaining = deadline(timeoutMs);
  try {
    release = acquireLock(home, 'run', project);
    const pin = process.env.PROMETHEUS_OPENSPEC_VERSION;
    runtime = pin ? cachedRuntime(home, pin) : selectedRuntime(home);
    if (!runtime) {
      const selection = await selectVersion(home, remaining);
      runtime = await ensureRuntime(home, selection, remaining);
      writeJson(path.join(home, 'selected.json'), { version: runtime.version, selectedAt: new Date().toISOString(), ...selection });
    }
    const result = await nodeProcess(runtime.entry, args, { cwd: project, remaining, inherit: true });
    return finish(result.code ? 'failed' : 'completed', { version: runtime.version, latestVerified: false, policy: pin ? 'operator-pin' : 'selected-cache', cliExitCode: result.code }, result.code);
  } catch (error) {
    return finish(error.status || 'failed', { version: runtime?.version, reason: error.message, output: error.output || '' }, error.status === 'pending' || error.status === 'contended' ? 75 : 1);
  } finally { release?.(); }
}
