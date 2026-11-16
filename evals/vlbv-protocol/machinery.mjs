#!/usr/bin/env node
// machinery.mjs — the six harness-independent VLBV acceptance tests
// (AT 05, 06, 09, 10, 15, 22), run once against the real KBD scripts
// (vlbv-r4-portability T4, scenario EVAL-MACH-1).
//
// Each AT builds an isolated project with the shared phase-closure conformance
// adapter (real `prometheus` binary, signed journal, isolated HOME/XDG/data),
// adopts its contract through kbd-contract-adopt.sh and runs
// kbd-phase-close-eval.sh. Nothing is mocked.
//
// Usage:
//   node evals/vlbv-protocol/machinery.mjs                     triggers -> evals/reports/machinery.{json,md}
//   node evals/vlbv-protocol/machinery.mjs --negative --at AT06   that AT with its trigger removed;
//                                                              the real script must ACCEPT
//   [--report-dir <dir>]  [--keep-scratch]
// Exit: 0 every AT behaved as required; 1 any FAIL; 2 any BLOCKED (setup or tool missing).
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fixture, step, surface } from '../../scripts/tests/lib/kbd-phase-closure-adapter.mjs';
import { scanPaths } from './scan-secrets.mjs';
import { ROOT, REPORTS_DIR, gitCommit, loadCorpus, parseArgs, run, stamp, which, writeJson, writeText } from './lib/common.mjs';

const BOTH_PASS = { 'CS-1': 'PASS', 'CS-2': 'PASS' };
const ACCEPT = { exit: 0, stdout: /^CLOSED /m };

/** Per-AT trigger and negative definitions over the conformance adapter's ops. */
const SCENARIOS = {
  AT05: {
    trigger: { steps: [], expect: { exit: 2, stdout: /^BLOCKED .*evidence-missing/m } },
    negative: { steps: [{ op: 'evidence', args: { statuses: BOTH_PASS } }], expect: ACCEPT },
  },
  AT06: {
    trigger: { steps: [{ op: 'evidence', args: { statuses: BOTH_PASS, blank: { 'CS-1': ['observed'] } } }],
      expect: { exit: 2, stdout: /^BLOCKED .*empty-evidence:CS-1/m } },
    negative: { steps: [{ op: 'evidence', args: { statuses: BOTH_PASS } }], expect: ACCEPT },
  },
  AT09: {
    trigger: { steps: [{ op: 'evidence', args: { statuses: BOTH_PASS, run: 'prefix' } }],
      expect: { exit: 2, stdout: /^BLOCKED .*evidence-run-mismatch/m } },
    negative: { steps: [{ op: 'evidence', args: { statuses: BOTH_PASS } }], expect: ACCEPT },
  },
  AT10: {
    trigger: { steps: [{ op: 'evidence', args: { statuses: { 'CS-1': 'PASS', 'CS-2': 'NOT_EXERCISED' } } }],
      expect: { exit: 2, stdout: /^BLOCKED .*not-pass:CS-2\(NOT_EXERCISED\)/m, packet: /## What remains\s+[\s\S]*CS-2 — NOT_EXERCISED/ } },
    negative: { steps: [{ op: 'evidence', args: { statuses: BOTH_PASS } }], expect: ACCEPT },
  },
  AT15: {
    trigger: { steps: [{ op: 'evidence', args: { statuses: BOTH_PASS } }, { op: 'edit-source' }],
      expect: { exit: 2, stdout: /^BLOCKED .*evidence-stale-sources\(stale\(src\.txt\)\)/m } },
    negative: { steps: [{ op: 'evidence', args: { statuses: BOTH_PASS } }], expect: ACCEPT },
  },
};

const SECRET_LINE = '{"access_token": "' + 'Zq7'.repeat(12) + '"}';

/** AT 22 runs the production secret scanner on a candidate artifact. */
function at22(scratch, negative) {
  const file = path.join(scratch, 'AT22', 'candidate-report.json');
  writeText(file, negative ? '{"summary": "all scenarios passed"}\n' : `${SECRET_LINE}\n`);
  const findings = scanPaths([file]);
  const refused = findings.length > 0;
  const ok = negative ? !refused : refused;
  return {
    verdict: ok ? 'PASS' : 'FAIL',
    observed: negative ? `scanner accepted the clean artifact: ${!refused}` : `scanner refused the secret artifact: ${refused} (${findings.map(f => f.pattern).join(',') || 'no findings'})`,
    command: ['node', 'evals/vlbv-protocol/scan-secrets.mjs', file],
  };
}

/** Run one adapter-driven AT (trigger or negative). */
async function runAt(id, scratch, prometheusBin, negative) {
  const def = SCENARIOS[id][negative ? 'negative' : 'trigger'];
  const observed = [];
  const ctx = fixture(scratch, `${id}${negative ? '-neg' : ''}`, prometheusBin, { observe: entry => observed.push(entry) });
  ctx.surface = surface('source', ROOT, { sourceLayout: true });
  const adopt = await step(ctx, { op: 'adopt' });
  if (adopt.exit !== 0) return { verdict: 'BLOCKED', observed: `fixture adoption failed: ${(adopt.stdout + adopt.stderr).slice(0, 400)}` };
  for (const s of def.steps) await step(ctx, s);
  const result = await step(ctx, { op: 'close-eval', args: {} });
  const packetFile = path.join(ctx.phaseDir, 'closure-packet.md');
  const packet = fs.existsSync(packetFile) ? fs.readFileSync(packetFile, 'utf8') : '';
  const problems = [];
  if (result.exit !== def.expect.exit) problems.push(`exit ${result.exit}, expected ${def.expect.exit}`);
  if (!def.expect.stdout.test(result.stdout)) problems.push(`stdout did not match ${def.expect.stdout}`);
  if (def.expect.packet && !def.expect.packet.test(packet)) problems.push(`closure packet did not match ${def.expect.packet}`);
  return {
    verdict: problems.length ? 'FAIL' : 'PASS',
    observed: `exit=${result.exit} stdout=${result.stdout.trim().slice(0, 300)}${problems.length ? ` | ${problems.join('; ')}` : ''}`,
    command: ['bash', 'shared/scripts/kbd-phase-close-eval.sh', '--phase', ctx.phase],
    runtimeCalls: observed.length,
  };
}

function renderMarkdown(report) {
  const rows = report.cases.map(c => `| ${c.id} | ${c.requirement} | ${c.verdict} | ${String(c.observed).replace(/\|/g, '\\|').slice(0, 160)} |`);
  return `# VLBV machinery cases${report.negative ? ' (negative controls)' : ''}\n\nGenerated ${report.generatedAt} at commit \`${report.commit}\` with ${report.prometheus}.\n` +
    `These cases are harness-independent: they run once and their verdicts are carried into every harness report.\n\n` +
    `| Case | Requirement | Verdict | Observed |\n|---|---|---|---|\n${rows.join('\n')}\n`;
}

async function main() {
  const args = parseArgs(process.argv.slice(2), ['negative', 'keep-scratch']);
  const corpus = loadCorpus();
  const machinery = corpus.cases.filter(c => c.class === 'machinery');
  const selected = args.at ? machinery.filter(c => c.id === args.at) : machinery;
  if (!selected.length) throw new Error(`no machinery case ${args.at}`);
  const negative = Boolean(args.negative);
  const reportDir = path.resolve(args['report-dir'] ?? (negative ? path.join(REPORTS_DIR, '_negative') : REPORTS_DIR));
  const prometheusBin = process.env.PROMETHEUS_CLI_TEST_BINARY || which('prometheus');
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'vlbv-machinery-'));
  const version = prometheusBin ? run(prometheusBin, ['--version'], { timeoutMs: 10_000 }).stdout.trim() : 'prometheus not found';

  const cases = [];
  try {
    for (const c of selected) {
      let outcome;
      try {
        if (c.id === 'AT22') outcome = at22(scratch, negative);
        else if (!prometheusBin) outcome = { verdict: 'BLOCKED', observed: 'prometheus binary not found on PATH (set PROMETHEUS_CLI_TEST_BINARY)' };
        else outcome = await runAt(c.id, scratch, prometheusBin, negative);
      } catch (error) {
        outcome = { verdict: 'BLOCKED', observed: `setup error: ${String(error?.message ?? error).slice(0, 400)}` };
      }
      cases.push({ id: c.id, at: c.at, requirement: c.requirement, class: 'machinery', mode: negative ? 'negative' : 'trigger',
        expected: negative ? c.negative : c.expected, ...outcome });
    }
  } finally {
    if (!args['keep-scratch']) fs.rmSync(scratch, { recursive: true, force: true });
  }

  const report = { schemaVersion: 1, kind: 'vlbv.machinery-report', negative, generatedAt: stamp(), commit: gitCommit(),
    prometheus: version, scratchRetained: Boolean(args['keep-scratch']), cases };
  const base = negative ? `machinery-${args.at ?? 'all'}` : 'machinery';
  writeJson(path.join(reportDir, `${base}.json`), report);
  writeText(path.join(reportDir, `${base}.md`), renderMarkdown(report));
  for (const c of cases) process.stdout.write(`${c.verdict} ${c.id}${negative ? ' (negative)' : ''}: ${c.observed}\n`);
  if (cases.some(c => c.verdict === 'BLOCKED')) return 2;
  return cases.every(c => c.verdict === 'PASS') ? 0 : 1;
}

main().then(code => { process.exitCode = code; }, error => {
  process.stderr.write(`machinery: ${error?.stack ?? error}\n`);
  process.exitCode = 2;
});
