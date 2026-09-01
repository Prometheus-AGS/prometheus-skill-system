#!/usr/bin/env node
/**
 * Run this host's leg of the activation verification matrix and record it.
 *
 * WHY THIS IS NOT A CI WORKFLOW
 *
 * The task list asks for a "four-leg CI matrix". This repository forbids that:
 * CLAUDE.md's local-only validation rule is marked MANDATORY, and
 * `scripts/check-workflow-policy.mjs` mechanically rejects any hosted workflow
 * that is not `docs-sync` or `docs-pages`, along with any workflow containing
 * `npm test`, `cargo test`, and friends. A hosted matrix would fail the
 * repository's own gate before it ever ran.
 *
 * So the matrix is assembled the way this repository assembles everything else:
 * each host runs its own leg LOCALLY and emits a signed-by-content receipt, and
 * `compare-host-legs.mjs` asserts the legs agree. The property the spec demands
 * -- identical bundle identity on every supported host, with any divergence
 * failing the change -- is unchanged. Only the place the legs run moves.
 *
 * THE LEG ID IS DERIVED, NEVER DECLARED
 *
 * A leg is identified by what the host's filesystem can actually do, measured
 * by the capability probe, not by what it says it is. That matters for exactly
 * one leg: a Windows host with Developer Mode enabled and one without are the
 * same platform and the same arch, and differ only in whether a directory
 * symlink can be created. Deriving the id from the probe is what makes
 * "a passing result cannot come from an elevated configuration" checkable
 * rather than promised.
 */

import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { probeFilesystemCapabilities, probeShell } from './lib/capabilities.js';
import { GOLDEN_DIGEST, buildGoldenPayload } from './lib/golden-payload.js';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const legsRoot = path.join(
  repoRoot,
  'openspec/changes/change-win-001-host-portable-activation/evidence/legs'
);
const home = os.homedir();

const sha256 = value => crypto.createHash('sha256').update(value).digest('hex');

/**
 * Remove this operator's identity from recorded output.
 *
 * Evidence is committed, so a home directory, a user name, and a temporary
 * directory must not travel with it. The HASH is taken over the raw bytes, so
 * redaction cannot be used to launder a failing result into a passing one.
 */
function redact(text) {
  return String(text ?? '')
    .split(home)
    .join('~')
    .split(home.split(path.sep).join('/'))
    .join('~')
    .split(os.tmpdir())
    .join('$TMPDIR')
    .replace(/\b[a-f0-9]{64}\b/g, digest => `${digest.slice(0, 12)}…`);
}

function record(name, command, args) {
  const result = spawnSync(command, args, { encoding: 'utf8', shell: false, cwd: repoRoot });
  const output = `${result.stdout ?? ''}${result.stderr ?? ''}`;
  const lines = redact(output).trim().split('\n').filter(Boolean);
  return {
    name,
    command: `${command} ${args.join(' ')}`,
    exitCode: result.status ?? null,
    outputSha256: sha256(output),
    // The tail carries the verdict; the head carries the context. Everything in
    // between is noise in a receipt.
    excerpt: lines.length > 6 ? [...lines.slice(0, 2), '…', ...lines.slice(-3)] : lines,
  };
}

const capabilities = probeFilesystemCapabilities(path.join(os.tmpdir(), 'prometheus-host-leg'));
const shell = probeShell();

// The leg id is (platform, directoryLinkStrategy). Nothing here is self-asserted.
const legId =
  process.platform === 'win32'
    ? `windows-${capabilities.directoryLinkStrategy}`
    : process.platform === 'darwin'
      ? 'macos'
      : process.platform === 'linux'
        ? 'linux'
        : `${process.platform}-${capabilities.directoryLinkStrategy}`;

const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-golden-'));
let golden;
try {
  golden = buildGoldenPayload(path.join(workspace, 'payload'), capabilities);
} finally {
  fs.rmSync(workspace, { recursive: true, force: true });
}

const release = JSON.parse(
  fs.readFileSync(path.join(repoRoot, 'shared/harnesses/generated/release-manifest.json'), 'utf8')
);

const checks = [
  record('generated artifacts', process.execPath, [
    'scripts/generate-harness-adapters.js',
    '--check',
  ]),
  record('harness runtime parity', process.execPath, ['scripts/check-harness-adapters.js']),
  record('workflow policy', process.execPath, ['scripts/check-workflow-policy.mjs']),
  record('host capability probing', process.execPath, [
    'scripts/tests/host-capability-probing.test.mjs',
  ]),
  record('owner-restricted key protection', process.execPath, [
    'scripts/tests/owner-restricted-key-protection.test.mjs',
  ]),
  record('portable generation identity', process.execPath, [
    'scripts/tests/portable-generation-identity.test.mjs',
  ]),
  record('activation pointer', process.execPath, ['scripts/tests/activation-pointer.test.mjs']),
  record('shell-free hook dispatch', process.execPath, ['scripts/tests/hook-dispatch.test.mjs']),
];

const receipt = {
  schemaVersion: 1,
  legId,
  host: { platform: process.platform, arch: process.arch, node: process.version },
  capabilities: {
    symlinkFile: capabilities.symlinkFile,
    symlinkDirectory: capabilities.symlinkDirectory,
    junction: capabilities.junction,
    hardlink: capabilities.hardlink,
    executableBit: capabilities.executableBit,
    posixModes: capabilities.posixModes,
    directoryLinkStrategy: capabilities.directoryLinkStrategy,
    fileLinkStrategy: capabilities.fileLinkStrategy,
    shell,
  },
  // The two values every leg must agree on. Everything else in this receipt is
  // context; these are the gate.
  identity: {
    bundleId: release.bundleId,
    goldenPayloadDigest: golden.digest,
    releaseManifestSchemaVersion: release.schemaVersion,
    dispatcherInterpreter: release.dispatcherInterpreter ?? null,
  },
  // Reported, never hashed into the identity: a host that had to degrade a link
  // still has to land on the same digest.
  degradations: golden.degradations,
  checks,
};

const failed = checks.filter(check => check.exitCode !== 0);
const goldenMatches = golden.digest === GOLDEN_DIGEST;

fs.mkdirSync(legsRoot, { recursive: true });
const file = path.join(legsRoot, `${legId}.json`);
fs.writeFileSync(file, `${JSON.stringify(receipt, null, 2)}\n`);

process.stdout.write(`host leg: ${legId}\n`);
process.stdout.write(
  `  capabilities: ${capabilities.directoryLinkStrategy} links, executableBit=${capabilities.executableBit}, posixModes=${capabilities.posixModes}\n`
);
process.stdout.write(`  bundleId: ${release.bundleId}\n`);
process.stdout.write(
  `  golden payload digest: ${golden.digest}${goldenMatches ? '' : ' (DIVERGED)'}\n`
);
for (const check of checks) {
  process.stdout.write(`  ${check.exitCode === 0 ? 'ok  ' : 'FAIL'} ${check.name}\n`);
}
process.stdout.write(`  receipt: ${path.relative(repoRoot, file)}\n`);

if (!goldenMatches) {
  process.stderr.write(
    `\nthis host computes a different identity for the golden payload than every other leg.\n` +
      `  expected ${GOLDEN_DIGEST}\n  computed ${golden.digest}\n` +
      'that is host dependence in the payload identity, which this change exists to remove.\n'
  );
}
if (failed.length) {
  process.stderr.write(`\n${failed.length} local check(s) failed on this leg:\n`);
  for (const check of failed) process.stderr.write(`  ${check.name}: ${check.command}\n`);
}
process.exitCode = failed.length || !goldenMatches ? 1 : 0;
