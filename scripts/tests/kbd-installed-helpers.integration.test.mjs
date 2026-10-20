// Real CLI + signed canonical state + generated payload/installer regression.
// No status files, decision sinks, helper stubs, live home, or live KBD phase.
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const scratch = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'kbd installed helpers ')));
const phase = 'packaging-regression';
const read = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const write = (file, value) => { fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, JSON.stringify(value, null, 2), { mode: 0o600 }); };
const hash = value => crypto.createHash('sha256').update(value).digest('hex');
function run(exe, args, cwd, env, expected = 0) {
  const result = spawnSync(exe, args, { cwd, env, encoding: 'utf8', timeout: 180_000, maxBuffer: 8 * 1024 * 1024 });
  assert.equal(result.status, expected, `${exe} ${args.join(' ')}\n${result.error ?? ''}\n${result.stdout}\n${result.stderr}`);
  return `${result.stdout}${result.stderr}`;
}
function isolated(label) {
  const base = path.join(scratch, label), home = path.join(base, 'home with spaces');
  fs.mkdirSync(home, { recursive: true });
  const env = { PATH: process.env.PATH, HOME: home, USERPROFILE: home,
    CODEX_HOME: path.join(home, '.codex'), CLAUDE_CONFIG_DIR: path.join(home, '.claude'),
    XDG_CONFIG_HOME: path.join(home, '.config'), XDG_DATA_HOME: path.join(home, '.local/share'),
    XDG_CACHE_HOME: path.join(home, '.cache'), TMPDIR: path.join(base, 'tmp'),
    PROMETHEUS_DATA_DIR: path.join(base, 'canonical data'), PROMETHEUS_DEVICE_KEY_FILE: path.join(base, 'device-key.json'),
    PROMETHEUS_KBD_CONTROL_PLANE: '0', PROMETHEUS_CONTROL_ENDPOINT: 'http://127.0.0.1:1',
    PROMETHEUS_LEARNING_CORTEX: '0', PROMETHEUS_LEARNING_PK: '0',
    PYTHONDONTWRITEBYTECODE: '1', GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: os.devNull,
  };
  fs.mkdirSync(env.TMPDIR, { recursive: true });
  const pair = crypto.generateKeyPairSync('ed25519');
  const pub = pair.publicKey.export({ format: 'jwk' }), priv = pair.privateKey.export({ format: 'jwk' });
  write(env.PROMETHEUS_DEVICE_KEY_FILE, { schemaVersion: '1', keyId: `ed25519:${hash(Buffer.from(pub.x, 'base64url'))}`,
    privateKey: Buffer.from(priv.d, 'base64url').toString('base64') });
  return { base, home, env };
}
function contract(status) {
  return { contractVersion: '1', kind: 'kbd.adopted-contract',
    identity: { projectId: status.projectId, runId: status.runId, phaseId: phase },
    digestNormalization: { algorithm: 'sha256', command: "jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)'", stalenessRule: 'latest adoption digest must match' },
    adoptionSource: 'isolated packaging regression', decisionOwner: 'local tester', amendmentPolicy: 'explicit supersedes',
    releaseClaim: { userOperation: 'Adopt and validate from an ordinary consumer', supportedEnvironments: ['local'], currentCapabilities: 'installed helpers' },
    statusVocabulary: ['PASS', 'FAIL', 'BLOCKED', 'NOT_EXERCISED'],
    scenarioMatrix: [{ id: 'PK-1', ownerTask: 'T1', productionEntry: 'installed helper', oracle: 'READY', negativeControl: 'missing contract', fixtures: 'temporary consumer', evidence: 'process output' }],
    functionalScope: ['PK-1'], exclusions: ['semantic changes'], requiredSafetyChecks: ['isolated canonical writer'],
    budgets: { timeMinutes: 10, toolCalls: 100, repairAttempts: 2, noProgressLimit: 2, tokenBudget: 'UNKNOWN', tokenFallback: 'time and tools', exhaustion: 'BLOCKED' },
    evidenceInvalidation: 'digest changes', workerRoutes: { agent: 'local', model: 'none' },
    boundaryPacket: { postImplementationOnly: true, singleRun: true, commands: ['integration'], stopConditions: ['failure'] },
    status: { implementationComplete: true } };
}
function exercise(label, payload, box, installed = false) {
  const { base, env } = box;
  const project = path.join(base, 'consumer with spaces'); fs.mkdirSync(project);
  fs.mkdirSync(path.join(project, '.kbd-orchestrator'));
  const native = (...args) => run('prometheus', ['kbd', '--path', project, ...args], project, env);
  native('status', '--json');
  native('phase', 'create', '--command-id', 'create', '--id', phase, '--title', 'Packaging regression');
  native('phase', 'activate', '--command-id', 'activate', '--id', phase);
  native('change', 'register', '--command-id', 'change', '--phase', phase, '--id', 'C1', '--title', 'Consumer helpers');
  native('task', 'register', '--command-id', 'task', '--phase', phase, '--change', 'C1', '--id', 'T1', '--title', 'Adopt contract');
  const status = JSON.parse(native('status', '--json'));
  const phaseDir = path.join(project, '.kbd-orchestrator/phases', phase);
  const relative = `.kbd-orchestrator/phases/${phase}/adopted-contract.json`, file = path.join(project, relative);
  write(file, contract(status));
  // Authored phase coverage input, separate from the CLI's canonical task registration.
  write(path.join(phaseDir, 'tasks.json'), { tasks: [{ id: 'T1', title: 'Adopt contract' }] });
  const source = payload === root;
  const orchestrator = path.join(payload, source ? 'skills/process/kbd-process-orchestrator' : 'skills/kbd-process-orchestrator');
  const gateFile = path.join(orchestrator, 'shared/lib/stage-gate.sh');
  const helpers = installed ? path.join(box.store, 'stable') : path.join(payload, 'shared/scripts');
  const adopt = (expected = 0) => run('bash', [path.join(helpers, 'kbd-contract-adopt.sh'), relative], project, env, expected);
  const ready = (expected = 0) => run('bash', [path.join(helpers, 'kbd-phase-readiness-check.sh'), '--phase', phase], project, env, expected);
  const gate = (expected = 0, cwd = project, library = gateFile, extra = {}) => run('bash', ['-c', '. "$1"; kbd_stage_gate execute "$2"', 'gate', library, phaseDir], cwd, { ...env, ...extra }, expected);
  assert.match(ready(2), /BLOCKED missing=adoption/);
  assert.match(adopt(), /^ADOPTED /);
  const stamped = fs.readFileSync(file), after = JSON.parse(native('status', '--json'));
  assert.equal(Object.keys(after.decisions).length, 1);
  assert.match(Object.values(after.decisions)[0].summary, new RegExp(`ADOPTED-CONTRACT path=${relative}`));
  assert.match(read(file).identity.contractDigest, /^[a-f0-9]{64}$/);
  const journal = native('audit', '--json');
  assert.match(adopt(), /^ALREADY-ADOPTED /);
  assert.equal(native('audit', '--json'), journal, 'repeat adoption must not append any canonical event');
  assert.deepEqual(fs.readFileSync(file), stamped);
  assert.equal(JSON.parse(native('status', '--json')).revision, after.revision);
  assert.match(ready(), /^READY /);
  run('bash', ['-c', '. "$1"; kbd_stage_handoff_write plan "Consumer contract adopted" "$2"', 'handoff', gateFile, phaseDir], project,
    { ...env, KBD_HANDOFF_IDENTITY_JSON: JSON.stringify(read(file).identity) });
  gate(); gate(0, base); // explicit consumer phase, unrelated working directory
  if (installed) gate(0, project, path.join(box.home, '.codex/skills/kbd-process-orchestrator/shared/lib/stage-gate.sh'));
  const skillDir = path.join(payload, source ? 'skills/process/kbd-process-orchestrator/skills' : 'skills');
  for (const [name, marker] of [['kbd-plan', 'ALREADY-ADOPTED'], ['kbd-execute', 'READY'], ['kbd-apply', 'READY']]) {
    const text = fs.readFileSync(path.join(skillDir, name, 'SKILL.md'), 'utf8');
    const block = [...text.matchAll(/```bash\n([\s\S]*?)```/g)].map(m => m[1]).find(s => s.includes('KBD_PACKAGE_ROOT='));
    assert.ok(block, `${name} must document package-root execution`);
    const command = block.replaceAll('<phase>', phase);
    const docEnv = installed ? env : { ...env, KBD_PACK_ROOT: payload };
    assert.match(run('bash', ['-c', command], project, docEnv), new RegExp(`^${marker} `));
  }
  fs.unlinkSync(file);
  assert.match(adopt(1), /contract not found/);
  assert.match(ready(2), /BLOCKED missing=contract/);
  assert.match(gate(2), /BLOCKED missing=contract/);
  fs.writeFileSync(file, '{invalid');
  assert.match(adopt(2), /not valid JSON/);
  assert.match(ready(5), /MALFORMED contract/);
  assert.match(gate(2), /MALFORMED contract/);
  const foreign = JSON.parse(stamped); foreign.identity.runId = 'foreign-run'; write(file, foreign);
  assert.match(ready(3), /REJECT-IDENTITY/);
  assert.match(gate(2), /REJECT-IDENTITY/);
  const stale = JSON.parse(stamped); stale.releaseClaim.currentCapabilities += ' changed'; write(file, stale);
  assert.match(ready(4), /REJECT-STALE/);
  assert.match(gate(2), /REJECT-STALE/);
  fs.writeFileSync(file, stamped);
  if (installed) {
    const checker = path.join(payload, 'shared/scripts/kbd-phase-readiness-check.sh');
    fs.renameSync(checker, `${checker}.saved`);
    try { assert.match(gate(2), /readiness checker missing.*broken install/s); }
    finally { fs.renameSync(`${checker}.saved`, checker); }
    for (const name of ['kbd-contract-adopt.sh', 'kbd-phase-readiness-check.sh']) {
      assert.deepEqual(fs.readFileSync(path.join(helpers, name)), fs.readFileSync(path.join(root, 'shared/scripts', name)));
    }
  }
  assert.match(ready(), /^READY /); gate();
  assert.equal(fs.existsSync(path.join(project, 'shared')), false);
  assert.equal(fs.existsSync(path.join(project, 'scripts')), false);
  console.log(`PASS ${label}: adoption, READY, real gate, idempotent journal, documented commands, missing/malformed/identity/stale state${installed ? ', missing helper, stable links, copied skill' : ''}; consumer has no scripts`);
}
try {
  exercise('source', root, isolated('source'));
  for (const platform of ['claude', 'codex']) {
    const payload = path.join(root, 'dist/plugins', platform, 'prometheus-skill-pack');
    exercise(`generated ${platform}`, payload, isolated(`generated ${platform}`));
    const box = isolated(`installed ${platform}`);
    box.store = path.join(box.home, '.prometheus/plugins/prometheus-skill-pack');
    console.log(`Installing ${platform} payload into temporary home`);
    run(process.execPath, [path.join(payload, 'scripts/install-plugin-generation.js'), '--source-root', payload,
      '--home', box.home, '--plugin-root', box.store, '--targets', 'codex'], box.base, box.env);
    const installed = fs.realpathSync(path.join(box.store, 'current'));
    exercise(`installed ${platform}`, installed, box, true);
  }
  console.log('PASS all isolated package surfaces');
} finally { fs.rmSync(scratch, { recursive: true, force: true }); }
