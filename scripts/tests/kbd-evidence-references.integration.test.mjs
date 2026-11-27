// Issue #256: evidence references in the closure packet. Runs the shipped
// packet generator (the script kbd-phase-close-eval.sh invokes) against real
// Git checkouts and reads the files it writes.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const PACKET = path.join(REPO, 'shared/scripts/kbd-closure-packet.mjs');
const PHASE = 'evidence-phase';

const CONTRACT = {
  contractVersion: 2,
  identity: { phaseId: PHASE },
  scenarioMatrix: [
    { id: 'SCN-1', name: 'Export writes every note', actor: 'A departing user', usefulOutcome: 'One file with every note', mandatory: true, scopeRefs: ['Export all notes'] },
    { id: 'SCN-2', name: 'Export refuses a corrupted store', actor: 'A departing user', usefulOutcome: 'A clear refusal', mandatory: true, scopeRefs: ['Refuse corrupted input'] },
  ],
};

function sandbox(t) {
  const project = realpathSync(mkdtempSync(path.join(os.tmpdir(), 'evidence-refs-')));
  t.after(() => rmSync(project, { recursive: true, force: true }));
  const git = (...args) => execFileSync('git', ['-C', project, '-c', 'user.name=Refs', '-c', 'user.email=refs@example.invalid', ...args], { encoding: 'utf8' }).trim();
  git('init', '-q', '-b', 'main');
  writeFileSync(path.join(project, 'src.txt'), 'first revision\n');
  git('add', '-A'); git('commit', '-qm', 'first');
  const phaseDir = path.join(project, '.kbd-orchestrator/phases', PHASE);
  mkdirSync(phaseDir, { recursive: true });
  writeFileSync(path.join(phaseDir, 'contract.json'), JSON.stringify(CONTRACT));
  const packet = evidence => {
    writeFileSync(path.join(phaseDir, 'boundary-results.json'), JSON.stringify(evidence));
    const run = spawnSync('node', [PACKET, '--snapshot', path.join(phaseDir, 'contract.json'), '--evidence', path.join(phaseDir, 'boundary-results.json'),
      '--out-dir', phaseDir, '--project-root', project, '--disposition', 'CLOSED', '--reason', 'all-pass'], { encoding: 'utf8' });
    assert.equal(run.status, 0, run.stderr);
    return { closure: JSON.parse(readFileSync(path.join(phaseDir, 'closure.json'), 'utf8')), text: readFileSync(path.join(phaseDir, 'closure-packet.md'), 'utf8') };
  };
  const candidate = () => ({ baseHead: git('rev-parse', 'HEAD'), treeDigest: git('rev-parse', 'HEAD^{tree}') });
  return { project, git, packet, candidate };
}

const entry = (id, status, refs = {}) => ({ id, status, attempted: 'ran the export', observed: 'as expected', evidence: 'out.md digest', ...refs });

test('recorded references are shown and a result for the current candidate keeps its verdict', t => {
  const box = sandbox(t);
  const { closure, text } = box.packet({
    phase: PHASE, candidate: box.candidate(), venue: 'local macOS arm64, scratch HOME',
    scenarios: [
      entry('SCN-1', 'PASS', { scopeRef: 'Export all notes', fixtureRef: 'fixtures/notes.sql@sha256:abc', oracleRef: 'three inserted notes' }),
      entry('SCN-2', 'PASS', { scopeRef: 'Some other scope' }),
    ],
  });
  assert.equal(closure.candidate.status, 'current');
  assert.equal(closure.venue, 'local macOS arm64, scratch HOME');
  const [first, second] = closure.scenarios;
  assert.equal(first.verdict, 'PASS');
  assert.equal(first.currentCandidateVerdict, 'PASS');
  assert.deepEqual(first.evidenceRefs, { scopeRef: 'Export all notes', fixtureRef: 'fixtures/notes.sql@sha256:abc', oracleRef: 'three inserted notes', scopeRefInContract: true });
  // A scope reference the contract does not list for the scenario is said so, not rejected.
  assert.equal(second.evidenceRefs.scopeRefInContract, false);
  assert.equal(second.verdict, 'PASS');
  assert.match(text, /^- candidate: CURRENT — /m);
  assert.match(text, /^- venue: local macOS arm64, scratch HOME$/m);
  assert.match(text, /^- SCN-1: recorded PASS; scope Export all notes; fixture fixtures\/notes\.sql@sha256:abc; oracle three inserted notes$/m);
  assert.match(text, /SCN-2: recorded PASS; scope Some other scope \(not a scope reference of this scenario in the contract\)/);
  assert.match(text, /^- SCN-1 — One file with every note/m);
  // Uncommitted phase files under .kbd-orchestrator do not make the candidate unknown.
  assert.equal(closure.candidate.current.uncommittedChanges, false);
});

test('a pass recorded for an earlier candidate is stale, and an unexercised scenario is never a failure', t => {
  const box = sandbox(t);
  const earlier = box.candidate();
  writeFileSync(path.join(box.project, 'src.txt'), 'second revision\n');
  box.git('add', '-A'); box.git('commit', '-qm', 'second');

  const { closure, text } = box.packet({
    phase: PHASE, candidate: earlier, venue: 'local',
    scenarios: [entry('SCN-1', 'PASS', { scopeRef: 'Export all notes' })],
  });
  assert.equal(closure.candidate.status, 'stale');
  assert.equal(closure.candidate.recorded.baseHead, earlier.baseHead);
  assert.notEqual(closure.candidate.current.baseHead, earlier.baseHead);
  const [first, second] = closure.scenarios;
  // The recorded verdict is kept as recorded; for the current candidate it is not a pass.
  assert.equal(first.verdict, 'PASS');
  assert.equal(first.currentCandidateVerdict, 'NOT_EXERCISED');
  // SCN-2 was never run: NOT_EXERCISED in both views, never FAIL.
  assert.equal(second.verdict, 'NOT_EXERCISED');
  assert.equal(second.currentCandidateVerdict, 'NOT_EXERCISED');
  assert.ok(!closure.scenarios.some(row => row.verdict === 'FAIL' || row.currentCandidateVerdict === 'FAIL'));
  assert.match(text, /^- candidate: STALE — the recorded base commit is not the current commit/m);
  assert.match(text, /nothing is established for the current candidate: 1 recorded PASS result\(s\) belong to an earlier candidate/);
  assert.match(text, /SCN-1: recorded PASS on an earlier candidate; NOT_EXERCISED on the current one/);
  assert.doesNotMatch(text, /^- SCN-1 — One file with every note/m);
  assert.match(text, /^- SCN-2 — NOT_EXERCISED$/m);
  // Report only: the disposition handed in is not changed by the label.
  assert.equal(closure.disposition, 'CLOSED');
});

test('evidence without references produces the same verdicts and says nothing was recorded', t => {
  const box = sandbox(t);
  const { closure, text } = box.packet({ phase: PHASE, scenarios: [entry('SCN-1', 'PASS'), entry('SCN-2', 'FAIL')] });
  assert.equal(closure.candidate.status, 'not-recorded');
  assert.equal(closure.venue, null);
  assert.deepEqual(closure.scenarios.map(row => [row.verdict, row.currentCandidateVerdict, row.evidenceRefs]),
    [['PASS', 'PASS', null], ['FAIL', 'FAIL', null]]);
  assert.match(text, /^- candidate: not recorded in boundary-results\.json$/m);
  assert.match(text, /^- venue: not recorded$/m);
  assert.match(text, /^- SCN-1 — One file with every note/m);
  assert.match(text, /^- SCN-2 — FAIL$/m);
});

test('uncommitted source changes make the comparison unknown, not current and not stale', t => {
  const box = sandbox(t);
  const recorded = box.candidate();
  writeFileSync(path.join(box.project, 'src.txt'), 'edited after the run\n');
  const { closure, text } = box.packet({ phase: PHASE, candidate: recorded, scenarios: [entry('SCN-1', 'PASS')] });
  assert.equal(closure.candidate.status, 'unknown');
  assert.equal(closure.scenarios[0].currentCandidateVerdict, 'PASS');
  assert.match(text, /^- candidate: UNKNOWN — the checkout has uncommitted changes outside \.kbd-orchestrator/m);
});
