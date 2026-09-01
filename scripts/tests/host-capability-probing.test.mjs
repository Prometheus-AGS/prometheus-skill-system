/**
 * Fixtures for the `host-capability-probing` specification.
 *
 * Every assertion here runs against a REAL filesystem in a REAL store root.
 * Nothing is stubbed: the link ladder actually creates a junction where a
 * junction is possible, actually writes a copy where it is not, and the
 * degradation record reports what actually happened.
 */

import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import {
  loadCapabilities,
  materializeLink,
  probeFilesystemCapabilities,
} from '../lib/capabilities.js';
import { jcs } from '../lib/jcs.js';
import { collectPayloadEntries } from '../lib/payload-manifest.js';

const results = [];
const skipped = [];
function check(name, body) {
  body();
  results.push(name);
}
function skip(name, why) {
  skipped.push(`${name} -- ${why}`);
}

const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-capability-'));
const digest = value => crypto.createHash('sha256').update(jcs(value)).digest('hex');

// ---------------------------------------------------------------------------
// Requirement: filesystem primitives are probed, never inferred
// ---------------------------------------------------------------------------

const storeA = path.join(workspace, 'store-a', 'generations');
const storeB = path.join(workspace, 'store-b', 'generations');

// Scenario: the probe runs in the store root, not in the probe default.
//
// Inferring from `os.tmpdir()` is the classic version of this bug: on Windows
// %TEMP% is nearly always on C:, so a store on another volume would inherit an
// answer measured somewhere else entirely.
check('probe executes inside the store root', () => {
  const tmpBefore = fs.readdirSync(os.tmpdir()).length;
  const record = probeFilesystemCapabilities(storeA);
  const relative = path.relative(path.resolve(storeA), record.probeRoot);
  assert.ok(
    relative && !relative.startsWith('..') && !path.isAbsolute(relative),
    `probe root ${record.probeRoot} is not inside the store root ${storeA}`
  );
  assert.equal(record.storeRoot, path.resolve(storeA));
  assert.equal(fs.existsSync(record.probeRoot), false, 'probe artifacts must be removed');
  assert.equal(
    fs.readdirSync(os.tmpdir()).length,
    tmpBefore,
    'the probe must not create entries in the temporary directory'
  );
});

check('each store root gets its own record', () => {
  const a = probeFilesystemCapabilities(storeA);
  const b = probeFilesystemCapabilities(storeB);
  assert.equal(a.storeRoot, path.resolve(storeA));
  assert.equal(b.storeRoot, path.resolve(storeB));
  assert.notEqual(a.probeRoot, b.probeRoot);
});

check('an unwritable store root is reported, not silently degraded', () => {
  const file = path.join(workspace, 'not-a-directory');
  fs.writeFileSync(file, 'x');
  assert.throws(
    () => probeFilesystemCapabilities(path.join(file, 'generations')),
    error => error.code === 'STORE_ROOT_UNWRITABLE' || error.code === 'ENOTDIR'
  );
});

// Scenario: the executable bit is unsupported.
const probe = probeFilesystemCapabilities(storeA);
if (probe.executableBit) {
  check('a host that reports an executable bit records why', () => {
    assert.equal(probe.unsupported.executableBit, undefined);
    const file = path.join(storeA, 'exec-probe');
    fs.writeFileSync(file, 'x');
    fs.chmodSync(file, 0o755);
    assert.notEqual(fs.statSync(file).mode & 0o100, 0);
  });
} else {
  check('an absent executable bit is recorded with its observed reason', () => {
    assert.ok(
      probe.unsupported.executableBit,
      'an absent executable bit must record the observation that produced it'
    );
    // Confirm the probe measured rather than assumed: chmod really does not move.
    const file = path.join(storeA, 'exec-probe');
    fs.writeFileSync(file, 'x');
    fs.chmodSync(file, 0o755);
    assert.equal(fs.statSync(file).mode & 0o100, 0);
  });
}

// Scenario: with no executable bit, the MANIFEST is the authority.
check('the manifest, not the filesystem, carries the executable bit', () => {
  const tree = path.join(workspace, 'authority');
  fs.mkdirSync(tree, { recursive: true });
  fs.writeFileSync(path.join(tree, 'run.sh'), '#!/bin/sh\n');
  const capabilities = { ...probe, executableBit: false };
  const [entry] = collectPayloadEntries(
    tree,
    () => ({ type: 'file', executable: true }),
    capabilities
  );
  assert.equal(entry.executable, true, 'recorded intent must survive a host that cannot store it');
});

// ---------------------------------------------------------------------------
// Requirement: probe results are cached and invalidated
// ---------------------------------------------------------------------------

check('a cached record is reused and an installer upgrade discards it', () => {
  const cacheFile = path.join(workspace, 'cache', 'capabilities.json');
  const first = loadCapabilities({ storeRoot: storeA, installerVersion: 'v-old', cacheFile });
  assert.equal(first.source, 'probe');
  const second = loadCapabilities({ storeRoot: storeA, installerVersion: 'v-old', cacheFile });
  assert.equal(second.source, 'cache', 'an unchanged installer must reuse the cached record');

  const upgraded = loadCapabilities({ storeRoot: storeA, installerVersion: 'v-new', cacheFile });
  assert.equal(upgraded.source, 'probe', 'an installer upgrade must discard the cached record');
  assert.equal(JSON.parse(fs.readFileSync(cacheFile, 'utf8')).installerVersion, 'v-new');

  const moved = loadCapabilities({ storeRoot: storeB, installerVersion: 'v-new', cacheFile });
  assert.equal(moved.source, 'probe', 'a different store root must re-probe');
});

check('a truncated cache is re-probed rather than trusted', () => {
  const cacheFile = path.join(workspace, 'cache-partial', 'capabilities.json');
  fs.mkdirSync(path.dirname(cacheFile), { recursive: true });
  fs.writeFileSync(
    cacheFile,
    JSON.stringify({ schemaVersion: 1, installerVersion: 'v1', storeRoot: path.resolve(storeA) })
  );
  const record = loadCapabilities({ storeRoot: storeA, installerVersion: 'v1', cacheFile });
  assert.equal(record.source, 'probe', 'a record missing probed fields must not be reused');
});

// ---------------------------------------------------------------------------
// Requirement: degradation is recorded out of band
// ---------------------------------------------------------------------------

/** Build the same two-entry payload with a given capability record. */
function materializePayload(name, capabilities) {
  const root = path.join(workspace, name);
  fs.mkdirSync(path.join(root, 'target'), { recursive: true });
  fs.writeFileSync(path.join(root, 'target', 'inner.txt'), 'inner\n');
  const degradations = [];
  const outcome = materializeLink({
    linkPath: path.join(root, 'alias'),
    target: 'target',
    capabilities,
  });
  if (outcome.realized !== 'symlink') {
    degradations.push({
      path: 'alias',
      intended: outcome.intended,
      realized: outcome.realized,
      reason: outcome.reason ?? null,
    });
  }
  const intentOf = relative => {
    if (relative === 'alias') return { type: 'symlink', target: 'target' };
    if (relative === 'target') return { type: 'directory' };
    return { type: 'file', executable: false };
  };
  const entries = collectPayloadEntries(root, intentOf, capabilities);
  return { root, outcome, degradations, entries };
}

const noLinks = { ...probe, symlinkFile: false, symlinkDirectory: false, junction: false };
const copied = materializePayload('degraded', noLinks);

check('a link with no primitive available is written as a copy', () => {
  assert.equal(copied.outcome.realized, 'copy');
  assert.deepEqual(copied.degradations, [
    { path: 'alias', intended: 'symlink', realized: 'copy', reason: 'NO_DIRECTORY_SYMLINK' },
  ]);
  assert.equal(fs.lstatSync(path.join(copied.root, 'alias')).isFile(), true);
  assert.equal(fs.readFileSync(path.join(copied.root, 'alias'), 'utf8'), 'target');
});

check('the link entry still hashes as a link over its recorded target', () => {
  const alias = copied.entries.find(entry => entry.path === 'alias');
  assert.equal(alias.type, 'symlink');
  assert.equal(alias.target, 'target');
  assert.equal(alias.sha256, crypto.createHash('sha256').update('target').digest('hex'));
  assert.equal(alias.size, Buffer.byteLength('target'));
});

if (probe.symlinkDirectory || probe.junction) {
  const linked = materializePayload('linked', probe);
  check('a real link and a degraded copy produce the same identity', () => {
    assert.notEqual(linked.outcome.realized, 'copy');
    assert.equal(linked.degradations.length, probe.symlinkDirectory ? 0 : 1);
    assert.equal(
      digest(linked.entries),
      digest(copied.entries),
      'the realization must not move the generation identity'
    );
    assert.equal(jcs(linked.entries), jcs(copied.entries));
  });

  check('the realized link satisfies the link assertion the runtime makes', () => {
    // Both assertions the runtime makes about an indirection survive a junction:
    // libuv sets S_IFLNK for ANY reparse point on lstat, so `isSymbolicLink()`
    // is true, and msys2-runtime reports a junction with a drive-letter
    // substitute name as a POSIX symlink so `[[ -L ]]` holds under git-bash.
    const alias = path.join(linked.root, 'alias');
    assert.equal(fs.lstatSync(alias).isSymbolicLink(), true);
    assert.equal(fs.readFileSync(path.join(alias, 'inner.txt'), 'utf8'), 'inner\n');
  });

  check('the ladder chose the strongest available primitive', () => {
    assert.equal(linked.outcome.realized, probe.symlinkDirectory ? 'symlink' : 'junction');
  });
} else {
  skip('real link vs degraded copy', 'this volume supports no directory link primitive at all');
}

fs.rmSync(workspace, { recursive: true, force: true });

process.stdout.write(`host-capability-probing: ${results.length} checks passed\n`);
for (const name of results) process.stdout.write(`  ok   ${name}\n`);
for (const note of skipped) process.stdout.write(`  SKIP ${note}\n`);
process.stdout.write(
  `  host: directoryLinkStrategy=${probe.directoryLinkStrategy} executableBit=${probe.executableBit} posixModes=${probe.posixModes}\n`
);
