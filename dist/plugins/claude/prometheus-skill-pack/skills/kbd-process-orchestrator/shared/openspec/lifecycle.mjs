import fs from 'node:fs';
import path from 'node:path';
import { cacheHome, findProject, deadline, acquireLock, receipt, writeJson, assertNodeFloor, PINNED_VERSION } from './state.mjs';
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
    assertNodeFloor();
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
    return finish(selection.policy === 'operator-pin' ? 'refreshed-operator-pin' : 'refreshed', {
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
    assertNodeFloor();
    release = acquireLock(home, 'run', project);
    const pin = process.env.PROMETHEUS_OPENSPEC_VERSION;
    // Commands always run the pack-pinned CLI (or an operator pin), never a
    // newer selection left behind by an earlier registry-latest policy.
    runtime = cachedRuntime(home, pin || PINNED_VERSION);
    if (!runtime) {
      const selection = await selectVersion(home, remaining);
      runtime = await ensureRuntime(home, selection, remaining);
      writeJson(path.join(home, 'selected.json'), { version: runtime.version, selectedAt: new Date().toISOString(), ...selection });
    }
    const result = await nodeProcess(runtime.entry, args, { cwd: project, remaining, inherit: true });
    return finish(result.code ? 'failed' : 'completed', { version: runtime.version, latestVerified: false, policy: pin ? 'operator-pin' : 'pack-pin', cliExitCode: result.code }, result.code);
  } catch (error) {
    return finish(error.status || 'failed', { version: runtime?.version, reason: error.message, output: error.output || '' }, error.status === 'pending' || error.status === 'contended' ? 75 : 1);
  } finally { release?.(); }
}

// Adopt OpenSpec as a project's spec engine: `openspec init` non-interactively.
// A project that already has an openspec/ root is left untouched.
export async function init({ project: requested = process.cwd(), tools = 'claude,codex', timeoutMs = 240000 } = {}) {
  const project = path.resolve(requested);
  if (fs.existsSync(path.join(project, 'openspec'))) {
    const home = cacheHome();
    return { code: 0, ...receipt(home, { operation: 'init', project, status: 'already-initialized' }) };
  }
  const result = await run({ project, timeoutMs, args: ['init', project, '--tools', tools, '--no-animation', '--no-copilot-cloud'] });
  return { ...result, operation: 'init', tools };
}

// Session-start check (CR-03): report whether the project's generated OpenSpec
// integration matches the pinned CLI. Never installs, updates or writes project
// files, so a new session (for example a harness switch) cannot change a bound
// phase's procedure text mid-phase. Mutating refresh runs at phase entry only.
export async function verify({ project: requested = process.cwd() } = {}) {
  const home = cacheHome();
  const project = findProject(requested);
  const base = { operation: 'verify', project: project || path.resolve(requested) };
  const finish = (status, details = {}) => ({ code: 0, ...receipt(home, { ...base, status, ...details }) });
  if (process.env.PROMETHEUS_OPENSPEC_DISABLE === '1') return finish('disabled', { policy: 'operator-disabled' });
  if (!project) return finish('skipped', { reason: 'No ancestor with OpenSpec and canonical or legacy KBD identity' });
  const version = process.env.PROMETHEUS_OPENSPEC_VERSION || PINNED_VERSION;
  const skills = path.join(project, '.agents', 'skills');
  const generated = [];
  if (fs.existsSync(skills)) {
    for (const name of fs.readdirSync(skills)) {
      const file = path.join(skills, name, 'SKILL.md');
      if (!name.startsWith('openspec-') || !fs.existsSync(file)) continue;
      const match = fs.readFileSync(file, 'utf8').match(/generatedBy:\s*"?([0-9.]+)"?/);
      generated.push({ skill: name, generatedBy: match ? match[1] : null });
    }
  }
  const stale = generated.filter(entry => entry.generatedBy !== version);
  const runtimeCached = Boolean(cachedRuntime(home, version));
  const status = !generated.length ? 'refresh-needed' : stale.length || !runtimeCached ? 'refresh-needed' : 'current';
  return finish(status, {
    version, runtimeCached, policy: 'verify-only', staleSkills: stale.map(entry => entry.skill),
    next: status === 'current' ? null : 'Run the managed refresh at phase entry (kbd-new-phase / kbd-next-phase), or: node cli.mjs refresh --project <root> --timeout-ms 120000',
  });
}
