#!/usr/bin/env node
// apply-capabilities.mjs — promote harness capability fields to "supported"
// only from PASS evidence (vlbv-r4-portability T5, CAP-UPDATE-1, FR-H7).
//
// A field moves to supported only when a report case of the same harness
// declares that field in its corpus capabilityFields AND its case verdict is
// PASS. FAIL, BLOCKED and NOT_EXERCISED are no evidence; such fields are left
// exactly as they are. After writing, the harness adapters are regenerated
// (constraint C-01); `generate-harness-adapters.js --check` then proves the
// regeneration is idempotent.
//
// Usage:
//   node evals/vlbv-protocol/apply-capabilities.mjs --reports evals/reports
//   node evals/vlbv-protocol/apply-capabilities.mjs --negative [--report-dir evals/reports/_negative]
//       Writes three defective copies of capabilities.json and expects
//       validate:harness-adapters to reject each: (a) supported with no
//       evidence, (b) supported citing a PASS case of a different capability,
//       (c) supported citing a FAIL case of the right capability.
// Exit: 0 applied (or all three copies rejected); 1 otherwise.
import fs from 'node:fs';
import path from 'node:path';
import { INSTALLED_HARNESSES, REPORTS_DIR, ROOT, loadCorpus, parseArgs, readJson, run, stamp, writeJson } from './lib/common.mjs';

const CAPABILITIES = path.join(ROOT, 'shared/harnesses/capabilities.json');
const VALIDATOR = path.join(ROOT, 'scripts/check-harness-adapters.js');
const GENERATOR = path.join(ROOT, 'scripts/generate-harness-adapters.js');

function promote(capabilities, corpus, reportsDir) {
  const next = JSON.parse(JSON.stringify(capabilities));
  const changes = [];
  for (const harness of INSTALLED_HARNESSES) {
    const file = path.join(reportsDir, `${harness}.json`);
    if (!fs.existsSync(file)) continue;
    const report = readJson(file);
    if (report.status !== 'run' || report.harness !== harness) continue;
    const profile = next.profiles?.[harness];
    if (!profile) continue;
    let promoted = false;
    for (const c of report.cases) {
      const def = corpus.cases.find(x => x.id === c.id);
      for (const field of def?.capabilityFields ?? []) {
        if (c.verdict !== 'PASS') { changes.push(`${harness}.${field}: left unchanged (${c.id} ${c.verdict})`); continue; }
        if (!profile[field] || typeof profile[field] !== 'object') { changes.push(`${harness}.${field}: no such profile field`); continue; }
        profile[field] = {
          ...profile[field], status: 'supported',
          evidence: { report: path.relative(ROOT, file), caseId: c.id, date: report.generatedAt, version: report.harnessVersion },
        };
        promoted = true;
        changes.push(`${harness}.${field}: supported (${c.id} PASS)`);
      }
    }
    if (promoted) {
      profile.verified = { ...profile.verified, date: stamp().slice(0, 10), version: report.harnessVersion,
        exercised: true, evidence: path.relative(ROOT, file) };
    }
  }
  return { next, changes };
}

function negative(reportDir) {
  const base = readJson(CAPABILITIES);
  const plantedReport = path.join(reportDir, 'planted', 'capabilities', 'claude-code.json');
  // A minimal planted report: S18 (declares rulesFiles) PASS, AT19 (declares canExecuteBoundary) FAIL.
  writeJson(plantedReport, { kind: 'vlbv.protocol-eval-report', harness: 'claude-code', status: 'run', generatedAt: stamp(),
    harnessVersion: 'planted', cases: [{ id: 'S18', verdict: 'PASS' }, { id: 'AT19', verdict: 'FAIL' }] });
  const reportRel = path.relative(ROOT, plantedReport);
  const variants = {
    a: { field: 'invocation', evidence: undefined, why: 'supported with no evidence' },
    b: { field: 'canExecuteBoundary', evidence: { report: reportRel, caseId: 'S18' }, why: 'cites a PASS case of a different capability' },
    c: { field: 'canExecuteBoundary', evidence: { report: reportRel, caseId: 'AT19' }, why: 'cites a FAIL case of the right capability' },
  };
  const results = [];
  for (const [label, v] of Object.entries(variants)) {
    const copy = JSON.parse(JSON.stringify(base));
    const field = copy.profiles['claude-code'][v.field];
    copy.profiles['claude-code'][v.field] = { ...field, status: 'supported', ...(v.evidence ? { evidence: v.evidence } : {}) };
    if (!v.evidence) delete copy.profiles['claude-code'][v.field].evidence;
    const file = path.join(reportDir, `capabilities-${label}.json`);
    writeJson(file, copy);
    const out = run(process.execPath, [VALIDATOR, '--capabilities', file], { cwd: ROOT, timeoutMs: 120_000 });
    const text = `${out.stdout}\n${out.stderr}`;
    const rejected = out.exit !== 0 && text.includes(`claude-code: ${v.field}`);
    results.push({ label, why: v.why, rejected, detail: rejected ? text.split('\n').find(l => l.includes(`claude-code: ${v.field}`)) : `exit ${out.exit}: ${text.trim().slice(0, 300)}` });
  }
  return results;
}

function main() {
  const args = parseArgs(process.argv.slice(2), ['negative']);
  if (args.negative) {
    const results = negative(path.resolve(args['report-dir'] ?? path.join(REPORTS_DIR, '_negative')));
    for (const r of results) process.stdout.write(`${r.rejected ? 'REJECTED (expected)' : 'NOT REJECTED'} (${r.label}) ${r.why}: ${r.detail}\n`);
    return results.every(r => r.rejected) ? 0 : 1;
  }
  const reportsDir = path.resolve(args.reports ?? REPORTS_DIR);
  const corpus = loadCorpus();
  const { next, changes } = promote(readJson(CAPABILITIES), corpus, reportsDir);
  writeJson(CAPABILITIES, next);
  // The file is prettier-formatted (check-format); reformat so the diff carries only the promotions.
  const prettier = path.join(ROOT, 'node_modules/.bin/prettier');
  if (fs.existsSync(prettier)) {
    const formatted = run(prettier, ['--write', CAPABILITIES], { cwd: ROOT, timeoutMs: 60_000 });
    if (formatted.exit !== 0) process.stdout.write(`prettier failed (exit ${formatted.exit}); capabilities.json left unformatted\n`);
  } else process.stdout.write('prettier not installed; capabilities.json left unformatted (run npm run format)\n');
  for (const c of changes) process.stdout.write(`${c}\n`);
  const generated = run(process.execPath, [GENERATOR], { cwd: ROOT, timeoutMs: 300_000 });
  process.stdout.write(`generate-harness-adapters: exit ${generated.exit}\n`);
  if (generated.exit !== 0) process.stdout.write(`${generated.stdout}${generated.stderr}\n`);
  return generated.exit === 0 ? 0 : 1;
}

process.exitCode = main();
