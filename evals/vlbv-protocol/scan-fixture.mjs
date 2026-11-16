#!/usr/bin/env node
// scan-fixture.mjs — drive the passive pre-authorization artifact scan through
// the production entry points (vlbv-r4-portability T6, scenario SCAN-PRE-1).
//
// Builds an isolated KBD project with the shared phase-closure conformance
// adapter (real prometheus runtime), declares runnerOutputGlobs in the contract,
// adopts it, records the B4 authorization event with kbd-boundary-authorize.mjs,
// dates three runner artifacts around that instant, writes PASS evidence and
// runs kbd-phase-close-eval.sh. The closure packet must list exactly the
// artifacts modified before authorization.
//
// Usage: node evals/vlbv-protocol/scan-fixture.mjs --case mixed|all-after [--keep-scratch]
//   mixed      two artifacts before, one after   -> exactly the two earlier paths listed
//   all-after  every artifact after authorization -> "none found" (negative control)
// Exit: 0 expectation met; 1 not met; 2 BLOCKED (setup or tool missing).
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fixture, step, surface } from '../../scripts/tests/lib/kbd-phase-closure-adapter.mjs';
import { REPORTS_DIR, ROOT, parseArgs, readJson, run, stamp, which, writeJson } from './lib/common.mjs';

const ARTIFACTS = ['out/report-a.json', 'out/report-b.json', 'out/report-c.json'];

async function main() {
  const args = parseArgs(process.argv.slice(2), ['keep-scratch']);
  const mode = args.case;
  if (!['mixed', 'all-after'].includes(mode)) throw new Error('--case mixed|all-after is required');
  const prometheusBin = process.env.PROMETHEUS_CLI_TEST_BINARY || which('prometheus');
  const result = { schemaVersion: 1, kind: 'vlbv.scan-fixture-report', case: mode, generatedAt: stamp() };
  if (!prometheusBin) {
    writeJson(path.join(REPORTS_DIR, `scan-${mode}.json`), { ...result, verdict: 'BLOCKED', observed: 'prometheus binary not found' });
    process.stdout.write('BLOCKED prometheus binary not found\n');
    return 2;
  }
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'vlbv-scan-'));
  try {
    const ctx = fixture(scratch, `scan-${mode}`, prometheusBin);
    ctx.surface = surface('source', ROOT, { sourceLayout: true });
    const contractFile = path.join(ctx.phaseDir, 'adopted-contract.json');
    const contract = readJson(contractFile);
    contract.boundaryPacket = { ...contract.boundaryPacket, runnerOutputGlobs: ['out/**'] };
    writeJson(contractFile, contract);
    const adopt = await step(ctx, { op: 'adopt' });
    if (adopt.exit !== 0) throw new Error(`adoption failed: ${adopt.stdout}${adopt.stderr}`);

    const authorize = run(process.execPath, [path.join(ROOT, 'shared/scripts/kbd-boundary-authorize.mjs'), '--phase', ctx.phase,
      '--authorizer', 'fixture owner', '--project', ctx.project], { cwd: ctx.project, env: ctx.env, timeoutMs: 60_000 });
    if (authorize.exit !== 0) throw new Error(`authorization failed: ${authorize.stdout}${authorize.stderr}`);
    const authorizedAt = readJson(path.join(ctx.phaseDir, 'boundary-authorization.json')).events.find(e => e.boundary === 'B4').authorizedAt;
    const authSec = Date.parse(authorizedAt) / 1000;

    const offsets = mode === 'mixed' ? [-3600, -1800, 60] : [60, 120, 180];
    ARTIFACTS.forEach((rel, i) => {
      const file = path.join(ctx.project, rel);
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, `{"artifact": "${rel}"}\n`);
      fs.utimesSync(file, authSec + offsets[i], authSec + offsets[i]);
    });

    await step(ctx, { op: 'evidence', args: { statuses: { 'CS-1': 'PASS', 'CS-2': 'PASS' } } });
    const close = await step(ctx, { op: 'close-eval', args: {} });
    const closure = readJson(path.join(ctx.phaseDir, 'closure.json'));
    const packet = fs.readFileSync(path.join(ctx.phaseDir, 'closure-packet.md'), 'utf8');
    const scan = closure.preAuthorizationArtifacts ?? {};
    const listed = (scan.artifacts ?? []).map(a => a.path).sort();
    const expected = mode === 'mixed' ? ARTIFACTS.slice(0, 2) : [];
    const problems = [];
    if (close.exit !== 0) problems.push(`close-eval exit ${close.exit}: ${close.stdout.trim().slice(0, 200)}`);
    if (JSON.stringify(listed) !== JSON.stringify(expected)) problems.push(`listed ${JSON.stringify(listed)}, expected ${JSON.stringify(expected)}`);
    if (scan.status !== (mode === 'mixed' ? 'found' : 'none')) problems.push(`scan status ${scan.status}`);
    if (mode === 'all-after' && !/## Pre-authorization runner artifacts\s+- none found/.test(packet)) problems.push('closure packet does not state "none found"');
    if (mode === 'mixed' && !expected.every(p => packet.includes(`- ${p} (modified`))) problems.push('closure packet does not list the earlier artifacts');
    const verdict = problems.length ? 'FAIL' : 'PASS';
    writeJson(path.join(REPORTS_DIR, `scan-${mode}.json`), { ...result, verdict, authorizedAt, offsetsSeconds: offsets, listed, expected, scan, problems,
      command: ['bash', 'shared/scripts/kbd-phase-close-eval.sh', '--phase', ctx.phase] });
    process.stdout.write(`${verdict} scan ${mode}: listed ${JSON.stringify(listed)}${problems.length ? ` | ${problems.join('; ')}` : ''}\n`);
    return verdict === 'PASS' ? 0 : 1;
  } catch (error) {
    writeJson(path.join(REPORTS_DIR, `scan-${mode}.json`), { ...result, verdict: 'BLOCKED', observed: String(error?.message ?? error).slice(0, 400) });
    process.stdout.write(`BLOCKED scan ${mode}: ${String(error?.message ?? error).slice(0, 400)}\n`);
    return 2;
  } finally {
    if (!args['keep-scratch']) fs.rmSync(scratch, { recursive: true, force: true });
  }
}

main().then(code => { process.exitCode = code; }, error => {
  process.stderr.write(`scan-fixture: ${error?.stack ?? error}\n`);
  process.exitCode = 2;
});
