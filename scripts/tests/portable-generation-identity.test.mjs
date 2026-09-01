/**
 * Fixtures for the `portable-generation-identity` specification.
 *
 * The centrepiece is a GOLDEN DIGEST over a fixed payload tree. Cross-host
 * identity equality cannot be observed from one host, so it is asserted as a
 * constant here: every leg of the verification matrix runs this same file and
 * must reproduce the same value. A host that computes anything else fails, and
 * the failure names which host diverged.
 */

import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

import { probeFilesystemCapabilities } from '../lib/capabilities.js';
import {
  GOLDEN_DIGEST,
  GOLDEN_PAYLOAD,
  buildGoldenPayload,
  goldenIntentOf,
} from '../lib/golden-payload.js';
import { jcs } from '../lib/jcs.js';
import {
  MANIFEST_SCHEMA_VERSION,
  applyManifestMode,
  collectPayloadEntries,
  resolveIngestIntent,
} from '../lib/payload-manifest.js';
import { __testing } from '../install-plugin-generation.js';

const results = [];
const skipped = [];
const check = (name, body) => {
  body();
  results.push(name);
};
const skip = (name, why) => skipped.push(`${name} -- ${why}`);

const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-identity-'));
const capabilities = probeFilesystemCapabilities(workspace);
const sha256 = value => crypto.createHash('sha256').update(value).digest('hex');

// ---------------------------------------------------------------------------
// A fixed payload, described entirely by intent
// ---------------------------------------------------------------------------

// The payload description and its golden digest are shared with
// `verify-host-leg.mjs`, so the value this fixture asserts and the value the
// matrix compares are the same value by construction rather than by discipline.
const PAYLOAD = GOLDEN_PAYLOAD;
const intentOf = goldenIntentOf;

function build(name, activeCapabilities) {
  const root = path.join(workspace, name);
  const { entries, digest, degradations } = buildGoldenPayload(root, activeCapabilities);
  return { root, entries, degradations, digest };
}

const primary = build('primary', capabilities);

check('the payload digest matches the cross-host golden value', () => {
  assert.equal(
    primary.digest,
    GOLDEN_DIGEST,
    `identity diverged on ${process.platform}: this host computed ${primary.digest}`
  );
});

check('no entry records a host-specific property', () => {
  const forbidden = [
    'mode',
    'mtime',
    'ctime',
    'atime',
    'birthtime',
    'uid',
    'gid',
    'owner',
    'ownerSid',
    'sid',
    'attributes',
    'fileAttributes',
    'acl',
    'dacl',
  ];
  for (const entry of primary.entries) {
    for (const key of forbidden) {
      assert.equal(key in entry, false, `entry ${entry.path} records ${key}`);
    }
    assert.ok(['file', 'directory', 'symlink'].includes(entry.type));
    if (entry.type === 'file') assert.equal(typeof entry.executable, 'boolean');
    if (entry.type === 'symlink') assert.equal(typeof entry.target, 'string');
  }
});

check('a link entry hashes its recorded target, not its realization', () => {
  const alias = primary.entries.find(entry => entry.path === 'alias');
  assert.equal(alias.type, 'symlink');
  assert.equal(alias.target, 'bin');
  assert.equal(alias.sha256, sha256('bin'));
  assert.equal(alias.size, 3);
});

check('an empty directory is part of the identity', () => {
  assert.ok(primary.entries.some(entry => entry.path === 'empty' && entry.type === 'directory'));
});

// Scenario: degraded materialization leaves identity unchanged.
const degraded = build('degraded', {
  ...capabilities,
  symlinkFile: false,
  symlinkDirectory: false,
  junction: false,
});

check('a host with no link primitive computes the same identity', () => {
  assert.equal(degraded.degradations.length, 2);
  assert.deepEqual(
    degraded.degradations.map(entry => entry.realized),
    ['copy', 'copy']
  );
  assert.equal(degraded.digest, primary.digest);
  assert.equal(jcs(degraded.entries), jcs(primary.entries));
});

// Scenario: differing umask.
if (capabilities.posixModes && typeof process.umask === 'function') {
  check('two umasks produce the same identity and the same materialized modes', () => {
    const previous = process.umask(0o077);
    const strict = build('umask-strict', capabilities);
    process.umask(0o000);
    const loose = build('umask-loose', capabilities);
    process.umask(previous);
    assert.equal(strict.digest, loose.digest);
    const strictMode = fs.statSync(path.join(strict.root, 'bin', 'run.sh')).mode & 0o777;
    const looseMode = fs.statSync(path.join(loose.root, 'bin', 'run.sh')).mode & 0o777;
    assert.equal(strictMode, 0o755, 'modes are re-applied from the manifest, not left to the umask');
    assert.equal(looseMode, 0o755);
    assert.equal(
      fs.statSync(path.join(strict.root, 'data', 'plain.txt')).mode & 0o777,
      0o644
    );
  });
} else {
  check('a volume without mode semantics applies nothing and records nothing', () => {
    assert.equal(applyManifestMode(primary.root, { type: 'directory' }, capabilities), null);
    assert.ok(primary.entries.every(entry => !('mode' in entry)));
  });
  skip(
    'differing umask',
    'this volume has no POSIX mode semantics; the umask leg belongs to Linux and macOS'
  );
}

// Scenario: unsupported entry type.
const mkfifo = spawnSync('mkfifo', ['--version'], { encoding: 'utf8' });
if (!capabilities.posixModes || mkfifo.error) {
  skip(
    'unsupported entry type',
    'no mkfifo on this host; a device or socket entry cannot be created to be rejected'
  );
} else {
  check('a device, socket, or fifo entry fails ingest rather than being approximated', () => {
    const root = path.join(workspace, 'unsupported');
    fs.mkdirSync(root, { recursive: true });
    const fifo = path.join(root, 'pipe');
    const made = spawnSync('mkfifo', [fifo]);
    assert.equal(made.status, 0, 'the fixture must actually create a fifo');
    assert.throws(
      () => resolveIngestIntent({ sourcePath: fifo, repoRoot: root, oracle: null, capabilities }),
      error => error.code === 'UNSUPPORTED_ENTRY_TYPE'
    );
    assert.throws(
      () => collectPayloadEntries(root, () => ({ type: 'file', executable: false }), capabilities),
      error => error.code === 'UNSUPPORTED_ENTRY_TYPE'
    );
  });
}

// ---------------------------------------------------------------------------
// Requirement: schema version is enforced asymmetrically
// ---------------------------------------------------------------------------

const manifestBody = {
  sourceVersion: '1.8.0',
  signerKeyId: 'a'.repeat(64),
  bundleId: 'b'.repeat(64),
  hookRuntime: { abi: 'hook-runtime-v1' },
  sourceProvenance: { sourceCommit: 'c'.repeat(40), sourceTreeState: 'clean', externalSources: [] },
  skillIndex: { sha256: 'd'.repeat(64), entryCount: 3 },
  executionComponent: { componentId: 'x' },
  files: primary.entries,
  targetPayloads: [],
};

check('the creator emits the current schema version', () => {
  assert.equal(MANIFEST_SCHEMA_VERSION, 2);
  assert.equal(__testing.MANIFEST_SCHEMA_VERSION, 2);
  assert.equal(__testing.RELEASE_MANIFEST_SCHEMA_VERSION, 2);
});

check('a pre-existing generation keeps the digest rule it was signed under', () => {
  const legacy = { ...manifestBody, schemaVersion: 1 };
  const expected = sha256(__testing.canonicalJson(__testing.generationIdentity(legacy)));
  assert.equal(__testing.generationDigest(legacy), expected);
});

check('a new generation digests RFC 8785 and is therefore a different value', () => {
  const current = { ...manifestBody, schemaVersion: MANIFEST_SCHEMA_VERSION };
  assert.equal(
    __testing.generationDigest(current),
    sha256(jcs(__testing.generationIdentity(current)))
  );
  assert.notEqual(
    __testing.generationDigest(current),
    __testing.generationDigest({ ...manifestBody, schemaVersion: 1 })
  );
});

check('an unknown schema version is refused outright', () => {
  const trust = path.join(workspace, 'trust.json');
  const pair = crypto.generateKeyPairSync('ed25519');
  const keyId = sha256(pair.publicKey.export({ type: 'spki', format: 'der' }));
  fs.writeFileSync(
    trust,
    JSON.stringify({
      schemaVersion: 1,
      signers: [
        {
          keyId,
          algorithm: 'Ed25519',
          publicKey: pair.publicKey.export({ type: 'spki', format: 'pem' }).toString(),
        },
      ],
    })
  );
  const identity = { keyId, privateKey: pair.privateKey };
  const value = { hello: 'world' };

  // The creator always emits envelope version 2 over RFC 8785.
  const current = __testing.signValue(value, identity);
  assert.equal(current.schemaVersion, __testing.SIGNATURE_SCHEMA_VERSION);
  assert.equal(current.canonicalization, 'RFC8785');
  assert.equal(__testing.verifySignedValue(value, current, trust), keyId);

  // A version-1 envelope written by an earlier installer still verifies.
  const legacy = {
    schemaVersion: 1,
    namespace: current.namespace,
    algorithm: 'Ed25519',
    signerKeyId: keyId,
    signature: crypto
      .sign(null, __testing.signaturePayload(value, 'legacy'), pair.privateKey)
      .toString('base64'),
  };
  assert.equal(__testing.verifySignedValue(value, legacy, trust), keyId);

  // A version-2 envelope claiming a canonicalization the verifier does not
  // implement is refused rather than guessed at.
  assert.throws(() =>
    __testing.verifySignedValue(value, { ...current, canonicalization: 'pretty' }, trust)
  );
  assert.throws(() => __testing.verifySignedValue(value, { ...current, schemaVersion: 3 }, trust));
  // A version-1 envelope cannot borrow a version-2 signature.
  assert.throws(() =>
    __testing.verifySignedValue(value, { ...current, schemaVersion: 1 }, trust)
  );
});

check('executable intent is read from the manifest, per schema version', () => {
  const v2 = __testing.manifestExecutableLookup({
    schemaVersion: 2,
    files: [{ path: 'bin/run.sh', type: 'file', executable: true }],
  });
  assert.equal(v2('bin/run.sh'), true);
  assert.equal(v2('missing'), null);
  const v1 = __testing.manifestExecutableLookup({
    schemaVersion: 1,
    files: [
      { path: 'bin/run.sh', type: 'file', mode: '0755' },
      { path: 'data/plain.txt', type: 'file', mode: '0644' },
    ],
  });
  assert.equal(v1('bin/run.sh'), true);
  assert.equal(v1('data/plain.txt'), false);
});

fs.rmSync(workspace, { recursive: true, force: true });

process.stdout.write(`portable-generation-identity: ${results.length} checks passed\n`);
for (const name of results) process.stdout.write(`  ok   ${name}\n`);
for (const note of skipped) process.stdout.write(`  SKIP ${note}\n`);
process.stdout.write(`  host: ${process.platform}, digest ${primary.digest}\n`);
