// Per-harness protocol eval reports: decision rule, deterministic re-check,
// rendering and validation (vlbv-r4-portability T3).
import fs from 'node:fs';
import path from 'node:path';
import { ROOT, passesRequired, sha256 } from './common.mjs';

export const VERDICTS = ['PASS', 'FAIL', 'BLOCKED', 'NOT_EXERCISED'];

/**
 * Recompute deterministic checks from the raw observations in an effects record.
 * @param {{unchanged?: Record<string,{before:string,after:string}>, present?: Record<string,boolean>, absent?: Record<string,boolean>}} effects
 * @returns {{kind: string, path: string, ok: boolean}[]}
 */
export function deterministicChecks(effects) {
  const checks = [];
  for (const [file, h] of Object.entries(effects.unchanged ?? {})) checks.push({ kind: 'unchanged', path: file, ok: h.before === h.after });
  for (const [file, exists] of Object.entries(effects.present ?? {})) checks.push({ kind: 'present', path: file, ok: exists === true });
  for (const [file, exists] of Object.entries(effects.absent ?? {})) checks.push({ kind: 'absent', path: file, ok: exists === false });
  return checks;
}

/** Case verdict from sample verdicts under the declared k-of-n rule. */
export function caseVerdict(samples, required, declared) {
  const pass = samples.filter(s => s.verdict === 'PASS').length;
  const fail = samples.filter(s => s.verdict === 'FAIL').length;
  if (pass >= required) return { verdict: 'PASS', reason: `${pass} of ${declared} samples passed (needs ${required})` };
  if (fail > declared - required) return { verdict: 'FAIL', reason: `${fail} of ${declared} samples failed; at most ${declared - required} allowed` };
  return { verdict: 'BLOCKED', reason: `${pass} pass, ${fail} fail, ${samples.length - pass - fail} blocked of ${declared}; the rule cannot be decided` };
}

/** Tally case verdicts. */
export function tally(cases) {
  const counts = Object.fromEntries(VERDICTS.map(v => [v, 0]));
  for (const c of cases) counts[c.verdict] = (counts[c.verdict] ?? 0) + 1;
  return { pass: counts.PASS, fail: counts.FAIL, blocked: counts.BLOCKED, notExercised: counts.NOT_EXERCISED, notRun: counts.BLOCKED + counts.NOT_EXERCISED };
}

/** Machinery verdicts carried into a harness report as harness-independent. */
export function machineryCases(machineryCorpus, machineryReport) {
  return machineryCorpus.map(c => {
    const found = machineryReport?.cases?.find(m => m.id === c.id && m.mode === 'trigger');
    return {
      id: c.id, at: c.at, requirement: c.requirement, class: 'machinery', harnessIndependent: true,
      source: 'evals/reports/machinery.json', samples: [],
      verdict: found ? found.verdict : 'BLOCKED',
      reason: found ? found.observed : 'machinery report missing (run evals/vlbv-protocol/machinery.mjs first)',
    };
  });
}

const rel = file => path.relative(ROOT, file);
const resolve = file => (path.isAbsolute(file) ? file : path.join(ROOT, file));

/**
 * Validate one harness report. Returns a list of problems (empty = valid).
 * @param {any} report @param {{corpus: any, machineryReport?: any}} context
 */
export function validateReport(report, { corpus, machineryReport }) {
  const problems = [];
  if (report?.kind !== 'vlbv.protocol-eval-report') return [`unexpected kind ${report?.kind}`];
  const expectedIds = corpus.cases.map(c => c.id).sort();
  const ids = (report.cases ?? []).map(c => c.id).sort();
  if (JSON.stringify(ids) !== JSON.stringify(expectedIds)) problems.push(`cases ${ids.length}/${expectedIds.length}: ${ids.join(',')} != ${expectedIds.join(',')}`);
  const declared = report.samplesDeclared;
  for (const c of report.cases ?? []) {
    const def = corpus.cases.find(x => x.id === c.id);
    if (!def) continue;
    if (!VERDICTS.includes(c.verdict)) problems.push(`${c.id}: verdict ${c.verdict} is not in the status vocabulary`);
    if (def.class === 'machinery') {
      if (c.harnessIndependent !== true) problems.push(`${c.id}: machinery case not marked harness-independent`);
      const m = machineryReport?.cases?.find(x => x.id === c.id && x.mode === 'trigger');
      if (m && m.verdict !== c.verdict) problems.push(`${c.id}: verdict ${c.verdict} differs from machinery report ${m.verdict}`);
      continue;
    }
    if (report.status === 'not-run') {
      if (c.verdict !== 'NOT_EXERCISED' || !c.reason) problems.push(`${c.id}: not-run harness must record NOT_EXERCISED with a reason`);
      if (c.samples?.length) problems.push(`${c.id}: not-run harness carries samples`);
      continue;
    }
    const samples = c.samples ?? [];
    if (samples.length !== declared && !(['BLOCKED', 'NOT_EXERCISED'].includes(c.verdict) && c.reason)) {
      problems.push(`${c.id}: ${samples.length} samples recorded, ${declared} declared, and no BLOCKED/NOT_EXERCISED reason`);
    }
    for (const s of samples) {
      const tag = `${c.id} sample ${s.n}`;
      if (s.verdict === 'PASS' || s.verdict === 'FAIL') {
        const t = s.transcript ? resolve(s.transcript.path) : null;
        if (!t || !fs.existsSync(t)) problems.push(`${tag}: ${s.verdict} cites transcript ${s.transcript?.path ?? '(none)'} that does not exist`);
        else if (sha256(fs.readFileSync(t)) !== s.transcript.sha256) problems.push(`${tag}: transcript digest mismatch`);
      }
      if (s.effects) {
        const e = resolve(s.effects);
        if (!fs.existsSync(e)) problems.push(`${tag}: effects record ${s.effects} missing`);
        else {
          const ok = deterministicChecks(JSON.parse(fs.readFileSync(e, 'utf8'))).every(x => x.ok);
          if (ok !== s.deterministicOk) problems.push(`${tag}: recorded deterministicOk=${s.deterministicOk} but retained effects recompute to ${ok}`);
          if (s.verdict === 'PASS' && !ok) problems.push(`${tag}: PASS contradicted by its deterministic check (retained effects show a forbidden effect)`);
        }
      } else if (s.verdict === 'PASS') problems.push(`${tag}: PASS without an effects record`);
      if (s.grade && s.grade.grader?.family === report.harnessFamily && report.harnessFamily !== 'unknown') {
        problems.push(`${tag}: grader family ${s.grade.grader.family} equals the harness family`);
      }
    }
    if (samples.length === declared) {
      const expected = caseVerdict(samples, passesRequired(corpus.decisionRule, def), declared).verdict;
      if (expected !== c.verdict) problems.push(`${c.id}: verdict ${c.verdict} but the declared rule gives ${expected}`);
    }
  }
  const counts = tally(report.cases ?? []);
  for (const key of Object.keys(counts)) {
    if (report.counts?.[key] !== counts[key]) problems.push(`counts.${key}=${report.counts?.[key]} but the cases tally ${counts[key]}`);
  }
  return problems;
}

/** Markdown view of a report. */
export function renderMarkdown(report) {
  const esc = s => String(s ?? '').replace(/\|/g, '\\|').replace(/\n/g, ' ').slice(0, 180);
  const header = `# VLBV protocol eval — ${report.harness}\n\n` +
    `Generated ${report.generatedAt} at commit \`${report.commit}\`. Status: **${report.status}**${report.reason ? ` (${report.reason})` : ''}.\n\n` +
    `Harness: ${report.harnessVersion ?? 'n/a'}; model: ${report.harnessModel ?? 'n/a'} (family ${report.harnessFamily ?? 'n/a'}). ` +
    `Grader: ${report.grader?.model ?? 'n/a'}. Samples declared: ${report.samplesDeclared}; decision rule: ${report.decisionRule?.samplePass ?? ''}\n\n` +
    `| Pass | Fail | Blocked | Not exercised |\n|---|---|---|---|\n| ${report.counts.pass} | ${report.counts.fail} | ${report.counts.blocked} | ${report.counts.notExercised} |\n\n`;
  const rows = report.cases.map(c => `| ${c.id} | ${c.class}${c.harnessIndependent ? ' (shared)' : ''} | ${c.verdict} | ${esc(c.reason)} |`);
  const iso = report.isolation
    ? `\n## Isolation (AUTH-ISO-1)\n\n- Real harness state unchanged: ${report.isolation.realStateUnchanged}\n- Keychain item listing unchanged: ${report.isolation.keychainUnchanged}\n- Copied auth files left behind: ${report.isolation.leftovers.length}\n- Auth files in the repository working tree: ${report.isolation.repoAuthFiles.length}\n${report.isolation.diffs.map(d => `- ${d}`).join('\n')}\n`
    : '';
  return `${header}| Case | Class | Verdict | Reason |\n|---|---|---|---|\n${rows.join('\n')}\n${iso}`;
}

export { rel };
