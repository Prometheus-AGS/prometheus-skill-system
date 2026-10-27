#!/usr/bin/env node
// A derived recovery index. Canonical decisions and the adopted contract remain authority.
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, realpathSync, mkdirSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const read = file => JSON.parse(readFileSync(file, 'utf8'));
const complete = status => ['complete', 'completed', 'done', 'cancelled', 'skipped'].includes(String(status).toLowerCase());
const command = (argv, root) => JSON.parse(execFileSync('prometheus', ['kbd', '--path', root, ...argv], { encoding: 'utf8', timeout: 30000, maxBuffer: 16 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] }));
const within = (root, target) => target === root || target.startsWith(root + path.sep);

function ownedPath(root, relative) {
  const target = path.resolve(root, relative);
  if (!within(root, target) || (existsSync(target) && !within(root, realpathSync(target)))) throw new Error('method reference escapes project root');
  return target;
}

function fileReference(root, relative) {
  let file;
  try { file = ownedPath(root, relative); } catch { return { path: relative, observation: 'unreadable', sha256: null }; }
  if (!existsSync(file)) return { path: relative, observation: 'missing', sha256: null };
  try { return { path: relative, observation: 'observed', sha256: hash(readFileSync(file)) }; } catch { return { path: relative, observation: 'unreadable', sha256: null }; }
}

// A reference path comes from a decision summary: one relative, control-character-free path inside the project.
const safeReferencePath = value => typeof value === 'string' && value.length > 0 && !/[\u0000-\u001f\u007f\\]/.test(value)
  && !path.isAbsolute(value) && !value.split('/').includes('..');
const safeMethodName = value => typeof value === 'string' && /^[A-Za-z0-9][A-Za-z0-9._ -]{0,63}$/.test(value);

function phaseDirectory(root, state, id) {
  const chain = []; const visited = new Set(); let cursor = id;
  while (cursor) {
    if (visited.has(cursor)) throw new Error('cyclic canonical phase ancestry');
    visited.add(cursor);
    const phase = state.phases?.[cursor];
    if (!phase) throw new Error(`canonical phase missing: ${cursor}`);
    const slug = phase.slug || phase.id;
    if (typeof slug !== 'string' || !/^[a-z0-9][a-z0-9._-]*$/.test(slug) || slug.includes('..')) throw new Error('unsafe phase projection slug');
    chain.unshift(slug); cursor = phase.parentPhaseId;
  }
  return ownedPath(root, path.join('.kbd-orchestrator', 'phases', ...chain.flatMap((slug, i) => i ? ['children', slug] : [slug])));
}

/** Read-only discovery. A caller may supply a canonical status already read by its lifecycle. */
export function discoverMethodContext({ project = process.cwd(), phase, canonicalStatus, disposition } = {}) {
  const root = realpathSync(project);
  if (!existsSync(path.join(root, '.kbd-orchestrator'))) throw new Error('project has no existing KBD context; discovery does not initialize one');
  const state = canonicalStatus ?? command(['status', '--json'], root);
  const phaseId = phase ?? state.activePath?.phaseId;
  if (!phaseId || !state.phases?.[phaseId]) throw new Error('method context needs a resolvable canonical phase');
  const directory = phaseDirectory(root, state, phaseId);
  const relativeDirectory = path.relative(root, directory);
  const contractPath = path.join(relativeDirectory, 'adopted-contract.json');
  const contractRef = fileReference(root, contractPath);
  const contract = contractRef.observation === 'observed' ? read(ownedPath(root, contractPath)) : null;
  const phaseDisposition = disposition ?? command(['phase', 'disposition', '--id', phaseId, '--json'], root);
  const binding = phaseDisposition.binding ?? null;
  const blockers = [];
  let digest = null;
  if (contract) {
    const normalized = execFileSync('jq', ['-S', '-c', 'del(.identity.contractDigest) | del(.identity.planRevision)', ownedPath(root, contractPath)], { timeout: 30000 });
    digest = hash(normalized);
    if (contract.identity?.projectId !== state.projectId || contract.identity?.runId !== state.runId || contract.identity?.phaseId !== phaseId) blockers.push('contract identity differs from canonical phase');
    if (!binding || binding.contractDigest !== digest) blockers.push('contract missing canonical binding or changed since adoption');
  } else blockers.push('adopted contract unavailable');

  // KBD-METHOD is a compact payload in the existing signed decision surface, not a new ledger.
  const decisions = Object.values(state.decisions ?? {});
  const retired = new Set(decisions.map(item => item.supersedes).filter(Boolean));
  const selections = [];
  for (const decision of decisions) {
    if (retired.has(decision.id) || typeof decision.summary !== 'string' || !decision.summary.startsWith('KBD-METHOD ')) continue;
    let selection;
    try { selection = JSON.parse(decision.summary.slice('KBD-METHOD '.length)); } catch { continue; }
    if (!selection || typeof selection !== 'object') continue;
    if (selection.phaseId === phaseId && selection.contractDigest === digest) selections.push({ decision, selection });
  }
  const selected = selections.length === 1 ? selections[0] : null;
  if (!selected) blockers.push(selections.length ? 'multiple current selected-method decisions' : 'no current contract-bound selected-method decision');
  let method = selected?.selection.method ?? 'UNKNOWN';
  if (selected && !safeMethodName(method)) { blockers.push('selected method name is not a short plain label'); method = 'UNKNOWN'; }
  const rawRefs = selected?.selection.references ?? [];
  const methodRefs = Array.isArray(rawRefs) ? rawRefs : [];
  if (!Array.isArray(rawRefs)) blockers.push('selected method references are not a list');
  const references = methodRefs.flatMap(expected => {
    if (!expected || !safeReferencePath(expected.path) || !/^[a-f0-9]{64}$/.test(expected.sha256 ?? '')) {
      blockers.push('selected method reference is not a safe project-relative path with SHA-256');
      return [];
    }
    const actual = fileReference(root, expected.path);
    const matches = actual.sha256 === expected.sha256;
    if (!matches) blockers.push(`selected method reference changed or missing: ${expected.path}`);
    return [{ ...actual, expectedSha256: expected.sha256, matches }];
  });
  const governingInstructions = ['AGENTS.md', 'CLAUDE.md', '.kbd-orchestrator/constraints.md'].map(file => fileReference(root, file));
  const policyConflicts = [];
  if (method === 'VLBV') {
    for (const ref of governingInstructions.filter(item => item.observation === 'observed')) {
      const text = readFileSync(ownedPath(root, ref.path), 'utf8');
      if (/compiler.{0,100}(exception|unblock)|(?:narrow|targeted|intermediate|package-scoped).{0,60}compiler check/is.test(text)) policyConflicts.push({ path: ref.path, issue: 'Host compiler exception may conflict with selected VLBV; do not infer permission to run generated-code checks during generation.' });
    }
  }
  const implementationComplete = Object.values(state.phases[phaseId].changes ?? {}).length > 0 && Object.values(state.phases[phaseId].changes).every(change => complete(change.implementationStatus ?? change.status));
  const stage = state.activePath?.phaseId === phaseId ? state.activePath.stageId ?? null : null;
  const preservation = [
    'latest non-superseded method decision and retiredMethods; old commands/memory are context only',
    'original contract snapshots, evidence and source/build/configuration/fixture/assertion identity',
    'all budget consumption/high-water marks, including UNKNOWN prior usage; never reset on recovery',
    'unresolved effects and cleanup/receipt obligations; reconcile uncertain effects before retry',
  ];
  return {
    schemaVersion: 1, kind: 'kbd.derived-method-context', generatedAt: new Date().toISOString(),
    authority: 'derived-read-only-index; canonical signed decisions, phase binding, contract and governing host policy remain authority',
    projectId: state.projectId, runId: state.runId, phaseId, sourceRevision: state.revision,
    phaseDirectory: relativeDirectory, contract: { ...contractRef, contractDigest: digest, binding, identity: contract?.identity ?? null },
    selectedMethod: method, methodDecision: selected?.decision ?? null, retiredMethods: selected?.selection.retiredMethods ?? [], references,
    includedValue: contract?.functionalScope ?? [], exclusions: contract?.exclusions ?? [],
    stage, stageAuthority: { operations: stage === 'execute' ? ['author agreed production without generated-code verification'] : ['discover/define/plan within governing policy'], implementationComplete, verificationAuthority: 'separately authorized whole-value-phase boundary required; this index grants none', executableTestAuthoring: 'governed by host policy; where that policy is production-first, defer until the complete agreed production work' },
    governingInstructions, policyConflicts, candidateAndEvidence: { freshness: contract?.evidenceInvalidation ?? 'UNKNOWN', paths: contract?.boundaryPacket?.receipts ?? [] },
    budgets: { adopted: contract?.budgets ?? null, consumption: state.phaseClosure?.phases?.[phaseId]?.consumptionHighWater ?? 'UNKNOWN; recover original canonical/evidence receipts before execution' },
    unresolved: { boundaryObligations: state.boundaryObligations ?? {}, blockers: Object.values(state.blockers ?? {}).filter(item => !item.resolved), instruction: 'Unknown side effects require reconciliation before retry, not a fresh attempt assumption.' },
    preserve: preservation,
    hostCapabilities: { platform: process.platform, nodeVersion: process.version, observedInterfaces: ['canonical status', 'typed phase disposition', 'project-owned file identity'], executionInterception: 'not provided by this helper', commandInventoryIsPermission: false },
    control: { mode: 'instruction-only', observed: ['canonical references and file identities'], enforced: ['only named existing signed runtime mutations; no generic shell/tool interception claimed'], unsupported: ['universal model obedience or arbitrary tool interception'] },
    status: blockers.length ? 'BLOCKED' : 'RESOLVED_REFERENCES', blockers,
  };
}

export function renderMethodContext(context) {
  return '# Derived method context\n\n' +
    `Method: ${context.selectedMethod}; phase: ${context.phaseId}; revision: ${context.sourceRevision}; status: ${context.status}.\n\n` +
    'This is a recovery index, not execution permission. Resolve current canonical decisions and contract before dependent action.\n\n' +
    '```json\n' + JSON.stringify(context, null, 2) + '\n```\n';
}

// Contain the nearest existing ancestor of a path that may not exist yet, so a symlinked ancestor cannot redirect mkdir.
function containedAncestor(root, target) {
  let cursor = target;
  while (!existsSync(cursor)) cursor = path.dirname(cursor);
  if (!within(root, realpathSync(cursor))) throw new Error('method context directory escapes project root');
}

// Fields that change on every lifecycle event without changing meaning; excluded so a refresh does not change provenance hashes.
const VOLATILE = ['generatedAt', 'sourceRevision'];
const semantic = context => JSON.stringify(Object.fromEntries(Object.entries(context).filter(([key]) => !VOLATILE.includes(key))));

export function writeMethodContext(project, context) {
  const root = realpathSync(project); const directory = ownedPath(root, context.phaseDirectory);
  containedAncestor(root, directory);
  mkdirSync(directory, { recursive: true });
  const existing = path.join(directory, 'method-context.json');
  try { if (existsSync(existing) && semantic(read(existing)) === semantic(context)) return; } catch { /* rewrite an unreadable index */ }
  for (const [name, bytes] of [['method-context.json', JSON.stringify(context, null, 2) + '\n'], ['method-context.md', renderMethodContext(context)]]) {
    const destination = ownedPath(root, path.join(context.phaseDirectory, name));
    const temporary = destination + `.${process.pid}.tmp`;
    try {
      writeFileSync(temporary, bytes, { flag: 'w', mode: 0o600 });
      renameSync(temporary, destination);
    } catch (error) { rmSync(temporary, { force: true }); throw error; }
  }
}

// Hooks reach this file through symlinked skill directories; compare real paths, as shared/openspec/cli.mjs does.
const invokedDirectly = () => { try { return Boolean(process.argv[1]) && realpathSync(process.argv[1]) === realpathSync(fileURLToPath(import.meta.url)); } catch { return false; } };
if (invokedDirectly()) {
  try {
    const args = process.argv.slice(2); let project = process.cwd(); let phase; let write = false; let json = false; let requireReady = false;
    for (let i = 0; i < args.length; i++) {
      if (args[i] === '--project' && args[i + 1]) project = args[++i];
      else if (args[i] === '--phase' && args[i + 1]) phase = args[++i];
      else if (args[i] === '--write') write = true;
      else if (args[i] === '--json') json = true;
      else if (args[i] === '--require-resolved') requireReady = true;
      else throw new Error(`unknown or incomplete argument: ${args[i]}`);
    }
    const context = discoverMethodContext({ project, phase });
    if (write) writeMethodContext(project, context);
    process.stdout.write(json ? JSON.stringify(context, null, 2) + '\n' : renderMethodContext(context));
    if (requireReady && context.status === 'BLOCKED') process.exitCode = 2;
  } catch (error) {
    process.stderr.write(`method-context: ${error.message}\n`); process.exitCode = 2;
  }
}
