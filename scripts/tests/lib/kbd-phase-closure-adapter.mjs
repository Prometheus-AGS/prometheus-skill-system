// Full-pack adapter for the shared KBD phase-closure conformance corpus
// (shared/references/conformance/kbd-phase-closure/v1). Every step drives a
// production entry point — the pack's shell helpers and the real `prometheus`
// binary — against a real signed journal and real ssh-keygen signatures in
// an isolated HOME/XDG/data root. Nothing is mocked.
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawn, spawnSync } from 'node:child_process';

export const NAMESPACE = 'prometheus-kbd-scope-amendment';
const OPERATOR = 'operator@kbd.test';
const INTRUDER = 'intruder@kbd.test';
const PHASE = 'closure-phase';
const STEP_TIMEOUT_MS = 180_000;
const BUDGETS = {
  ok: { timeMinutes: 1, toolCalls: 1, repairAttempts: 0 },
  'at-ceiling': { timeMinutes: 10, toolCalls: 100, repairAttempts: 2 },
  exceeded: { timeMinutes: 10, toolCalls: 101, repairAttempts: 2 },
};

const sha256 = (value) => crypto.createHash('sha256').update(value).digest('hex');
const writeFile = (file, value) => {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, typeof value === 'string' ? value : `${JSON.stringify(value, null, 2)}\n`, { mode: 0o600 });
};
const readJson = (file) => JSON.parse(fs.readFileSync(file, 'utf8'));

/** Canonical JSON with recursively sorted keys (JCS for ASCII string data). */
export function canonical(value) {
  if (value === null || typeof value !== 'object') return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(',')}}`;
}

function run(exe, args, { cwd, env, input } = {}) {
  const result = spawnSync(exe, args, { cwd, env, input, encoding: 'utf8', timeout: STEP_TIMEOUT_MS, maxBuffer: 16 * 1024 * 1024 });
  return { exit: result.status ?? (result.error ? 127 : 1), stdout: result.stdout ?? '', stderr: `${result.stderr ?? ''}${result.error ?? ''}` };
}

function runAsync(exe, args, { cwd, env }) {
  return new Promise((resolve) => {
    const child = spawn(exe, args, { cwd, env });
    let stdout = '', stderr = '';
    child.stdout.on('data', (chunk) => { stdout += chunk; });
    child.stderr.on('data', (chunk) => { stderr += chunk; });
    child.on('close', (code) => resolve({ exit: code ?? 1, stdout, stderr }));
  });
}

function deviceKey(file) {
  const pair = crypto.generateKeyPairSync('ed25519');
  const pub = pair.publicKey.export({ format: 'jwk' });
  const priv = pair.privateKey.export({ format: 'jwk' });
  writeFile(file, { schemaVersion: '1', keyId: `ed25519:${sha256(Buffer.from(pub.x, 'base64url'))}`,
    privateKey: Buffer.from(priv.d, 'base64url').toString('base64') });
}

function sshKey(file) {
  const made = run('ssh-keygen', ['-q', '-t', 'ed25519', '-N', '', '-C', path.basename(file), '-f', file]);
  if (made.exit !== 0) throw new Error(`ssh-keygen is required for scope-authority cases: ${made.stderr}`);
  return fs.readFileSync(`${file}.pub`, 'utf8').trim();
}

/** Resolve the entry points of one package surface (source tree or generated payload). */
export function surface(label, packageRoot, { sourceLayout }) {
  const orchestrator = path.join(packageRoot, sourceLayout ? 'skills/process/kbd-process-orchestrator' : 'skills/kbd-process-orchestrator');
  return {
    label,
    packageRoot,
    scripts: path.join(packageRoot, 'shared/scripts'),
    stageGate: path.join(orchestrator, 'shared/lib/stage-gate.sh'),
    nextPhase: path.join(orchestrator, 'skills/kbd-next-phase/scripts/kbd-next-phase.sh'),
  };
}

function baseContract(status) {
  const scenario = (id, oracle) => ({ id, ownerTask: 'T1', productionEntry: 'terminal evaluator', oracle,
    negativeControl: 'refusal recorded', fixtures: 'isolated scratch project', evidence: 'process output' });
  return {
    contractVersion: '1', kind: 'kbd.adopted-contract',
    identity: { projectId: status.projectId, runId: status.runId, phaseId: PHASE },
    digestNormalization: { algorithm: 'sha256', command: "jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)'", stalenessRule: 'bound digest must match' },
    adoptionSource: 'conformance fixture', decisionOwner: 'conformance operator', amendmentPolicy: 'signed amend-contract manifest',
    releaseClaim: { userOperation: 'close a phase exactly once', supportedEnvironments: ['local'], currentCapabilities: 'conformance' },
    statusVocabulary: ['PASS', 'FAIL', 'BLOCKED', 'NOT_EXERCISED'],
    scenarioMatrix: [scenario('CS-1', 'evaluator closes on PASS'), scenario('CS-2', 'dispatch refused when closed')],
    functionalScope: ['CS-1', 'CS-2'], exclusions: ['none'], requiredSafetyChecks: ['isolated canonical writer'],
    budgets: { timeMinutes: 10, toolCalls: 100, repairAttempts: 2, noProgressLimit: 2, tokenBudget: 'UNKNOWN', tokenFallback: 'time and tools', exhaustion: 'BLOCKED' },
    evidenceInvalidation: 'digest changes', workerRoutes: { agent: 'local', model: 'none' },
    boundaryPacket: { postImplementationOnly: true, singleRun: true, commands: ['integration'], stopConditions: ['failure'] },
    status: { implementationComplete: true },
  };
}

/** One isolated project at the corpus fixture state. */
export function fixture(scratch, caseId, prometheusBin) {
  const base = path.join(scratch, caseId);
  const home = path.join(base, 'home');
  fs.mkdirSync(path.join(base, 'tmp'), { recursive: true });
  const env = {
    PATH: `${path.dirname(prometheusBin)}${path.delimiter}${process.env.PATH}`,
    HOME: home, USERPROFILE: home, CODEX_HOME: path.join(home, '.codex'), CLAUDE_CONFIG_DIR: path.join(home, '.claude'),
    XDG_CONFIG_HOME: path.join(home, '.config'), XDG_DATA_HOME: path.join(home, '.local/share'), XDG_CACHE_HOME: path.join(home, '.cache'),
    TMPDIR: path.join(base, 'tmp'), PROMETHEUS_DATA_DIR: path.join(base, 'data'), PROMETHEUS_DEVICE_KEY_FILE: path.join(base, 'device-key.json'),
    PROMETHEUS_KBD_CONTROL_PLANE: '0', PROMETHEUS_CONTROL_ENDPOINT: 'http://127.0.0.1:1', PROMETHEUS_LEARNING_CORTEX: '0', PROMETHEUS_LEARNING_PK: '0',
    PYTHONDONTWRITEBYTECODE: '1', GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: os.devNull,
  };
  deviceKey(env.PROMETHEUS_DEVICE_KEY_FILE);
  const keys = { operator: path.join(base, 'keys/operator'), intruder: path.join(base, 'keys/intruder') };
  fs.mkdirSync(path.dirname(keys.operator), { recursive: true });
  const operatorPub = sshKey(keys.operator);
  sshKey(keys.intruder);
  writeFile(path.join(env.XDG_CONFIG_HOME, 'prometheus/kbd-scope-allowed-signers'),
    `${OPERATOR} namespaces="${NAMESPACE}" ${operatorPub}\n`);
  const project = path.join(base, 'project');
  fs.mkdirSync(path.join(project, '.kbd-orchestrator'), { recursive: true });
  const ctx = { base, env, keys, project, prometheusBin, phase: PHASE, labels: {}, phaseDir: path.join(project, '.kbd-orchestrator/phases', PHASE) };
  const setup = [
    ['status', '--json'],
    ['phase', 'create', '--command-id', 'fixture-create', '--id', PHASE, '--title', 'Closure phase'],
    ['phase', 'activate', '--command-id', 'fixture-activate', '--id', PHASE],
    ['guard', 'evaluate', '--boundary', 'phase', '--edge', 'before', '--subject', PHASE, '--json', '--repair-projections'],
    ['phase', 'transition', '--command-id', 'fixture-start', '--id', PHASE, '--status', 'in-progress'],
    ['change', 'register', '--command-id', 'fixture-change', '--phase', PHASE, '--id', 'C1', '--title', 'Closure change'],
    ['task', 'register', '--command-id', 'fixture-task', '--phase', PHASE, '--change', 'C1', '--id', 'T1', '--title', 'Closure task'],
  ];
  for (const args of setup) {
    const out = kbd(ctx, args);
    if (out.exit !== 0) throw new Error(`fixture step ${args.join(' ')} failed: ${out.stdout}${out.stderr}`);
  }
  writeFile(path.join(project, 'src.txt'), 'production source\n');
  writeFile(path.join(ctx.phaseDir, 'adopted-contract.json'), baseContract(status(ctx)));
  writeFile(path.join(ctx.phaseDir, 'tasks.json'), { tasks: [{ id: 'T1', title: 'Closure task' }] });
  writeFile(path.join(ctx.phaseDir, 'reflection.md'), '# Reflection\n\n## Next Phase Seed\n\n- proposals carried forward\n');
  return ctx;
}

const kbd = (ctx, args, env = ctx.env) => run(ctx.prometheusBin, ['kbd', '--path', ctx.project, ...args], { cwd: ctx.project, env });
const status = (ctx) => JSON.parse(kbd(ctx, ['status', '--json']).stdout);
export function disposition(ctx, phase = ctx.phase) {
  const out = kbd(ctx, ['phase', 'disposition', '--id', phase, '--json']);
  if (out.exit !== 0) throw new Error(`phase disposition ${phase} failed: ${out.stdout}${out.stderr}`);
  return JSON.parse(out.stdout);
}

/** Count journal events by kind across the project's replica journals. */
export function eventCounts(ctx) {
  const counts = {};
  const projects = path.join(ctx.env.PROMETHEUS_DATA_DIR, 'prometheus/kbd/projects');
  for (const project of fs.existsSync(projects) ? fs.readdirSync(projects) : []) {
    const replicas = path.join(projects, project, 'replicas');
    for (const replica of fs.existsSync(replicas) ? fs.readdirSync(replicas) : []) {
      const journal = path.join(replicas, replica, 'events.jsonl');
      if (!fs.existsSync(journal)) continue;
      for (const line of fs.readFileSync(journal, 'utf8').split('\n').filter(Boolean)) {
        const kind = JSON.parse(line).kind?.type;
        counts[kind] = (counts[kind] ?? 0) + 1;
      }
    }
  }
  return counts;
}

function expand(ctx, text) {
  const s = status(ctx);
  const bound = disposition(ctx).binding?.contractDigest ?? '';
  return text.replaceAll('{{phase}}', ctx.phase).replaceAll('{{runId}}', s.runId)
    .replaceAll('{{projectId}}', s.projectId).replaceAll('{{boundDigest}}', bound);
}

function contractFile(ctx) { return path.join(ctx.phaseDir, 'adopted-contract.json'); }
function relContract(ctx) { return path.relative(ctx.project, contractFile(ctx)); }

function editContract(ctx, args) {
  const contract = readJson(contractFile(ctx));
  if (args.addScenario) {
    contract.scenarioMatrix.push({ id: args.addScenario, ownerTask: 'T1', productionEntry: 'imported requirement',
      oracle: 'imported oracle', negativeControl: 'imported', fixtures: 'imported', evidence: 'imported' });
  }
  if (args.modifyScenario) {
    const entry = contract.scenarioMatrix.find((s) => s.id === args.modifyScenario);
    entry.oracle = `${entry.oracle} (amended)`;
  }
  Object.assign(contract, args.setField ?? {});
  writeFile(contractFile(ctx), contract);
}

/** Produce a signed manifest for a spec; returns { manifest, signature } paths. */
function signedManifest(ctx, spec, successor) {
  if (spec.reuse) return ctx.labels[spec.reuse];
  const dir = fs.mkdtempSync(path.join(ctx.base, 'manifest-'));
  const manifest = path.join(dir, 'manifest.json');
  const templateArgs = spec.action === 'amend-contract'
    ? ['amend-contract', '--phase', ctx.phase, '--contract', relContract(ctx)]
    : ['start-successor', '--phase', ctx.phase, '--successor', successor, '--title', `Successor ${successor}`];
  const out = run('bash', [path.join(ctx.surface.scripts, 'kbd-scope-authority-template.sh'), ...templateArgs,
    '--reason', 'operator authorizes this conformance scope change', '--approver', OPERATOR], { cwd: ctx.project, env: ctx.env });
  let body;
  if (out.exit === 0) {
    body = JSON.parse(out.stdout);
  } else if (spec.action === 'start-successor') {
    // The template refuses once a successor exists; a hostile caller can
    // still hand-craft the bytes, which is exactly what the runtime must refuse.
    const d = disposition(ctx), s = status(ctx);
    body = { schemaVersion: '1', kind: 'kbd.scope-authority', namespace: NAMESPACE, action: 'start-successor',
      projectId: s.projectId, runId: s.runId, phaseId: ctx.phase, predecessorClosureKey: d.closure?.closureKey ?? '0'.repeat(64),
      successorPhaseId: successor, successorTitle: `Successor ${successor}`, reason: 'hand-crafted second successor request',
      approver: OPERATOR, timestamp: new Date().toISOString().replace(/\.\d+Z$/, 'Z'), nonce: crypto.randomBytes(16).toString('hex') };
  } else {
    throw new Error(`scope template failed: ${out.stdout}${out.stderr}`);
  }
  if (spec.tamper === 'phase') body.phaseId = 'some-other-phase';
  if (spec.tamper === 'diff') body.scenarioChanges = { added: ['CS-77'], modified: [], removed: [] };
  writeFile(manifest, `${canonical(body)}\n`);
  const signature = `${manifest}.sig`;
  if (spec.tamper === 'garbage-signature') {
    writeFile(signature, 'this is not an ssh signature\n');
  } else {
    const ns = spec.namespace === 'other' ? 'file' : NAMESPACE;
    const signed = run('ssh-keygen', ['-q', '-Y', 'sign', '-f', ctx.keys[spec.signer ?? 'operator'], '-n', ns, manifest], { env: ctx.env });
    if (signed.exit !== 0) throw new Error(`ssh-keygen sign failed: ${signed.stderr}`);
  }
  const pair = { manifest, signature };
  if (spec.label) ctx.labels[spec.label] = pair;
  return pair;
}

function writeEvidence(ctx, args) {
  const s = status(ctx), d = disposition(ctx);
  const working = sha256(`${run('jq', ['-S', '-c', 'del(.identity.contractDigest) | del(.identity.planRevision)', contractFile(ctx)]).stdout}`);
  const contractDigest = d.binding?.contractDigest ?? working;
  const entry = (id, statusValue) => ({ id, status: statusValue, attempted: 'ran the production entry point',
    observed: `observed result${args.marker ? ` (${args.marker})` : ''}`, evidence: 'process output' });
  const scenarios = Object.entries(args.statuses ?? {}).map(([id, value]) => {
    const item = entry(id, value);
    if ((args.carry ?? []).includes(id)) {
      item.scenarioDigest = d.binding?.scenarioDigests?.[id];
      item.carriedFrom = { contractDigest: 'previous-binding' };
    }
    return item;
  });
  for (const extra of args.extra ?? []) scenarios.push(entry(extra.id, extra.status));
  const measured = typeof args.budget === 'object' ? args.budget : BUDGETS[args.budget ?? 'ok'];
  const values = Object.fromEntries(Object.entries(measured ?? BUDGETS.ok).map(([k, v]) => [k, { value: v, class: 'measured' }]));
  if (args.budget === 'unmeasured') values.timeMinutes = { value: null, class: 'unknown' };
  values.noProgress = { value: 0, class: 'measured' };
  values.tokens = { value: null, class: 'unknown' };
  const evidence = { phase: ctx.phase, projectId: s.projectId, runId: args.run === 'prefix' ? `${s.runId}x` : s.runId,
    contractDigest, sourceDigests: { 'src.txt': sha256(fs.readFileSync(path.join(ctx.project, 'src.txt'))) },
    budgetConsumption: { source: 'conformance fixture (self-reported)', values }, scenarios };
  if (args.approvedBy) evidence.approvedBy = args.approvedBy;
  writeFile(path.join(ctx.phaseDir, 'boundary-results.json'), evidence);
}

async function closeEval(ctx, args) {
  const env = { ...ctx.env };
  if (args.writer === 'unenrolled') {
    env.PROMETHEUS_DEVICE_KEY_FILE = path.join(ctx.base, 'unenrolled-key.json');
    if (!fs.existsSync(env.PROMETHEUS_DEVICE_KEY_FILE)) deviceKey(env.PROMETHEUS_DEVICE_KEY_FILE);
  }
  const argv = [path.join(ctx.surface.scripts, 'kbd-phase-close-eval.sh'), '--phase', ctx.phase];
  if (args.parallel) {
    const runs = await Promise.all(Array.from({ length: args.parallel }, () => runAsync('bash', argv, { cwd: ctx.project, env })));
    const worst = runs.find((r) => r.exit !== 0) ?? runs[0];
    return { ...worst, stdout: runs.map((r) => r.stdout).join('') };
  }
  return run('bash', argv, { cwd: ctx.project, env });
}

function stageGate(ctx) {
  const identity = readJson(contractFile(ctx)).identity;
  const write = run('bash', ['-c', '. "$1"; kbd_stage_handoff_write plan "conformance plan" "$2"', 'handoff', ctx.surface.stageGate, ctx.phaseDir],
    { cwd: ctx.project, env: { ...ctx.env, KBD_HANDOFF_IDENTITY_JSON: JSON.stringify(identity) } });
  if (write.exit !== 0) return write;
  return run('bash', ['-c', '. "$1"; kbd_stage_gate execute "$2"', 'gate', ctx.surface.stageGate, ctx.phaseDir], { cwd: ctx.project, env: ctx.env });
}

function nextPhase(ctx, args) {
  const argv = [ctx.surface.nextPhase, args.successor];
  if (args.authorization && args.authorization !== 'none') {
    const pair = signedManifest(ctx, args.authorization, args.successor);
    argv.push('--authorize-successor', pair.manifest, '--signature', pair.signature);
  }
  return run('bash', argv, { cwd: ctx.project, env: { ...ctx.env, KBD_ORCHESTRATOR_ROOT: '' } });
}

/** Execute one step; returns the process result (or null for state-only steps). */
export async function step(ctx, { op, args = {} }) {
  switch (op) {
    case 'adopt': {
      const argv = [path.join(ctx.surface.scripts, 'kbd-contract-adopt.sh'), relContract(ctx)];
      if (args.amendment) {
        const pair = signedManifest(ctx, args.amendment);
        argv.push('--supersedes', disposition(ctx).binding.adoptionDecisionId, '--amendment', pair.manifest, '--signature', pair.signature);
      }
      return run('bash', argv, { cwd: ctx.project, env: ctx.env });
    }
    case 'edit-contract': editContract(ctx, args); return null;
    case 'evidence': writeEvidence(ctx, args); return null;
    case 'close-eval': return closeEval(ctx, args);
    case 'cli-close': return kbd(ctx, ['phase', 'close', '--id', ctx.phase, '--evidence-file',
      path.join(ctx.phaseDir, 'boundary-results.json'), '--disposition', args.disposition]);
    case 'stage-gate-execute': return stageGate(ctx);
    case 'next-phase': return nextPhase(ctx, args);
    case 'cli-successor': {
      const pair = signedManifest(ctx, args.authorization, args.successor);
      return kbd(ctx, ['phase', 'successor', 'start', '--predecessor', ctx.phase, '--id', args.successor,
        '--title', `Successor ${args.successor}`, '--authorization', pair.manifest, '--signature', pair.signature]);
    }
    case 'forge-decision': return kbd(ctx, ['decision', 'record', '--command-id', `forge-${Date.now()}`,
      '--id', `forged-${crypto.randomBytes(4).toString('hex')}`, '--summary', expand(ctx, args.summary)]);
    case 'phase-transition': return kbd(ctx, ['phase', 'transition', '--command-id', `direct-${args.status}-${Date.now()}`,
      '--id', ctx.phase, '--status', args.status]);
    case 'phase-create': return kbd(ctx, ['phase', 'create', '--command-id', `direct-create-${args.id}`, '--id', args.id, '--title', args.id]);
    case 'phase-activate': return kbd(ctx, ['phase', 'activate', '--command-id', `direct-activate-${expand(ctx, args.id)}-${Date.now()}`,
      '--id', expand(ctx, args.id)]);
    case 'restart': {
      const projects = path.join(ctx.env.PROMETHEUS_DATA_DIR, 'prometheus/kbd/projects');
      for (const project of fs.readdirSync(projects)) fs.rmSync(path.join(projects, project, 'checkpoints'), { recursive: true, force: true });
      return kbd(ctx, ['status', '--json']);
    }
    case 'assert': return null;
    default: throw new Error(`unknown corpus op ${op}`);
  }
}

/** Evaluate an `assert` step; returns a list of mismatch strings. */
export function checkAssert(ctx, args) {
  const problems = [];
  const d = disposition(ctx), s = status(ctx), counts = eventCounts(ctx);
  const want = (label, actual, expected) => { if (expected !== undefined && actual !== expected) problems.push(`${label}: expected ${expected}, got ${actual}`); };
  want('disposition', d.disposition, args.disposition);
  want('phaseStatus', d.phaseStatus, args.phaseStatus);
  want('bindingRevision', d.binding?.bindingRevision, args.bindingRevision);
  want('successor', d.successor?.phaseId, args.successor);
  want('activePhase', s.activePath?.phaseId, args.activePhase);
  if (args.successorStatus) want('successorStatus', s.phases?.[args.successor]?.status, args.successorStatus);
  for (const [kind, count] of Object.entries(args.events ?? {})) want(`events.${kind}`, counts[kind] ?? 0, count);
  return problems;
}

export function checkExpect(result, expect = {}) {
  const problems = [];
  if (expect.exit !== undefined && result.exit !== expect.exit) problems.push(`exit: expected ${expect.exit}, got ${result.exit}`);
  if (expect.stdout && !new RegExp(expect.stdout, 'm').test(result.stdout)) problems.push(`stdout !~ /${expect.stdout}/`);
  if (expect.stderr && !new RegExp(expect.stderr, 'm').test(result.stderr)) problems.push(`stderr !~ /${expect.stderr}/`);
  if (expect.notStdout && new RegExp(expect.notStdout, 'm').test(result.stdout)) problems.push(`stdout =~ /${expect.notStdout}/ (forbidden)`);
  return problems;
}
