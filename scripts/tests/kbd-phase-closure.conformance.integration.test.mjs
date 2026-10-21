// Canonical phase closure + scope authority conformance (issue #192).
// Runs the shared corpus through the source tree and both generated payloads
// with a real `prometheus` binary, real shell entry points, a real signed
// journal and real ssh-keygen signatures, in isolated HOME/XDG/data roots.
// PROMETHEUS_CLI_TEST_BINARY must name a prometheus build with typed phase
// closure; a missing binary or ssh-keygen fails the suite rather than skipping.
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { canonical, checkAssert, checkExpect, fixture, step, surface } from './lib/kbd-phase-closure-adapter.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const corpus = path.join(root, 'shared/references/conformance/kbd-phase-closure/v1');
const prometheusBin = path.resolve(process.env.PROMETHEUS_CLI_TEST_BINARY ?? '');
const receiptsDir = process.env.KBD_CONFORMANCE_RECEIPTS ?? fs.mkdtempSync(path.join(os.tmpdir(), 'kbd-closure-receipts-'));
const cases = fs.readdirSync(path.join(corpus, 'cases')).filter((f) => f.endsWith('.json')).sort()
  .map((f) => JSON.parse(fs.readFileSync(path.join(corpus, 'cases', f), 'utf8')));
const sha256 = (value) => crypto.createHash('sha256').update(value).digest('hex');
const version = (cmd, args) => (spawnSync(cmd, args, { encoding: 'utf8' }).stdout || spawnSync(cmd, args, { encoding: 'utf8' }).stderr || '').trim().split('\n')[0];

/** Install a generated payload into a scratch home with the real installer. */
function installed(platform) {
  const payload = path.join(root, 'dist/plugins', platform, 'prometheus-skill-pack');
  const home = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), `kbd-closure-install-${platform}-`)));
  const store = path.join(home, '.prometheus/plugins/prometheus-skill-pack');
  const env = { PATH: process.env.PATH, HOME: home, USERPROFILE: home, CODEX_HOME: path.join(home, '.codex'),
    CLAUDE_CONFIG_DIR: path.join(home, '.claude'), XDG_CONFIG_HOME: path.join(home, '.config'),
    XDG_DATA_HOME: path.join(home, '.local/share'), PROMETHEUS_PLUGIN_ROOT: store, PYTHONDONTWRITEBYTECODE: '1' };
  const result = spawnSync(process.execPath, [path.join(payload, 'scripts/install-plugin-generation.js'), '--source-root', payload,
    '--home', home, '--plugin-root', store, '--targets', 'codex'], { env, encoding: 'utf8', timeout: 600_000 });
  assert.equal(result.status, 0, `installer for ${platform} failed:\n${result.stdout}\n${result.stderr}`);
  return surface(`installed ${platform}`, fs.realpathSync(path.join(store, 'current')), { sourceLayout: false });
}

const SURFACES = [
  () => surface('source', root, { sourceLayout: true }),
  () => surface('generated claude', path.join(root, 'dist/plugins/claude/prometheus-skill-pack'), { sourceLayout: false }),
  () => surface('generated codex', path.join(root, 'dist/plugins/codex/prometheus-skill-pack'), { sourceLayout: false }),
  () => installed('claude'),
  () => installed('codex'),
];
const SURFACE_LABELS = ['source', 'generated claude', 'generated codex', 'installed claude', 'installed codex'];

test('prerequisites: typed-closure prometheus and ssh-keygen are present', () => {
  assert.ok(process.env.PROMETHEUS_CLI_TEST_BINARY, 'set PROMETHEUS_CLI_TEST_BINARY to the built prometheus binary');
  const help = spawnSync(prometheusBin, ['kbd', 'phase', '--help'], { encoding: 'utf8' });
  assert.match(help.stdout, /disposition/, 'prometheus binary lacks `kbd phase disposition`');
  assert.equal(spawnSync('ssh-keygen', ['-Y', 'check-novalidate', '-n', 'x', '-s', os.devNull], { encoding: 'utf8' }).error, undefined);
});

test('golden vectors: contract digest, scenario digests, closure key, manifest bytes', () => {
  const golden = path.join(corpus, 'golden');
  const digests = JSON.parse(fs.readFileSync(path.join(golden, 'digests.json'), 'utf8'));
  const normalized = spawnSync('jq', ['-S', '-c', 'del(.identity.contractDigest) | del(.identity.planRevision)', path.join(golden, 'contract.json')], { encoding: 'utf8' }).stdout;
  assert.equal(sha256(normalized), digests.contractDigest, 'jq normalization digest');
  assert.equal(sha256(fs.readFileSync(path.join(golden, 'contract.snapshot.json'))), digests.contractDigest, 'snapshot bytes hash to the digest');
  for (const [id, digest] of Object.entries(digests.scenarioDigests)) {
    const entry = spawnSync('jq', ['-S', '-c', '--arg', 'i', id, '.scenarioMatrix[] | select(.id == $i)', path.join(golden, 'contract.json')], { encoding: 'utf8' }).stdout;
    assert.equal(sha256(entry), digest, `scenario ${id}`);
  }
  const vectors = JSON.parse(fs.readFileSync(path.join(golden, 'vectors.json'), 'utf8'));
  const identity = fs.readFileSync(path.join(golden, vectors.closureIdentityFile), 'utf8');
  assert.equal(identity, canonical(JSON.parse(identity)), 'closure identity is canonical');
  assert.equal(sha256(identity), vectors.closureKey);
  const manifestFile = fs.readFileSync(path.join(golden, vectors.manifestFile), 'utf8');
  assert.ok(manifestFile.endsWith('}\n') && !manifestFile.endsWith('\n\n'), 'manifest file carries exactly one trailing newline');
  const raw = manifestFile.slice(0, -1);
  assert.equal(raw, canonical(JSON.parse(raw)), 'manifest is canonical');
  assert.equal(sha256(raw), vectors.manifestSha256);
});

for (const [index, makeSurface] of SURFACES.entries()) {
  test(`corpus on ${SURFACE_LABELS[index]}`, async () => {
    const target = makeSurface();
    assert.ok(fs.existsSync(target.stageGate), `${target.label}: stage gate missing at ${target.stageGate}`);
    const scratch = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'kbd-closure ')));
    const receipt = { schema: 'kbd.conformance-receipt/v1', corpus: 'kbd-phase-closure/v1', surface: target.label,
      packageRoot: target.packageRoot, commit: spawnSync('git', ['-C', root, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).stdout.trim(),
      dirtyTree: spawnSync('git', ['-C', root, 'status', '--porcelain'], { encoding: 'utf8' }).stdout.trim().length > 0,
      versions: { prometheus: version(prometheusBin, ['--version']), bash: version('bash', ['--version']), jq: version('jq', ['--version']),
        sshKeygen: version('ssh', ['-V']), node: process.version, os: `${os.platform()} ${os.release()}` },
      prometheusBinary: prometheusBin, startedAt: new Date().toISOString(), cases: [],
      notRun: ['remote control plane (PROMETHEUS_KBD_CONTROL_PLANE=0)', 'Windows and Linux hosts', 'harness command mirrors and slash-command prompts (guidance text only)', 'live operator homes and journals'] };
    const failures = [];
    try {
      for (const kase of cases) {
        const ctx = fixture(scratch, kase.id, prometheusBin);
        ctx.surface = target;
        const record = { id: kase.id, acceptance: kase.acceptance, steps: [], result: 'PASS' };
        for (const [index, spec] of kase.steps.entries()) {
          const outcome = await step(ctx, spec);
          const problems = spec.op === 'assert' ? checkAssert(ctx, spec.args ?? {}) : (outcome ? checkExpect(outcome, spec.expect) : []);
          record.steps.push({ index, op: spec.op, exit: outcome?.exit ?? null, problems });
          if (problems.length) {
            record.result = 'FAIL';
            failures.push(`${kase.id} step ${index} (${spec.op}): ${problems.join('; ')}\n  stdout: ${outcome?.stdout?.slice(-1200) ?? ''}\n  stderr: ${outcome?.stderr?.slice(-1200) ?? ''}`);
            break;
          }
        }
        receipt.cases.push(record);
      }
    } finally {
      receipt.finishedAt = new Date().toISOString();
      receipt.passed = receipt.cases.filter((c) => c.result === 'PASS').length;
      receipt.total = cases.length;
      fs.mkdirSync(receiptsDir, { recursive: true });
      fs.writeFileSync(path.join(receiptsDir, `${target.label.replace(/\s+/g, '-')}.receipt.json`), `${JSON.stringify(receipt, null, 2)}\n`);
      if (!failures.length) fs.rmSync(scratch, { recursive: true, force: true });
    }
    assert.deepEqual(failures, [], `${target.label}: ${failures.length} corpus case(s) failed (scratch kept for inspection)\n${failures.join('\n')}`);
  });
}
