import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { bindingAt, object, readJson, runJson, writeJson } from './io.mjs';

/** Read-only canonical authority; never substitute a possibly stale waypoint projection. */
export function canonicalSnapshot(project) {
  const state = runJson(process.env.PROMETHEUS_BIN || 'prometheus', ['kbd', '--path', project, 'status', '--json'], project, undefined, 12000);
  const phaseId = state.activePath?.phaseId;
  const phase = state.phases?.[phaseId];
  if (!object(state.phases) || !phase || !Array.isArray(state.activePath.phasePath) || !Number.isInteger(state.revision)) {
    throw new Error('Canonical KBD status lacks phase identity, path, or revision');
  }
  return {
    schemaVersion: 1, source: 'prometheus-kbd-status', projectId: state.projectId, runId: state.runId,
    phaseId, parentPhaseId: phase.parentPhaseId ?? null, path: state.activePath.phasePath,
    revision: state.revision, lifecycle: state.lifecycle, approvals: state.approvals ?? null,
    phases: state.phases, decisions: state.decisions ?? {},
  };
}

export function registerKbd(project, entry) {
  const file = path.join(project, '.kbd-orchestrator', 'hooks-config.json');
  fs.mkdirSync(path.dirname(file), { recursive: true });
  const lock = fs.openSync(`${file}.cadence.lock`, 'wx');
  try {
    const config = fs.existsSync(file) ? readJson(file) : { hooks: [] };
    if (!object(config) || !Array.isArray(config.hooks)) throw new Error('Project hooks configuration must contain a hooks array');
    const hook = { id: 'delivery-cadence-kbd-reconcile', event: 'child:*', mode: 'augment', enabled: true,
      action: { command: 'node', args: [entry, 'reconcile', '--project', project], timeout: 30, on_failure: 'warn' } };
    const prior = config.hooks.find(item => item.id === hook.id);
    if (prior && JSON.stringify(prior) !== JSON.stringify(hook)) throw new Error('Existing cadence hook differs; retain it and resolve the registration explicitly');
    if (!prior) writeJson(file, { ...config, hooks: [...config.hooks, hook] });
    return { status: prior ? 'unchanged' : 'registered', file, hook,
      canonicalCommand: { command: 'node', args: [entry, 'snapshot', '--project', project], cwd: project } };
  } finally { fs.closeSync(lock); fs.rmSync(`${file}.cadence.lock`); }
}

/** Keep the approved operation intact across planning dispatch and child scope splits. */
export function dispatchContext(project, input) {
  if (!object(input) || !object(input.featureOperation)) {
    throw new Error('Dispatch requires the approved featureOperation; name its creation task if the procedure is not implemented');
  }
  const operation = input.featureOperation;
  if (!operation.id || !(operation.promisedCapability || operation.outcome) || !operation.procedure || !operation.checkpointId) {
    throw new Error('Dispatch featureOperation requires id, capability, procedure and checkpointId');
  }
  if (!operation.entrypoint && !operation.creationTaskRef) {
    throw new Error('Dispatch requires a production entrypoint or an approved operation creationTaskRef');
  }
  const canonical = canonicalSnapshot(project);
  if (input.phaseId && input.phaseId !== canonical.phaseId) {
    throw new Error('Dispatch phase differs from the canonical active phase; reconcile the scope before dispatch');
  }
  return { ...input, phaseId: canonical.phaseId, canonical,
    lifecycleAuthority: 'kbd', operationContractSource: input.operationContractSource ?? null };
}

export function reconcileKbd(project) {
  const binding = bindingAt(project);
  if (!binding) return { status: 'degraded', reason: 'cadence-binding-absent' };
  if (typeof binding.stateRoot !== 'string' || typeof binding.skillRoot !== 'string') throw new Error('Cadence binding requires stateRoot and skillRoot');
  const entry = path.resolve(project, binding.skillRoot, 'scripts', 'cadence.mjs');
  const stateRoot = path.resolve(project, binding.stateRoot);
  const state = runJson(process.execPath, [entry, 'status', '--root', stateRoot], project);
  if (!state.activeIterationId) return { status: 'idle', reason: 'no-active-cadence-iteration' };
  const canonical = canonicalSnapshot(project);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'cadence-kbd-'));
  try {
    const input = path.join(directory, 'canonical.json');
    const authored = binding.dispatchFile ? readJson(path.resolve(project, binding.dispatchFile)) : {};
    if (!object(authored)) throw new Error('Cadence dispatchFile must contain an authored JSON object');
    // Only context flows through the hook; it never creates tasks or certifies a return.
    writeJson(input, { canonical, featureOperation: authored.featureOperation ?? null,
      operationContractSource: binding.dispatchFile ?? null,
      candidateId: authored.candidateId ?? null, workAheadId: authored.workAheadId ?? null });
    return runJson(process.execPath, [entry, 'child', 'reconcile', '--root', stateRoot, '--input', input], project);
  } finally { fs.rmSync(directory, { recursive: true, force: true }); }
}

export async function kbdMain(args, options) {
  const project = path.resolve(args.project ?? process.cwd());
  const action = args._[0] ?? 'reconcile';
  if (action === 'snapshot') return canonicalSnapshot(project);
  if (action === 'register') return registerKbd(project, options.entry);
  if (action === 'dispatch') {
    if (!args.input) throw new Error('Dispatch requires --input with the approved request');
    return dispatchContext(project, readJson(path.resolve(project, args.input)));
  }
  if (action === 'reconcile') {
    try { return reconcileKbd(project); }
    catch { return { status: 'degraded', reason: 'canonical-or-cadence-reconciliation-unavailable' }; }
  }
  throw new Error('Use register, snapshot, dispatch, or reconcile');
}
