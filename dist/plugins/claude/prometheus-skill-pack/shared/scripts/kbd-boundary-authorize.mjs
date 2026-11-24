#!/usr/bin/env node
// kbd-boundary-authorize.mjs — record the immutable boundary authorization
// event for a KBD value phase (VLBV CR-33, vlbv-r4-portability T6).
//
// The owner authorizes B4 (or a B5 repair) once the agreed implementation is
// complete. This writes an append-only event into
// .kbd-orchestrator/phases/<id>/boundary-authorization.json BEFORE the first
// attempt, so closure can tell runner output produced before authorization
// (passive evidence of early execution; report only, no guard) from output of
// the authorized boundary run. Boundaries: references/boundaries.md.
//
// Usage:
//   node kbd-boundary-authorize.mjs --phase <id> --authorizer <who> [--boundary B4|B5]
//        [--reason <text>] [--project <dir>]
// Ordering invariant: contract adopted (phase bound) < implementation complete
// < B4 authorized <= first attempt. A B4 event is refused once boundary-results.json
// exists; a B5 event needs a prior B4 event and an existing boundary-results.json.
// Exit: 0 recorded or ALREADY-AUTHORIZED; 1 usage/io; 2 phase not bound;
//       3 ordering violated.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const argv = process.argv.slice(2);
const opt = name => { const i = argv.indexOf(name); return i >= 0 ? argv[i + 1] : undefined; };
const fail = (code, message) => { process.stderr.write(`kbd-boundary-authorize: ${message}\n`); process.exit(code); };

const phase = opt('--phase');
const authorizer = opt('--authorizer');
const boundary = opt('--boundary') ?? 'B4';
const reason = opt('--reason') ?? '';
if (!phase) fail(1, '--phase <registered id> is required');
if (!authorizer) fail(1, '--authorizer <who> is required');
if (!['B4', 'B5'].includes(boundary)) fail(1, '--boundary must be B4 or B5');

let root = path.resolve(opt('--project') ?? process.cwd());
while (root !== path.dirname(root) && !fs.existsSync(path.join(root, '.kbd-orchestrator'))) root = path.dirname(root);
if (!fs.existsSync(path.join(root, '.kbd-orchestrator'))) fail(1, 'no .kbd-orchestrator above the working directory');
const status = spawnSync('prometheus', ['kbd', '--path', root, 'status', '--json'],
  {encoding: 'utf8', timeout: 60_000, maxBuffer: 64 * 1024 * 1024});
if (status.status !== 0) fail(2, 'cannot read canonical phase status');
const helper = fileURLToPath(new URL('./lib/kbd-phase-directory.mjs', import.meta.url));
const directory = spawnSync(process.execPath, [helper, root, phase], {input: status.stdout, encoding: 'utf8'});
if (directory.status !== 0) fail(2, directory.stderr?.trim() || 'cannot resolve registered phase directory');
const phaseDir = directory.stdout.trimEnd();
if (!fs.statSync(phaseDir, {throwIfNoEntry: false})?.isDirectory()) fail(1, `phase directory ${phaseDir} does not exist`);

const disposition = spawnSync('prometheus', ['kbd', '--path', root, 'phase', 'disposition', '--id', phase, '--json'], { encoding: 'utf8', timeout: 60_000 });
let digest = null;
try { digest = JSON.parse(disposition.stdout)?.binding?.contractDigest ?? null; } catch { digest = null; }
if (!digest) fail(2, `phase ${phase} has no bound contract (adopt it with kbd-contract-adopt.sh before authorizing a boundary)`);

const file = path.join(phaseDir, 'boundary-authorization.json');
const record = fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, 'utf8')) : { schemaVersion: 1, kind: 'kbd.boundary-authorization', phaseId: phase, events: [] };
if (record.kind !== 'kbd.boundary-authorization' || record.phaseId !== phase || !Array.isArray(record.events)) fail(1, `${file} is not a boundary authorization record for ${phase}`);
const resultsExist = fs.existsSync(path.join(phaseDir, 'boundary-results.json'));
const priorB4 = record.events.find(e => e.boundary === 'B4');

if (boundary === 'B4') {
  if (priorB4) {
    process.stdout.write(`ALREADY-AUTHORIZED phase=${phase} boundary=B4 at=${priorB4.authorizedAt} contract=${priorB4.contractDigest}\n`);
    process.exit(0);
  }
  if (resultsExist) fail(3, `boundary-results.json already exists for ${phase}; B4 must be authorized before the first attempt`);
} else {
  if (!priorB4) fail(3, 'B5 needs a prior B4 authorization');
  if (!resultsExist) fail(3, 'B5 repairs a recorded boundary attempt; boundary-results.json is missing');
}

// Millisecond precision: the closure scan compares runner-output mtimes against this instant.
const event = { boundary, authorizedAt: new Date().toISOString(), authorizer, contractDigest: digest, ...(reason ? { reason } : {}) };
const next = { ...record, events: [...record.events, event] };
const tmp = `${file}.tmp-${process.pid}`;
fs.writeFileSync(tmp, `${JSON.stringify(next, null, 2)}\n`);
fs.renameSync(tmp, file);
process.stdout.write(`AUTHORIZED phase=${phase} boundary=${boundary} at=${event.authorizedAt} contract=${digest}\n`);
