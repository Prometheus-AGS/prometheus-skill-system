#!/usr/bin/env node
// validate-report.mjs — validate a per-harness protocol eval report by
// re-deriving every verdict from the retained run artifacts
// (vlbv-r4-portability T3, oracle of EVAL-<HARNESS>-1).
//
// Usage:
//   node evals/vlbv-protocol/validate-report.mjs --harness <id> [--report <file>]
//   node evals/vlbv-protocol/validate-report.mjs --negative --harness <id>
//       Plants two defects into a copy of the harness report and expects BOTH to
//       be rejected: (a) a PASS citing a transcript that does not exist, and
//       (b) a PASS whose retained effects record a forbidden effect (the marker
//       file of a forbidden command was present after the run).
// Exit: 0 valid (or, with --negative, both planted defects rejected);
//       1 invalid (or a planted defect accepted); 2 report missing or unusable.
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { validateReport } from './lib/report.mjs';
import { REPORTS_DIR, ROOT, loadCorpus, parseArgs, readJson, writeJson } from './lib/common.mjs';

const cryptoDigest = text => crypto.createHash('sha256').update(text).digest('hex');

function loadMachinery() {
  const file = path.join(REPORTS_DIR, 'machinery.json');
  return fs.existsSync(file) ? readJson(file) : null;
}

/** Copy a report with one agent sample replaced by a planted PASS. */
function plant(report, mutate) {
  const copy = JSON.parse(JSON.stringify(report));
  const target = copy.cases.find(c => c.class === 'agent' && c.samples?.length);
  if (!target) return null;
  target.samples[0] = mutate(target, target.samples[0]);
  return copy;
}

function negative(report, corpus, machineryReport, harness) {
  const plantedDir = path.join(REPORTS_DIR, '_negative', 'planted', harness);
  fs.mkdirSync(plantedDir, { recursive: true });
  const results = [];

  const a = plant(report, (c, s) => ({ ...s, verdict: 'PASS',
    transcript: { path: path.relative(ROOT, path.join(plantedDir, 'missing-transcript.txt')), sha256: '0'.repeat(64) } }));

  const b = plant(report, (c, s) => {
    const def = corpus.cases.find(x => x.id === c.id);
    const forbidden = def.deterministic?.absent?.[0] ?? '.eval-markers/test-ran';
    const transcriptFile = path.join(plantedDir, 'forbidden-transcript.txt');
    const effectsFile = path.join(plantedDir, 'forbidden-effects.json');
    const transcript = `# planted transcript\n$ npm test\n(${forbidden} written)\n`;
    fs.writeFileSync(transcriptFile, transcript);
    writeJson(effectsFile, { case: c.id, sample: s.n, unchanged: {}, present: {}, absent: { [forbidden]: true } });
    // The planted sample claims deterministicOk so only the re-derivation can catch it.
    return { ...s, verdict: 'PASS', deterministicOk: true,
      transcript: { path: path.relative(ROOT, transcriptFile), sha256: cryptoDigest(transcript) },
      effects: path.relative(ROOT, effectsFile) };
  });

  for (const [label, planted, expect] of [
    ['a: PASS citing a missing transcript', a, /does not exist/],
    ['b: PASS contradicted by retained effects', b, /contradicted by its deterministic check|recompute to false/],
  ]) {
    if (!planted) { results.push({ label, ok: false, detail: 'no agent sample to plant into (run the harness first)' }); continue; }
    const problems = validateReport(planted, { corpus, machineryReport });
    const caught = problems.some(p => expect.test(p));
    results.push({ label, ok: caught, detail: caught ? problems.find(p => expect.test(p)) : `not rejected; problems: ${problems.join('; ') || 'none'}` });
  }
  return results;
}

function main() {
  const args = parseArgs(process.argv.slice(2), ['negative']);
  if (!args.harness) { process.stderr.write('validate-report: --harness <id> is required\n'); return 2; }
  const file = path.resolve(args.report ?? path.join(REPORTS_DIR, `${args.harness}.json`));
  if (!fs.existsSync(file)) { process.stderr.write(`validate-report: ${file} not found\n`); return 2; }
  const corpus = loadCorpus();
  const machineryReport = loadMachinery();
  const report = readJson(file);
  if (args.negative) {
    const results = negative(report, corpus, machineryReport, args.harness);
    for (const r of results) process.stdout.write(`${r.ok ? 'REJECTED (expected)' : 'NOT REJECTED'} ${r.label}: ${r.detail}\n`);
    return results.every(r => r.ok) ? 0 : 1;
  }
  const problems = validateReport(report, { corpus, machineryReport });
  for (const p of problems) process.stdout.write(`invalid: ${p}\n`);
  process.stdout.write(problems.length ? `${args.harness}: report INVALID (${problems.length})\n` : `${args.harness}: report valid\n`);
  return problems.length ? 1 : 0;
}

process.exitCode = main();
