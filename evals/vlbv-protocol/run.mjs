#!/usr/bin/env node
// run.mjs — VLBV protocol evals per harness (vlbv-r4-portability CR-31).
//
// For each harness: every agent case runs N samples (declared before results),
// each in a fresh scratch project and scratch home outside the repository with
// only the minimum auth material copied in. Transcripts and file effects are
// retained under evals/runs/ (gitignored local evidence). Deterministic checks
// read file effects; a cross-family grader judges the rest. Machinery verdicts
// come from evals/reports/machinery.json. Real harness state and the
// harness-related keychain items are hashed before and after.
//
// Usage:
//   node evals/vlbv-protocol/run.mjs --harness <id[,id]|all> [--samples 3]
//   node evals/vlbv-protocol/run.mjs --stub exit0 --samples 3           (EVAL-NOTRUN-1 negative)
//   [--report-dir <dir>] [--concurrency 3] [--timeout-min 10]
//   [--keep-scratch --expect-leftover]                                  (AUTH-ISO-1 negative)
// Exit: 0 reports written and valid with isolation intact; 1 a report failed
// validation; 3 isolation violated (later harnesses are not started);
// with --expect-leftover, 0 only if the leftover check reported the kept auth file.
import fs from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { adapterFor } from './adapters/index.mjs';
import { buildProject } from './lib/fixture.mjs';
import { gradeSample, graderFor } from './lib/grade.mjs';
import { caseVerdict, machineryCases, renderMarkdown, tally, validateReport, deterministicChecks } from './lib/report.mjs';
import {
  INSTALLED_HARNESSES, REPORTS_DIR, RUNS_DIR, ROOT, casesByClass, gitCommit, hashPath, loadCorpus,
  parseArgs, passesRequired, readJson, run, sha256, stamp, writeJson, writeText,
} from './lib/common.mjs';
import { compareManifests, createScratch, destroyScratch, realStateManifest, repoAuthFiles } from './lib/isolation.mjs';

// Interruption safety (B4 attempt 1, defect 3): an interrupted run must not leave
// harness sessions running or copied auth material in scratch roots.
const ACTIVE_SCRATCH = new Set();
const ACTIVE_CHILDREN = new Set();
for (const signal of ['SIGINT', 'SIGTERM', 'SIGHUP']) {
  process.on(signal, () => {
    for (const child of ACTIVE_CHILDREN) { try { child.kill('SIGKILL'); } catch { /* already gone */ } }
    for (const scratch of ACTIVE_SCRATCH) fs.rmSync(scratch.root, { recursive: true, force: true });
    process.stderr.write(`run: ${signal}: stopped ${ACTIVE_CHILDREN.size} harness session(s), removed ${ACTIVE_SCRATCH.size} scratch root(s)\n`);
    process.exit(signal === 'SIGINT' ? 130 : 143);
  });
}

/** Spawn without a shell; resolves with exit/stdout/stderr, killing on timeout. */
function spawnAsync(exe, args, { cwd, env, timeoutMs }) {
  return new Promise(resolve => {
    const started = Date.now();
    let stdout = '', stderr = '', timedOut = false, settled = false;
    let child;
    try {
      child = spawn(exe, args, { cwd, env, stdio: ['ignore', 'pipe', 'pipe'] });
    } catch (error) {
      resolve({ exit: 127, stdout: '', stderr: String(error), timedOut: false, durationMs: 0 });
      return;
    }
    ACTIVE_CHILDREN.add(child);
    const timer = setTimeout(() => { timedOut = true; child.kill('SIGTERM'); setTimeout(() => child.kill('SIGKILL'), 5_000); }, timeoutMs);
    child.stdout.on('data', chunk => { stdout += chunk; });
    child.stderr.on('data', chunk => { stderr += chunk; });
    const finish = (code, extra = '') => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      ACTIVE_CHILDREN.delete(child);
      resolve({ exit: code ?? 1, stdout, stderr: stderr + extra, timedOut, durationMs: Date.now() - started });
    };
    child.on('error', error => finish(127, String(error)));
    child.on('close', code => finish(code));
  });
}

/** Raw observations for a case's deterministic checks. */
function observe(project, def, phase) {
  const d = def.deterministic ?? {};
  const at = rel => path.join(project, rel);
  if (phase === 'before') return { unchanged: Object.fromEntries((d.unchanged ?? []).map(p => [p, hashPath(at(p))])) };
  return {
    unchanged: Object.fromEntries((d.unchanged ?? []).map(p => [p, hashPath(at(p))])),
    present: Object.fromEntries((d.present ?? []).map(p => [p, fs.existsSync(at(p))])),
    absent: Object.fromEntries((d.absent ?? []).map(p => [p, fs.existsSync(at(p))])),
  };
}

/** Run one sample end to end; returns the sample record (ungraded). */
async function runSample(adapter, def, n, runDir, options) {
  const scratch = createScratch(`${adapter.id}-${def.id}-s${n}`);
  ACTIVE_SCRATCH.add(scratch);
  const sampleDir = path.join(runDir, def.id, `s${n}`);
  let leftovers = [];
  try {
    const { project } = buildProject(scratch.project, def);
    const { env, copied } = adapter.prepare(scratch);
    if (options.keepScratch) {
      // AUTH-ISO-1 negative control: an auth-named, non-secret probe proves the
      // leftover check finds kept auth material even for harnesses (Claude Code)
      // whose real login lives in the keychain and copies no file.
      writeText(path.join(scratch.home, '.eval-leftover-probe', 'auth.json'), '{"probe": "not a credential"}\n');
    }
    const before = observe(project, def, 'before');
    const binary = adapter.resolveBinary();
    const argv = adapter.argv(def.prompt, project);
    const out = await spawnAsync(binary, argv, { cwd: project, env, timeoutMs: options.timeoutMs });
    const after = observe(project, def, 'after');
    const effects = {
      case: def.id, sample: n, harness: adapter.id,
      unchanged: Object.fromEntries(Object.entries(before.unchanged).map(([p, h]) => [p, { before: h, after: after.unchanged[p] }])),
      present: after.present, absent: after.absent,
      copiedAuthFiles: copied.map(p => path.basename(p)),
    };
    const checks = deterministicChecks(effects);
    const deterministicOk = checks.every(c => c.ok);
    const transcript = [
      `# harness: ${adapter.id}`,
      `# argv: ${[path.basename(binary ?? '?'), ...argv.map(a => (a === def.prompt ? '<case prompt>' : a.replace(project, '<project>')))].join(' ')}`,
      `# exit: ${out.exit}${out.timedOut ? ' (timed out)' : ''} after ${out.durationMs} ms`,
      '## stdout', out.stdout, '## stderr', out.stderr,
    ].join('\n');
    writeText(path.join(sampleDir, 'transcript.txt'), transcript);
    writeJson(path.join(sampleDir, 'effects.json'), { ...effects, checks });
    const text = adapter.text(out.stdout);
    // A forbidden effect (absent/unchanged violated) is a behaviour FAIL even if the
    // harness then errored: the agent did it. A missing REQUIRED effect (present)
    // only counts as FAIL when the harness actually ran; if it never ran
    // (error exit, timeout, no output) the sample is BLOCKED, not FAIL.
    const forbidden = checks.filter(c => !c.ok && c.kind !== 'present');
    const missing = checks.filter(c => !c.ok && c.kind === 'present');
    const harnessRan = !out.timedOut && out.exit === 0 && Boolean(String(text).trim());
    const notRan = out.timedOut ? 'harness timed out'
      : !String(text).trim() ? 'no transcript (harness produced no output)'
        : out.exit !== 0 ? `harness exited ${out.exit}: ${(out.stderr || out.stdout).trim().slice(0, 200)}` : '';
    let verdict = null, reason = '';
    if (forbidden.length) { verdict = 'FAIL'; reason = `deterministic check failed: ${forbidden.map(c => `${c.kind} ${c.path}`).join(', ')}`; }
    else if (!harnessRan) { verdict = 'BLOCKED'; reason = missing.length ? `${notRan} (required effect not produced: ${missing.map(c => c.path).join(', ')})` : notRan; }
    else if (missing.length) { verdict = 'FAIL'; reason = `deterministic check failed: ${missing.map(c => `${c.kind} ${c.path}`).join(', ')}`; }
    return {
      n, verdict, reason, deterministicOk, exit: out.exit, timedOut: out.timedOut, durationMs: out.durationMs,
      transcript: { path: path.relative(ROOT, path.join(sampleDir, 'transcript.txt')), sha256: sha256(transcript) },
      effects: path.relative(ROOT, path.join(sampleDir, 'effects.json')),
      text,
    };
  } catch (error) {
    return { n, verdict: 'BLOCKED', reason: `runner error: ${String(error?.message ?? error).slice(0, 300)}`, deterministicOk: null, text: '' };
  } finally {
    ACTIVE_SCRATCH.delete(scratch);
    leftovers = destroyScratch(scratch, { keep: options.keepScratch });
    options.leftovers.push(...leftovers);
    if (options.keepScratch) options.keptRoots.push(scratch.root);
  }
}

/** Run every agent case for one harness and build its report. */
async function runHarness(adapter, corpus, machineryReport, options) {
  const { agent: allAgent, machinery } = casesByClass(corpus);
  // The leftover negative control needs one session, not the whole corpus.
  const agent = options.firstCaseOnly ? allAgent.slice(0, 1) : allAgent;
  const family = adapter.family();
  const grader = graderFor(family);
  const runDir = path.join(options.runsDir, adapter.id);
  fs.rmSync(runDir, { recursive: true, force: true });
  const before = realStateManifest();
  const runCase = async def => {
    const samples = [];
    for (let start = 1; start <= options.samples; start += options.concurrency) {
      const batch = [];
      for (let n = start; n < start + options.concurrency && n <= options.samples; n += 1) batch.push(runSample(adapter, def, n, runDir, options));
      samples.push(...await Promise.all(batch));
    }
    await Promise.all(samples.map(async s => {
      if (s.verdict === null) {
        const g = await gradeSample({ testCase: def, transcript: s.text, harnessFamily: family });
        s.grade = g;
        s.verdict = g.verdict;
        s.reason = g.reason;
      } else s.grade = null;
      delete s.text;
    }));
    const required = passesRequired(corpus.decisionRule, def);
    const decided = caseVerdict(samples, required, options.samples);
    process.stdout.write(`  ${adapter.id} ${def.id}: ${decided.verdict} (${decided.reason})\n`);
    return { id: def.id, at: def.at, requirement: def.requirement, class: 'agent', capabilityFields: def.capabilityFields ?? [],
      passesRequired: required, ...decided, samples };
  };
  // Cases run in batches of options.caseConcurrency (wall-clock; each sample keeps
  // its own scratch root, so batches do not share state). Corpus order is kept.
  const cases = [];
  for (let i = 0; i < agent.length; i += options.caseConcurrency) {
    cases.push(...await Promise.all(agent.slice(i, i + options.caseConcurrency).map(runCase)));
  }
  cases.push(...machineryCases(machinery, machineryReport));
  const after = realStateManifest();
  const diffs = compareManifests(before, after);
  const repo = repoAuthFiles();
  const isolation = {
    ok: diffs.length === 0 && options.leftovers.length === 0 && repo.length === 0,
    realStateUnchanged: !diffs.some(d => !d.includes('keychain')),
    keychainUnchanged: !diffs.some(d => d.includes('keychain')),
    diffs, leftovers: options.leftovers.map(f => f.replace(/^.*vlbv-eval-/, '<scratch>/vlbv-eval-')), repoAuthFiles: repo,
  };
  const version = run(adapter.resolveBinary(), ['--version'], { timeoutMs: 20_000 });
  return {
    schemaVersion: 1, kind: 'vlbv.protocol-eval-report', harness: adapter.id, status: 'run',
    generatedAt: stamp(), commit: gitCommit(), harnessVersion: version.stdout.trim().split('\n')[0] || 'unknown',
    harnessModel: adapter.model(), harnessFamily: family, grader, decisionRule: corpus.decisionRule,
    samplesDeclared: options.samples, counts: tally(cases), isolation, cases,
  };
}

function notRunReport(id, reason, corpus, machineryReport, samples) {
  const { agent, machinery } = casesByClass(corpus);
  const cases = [
    ...agent.map(def => ({ id: def.id, at: def.at, requirement: def.requirement, class: 'agent', capabilityFields: def.capabilityFields ?? [],
      verdict: 'NOT_EXERCISED', reason, samples: [] })),
    ...machineryCases(machinery, machineryReport),
  ];
  return { schemaVersion: 1, kind: 'vlbv.protocol-eval-report', harness: id, status: 'not-run', reason,
    generatedAt: stamp(), commit: gitCommit(), decisionRule: corpus.decisionRule, samplesDeclared: samples, counts: tally(cases), cases };
}

async function main() {
  const args = parseArgs(process.argv.slice(2), ['keep-scratch', 'expect-leftover']);
  const corpus = loadCorpus();
  const samples = Number(args.samples ?? corpus.decisionRule.samples);
  if (!Number.isInteger(samples) || samples < 1) throw new Error('--samples must be a positive integer');
  const reportDir = path.resolve(args['report-dir'] ?? REPORTS_DIR);
  const isNegativeDir = reportDir !== path.resolve(REPORTS_DIR);
  const options = {
    samples,
    concurrency: Math.max(1, Number(args.concurrency ?? 3)),
    caseConcurrency: Math.max(1, Number(args['case-concurrency'] ?? process.env.VLBV_EVAL_CASE_CONCURRENCY ?? 4)),
    timeoutMs: Math.max(1, Number(args['timeout-min'] ?? 10)) * 60_000,
    keepScratch: Boolean(args['keep-scratch']),
    firstCaseOnly: Boolean(args['expect-leftover']),
    runsDir: isNegativeDir ? path.join(reportDir, 'runs') : RUNS_DIR,
    leftovers: [],
    keptRoots: [],
  };
  const machineryFile = path.join(REPORTS_DIR, 'machinery.json');
  const machineryReport = fs.existsSync(machineryFile) ? readJson(machineryFile) : null;
  const harnesses = args.stub ? ['stub-exit0']
    : String(args.harness ?? '').split(',').flatMap(h => (h === 'all' ? INSTALLED_HARNESSES : [h.trim()])).filter(Boolean);
  if (!harnesses.length) throw new Error('--harness <id[,id]|all> or --stub exit0 is required');

  let exitCode = 0;
  for (const id of harnesses) {
    options.leftovers = [];
    const adapter = adapterFor(id, { stub: args.stub });
    const availability = adapter.available();
    process.stdout.write(`${id}: ${availability.ok ? 'running' : `not run — ${availability.reason}`}\n`);
    const report = availability.ok
      ? await runHarness(adapter, corpus, machineryReport, options)
      : notRunReport(id, availability.reason, corpus, machineryReport, samples);
    writeJson(path.join(reportDir, `${id}.json`), report);
    writeText(path.join(reportDir, `${id}.md`), renderMarkdown(report));
    // The one-case leftover control is not a full report; it is checked for isolation only.
    const problems = options.firstCaseOnly ? [] : validateReport(report, { corpus, machineryReport });
    process.stdout.write(`${id}: pass ${report.counts.pass}, fail ${report.counts.fail}, blocked ${report.counts.blocked}, not exercised ${report.counts.notExercised}; report ${problems.length ? 'INVALID' : 'valid'}\n`);
    for (const p of problems) process.stdout.write(`  invalid: ${p}\n`);
    if (problems.length) exitCode = Math.max(exitCode, 1);
    if (args.stub) {
      // EVAL-NOTRUN-1 negative: a no-output harness must yield BLOCKED (no transcript), never PASS.
      const agentCases = report.cases.filter(c => c.class === 'agent');
      const ok = agentCases.every(c => c.verdict === 'BLOCKED') && agentCases.every(c => c.samples.every(s => /no transcript/.test(s.reason)));
      process.stdout.write(ok ? 'stub harness: every agent case BLOCKED with no transcript, zero PASS (expected)\n' : 'stub harness produced a non-BLOCKED verdict\n');
      return ok && !problems.length ? 0 : 1;
    }
    if (args['expect-leftover']) {
      const detected = report.isolation && !report.isolation.ok && report.isolation.leftovers.length > 0;
      process.stdout.write(detected ? 'expected leftover detected by the post-run check\n' : 'leftover NOT detected\n');
      // The control is over: remove the kept scratch roots so nothing persists (SAFE-1).
      for (const root of options.keptRoots) fs.rmSync(root, { recursive: true, force: true });
      return detected ? 0 : 1;
    }
    if (report.isolation && !report.isolation.ok) {
      process.stdout.write(`${id}: ISOLATION VIOLATED (${[...report.isolation.diffs, ...report.isolation.leftovers, ...report.isolation.repoAuthFiles].join('; ')}); stopping before any further harness\n`);
      return 3;
    }
  }
  return exitCode;
}

main().then(code => { process.exitCode = code; }, error => {
  process.stderr.write(`run: ${error?.stack ?? error}\n`);
  process.exitCode = 2;
});
