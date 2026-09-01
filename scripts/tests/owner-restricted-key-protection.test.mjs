/**
 * Fixtures for the `owner-restricted-key-protection` specification.
 *
 * The Windows cases operate on REAL files with REAL access control lists,
 * created here with `icacls` and read back through the operating system. The
 * predicate under test never sees a hand-written descriptor.
 */

import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { probeFilesystemCapabilities } from '../lib/capabilities.js';
import {
  UNAVOIDABLE_TRUSTEE_SIDS,
  assertKeyProtection,
  evaluatePosixKeyProtection,
  evaluateWindowsKeyProtection,
} from '../lib/key-protection.js';
import {
  currentUserSid,
  icacls,
  referenceWindowsInspector,
} from './lib/windows-security-reference.mjs';

const results = [];
const skipped = [];
const check = (name, body) => {
  body();
  results.push(name);
};
const skip = (name, why) => skipped.push(`${name} -- ${why}`);

const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-key-protection-'));
const capabilities = probeFilesystemCapabilities(workspace);

// ---------------------------------------------------------------------------
// Requirement: private key material is owner-restricted -- POSIX assertion
// ---------------------------------------------------------------------------

check('POSIX: mode 0600 owned by the running user passes', () => {
  const verdict = evaluatePosixKeyProtection({
    keyPath: '/home/user/key.pem',
    stat: { mode: 0o100600, uid: 1000 },
    processUid: 1000,
  });
  assert.equal(verdict.ok, true);
  assert.equal(verdict.reason, 'OK');
});

check('POSIX: a group- or world-readable key fails and names the remediation', () => {
  const verdict = evaluatePosixKeyProtection({
    keyPath: '/home/user/key.pem',
    stat: { mode: 0o100644, uid: 1000 },
    processUid: 1000,
  });
  assert.equal(verdict.ok, false);
  assert.equal(verdict.reason, 'POSIX_MODE');
  assert.match(verdict.detail, /0644/);
  assert.match(verdict.remediation, /^chmod 600 /);
});

check('POSIX: a key owned by another user fails', () => {
  const verdict = evaluatePosixKeyProtection({
    keyPath: '/home/user/key.pem',
    stat: { mode: 0o100600, uid: 0 },
    processUid: 1000,
  });
  assert.equal(verdict.ok, false);
  assert.equal(verdict.reason, 'POSIX_OWNER');
});

if (capabilities.posixModes) {
  check('POSIX: the predicate dispatches on probed mode semantics, on a real file', () => {
    const key = path.join(workspace, 'real-key.pem');
    fs.writeFileSync(key, 'key', { mode: 0o600 });
    fs.chmodSync(key, 0o600);
    assert.equal(assertKeyProtection(key, { capabilities }).ok, true);
    fs.chmodSync(key, 0o644);
    const verdict = assertKeyProtection(key, { capabilities });
    assert.equal(verdict.ok, false);
    assert.equal(verdict.reason, 'POSIX_MODE');
  });
} else {
  skip(
    'POSIX real-file dispatch',
    'this volume reports no POSIX mode semantics, so the mode assertion is not the applicable one'
  );
}

// ---------------------------------------------------------------------------
// Requirement: trustees are compared as security identifiers
// ---------------------------------------------------------------------------

const OWNER = 'S-1-5-21-1111111111-2222222222-3333333333-1001';

check('a localized descriptor is judged on its security identifiers alone', () => {
  // Every principal here carries a German display name in a field the predicate
  // must never read. A predicate comparing names would reject this descriptor.
  const verdict = evaluateWindowsKeyProtection({
    keyPath: 'C:\\key.pem',
    descriptor: {
      schemaVersion: 1,
      model: 'windows-security-descriptor',
      path: 'C:\\key.pem',
      processOwnerSid: OWNER,
      ownerSid: OWNER,
      daclPresent: true,
      daclProtected: true,
      inheritedAceCount: 0,
      aces: [
        { sid: OWNER, displayName: 'VORDEFINIERT\\Benutzer', kind: 'allow', inherited: false, accessMask: 2032127 },
        { sid: 'S-1-5-18', displayName: 'NT-AUTORITÄT\\SYSTEM', kind: 'allow', inherited: false, accessMask: 2032127 },
        { sid: 'S-1-5-32-544', displayName: 'VORDEFINIERT\\Administratoren', kind: 'allow', inherited: false, accessMask: 2032127 },
      ],
    },
  });
  assert.equal(verdict.ok, true, verdict.detail);
});

check('a malformed report fails closed rather than passing', () => {
  for (const descriptor of [null, {}, { schemaVersion: 1, model: 'posix', supported: false }]) {
    const verdict = evaluateWindowsKeyProtection({ keyPath: 'C:\\key.pem', descriptor });
    assert.equal(verdict.ok, false);
    assert.equal(verdict.reason, 'UNSUPPORTED_DESCRIPTOR');
  }
});

check('a NULL discretionary access control list is the worst case, not the best', () => {
  const verdict = evaluateWindowsKeyProtection({
    keyPath: 'C:\\key.pem',
    descriptor: {
      schemaVersion: 1,
      model: 'windows-security-descriptor',
      path: 'C:\\key.pem',
      processOwnerSid: OWNER,
      ownerSid: OWNER,
      daclPresent: false,
      daclProtected: true,
      inheritedAceCount: 0,
      aces: [],
    },
  });
  assert.equal(verdict.ok, false);
  assert.equal(verdict.reason, 'DACL_ABSENT');
});

check('a host that can assert neither mechanism refuses rather than passes', () => {
  const verdict = assertKeyProtection('C:\\key.pem', {
    capabilities: { ...capabilities, posixModes: false },
    inspect: () => ({ descriptor: null, error: 'inspector is not installed' }),
  });
  assert.equal(verdict.ok, false);
  assert.equal(verdict.reason, 'INSPECTOR_FAILED');
  assert.match(verdict.remediation, /prometheus-exec/);
});

// ---------------------------------------------------------------------------
// Windows assertion against REAL access control lists
// ---------------------------------------------------------------------------

const ownerSid = process.platform === 'win32' ? currentUserSid() : null;
if (!ownerSid) {
  skip(
    'real Windows access control lists',
    'no Windows security descriptor source on this host; run this fixture on a Windows leg'
  );
} else {
  const grants = [ownerSid, ...UNAVOIDABLE_TRUSTEE_SIDS].map(sid => `*${sid}:F`);

  // Case 1: owner-restricted -- protected list, owner plus the two principals
  // Windows cannot exclude.
  const restricted = path.join(workspace, 'restricted-key.pem');
  fs.writeFileSync(restricted, 'key');
  const applied = icacls([restricted, '/inheritance:r', ...grants.flatMap(g => ['/grant:r', g])]);
  assert.equal(applied.status, 0, `icacls failed: ${applied.stderr || applied.stdout}`);

  // Case 2: inherited -- a plain new file that inherits its parent's list.
  const inherited = path.join(workspace, 'inherited-key.pem');
  fs.writeFileSync(inherited, 'key');

  // Case 3: over-granted -- owner-restricted, then Everyone is given read.
  const overGranted = path.join(workspace, 'over-granted-key.pem');
  fs.writeFileSync(overGranted, 'key');
  icacls([overGranted, '/inheritance:r', ...grants.flatMap(g => ['/grant:r', g])]);
  const granted = icacls([overGranted, '/grant', '*S-1-1-0:R']);
  assert.equal(granted.status, 0, `icacls grant failed: ${granted.stderr || granted.stdout}`);

  const verdictFor = file => {
    const { descriptor, error } = referenceWindowsInspector(file);
    assert.equal(error, null, `reference reader failed for ${file}: ${error}`);
    return { descriptor, verdict: evaluateWindowsKeyProtection({ keyPath: file, descriptor }) };
  };

  check('Windows: an owner-restricted key passes', () => {
    const { descriptor, verdict } = verdictFor(restricted);
    assert.equal(descriptor.ownerSid, descriptor.processOwnerSid);
    assert.equal(descriptor.daclProtected, true);
    assert.equal(descriptor.inheritedAceCount, 0);
    assert.equal(verdict.ok, true, `${verdict.reason}: ${verdict.detail}`);
  });

  check('Windows: an inherited access control list fails', () => {
    const { descriptor, verdict } = verdictFor(inherited);
    assert.ok(
      descriptor.inheritedAceCount > 0 || !descriptor.daclProtected,
      'the inherited fixture must actually inherit'
    );
    assert.equal(verdict.ok, false);
    assert.equal(verdict.reason, 'DACL_INHERITED');
    assert.match(verdict.remediation, /\/inheritance:r/);
  });

  check('Windows: an over-granted access control list fails and names the trustee', () => {
    const { verdict } = verdictFor(overGranted);
    assert.equal(verdict.ok, false);
    assert.equal(verdict.reason, 'UNEXPECTED_TRUSTEE');
    assert.match(verdict.detail, /S-1-1-0/, 'the unexpected trustee must be named as a SID');
  });

  check('Windows: remediation is reported and never applied', () => {
    const before = fs.readFileSync(overGranted);
    const beforeAcl = referenceWindowsInspector(overGranted).descriptor;
    const { verdict } = verdictFor(overGranted);
    const afterAcl = referenceWindowsInspector(overGranted).descriptor;
    assert.equal(verdict.ok, false);
    assert.match(verdict.remediation, /^icacls /);
    assert.match(verdict.remediation, /\*S-1-5-18:F/);
    assert.match(verdict.remediation, /\*S-1-5-32-544:F/);
    assert.deepEqual(fs.readFileSync(overGranted), before, 'the key bytes must be untouched');
    assert.deepEqual(afterAcl.aces, beforeAcl.aces, 'the access control list must be untouched');
    assert.equal(afterAcl.daclProtected, beforeAcl.daclProtected);
  });

  check('Windows: the predicate dispatches here on probed capability, not on platform', () => {
    // `capabilities.posixModes` is false on this volume, which is the empirical
    // reason the security-descriptor assertion applies -- no `process.platform`
    // is consulted anywhere on the path.
    assert.equal(capabilities.posixModes, false);
    const verdict = assertKeyProtection(restricted, {
      capabilities,
      inspect: referenceWindowsInspector,
    });
    assert.equal(verdict.ok, true, `${verdict.reason}: ${verdict.detail}`);
  });
}

fs.rmSync(workspace, { recursive: true, force: true });

process.stdout.write(`owner-restricted-key-protection: ${results.length} checks passed\n`);
for (const name of results) process.stdout.write(`  ok   ${name}\n`);
for (const note of skipped) process.stdout.write(`  SKIP ${note}\n`);
