#!/usr/bin/env node
// kbd-closure-packet.mjs — derived closure view for a KBD value phase (VLBV CR-21).
//
// Written by kbd-phase-close-eval.sh for every outcome (CLOSED or refused). It is
// a readable *view* generated from the bound contract snapshot and
// boundary-results.json; the canonical disposition recorded through
// `prometheus kbd phase close` remains the authority.
//
// Usage: node kbd-closure-packet.mjs --snapshot <contract.json> --evidence <boundary-results.json>
//          --out-dir <phase-dir> --disposition <CLOSED|REFUSED_BLOCKED|REFUSED_FAIL> --reason <text>
//
// Optional evidence references (issue #256), shown when boundary-results.json
// carries them and never verdict-changing: `candidate` {baseHead, treeDigest},
// `venue`, and per scenario `scopeRef`, `fixtureRef`, `oracleRef`.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';

const args = process.argv.slice(2);
const opt = name => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};
const readJson = file => {
  try {
    return JSON.parse(fs.readFileSync(file, 'utf8'));
  } catch {
    return null;
  }
};
const contract = readJson(opt('--snapshot')) ?? {};
const evidencePath = opt('--evidence');
const evidence = readJson(evidencePath) ?? {};
const outDir = opt('--out-dir');
const disposition = opt('--disposition') ?? 'UNKNOWN';
const reason = opt('--reason') ?? '';
if (!outDir) {
  process.stderr.write('kbd-closure-packet: --out-dir is required\n');
  process.exit(1);
}

// For signed exits the caller passes the manifest returned by the canonical
// disposition reader. The signed contract and its original scenario rows remain intact.
const operatorDisposition = ['CLOSED_SUPERSEDED', 'CLOSED_UNMET'].includes(disposition);
const operatorExit = operatorDisposition ? readJson(opt('--exit-manifest')) : null;
if (operatorDisposition && (!operatorExit || !Array.isArray(operatorExit.exercised) || !Array.isArray(operatorExit.withdrawn))) {
  throw new Error('canonical operator exit manifest required for signed closure');
}

const scenarios = contract.scenarioMatrix ?? [];
const entries = evidence.scenarios ?? [];
const verdictOf = id => {
  if (operatorExit) {
    return operatorExit.exercised.find(item => item.id === id)?.verdict ?? 'WITHDRAWN';
  }
  const own = entries.filter(e => e.id === id || String(e.id).startsWith(`${id}-`));
  if (!own.length) return 'NOT_EXERCISED';
  if (own.some(e => e.status === 'FAIL')) return 'FAIL';
  if (own.every(e => e.status === 'PASS')) return 'PASS';
  return own.some(e => e.status === 'BLOCKED') ? 'BLOCKED' : 'NOT_EXERCISED';
};
// Evidence references (issue #256). The recorded candidate is compared with the
// checkout the packet is generated in. A result recorded for another candidate
// is labelled stale: it is not a result for the current one. Report only.
const refProjectRoot = path.resolve(opt('--project-root') ?? findProjectRoot(outDir));
function git(...gitArgs) {
  const run = spawnSync('git', ['-C', refProjectRoot, ...gitArgs], { encoding: 'utf8' });
  return run.status === 0 ? run.stdout.trim() : null;
}
function candidateReport() {
  const recorded = evidence.candidate && typeof evidence.candidate === 'object' ? evidence.candidate : null;
  const baseHead = typeof recorded?.baseHead === 'string' ? recorded.baseHead : null;
  const treeDigest = typeof recorded?.treeDigest === 'string' ? recorded.treeDigest : null;
  if (!baseHead && !treeDigest) return { status: 'not-recorded', recorded: null, current: null, reason: 'boundary-results.json records no candidate' };
  const head = git('rev-parse', 'HEAD');
  if (!head) return { status: 'unknown', recorded: { baseHead, treeDigest }, current: null, reason: 'the current checkout is not readable through git' };
  const dirty = (git('status', '--porcelain', '--', '.', ':(exclude).kbd-orchestrator') ?? '') !== '';
  const current = { baseHead: head, treeDigest: git('rev-parse', 'HEAD^{tree}'), uncommittedChanges: dirty };
  const recordedInfo = { baseHead, treeDigest };
  if (baseHead && baseHead !== head) return { status: 'stale', recorded: recordedInfo, current, reason: 'the recorded base commit is not the current commit' };
  if (treeDigest && current.treeDigest && treeDigest !== current.treeDigest) return { status: 'stale', recorded: recordedInfo, current, reason: 'the recorded tree is not the current committed tree' };
  if (dirty) return { status: 'unknown', recorded: recordedInfo, current, reason: 'the checkout has uncommitted changes outside .kbd-orchestrator, so the tree cannot be compared' };
  return { status: 'current', recorded: recordedInfo, current, reason: 'the recorded candidate is the current commit and tree' };
}
const candidate = candidateReport();
const venue = typeof evidence.venue === 'string' && evidence.venue.trim() ? evidence.venue.trim() : null;
const refsOf = scenario => {
  if (operatorExit) return null;
  const own = entries.filter(e => e.id === scenario.id || String(e.id).startsWith(`${scenario.id}-`));
  const pick = key => own.map(e => e[key]).find(value => typeof value === 'string' && value.trim()) ?? null;
  const refs = { scopeRef: pick('scopeRef'), fixtureRef: pick('fixtureRef'), oracleRef: pick('oracleRef') };
  if (!refs.scopeRef && !refs.fixtureRef && !refs.oracleRef) return null;
  return { ...refs, scopeRefInContract: refs.scopeRef ? (scenario.scopeRefs ?? []).includes(refs.scopeRef) : null };
};

const rows = scenarios.map(s => ({
  id: s.id,
  name: s.name,
  actor: s.actor ?? null,
  usefulOutcome: s.usefulOutcome ?? null,
  mandatory: s.mandatory ?? true,
  verdict: verdictOf(s.id),
  // What the recorded verdict says about the current candidate. A result for
  // another candidate was not exercised on this one; it is never a failure.
  currentCandidateVerdict: candidate.status === 'stale' ? 'NOT_EXERCISED' : verdictOf(s.id),
  evidenceRefs: refsOf(s),
  limitations: s.limitations ?? [],
  substitutedBoundaries: s.substitutedBoundaries ?? [],
  realCollaborators: s.realCollaborators ?? [],
}));
const mandatory = rows.filter(r => r.mandatory !== false);
const methodContext = readJson(path.join(outDir, 'method-context.json'));

// VLBV CR-33 passive scan (report only; never a guard and never verdict-changing):
// runner output under the contract's declared runnerOutputGlobs whose mtime is
// earlier than the recorded B4 authorization event.
function findProjectRoot(start) {
  let dir = path.resolve(start);
  while (dir !== path.dirname(dir) && !fs.existsSync(path.join(dir, '.kbd-orchestrator'))) dir = path.dirname(dir);
  return dir;
}
function globRegex(glob) {
  let out = '';
  for (let i = 0; i < glob.length; i += 1) {
    const ch = glob[i];
    if (ch === '*' && glob[i + 1] === '*') {
      if (glob[i + 2] === '/') { out += '(?:.*/)?'; i += 2; } else { out += '.*'; i += 1; }
    } else if (ch === '*') out += '[^/]*';
    else if (ch === '?') out += '[^/]';
    else out += ch.replace(/[.+^${}()|[\]\\]/g, '\\$&');
  }
  return new RegExp(`^${out}$`);
}
function walkFiles(root, base) {
  const start = path.join(root, base);
  if (!fs.existsSync(start)) return [];
  const found = [];
  const walk = dir => {
    for (const name of fs.readdirSync(dir)) {
      if (name === '.git' || name === 'node_modules') continue;
      const full = path.join(dir, name);
      const stat = fs.lstatSync(full);
      if (stat.isDirectory()) walk(full);
      else if (stat.isFile()) found.push({ rel: path.relative(root, full).split(path.sep).join('/'), mtimeMs: stat.mtimeMs });
    }
  };
  if (fs.lstatSync(start).isFile()) {
    found.push({ rel: base, mtimeMs: fs.lstatSync(start).mtimeMs });
    return found;
  }
  walk(start);
  return found;
}
function preAuthorizationScan() {
  const globs = contract.boundaryPacket?.runnerOutputGlobs ?? [];
  const authorization = readJson(path.join(outDir, 'boundary-authorization.json'));
  const event = (authorization?.events ?? []).find(e => e.boundary === 'B4');
  const authorizedAt = event?.authorizedAt ?? null;
  if (!globs.length) return { status: 'no-globs', authorizedAt, globs, scanned: 0, artifacts: [] };
  if (!authorizedAt || Number.isNaN(Date.parse(authorizedAt))) return { status: 'no-authorization', authorizedAt: null, globs, scanned: 0, artifacts: [] };
  const projectRoot = path.resolve(opt('--project-root') ?? findProjectRoot(outDir));
  const authMs = Date.parse(authorizedAt);
  const seen = new Map();
  for (const glob of globs) {
    const regex = globRegex(glob);
    const base = glob.split('/').filter((part, i, parts) => !/[*?]/.test(parts.slice(0, i + 1).join('/'))).join('/');
    for (const file of walkFiles(projectRoot, base)) if (regex.test(file.rel)) seen.set(file.rel, file.mtimeMs);
  }
  const artifacts = [...seen.entries()].filter(([, mtimeMs]) => mtimeMs < authMs).sort(([a], [b]) => a.localeCompare(b))
    .map(([file, mtimeMs]) => ({ path: file, mtime: new Date(mtimeMs).toISOString() }));
  return { status: artifacts.length ? 'found' : 'none', authorizedAt, globs, scanned: seen.size, artifacts };
}
const preAuthorization = preAuthorizationScan();
const statusComplete =
  contract.closure?.implementationComplete ?? contract.status?.implementationComplete ?? false;
const behaviorAccepted = disposition === 'CLOSED' && mandatory.every(r => r.verdict === 'PASS');
const closure = {
  schemaVersion: 1,
  kind: 'kbd.closure-view',
  generatedAt: new Date().toISOString(),
  authority:
    'derived view; the canonical disposition recorded by prometheus kbd phase close is authoritative',
  phaseId: contract.identity?.phaseId ?? evidence.phase ?? null,
  contractVersion: contract.contractVersion ?? null,
  contractDigest: operatorExit?.boundContractDigest ?? evidence.contractDigest ?? null,
  ...(operatorExit ? { operatorExit } : {}),
  evidenceDigest:
    evidencePath && fs.existsSync(evidencePath)
      ? crypto.createHash('sha256').update(fs.readFileSync(evidencePath)).digest('hex')
      : null,
  disposition,
  reason,
  states: {
    implementationComplete: Boolean(statusComplete),
    behaviorAccepted,
    releaseEligible: behaviorAccepted && rows.every(r => r.verdict === 'PASS'),
    releaseAuthorized: Boolean(contract.closure?.releaseAuthorized) && behaviorAccepted,
  },
  method: methodContext
    ? {
        name: methodContext.selectedMethod,
        source: methodContext.methodSource ?? null,
        deviation: methodContext.deviation ?? null,
      }
    : { name: contract.method?.name ?? 'VLBV', source: 'contract', deviation: null },
  executedBy: evidence.executedBy ?? null,
  candidate,
  venue,
  preAuthorizationArtifacts: preAuthorization,
  scenarios: rows,
};
fs.mkdirSync(outDir, { recursive: true });
fs.writeFileSync(path.join(outDir, 'closure.json'), JSON.stringify(closure, null, 2) + '\n');

const list = (items, empty = 'none') =>
  items.length ? items.map(i => `- ${i}`).join('\n') : `- ${empty}`;
const works = rows.filter(r => r.verdict === 'PASS');
const remains = rows.filter(r => r.verdict !== 'PASS');
const claim = contract.releaseClaim ?? {};
const md = `# Closure packet — ${closure.phaseId ?? 'phase'}

Disposition: **${disposition}**${reason ? ` (${reason})` : ''}. This packet is a derived view of
the bound contract (${closure.contractDigest ? `sha256:${closure.contractDigest}` : 'digest unknown'}) and
\`boundary-results.json\`${closure.evidenceDigest ? ` (sha256:${closure.evidenceDigest})` : ''}; the canonical KBD disposition is authoritative.

## What works, and for whom

${candidate.status === 'stale'
  ? `- nothing is established for the current candidate: ${works.length} recorded PASS result(s) belong to an earlier candidate (${candidate.reason})`
  : list(
      works.map(r => `${r.id} — ${r.usefulOutcome ?? r.name}${r.actor ? ` (for ${r.actor})` : ''}`),
      'no scenario passed'
    )}

## Under which conditions

${list([
  ...(claim.supportedEnvironments ?? []).map(e => `environment: ${e}`),
  ...rows.flatMap(r => r.realCollaborators.map(c => `${r.id} real collaborator: ${c}`)),
  ...rows.flatMap(r =>
    r.substitutedBoundaries.map(b => `${r.id} SUBSTITUTED boundary (not a real claim): ${b}`)
  ),
])}

## Limits

${list([...rows.flatMap(r => r.limitations.map(l => `${r.id}: ${l}`)), ...(claim.explicitUnknowns ?? []).map(u => `unknown: ${u}`)])}

## What remains

${list(
  remains.map(r => `${r.id} — ${r.verdict}${r.mandatory === false ? ' (not mandatory)' : ''}`),
  'nothing: every scenario passed'
)}

## Evidence references

${[
  candidate.status === 'not-recorded'
    ? '- candidate: not recorded in boundary-results.json'
    : `- candidate: ${candidate.status.toUpperCase()} — ${candidate.reason} (recorded base ${candidate.recorded.baseHead ?? 'not recorded'}, tree ${candidate.recorded.treeDigest ?? 'not recorded'}${candidate.current ? `; current base ${candidate.current.baseHead}, tree ${candidate.current.treeDigest}` : ''})`,
  `- venue: ${venue ?? 'not recorded'}`,
  ...rows.filter(r => r.evidenceRefs).map(r =>
    `- ${r.id}: recorded ${r.verdict}${candidate.status === 'stale' ? ' on an earlier candidate; NOT_EXERCISED on the current one' : ''}; scope ${r.evidenceRefs.scopeRef ?? 'not recorded'}${r.evidenceRefs.scopeRefInContract === false ? ' (not a scope reference of this scenario in the contract)' : ''}; fixture ${r.evidenceRefs.fixtureRef ?? 'not recorded'}; oracle ${r.evidenceRefs.oracleRef ?? 'not recorded'}`),
].join('\n')}

References are reported as recorded. They change no verdict and no disposition.

## Pre-authorization runner artifacts

${(() => {
  const p = preAuthorization;
  if (p.status === 'no-globs') return '- no runnerOutputGlobs declared in the contract; scan not possible';
  if (p.status === 'no-authorization') return '- no B4 authorization event recorded (boundary-authorization.json); scan not possible';
  if (p.status === 'none') return `- none found (${p.scanned} runner artifact(s) under ${p.globs.join(', ')}, all modified at or after the B4 authorization at ${p.authorizedAt})`;
  return p.artifacts.map(a => `- ${a.path} (modified ${a.mtime}, before the B4 authorization at ${p.authorizedAt})`).join('\n');
})()}

This scan is passive evidence (VLBV 04, 10): it reports and never blocks.

## Closure states

| State | Value |
|---|---|
| Implementation complete | ${closure.states.implementationComplete} |
| Behavior accepted | ${closure.states.behaviorAccepted} |
| Release eligible | ${closure.states.releaseEligible} |
| Release authorized | ${closure.states.releaseAuthorized} (a separate owner decision) |

Method: ${closure.method.name}${closure.method.deviation ? ` — **deviation** to ${closure.method.deviation.method}: cadence "${closure.method.deviation.cadence}", reason "${closure.method.deviation.reason}" (decision ${closure.method.deviation.decisionId})` : ''}.
Executed by: ${closure.executedBy ? `${closure.executedBy.harness ?? '?'} ${closure.executedBy.version ?? ''} (${closure.executedBy.model ?? 'model unknown'})` : 'not recorded in boundary-results.json'}.
`;
if (operatorExit) {
  const { operatorDecision, budgetAccounting, exercised, withdrawn, limits } = operatorExit;
  const signed = [
    `# Closure packet — ${closure.phaseId}`, '', `Disposition: **${disposition}**`,
    `Bound contract: ${operatorExit.boundContractDigest}`,
    '', '## Operator decision', operatorDecision.reference, operatorDecision.text,
    '', '## Exercised', list(exercised.map(item =>
      `${item.id} — ${item.verdict}; evidence: ${item.evidence.map(file => `${file.path} (sha256:${file.sha256})`).join(', ')}`)),
    '', '## Withdrawn', list(withdrawn.map(item => `${item.kind} ${item.id} → ${item.destination}`)),
    '', '## Budget accounting', JSON.stringify(budgetAccounting),
    '', '## Limits', list(limits),
    '', 'Withdrawn work carries no passing verdict. Release authorization requires its own evidence.', '',
  ];
  fs.writeFileSync(path.join(outDir, 'closure-packet.md'), signed.join('\n'));
} else {
  fs.writeFileSync(path.join(outDir, 'closure-packet.md'), md);
}
