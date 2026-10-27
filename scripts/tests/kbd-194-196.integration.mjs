#!/usr/bin/env node
// Final-boundary integration driver for phase issues-194-195-196.
//
// Every scenario drives a production entry point (the built `prometheus` CLI, the
// shipped kbd-apply.sh, the generator/installer dispatch, local harness sessions)
// against a real signed journal in isolated HOME/XDG/data roots. Nothing is mocked.
// This file is acceptance infrastructure, not a replacement runner: it records the
// original commands and observations, then states PASS / FAIL / BLOCKED /
// NOT_EXERCISED per scenario. A missing prerequisite never becomes a pass.
//
//   node scripts/tests/kbd-194-196.integration.mjs --contract <adopted-contract.json> \
//     --prometheus <built prometheus> --evidence-root <dir> [--only ID,ID] [--seeder <bin>]
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { scenarioModules } from './lib/kbd-194-196-scenarios.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const LIB = path.join(path.dirname(fileURLToPath(import.meta.url)), 'lib');
const argv = process.argv.slice(2);
const opt = (name, fallback) => { const i = argv.indexOf(`--${name}`); return i >= 0 ? argv[i + 1] : fallback; };
const required = (name) => {
  const value = opt(name);
  if (!value) { console.error(`usage error: --${name} is required`); process.exit(2); }
  return path.resolve(value);
};
const contractFile = required('contract');
const prometheus = required('prometheus');
const evidenceRoot = required('evidence-root');
const seeder = path.resolve(opt('seeder', path.join(ROOT, 'substrate/kbd-runtime/target/debug/examples/seed_damaged_identity')));
const only = opt('only') ? new Set(opt('only').split(',')) : null;

for (const [label, file] of [['contract', contractFile], ['prometheus', prometheus], ['seeder', seeder]]) {
  if (!fs.existsSync(file) || !fs.statSync(file).isFile()) { console.error(`BLOCKED prerequisite missing: ${label} ${file}`); process.exit(2); }
}
const contract = JSON.parse(fs.readFileSync(contractFile, 'utf8'));
const modules = new Map(scenarioModules);
const declaredIds = contract.scenarioMatrix.map((s) => s.id);
for (const id of only ?? []) {
  if (!declaredIds.includes(id)) { console.error(`usage error: --only ${id} is not a contract scenario`); process.exit(2); }
}
const sha256 = (v) => crypto.createHash('sha256').update(v).digest('hex');
const fileSha = (f) => sha256(fs.readFileSync(f));
const git = (...args) => spawnSync('git', ['-C', ROOT, ...args], { encoding: 'utf8' }).stdout ?? '';
fs.mkdirSync(evidenceRoot, { recursive: true });

/** Identity frozen before any scenario runs: the dirty candidate tree, the binaries and every file holding an assertion. */
const productionEntries = [
  'skills/process/kbd-process-orchestrator/skills/kbd-apply/kbd-apply.sh',
  'skills/process/kbd-process-orchestrator/skills/kbd-apply/reconcile.mjs',
  'skills/process/kbd-process-orchestrator/shared/method-context.mjs',
  'substrate/kbd-runtime/src/change_identity.rs',
];
const identity = {
  startedAt: new Date().toISOString(),
  baseHead: git('rev-parse', 'HEAD').trim(),
  candidateTree: { trackedDiffSha256: sha256(git('diff', 'HEAD', '--', '.', ':!.kbd-orchestrator', ':!dist', ':!.agents')),
    untrackedSha256: sha256(git('ls-files', '--others', '--exclude-standard', '--', 'substrate', 'skills', 'scripts', 'docs', 'tools')) },
  productionEntries: Object.fromEntries(productionEntries.map((f) => [f, fileSha(path.join(ROOT, f))])),
  contractFileSha256: fileSha(contractFile),
  prometheusBinarySha256: fileSha(prometheus),
  seederBinarySha256: fileSha(seeder),
  driverSha256: fileSha(fileURLToPath(import.meta.url)),
  scenarioLibSha256: Object.fromEntries(fs.readdirSync(LIB).filter((f) => f.startsWith('kbd-194-196-')).sort().map((f) => [f, fileSha(path.join(LIB, f))])),
  node: process.version, platform: `${os.platform()} ${os.release()}`,
};
fs.writeFileSync(path.join(evidenceRoot, 'identity.json'), `${JSON.stringify(identity, null, 2)}\n`);

const results = [];
// The adopted contract, not the driver's own registry, defines what must be reported.
for (const declared of contract.scenarioMatrix) {
  const id = declared.id;
  if (only && !only.has(id)) continue;
  const scenario = modules.get(id);
  if (!scenario) { results.push({ id, status: 'NOT_EXERCISED', attempted: 'none', observed: 'the adopted contract declares this scenario but the driver has no module for it', evidence: [], limitations: [] }); continue; }
  const dir = path.join(evidenceRoot, id);
  fs.mkdirSync(dir, { recursive: true });
  const started = Date.now();
  let outcome;
  try {
    outcome = await scenario({ ROOT, prometheus, seeder, evidenceDir: dir, declared });
  } catch (error) {
    // An unclassified exception is not evidence of a pass; it stays a failure until triaged.
    outcome = { status: 'FAIL', attempted: 'scenario execution', observed: `unclassified driver or product exception, triage required: ${error.stack ?? error}`, assertions: [], commands: [] };
  }
  const assertions = outcome.assertions ?? [];
  const failed = assertions.filter((a) => !a.ok);
  const status = outcome.status ?? (failed.length ? 'FAIL' : assertions.length ? 'PASS' : 'NOT_EXERCISED');
  fs.writeFileSync(path.join(dir, 'assertions.json'), `${JSON.stringify(assertions, null, 2)}\n`);
  fs.writeFileSync(path.join(dir, 'commands.json'), `${JSON.stringify(outcome.commands ?? [], null, 2)}\n`);
  for (const [name, text] of Object.entries(outcome.artifacts ?? {})) fs.writeFileSync(path.join(dir, name), text);
  const result = {
    id, status,
    attempted: outcome.attempted,
    observed: outcome.observed ?? (failed.length ? `unmet: ${failed.map((a) => a.name).join('; ')}` : assertions.length ? `${assertions.length} assertions observed` : 'no assertion was made'),
    evidence: [`${id}/assertions.json`, `${id}/commands.json`, ...Object.keys(outcome.artifacts ?? {}).map((n) => `${id}/${n}`)],
    limitations: outcome.limitations ?? [], elapsedMs: Date.now() - started,
  };
  results.push(result);
  console.log(`${status.padEnd(13)} ${id}  ${result.observed.slice(0, 160)}`);
}
fs.writeFileSync(path.join(evidenceRoot, only ? `results-${[...only].join('_')}.json` : 'results.json'),
  `${JSON.stringify({ identity, scenarios: results }, null, 2)}\n`);
process.exit(results.some((r) => r.status !== 'PASS') ? 1 : 0);
