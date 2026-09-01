#!/usr/bin/env node

import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { fileURLToPath } from 'node:url';

import { loadCapabilities, materializeLink } from './lib/capabilities.js';
import { jcs } from './lib/jcs.js';
import { assertKeyProtection } from './lib/key-protection.js';
import {
  MANIFEST_SCHEMA_VERSION,
  MATERIALIZATION_RECORD,
  MATERIALIZATION_SCHEMA_VERSION,
  applyManifestMode,
  collectPayloadEntries,
  entriesEqual,
  manifestIntentLookup,
  readIngestOracle,
  resolveIngestIntent,
} from './lib/payload-manifest.js';
import {
  assertMinimumActiveVersion,
  collectDistributionSkills,
  compareVersions,
  readSkillSystem,
  targetsById,
} from './lib/skill-system.js';
import { POINTER_PATTERN, isWithin } from './lib/store-paths.js';

/**
 * Probed filesystem capabilities for this run.
 *
 * A module-level value rather than a threaded parameter because it is measured
 * exactly once, in the generation store root, before anything is materialized —
 * and because the alternative is adding an argument to nine call sites that all
 * pass the same value. Nothing in this file may infer a capability from
 * `process.platform`; every decision reads this record.
 */
let CAPABILITIES = null;

const TARGETS = [
  '.claude/skills',
  '.opencode/skills',
  '.kimi-code/skills',
  '.minimax/skills',
  '.cursor/skills',
  '.codex/skills',
  '.gemini/skills',
  '.roo/skills',
  '.devin/skills',
  '.codeium/windsurf/skills',
  '.agents/skills',
  '.config/zed/skills',
  '.zed/skills',
  '.cline/skills',
];
const COPY_TARGETS = new Set(['.minimax/skills', '.codex/skills']);
const REQUIRED_SCRIPTS = [
  'hook-runtime-v1.sh',
  'bootstrap-hook-runtime.sh',
  'generated/hook-dispatch-v1.sh',
  'karpathy-hook-dispatch.sh',
  'detect-project-context.sh',
  'memory-outbox-flush.sh',
  'pk-health.sh',
  'enqueue-learning-job.py',
  'enqueue-memory-operation.py',
];
const STABLE_SCRIPTS = [
  'karpathy-hook-dispatch.sh',
  'detect-project-context.sh',
  'memory-outbox-flush.sh',
  'pk-health.sh',
];
const STABLE_HELPERS = ['enqueue-learning-job.py', 'enqueue-memory-operation.py'];
const STABLE_DIRECTORIES = ['lib'];
const PAYLOAD_ROOTS = [
  'agents',
  // Prebuilt hook dispatchers, one directory per target. Part of the PAYLOAD
  // rather than read from the source tree because the installer executes what
  // it finds here, and only a payload entry is manifested, hashed, and signed.
  'bin',
  'hooks',
  'shared',
  'scripts',
  '.claude-plugin',
  '.codex-plugin',
  '.agents/plugins',
  '.mcp.json',
  'skill-system.json',
];
const MANIFEST_SIGNATURE = 'manifest.sig.json';
const SKILL_INDEX_SCHEMA = 'prometheus-skill-index-v1';
const COMPONENT_INDEX_SCHEMA = 'prometheus-exec-component-index-v1';
const EXEC_COMPONENT_DESCRIPTOR = 'config/prometheus-exec-component.json';
const EXEC_HOST_IMPORTS = [
  'prometheus:component/types@0.1.0',
  'prometheus:component/log@0.1.0',
  'prometheus:component/kv-store@0.1.0',
  'prometheus:component/input@0.1.0',
  'prometheus:component/output@0.1.0',
  'prometheus:component/clock@0.1.0',
  'prometheus:component/random@0.1.0',
];
const EXEC_WASI_IMPORTS = [
  'wasi:io/poll@0.2.9',
  'wasi:clocks/monotonic-clock@0.2.9',
  'wasi:io/error@0.2.9',
  'wasi:io/streams@0.2.9',
  'wasi:cli/stdout@0.2.9',
  'wasi:cli/stderr@0.2.9',
  'wasi:cli/stdin@0.2.9',
  'wasi:cli/environment@0.2.9',
  'wasi:cli/exit@0.2.9',
  'wasi:cli/terminal-input@0.2.9',
  'wasi:cli/terminal-output@0.2.9',
  'wasi:cli/terminal-stdin@0.2.9',
  'wasi:cli/terminal-stdout@0.2.9',
  'wasi:cli/terminal-stderr@0.2.9',
];
const SIGNATURE_NAMESPACE = 'prometheus-plugin-generation-v1';
const RELEASE_MANIFEST_SCHEMA_VERSION = 2;

function fail(message) {
  throw new Error(message);
}

function parseArgs(argv) {
  const args = {
    sourceRoot: process.cwd(),
    pluginRoot: path.join(os.homedir(), '.prometheus/plugins/prometheus-skill-pack'),
    home: os.homedir(),
    verify: false,
    rollback: false,
    uninstall: false,
    pruneObsolete: false,
    dryRun: false,
    targets: 'all',
    expectedBundle: null,
    expectedSourceCommit: null,
    requireCleanSource: false,
    signingKey: null,
    trustStore: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--verify') args.verify = true;
    else if (value === '--rollback') args.rollback = true;
    else if (value === '--uninstall') args.uninstall = true;
    else if (value === '--prune-obsolete') args.pruneObsolete = true;
    else if (value === '--dry-run') args.dryRun = true;
    else if (value === '--targets') args.targets = argv[++index];
    else if (value === '--require-clean-source') args.requireCleanSource = true;
    else if (value === '--expected-bundle') args.expectedBundle = argv[++index];
    else if (value === '--expected-source-commit') args.expectedSourceCommit = argv[++index];
    else if (value === '--source-root') args.sourceRoot = argv[++index];
    else if (value === '--plugin-root') args.pluginRoot = argv[++index];
    else if (value === '--home') args.home = argv[++index];
    else if (value === '--signing-key') args.signingKey = argv[++index];
    else if (value === '--trust-store') args.trustStore = argv[++index];
    else fail(`unknown argument: ${value}`);
  }
  for (const key of ['sourceRoot', 'pluginRoot', 'home']) {
    if (!args[key])
      fail(`missing value for --${key.replace(/[A-Z]/g, c => `-${c.toLowerCase()}`)}`);
    args[key] = path.resolve(args[key]);
  }
  args.signingKey = path.resolve(
    args.signingKey ?? path.join(args.home, '.prometheus/plugin-signing/ed25519-private.pem')
  );
  args.trustStore = path.resolve(
    args.trustStore ?? path.join(args.pluginRoot, 'trust/allowed-signers.json')
  );
  if (args.expectedBundle && !/^[a-f0-9]{64}$/.test(args.expectedBundle)) {
    fail('invalid value for --expected-bundle');
  }
  if (args.expectedSourceCommit && !/^[a-f0-9]{40,64}$/.test(args.expectedSourceCommit)) {
    fail('invalid value for --expected-source-commit');
  }
  return args;
}

function assertSafeRoot(root, home) {
  const parsed = path.parse(root);
  if (home === path.parse(home).root) fail(`refusing unsafe home target: ${home}`);
  if (root === parsed.root || root === home || root.length < parsed.root.length + 8) {
    fail(`refusing unsafe plugin root: ${root}`);
  }
}

function sha256(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

function keyId(publicKey) {
  return sha256(publicKey.export({ type: 'spki', format: 'der' }));
}

function ensureSigningIdentity(privateKeyPath, trustStorePath) {
  ensureDirectory(path.dirname(privateKeyPath), 0o700);
  const trustExists = fs.existsSync(trustStorePath);
  let created = false;
  if (!fs.existsSync(privateKeyPath)) {
    if (trustExists) {
      fail(`plugin signing key is missing but a trust store already exists: ${privateKeyPath}`);
    }
    const pair = crypto.generateKeyPairSync('ed25519');
    atomicWrite(privateKeyPath, pair.privateKey.export({ type: 'pkcs8', format: 'pem' }), 0o600);
    created = true;
  }
  // Owner-only protection is asserted for a key this run just created as well as
  // for one it found. On a volume with no POSIX mode semantics the 0o600 passed
  // to atomicWrite above is inert -- libuv maps chmod onto FILE_ATTRIBUTE_READONLY
  // and nothing else -- so a freshly written key inherits its parent's DACL and
  // must be restricted before it is trusted to sign anything. The key is left in
  // place deliberately: `icacls /inheritance:r` needs a file to operate on, and
  // deleting it would make the remediation impossible to carry out.
  const protection = assertKeyProtection(privateKeyPath, { capabilities: CAPABILITIES });
  if (!protection.ok) {
    fail(
      [
        `plugin signing key is not owner-restricted: ${privateKeyPath}`,
        `  reason: ${protection.reason}`,
        `  detail: ${protection.detail}`,
        created ? '  note: this run created the key; it has not signed anything' : null,
        protection.remediation ? `  remediation (not applied): ${protection.remediation}` : null,
      ]
        .filter(Boolean)
        .join('\n')
    );
  }
  const privateKey = crypto.createPrivateKey(fs.readFileSync(privateKeyPath));
  const publicKey = crypto.createPublicKey(privateKey);
  const signer = keyId(publicKey);
  const encoded = publicKey.export({ type: 'spki', format: 'pem' }).toString();
  ensureDirectory(path.dirname(trustStorePath), 0o700);
  let trust = { schemaVersion: 1, signers: [] };
  if (trustExists) trust = JSON.parse(fs.readFileSync(trustStorePath, 'utf8'));
  if (!Array.isArray(trust.signers)) fail('plugin trust store has no signers array');
  const existing = trust.signers.find(entry => entry.keyId === signer);
  if (existing && existing.publicKey !== encoded) fail(`plugin signer collision: ${signer}`);
  if (!existing && trustExists) fail(`plugin signer is not enrolled: ${signer}`);
  if (!existing) {
    trust.signers.push({ keyId: signer, algorithm: 'Ed25519', publicKey: encoded });
    trust.signers.sort((left, right) => left.keyId.localeCompare(right.keyId));
    atomicWrite(trustStorePath, canonicalJson(trust), 0o600);
  }
  return { privateKey, publicKey, keyId: signer };
}

function readTrustedKey(trustStorePath, signer) {
  if (!fs.existsSync(trustStorePath)) fail(`plugin trust store is missing: ${trustStorePath}`);
  const trust = JSON.parse(fs.readFileSync(trustStorePath, 'utf8'));
  const entry = trust.signers?.find(candidate => candidate.keyId === signer);
  if (!entry || entry.algorithm !== 'Ed25519') fail(`untrusted plugin signer: ${signer}`);
  const publicKey = crypto.createPublicKey(entry.publicKey);
  if (keyId(publicKey) !== signer) fail(`plugin trust-store key fingerprint mismatch: ${signer}`);
  return publicKey;
}

// Signature envelope versions.
//
// Version 1 signs over `canonicalJson` -- sorted keys, two-space indentation.
// That is deterministic but is not RFC 8785, so no non-JavaScript verifier can
// reproduce it. Version 2 signs over JCS, matching the receipt signing already
// performed by `prometheus-exec`. The verifier accepts both so that receipts and
// manifests written by an earlier installer keep verifying; the creator emits
// only version 2.
const SIGNATURE_SCHEMA_VERSION = 2;

function signaturePayload(value, canonicalization) {
  const body = canonicalization === 'RFC8785' ? jcs(value) : canonicalJson(value);
  return Buffer.from(`${SIGNATURE_NAMESPACE}\n${body}`);
}

function signValue(value, identity) {
  return {
    schemaVersion: SIGNATURE_SCHEMA_VERSION,
    namespace: SIGNATURE_NAMESPACE,
    algorithm: 'Ed25519',
    canonicalization: 'RFC8785',
    signerKeyId: identity.keyId,
    signature: crypto
      .sign(null, signaturePayload(value, 'RFC8785'), identity.privateKey)
      .toString('base64'),
  };
}

function verifySignedValue(value, signature, trustStorePath) {
  if (
    ![1, SIGNATURE_SCHEMA_VERSION].includes(signature?.schemaVersion) ||
    signature?.namespace !== SIGNATURE_NAMESPACE ||
    signature?.algorithm !== 'Ed25519'
  ) {
    fail('invalid plugin signature envelope');
  }
  const canonicalization = signature.schemaVersion === 1 ? 'legacy' : signature.canonicalization;
  if (signature.schemaVersion === SIGNATURE_SCHEMA_VERSION && canonicalization !== 'RFC8785') {
    fail('invalid plugin signature canonicalization');
  }
  const publicKey = readTrustedKey(trustStorePath, signature.signerKeyId);
  if (
    !crypto.verify(
      null,
      signaturePayload(value, canonicalization),
      publicKey,
      Buffer.from(signature.signature ?? '', 'base64')
    )
  ) {
    fail('plugin signature verification failed');
  }
  return signature.signerKeyId;
}

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map(key => [key, canonical(value[key])])
    );
  }
  return value;
}

function canonicalJson(value) {
  return `${JSON.stringify(canonical(value), null, 2)}\n`;
}

function modeString(mode) {
  return (mode & 0o7777).toString(8).padStart(4, '0');
}

function ensureDirectory(directory, mode = 0o755) {
  fs.mkdirSync(directory, { recursive: true, mode });
}

// FlushFileBuffers requires WRITE access to the handle. A descriptor opened
// read-only returns ERROR_ACCESS_DENIED, which libuv surfaces as EPERM, so this
// used to fail on Windows for every payload file it tried to make durable --
// POSIX `fsync` has no such requirement, which is why opening 'r' worked
// everywhere else. The handle is opened for writing and the read-only open is
// kept only for the cases that cannot be: a directory, or a file this process
// may read and not write.
function syncPath(target) {
  let descriptor;
  try {
    descriptor = fs.openSync(target, 'r+');
  } catch (error) {
    if (!['EACCES', 'EPERM', 'EROFS', 'EISDIR'].includes(error.code)) throw error;
    descriptor = fs.openSync(target, 'r');
  }
  try {
    fs.fsyncSync(descriptor);
  } finally {
    fs.closeSync(descriptor);
  }
}

// Flushing a DIRECTORY is a POSIX durability idiom with no Windows equivalent:
// there is no directory-flush API, and FlushFileBuffers on a directory handle
// returns ERROR_ACCESS_DENIED, which libuv surfaces as EPERM. NTFS orders the
// metadata for a rename through the file flush that already happened in
// `atomicWrite`, so the guarantee is expressed differently rather than dropped.
// EPERM and EACCES join the errors already tolerated for the same reason on
// other platforms; on Linux and macOS a directory fsync succeeds and none of
// them occur.
const UNSUPPORTED_DIRECTORY_SYNC = ['EINVAL', 'ENOTSUP', 'EISDIR', 'EPERM', 'EACCES'];

function syncDirectory(directory) {
  try {
    syncPath(directory);
  } catch (error) {
    if (!UNSUPPORTED_DIRECTORY_SYNC.includes(error.code)) throw error;
  }
}

function atomicWrite(file, content, mode = 0o644) {
  ensureDirectory(path.dirname(file));
  const temporary = path.join(path.dirname(file), `.${path.basename(file)}.${process.pid}.tmp`);
  const descriptor = fs.openSync(temporary, 'wx', mode);
  try {
    fs.writeFileSync(descriptor, content);
    fs.fsyncSync(descriptor);
  } finally {
    fs.closeSync(descriptor);
  }
  fs.renameSync(temporary, file);
  syncDirectory(path.dirname(file));
}

// ---------------------------------------------------------------------------
// Activation: a pointer FILE is the record of truth, a link is a convenience
// ---------------------------------------------------------------------------
//
// `atomicSymlink()` used to create a temporary link and rename it into place.
// That is atomic on POSIX and impossible on Windows: MoveFileExW with
// MOVEFILE_REPLACE_EXISTING fails when the destination is a directory, and BOTH
// directory symlinks and junctions carry FILE_ATTRIBUTE_DIRECTORY. There is no
// atomic directory-link swap to be had.
//
// So the authoritative pointer became a small FILE holding `generations/<id>`,
// swapped by rename -- atomic over an existing file everywhere. That is
// strictly stronger than the link it replaces: a byte string can be hashed,
// signed, and swapped atomically, and a link can be none of those things.
//
// The link is still created wherever a link primitive exists, because
// `stable/*`, the platform skill targets, and `current/skills/...` all need a
// real directory alias on disk. Its replacement no longer has to be atomic,
// since the pointer file already carries that guarantee, so it is done as
// unlink-then-create under the store mutex with a breadcrumb that makes an
// interrupted swap recoverable.

/**
 * The store mutex.
 *
 * Deliberately the SAME path and the SAME protocol as the `mkdir` lock in
 * `bootstrap-hook-runtime.sh`. Two different mutexes guarding one store is not
 * mutual exclusion: a hook-triggered bootstrap and a hand-run installer would
 * each hold their own and swap pointers at the same time.
 *
 * A `mkdir` lock is not released by the operating system when the holder dies,
 * so a dead holder is detected and its lock broken -- the same fallible
 * approach the shell side already takes. The OS-released advisory lock that
 * removes stale-holder detection entirely belongs to the compiled dispatcher.
 */
const STORE_LOCK_DIRECTORY = '.bootstrap-lock';
const STORE_LOCK_TIMEOUT_MS = 60_000;

function sleepSync(milliseconds) {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, milliseconds);
}

function processAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    // EPERM means the process exists and belongs to somebody else.
    return error.code === 'EPERM';
  }
}

function withStoreLock(pluginRoot, body) {
  // `bootstrap-hook-runtime.sh` acquires the lock and then invokes this
  // installer. Re-acquiring it there would deadlock against its own caller.
  if (process.env.PROMETHEUS_STORE_LOCK_HELD === '1') return body();
  ensureDirectory(pluginRoot);
  const lock = path.join(pluginRoot, STORE_LOCK_DIRECTORY);
  const holderFile = path.join(lock, 'pid');
  const deadline = Date.now() + STORE_LOCK_TIMEOUT_MS;
  for (;;) {
    try {
      fs.mkdirSync(lock);
      fs.writeFileSync(holderFile, `${process.pid}\n`);
      break;
    } catch (error) {
      if (error.code !== 'EEXIST') throw error;
    }
    let holder = null;
    try {
      holder = Number.parseInt(fs.readFileSync(holderFile, 'utf8').trim(), 10);
    } catch {
      holder = null;
    }
    if (Number.isInteger(holder) && holder > 0 && !processAlive(holder)) {
      fs.rmSync(lock, { recursive: true, force: true });
      continue;
    }
    if (Date.now() > deadline) {
      fail(
        `could not acquire the generation store lock at ${lock} (holder pid ${holder ?? 'unknown'})`
      );
    }
    sleepSync(100);
  }
  try {
    return body();
  } finally {
    fs.rmSync(lock, { recursive: true, force: true });
  }
}

function pointerFile(pluginRoot, name) {
  return path.join(pluginRoot, 'pointers', ...name.split('/'));
}

function breadcrumbFile(pluginRoot, name) {
  return path.join(pluginRoot, 'pointers/.pending', `${name.split('/').join('__')}.json`);
}

/** Read an activation pointer, or null. A malformed pointer is an error. */
function readPointer(pluginRoot, name) {
  const file = pointerFile(pluginRoot, name);
  if (!fs.existsSync(file)) return null;
  const value = fs.readFileSync(file, 'utf8').split('\n')[0].trim();
  if (!POINTER_PATTERN.test(value)) {
    fail(`activation pointer does not name a generation: ${name} -> ${value}`);
  }
  return value;
}

function writePointer(pluginRoot, name, target) {
  if (!POINTER_PATTERN.test(target)) {
    fail(`refusing to write an activation pointer that does not name a generation: ${target}`);
  }
  atomicWrite(pointerFile(pluginRoot, name), `${target}\n`);
}

function removeEntry(target) {
  const stat = fs.lstatSync(target, { throwIfNoEntry: false });
  if (!stat) return;
  // A junction and a directory symlink both report isDirectory() through stat
  // but must be removed as links, not walked. lstat plus isSymbolicLink() is
  // the only reliable discriminator, and rmSync handles a real directory.
  if (stat.isSymbolicLink() || stat.isFile()) fs.unlinkSync(target);
  else fs.rmSync(target, { recursive: true, force: true });
}

/**
 * Replace a convenience link, leaving a breadcrumb across the window where it
 * does not exist.
 *
 * `name` is the link's path relative to the plugin root and doubles as the
 * breadcrumb key, so a crash between the unlink and the create is recoverable
 * by name alone.
 */
function replaceConvenienceLink(
  pluginRoot,
  name,
  target,
  degradedCopy = 'contents',
  kind = 'directory'
) {
  const linkPath = path.join(pluginRoot, ...name.split('/'));
  const breadcrumb = breadcrumbFile(pluginRoot, name);
  atomicWrite(
    breadcrumb,
    canonicalJson({ schemaVersion: 1, link: name, target, degradedCopy, kind, pid: process.pid })
  );
  removeEntry(linkPath);
  const outcome = materializeLink({
    linkPath,
    target,
    capabilities: CAPABILITIES,
    degradedCopy,
    kind,
  });
  fs.rmSync(breadcrumb, { force: true });
  syncDirectory(path.dirname(linkPath));
  return outcome;
}

/**
 * Finish any link swap that was interrupted.
 *
 * The breadcrumb records a decision that was already durably justified -- the
 * generation is on disk and verified before any pointer moves -- so recovery
 * COMPLETES the swap rather than reverting it. Reverting would leave the
 * pointer file and the link disagreeing, which is the one state nothing else
 * in the store can express.
 */
function recoverPendingLinks(pluginRoot) {
  const pending = path.join(pluginRoot, 'pointers/.pending');
  if (!fs.existsSync(pending)) return [];
  const recovered = [];
  for (const file of fs.readdirSync(pending).sort()) {
    if (!file.endsWith('.json')) continue;
    const absolute = path.join(pending, file);
    let record = null;
    try {
      record = JSON.parse(fs.readFileSync(absolute, 'utf8'));
    } catch {
      fs.rmSync(absolute, { force: true });
      continue;
    }
    if (
      record?.schemaVersion !== 1 ||
      typeof record.link !== 'string' ||
      typeof record.target !== 'string'
    ) {
      fs.rmSync(absolute, { force: true });
      continue;
    }
    replaceConvenienceLink(
      pluginRoot,
      record.link,
      record.target,
      record.degradedCopy ?? 'contents',
      record.kind ?? 'directory'
    );
    recovered.push(record.link);
  }
  return recovered;
}

/**
 * Move an activation pointer: the file first, then the link.
 *
 * Order matters. The file is the record of truth, so it is written first and a
 * crash before the link is refreshed leaves the store correct and merely
 * missing a convenience -- which `recoverPendingLinks` then restores.
 */
function setActivationPointer(pluginRoot, name, generationTarget, linkTarget) {
  writePointer(pluginRoot, name, generationTarget);
  try {
    replaceConvenienceLink(pluginRoot, name, linkTarget);
  } catch (error) {
    if (error.code !== 'NO_DIRECTORY_LINK_PRIMITIVE') throw error;
    // The pointer file is already correct and is what everything authoritative
    // reads. Report the missing convenience rather than failing activation.
    process.stderr.write(
      `install-plugin-generation: ${name} has no convenience link on this volume ` +
        '(no directory symlink and no junction); the activation pointer file is authoritative\n'
    );
  }
}

// An imported submodule carries its OWN KBD lifecycle state, including device
// and browser test evidence: Playwright traces, .webm screen recordings, and
// built iOS .app bundles. None of it is read at runtime — nothing in scripts/,
// shared/scripts/, or hooks/ references a submodule evidence path — but all of
// it was being copied into every generation and shipped to all 14 targets.
//
// Measured on generation 7a88b914: 97M of a 188M payload was
// skills/imported/prometheus-entity-management/.kbd-orchestrator/.../evidence,
// including a single 28M Mach-O iOS binary.
//
// The match is deliberately PATH-scoped, not name-scoped. This repository's own
// top-level .kbd-orchestrator/ is load-bearing (waypoints, progress.json, phase
// plans) and must keep shipping; only evidence under skills/imported/** is
// disposable. A bare `name === '.kbd-orchestrator'` check would silently drop
// the active KBD state the pack depends on.
const IMPORTED_EVIDENCE_RE =
  /(^|\/)skills\/imported\/[^/]+\/\.kbd-orchestrator\/phases\/[^/]+\/evidence$/;

function isExcludedPayloadEntry(name, sourcePath, repoRoot) {
  if (name === 'node_modules' || name === 'target' || name === '.git') return true;
  const relative = path.relative(repoRoot, sourcePath).split(path.sep).join('/');
  return IMPORTED_EVIDENCE_RE.test(relative);
}

/**
 * Materialize one payload entry, recording intent and any degradation.
 *
 * `ctx` carries the probed capabilities, the intent oracle, the map of recorded
 * intents keyed by destination-relative path, and the degradation log. Modes are
 * applied FROM the resolved intent rather than copied from the source, which is
 * what removes the umask dependence that made schema-1 identities differ between
 * two Linux hosts.
 */
function copyEntry(source, destination, durable = true, repoRoot = null, ctx = null) {
  if (!ctx) fail(`internal: payload materialization requires a capability context: ${source}`);
  const root = repoRoot ?? source;
  const intent = ctx.intentFor(source);
  const relative = toPosix(path.relative(ctx.destinationRoot, destination));

  if (intent.type === 'symlink') {
    ensureDirectory(path.dirname(destination));
    const outcome = materializeLink({
      linkPath: destination,
      target: intent.target,
      capabilities: ctx.capabilities,
      // No junction inside a payload: its target is absolute and would still
      // point at the staging directory after the rename into `generations/`.
      allowJunction: false,
    });
    ctx.intents?.set(relative, { type: 'symlink', target: intent.target });
    if (outcome.realized !== 'symlink') {
      ctx.degradations?.push({
        path: relative,
        intended: outcome.intended,
        realized: outcome.realized,
        reason: outcome.reason ?? null,
      });
    }
    if (durable && outcome.realized === 'copy') syncPath(destination);
    return;
  }

  if (intent.type === 'directory') {
    ensureDirectory(destination);
    for (const name of fs.readdirSync(source).sort()) {
      if (isExcludedPayloadEntry(name, path.join(source, name), root)) continue;
      copyEntry(path.join(source, name), path.join(destination, name), durable, root, ctx);
    }
    applyManifestMode(destination, intent, ctx.capabilities);
    if (relative) ctx.intents?.set(relative, { type: 'directory' });
    if (durable) syncDirectory(destination);
    return;
  }

  ensureDirectory(path.dirname(destination));
  fs.copyFileSync(source, destination, fs.constants.COPYFILE_EXCL);
  applyManifestMode(destination, intent, ctx.capabilities);
  ctx.intents?.set(relative, { type: 'file', executable: Boolean(intent.executable) });
  if (durable) syncPath(destination);
}

function toPosix(value) {
  return value.split(path.sep).join('/');
}

/** Re-apply 0755 to every directory in a staged payload. No-op without modes. */
function normalizeDirectoryModes(root) {
  if (!CAPABILITIES.posixModes) return;
  const walk = directory => {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      if (!entry.isDirectory()) continue;
      const absolute = path.join(directory, entry.name);
      walk(absolute);
      fs.chmodSync(absolute, 0o755);
    }
  };
  walk(root);
  fs.chmodSync(root, 0o755);
}

/** Materialization context for staging a payload out of a source checkout. */
function ingestContext({ repoRoot, destinationRoot, oracle }) {
  return {
    capabilities: CAPABILITIES,
    destinationRoot,
    intents: new Map(),
    degradations: [],
    intentFor: sourcePath =>
      resolveIngestIntent({ sourcePath, repoRoot, oracle, capabilities: CAPABILITIES }),
  };
}

/**
 * Materialization context for projecting an installed generation into a
 * copy-mode platform target. The generation's signed manifest is the authority
 * here, which is what lets a projection carry executable intent onto a volume
 * that cannot represent it.
 */
function projectionContext({ generationPath, destinationRoot, manifest }) {
  const lookup = manifestIntentLookup(manifest.files);
  return {
    capabilities: CAPABILITIES,
    destinationRoot,
    intents: null,
    degradations: [],
    intentFor: sourcePath => {
      const relative = toPosix(path.relative(generationPath, sourcePath));
      const recorded = lookup(relative);
      if (recorded) return recorded;
      const stat = fs.lstatSync(sourcePath);
      if (stat.isDirectory() && !stat.isSymbolicLink()) return { type: 'directory' };
      fail(`projection source is not manifested: ${relative}`);
      return null;
    },
  };
}

/**
 * Schema-1 entry collection, retained verbatim so that generations created by an
 * earlier installer keep verifying under the rules they were signed with. It
 * records a full permission mode, which is exactly the host dependence schema 2
 * removes; it must never be used to CREATE an entry list.
 */
function collectManifestFilesV1(root, relative = '') {
  const result = [];
  const absolute = path.join(root, relative);
  for (const name of fs.readdirSync(absolute).sort()) {
    const itemRelative = path.posix.join(relative.split(path.sep).join('/'), name);
    if (itemRelative === 'manifest.json' || itemRelative === MANIFEST_SIGNATURE) continue;
    const itemAbsolute = path.join(root, ...itemRelative.split('/'));
    const stat = fs.lstatSync(itemAbsolute);
    if (stat.isDirectory()) {
      result.push(...collectManifestFilesV1(root, itemRelative));
    } else if (stat.isSymbolicLink()) {
      const target = fs.readlinkSync(itemAbsolute);
      result.push({
        path: itemRelative,
        type: 'symlink',
        sha256: sha256(target),
        size: Buffer.byteLength(target),
        mode: modeString(stat.mode),
      });
    } else if (stat.isFile()) {
      const bytes = fs.readFileSync(itemAbsolute);
      result.push({
        path: itemRelative,
        type: 'file',
        sha256: sha256(bytes),
        size: bytes.length,
        mode: modeString(stat.mode),
      });
    }
  }
  return result.sort((left, right) => left.path.localeCompare(right.path));
}

function readSkillName(skillFile) {
  const text = fs.readFileSync(skillFile, 'utf8');
  const frontmatter = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  const match = frontmatter?.[1].match(/^name:\s*['\"]?([^'\"\r\n]+)['\"]?/m);
  const name = match?.[1].trim() || path.basename(path.dirname(skillFile));
  if (!name || name === '.' || name === '..' || name.includes('/') || name.includes('\\')) {
    fail(`unsafe skill name in ${skillFile}: ${name}`);
  }
  return name;
}

function readSkillDescription(skillFile) {
  const text = fs.readFileSync(skillFile, 'utf8').replace(/\r\n/g, '\n');
  const frontmatter = text.match(/^---\n([\s\S]*?)\n---/)?.[1] ?? '';
  const inline = frontmatter.match(/^description:\s*(?![>|])['"]?([^'"\n]+)['"]?\s*$/m);
  if (inline) return inline[1].replace(/\s+/g, ' ').trim();
  const folded = frontmatter.match(/^description:\s*[>|][-+]?\s*\n((?:[ \t]+\S[^\n]*\n?)+)/m);
  return (folded?.[1] ?? '')
    .split('\n')
    .map(line => line.trim())
    .filter(Boolean)
    .join(' ');
}

function collectSkills(skillsRoot) {
  const skills = [];
  const immediateSkillDirectories = fs
    .readdirSync(skillsRoot)
    .sort()
    .filter(name => {
      const entry = path.join(skillsRoot, name);
      return fs.lstatSync(entry).isDirectory() && fs.existsSync(path.join(entry, 'SKILL.md'));
    });
  if (immediateSkillDirectories.length) {
    for (const name of immediateSkillDirectories) {
      const skillFile = path.join(skillsRoot, name, 'SKILL.md');
      skills.push({
        name: readSkillName(skillFile),
        description: readSkillDescription(skillFile),
        relative: name,
      });
    }
    skills.sort((left, right) => left.name.localeCompare(right.name));
    return skills;
  }
  function visit(directory) {
    for (const name of fs.readdirSync(directory).sort()) {
      const entry = path.join(directory, name);
      const stat = fs.lstatSync(entry);
      if (!stat.isDirectory() || stat.isSymbolicLink()) continue;
      const relative = path.relative(skillsRoot, entry).split(path.sep);
      if (relative.some(part => ['imported', 'tests', 'fixtures'].includes(part))) continue;
      const skillFile = path.join(entry, 'SKILL.md');
      if (fs.existsSync(skillFile))
        skills.push({
          name: readSkillName(skillFile),
          description: readSkillDescription(skillFile),
          relative: path.relative(skillsRoot, entry).split(path.sep).join('/'),
        });
      visit(entry);
    }
  }
  visit(skillsRoot);
  skills.sort((left, right) => left.name.localeCompare(right.name));
  for (let index = 1; index < skills.length; index += 1) {
    if (skills[index - 1].name === skills[index].name) {
      fail(`duplicate installed skill name: ${skills[index].name}`);
    }
  }
  return skills;
}

function buildSkillIndex(skills) {
  const entries = skills.map(skill => ({
    id: skill.name,
    description: skill.description,
    relativePath: skill.relative,
    searchText: `${skill.name} ${skill.description}`.toLocaleLowerCase('en-US'),
  }));
  const body = { schemaVersion: SKILL_INDEX_SCHEMA, entries };
  return { ...body, sha256: sha256(canonicalJson(body)) };
}

function sourceProvenance(sourceRoot) {
  const git = (...args) => {
    const result = spawnSync('git', args, { cwd: sourceRoot, encoding: 'utf8' });
    return result.status === 0 ? result.stdout.trim() : null;
  };
  const sourceCommit = git('rev-parse', 'HEAD');
  const sourceTreeState = git('status', '--porcelain') ? 'modified' : 'clean';
  const configuredPaths = git('config', '-f', '.gitmodules', '--get-regexp', 'path');
  const externalSources = [];
  if (configuredPaths) {
    for (const line of configuredPaths.split('\n').filter(Boolean)) {
      const sourcePath = line.trim().split(/\s+/).at(-1);
      const treeEntry = git('ls-tree', 'HEAD', '--', sourcePath);
      const match = treeEntry?.match(/^160000 commit ([a-f0-9]{40,64})\t/);
      if (!match) fail(`external source has no commit gitlink: ${sourcePath}`);
      const checkedOutCommit = fs.existsSync(path.join(sourceRoot, sourcePath, '.git'))
        ? git('-C', sourcePath, 'rev-parse', 'HEAD')
        : null;
      if (checkedOutCommit && checkedOutCommit !== match[1]) {
        fail(`external source checkout differs from its commit pin: ${sourcePath}`);
      }
      externalSources.push({ path: sourcePath, commit: match[1] });
    }
  }
  externalSources.sort((left, right) => left.path.localeCompare(right.path));
  return { sourceCommit, sourceTreeState, externalSources };
}

function validateSourceProvenance(provenance, args, stage) {
  if (args.expectedSourceCommit && provenance.sourceCommit !== args.expectedSourceCommit) {
    fail(
      `source commit mismatch ${stage}: expected ${args.expectedSourceCommit}, ` +
        `found ${provenance.sourceCommit ?? 'unavailable'} at ${args.sourceRoot}`
    );
  }
  if (args.requireCleanSource && provenance.sourceTreeState !== 'clean') {
    fail(`source tree is modified ${stage}: ${args.sourceRoot}`);
  }
}

function sameExternalSources(left, right) {
  return canonicalJson(left) === canonicalJson(right);
}

function stageExecutionComponent(source, staging, expectedRelease, ingest) {
  const descriptorPath = path.join(source, ...EXEC_COMPONENT_DESCRIPTOR.split('/'));
  if (!fs.existsSync(descriptorPath)) {
    fail(`execution component descriptor is missing: ${EXEC_COMPONENT_DESCRIPTOR}`);
  }
  const descriptor = JSON.parse(fs.readFileSync(descriptorPath, 'utf8'));
  if (
    descriptor.schemaVersion !== 1 ||
    descriptor.release !== expectedRelease ||
    !/^[a-z0-9][a-z0-9-]*$/.test(descriptor.componentId ?? '') ||
    descriptor.world !== 'prometheus:component@0.1.0' ||
    !/^[a-f0-9]{64}$/.test(descriptor.sha256 ?? '') ||
    !Number.isSafeInteger(descriptor.sizeBytes) ||
    canonicalJson(descriptor.capabilities?.hostImports) !== canonicalJson(EXEC_HOST_IMPORTS) ||
    canonicalJson(descriptor.capabilities?.wasiAdapterImports) !== canonicalJson(EXEC_WASI_IMPORTS)
  ) {
    fail('execution component descriptor is invalid');
  }
  const sourceRelative = path.posix.normalize(descriptor.sourcePath ?? '');
  if (
    !sourceRelative ||
    sourceRelative !== descriptor.sourcePath ||
    sourceRelative.startsWith('../') ||
    path.isAbsolute(sourceRelative)
  ) {
    fail('execution component source path is unsafe');
  }
  const sourceArtifact = path.join(source, ...sourceRelative.split('/'));
  const artifactBytes = fs.readFileSync(sourceArtifact);
  if (
    sha256(artifactBytes) !== descriptor.sha256 ||
    artifactBytes.length !== descriptor.sizeBytes
  ) {
    fail('execution component bytes differ from the checked capability descriptor');
  }
  const stagedArtifact = path.join(staging, ...sourceRelative.split('/'));
  if (!fs.existsSync(stagedArtifact))
    copyEntry(sourceArtifact, stagedArtifact, true, source, ingest);

  const componentRoot = path.posix.join('components', descriptor.componentId);
  const artifactPath = sourceRelative;
  const capabilityMetadataPath = path.posix.join(componentRoot, 'component.json');
  const descriptorBytes = Buffer.from(canonicalJson(descriptor));
  atomicWrite(path.join(staging, ...capabilityMetadataPath.split('/')), descriptorBytes);

  const componentEntry = {
    componentId: descriptor.componentId,
    world: descriptor.world,
    artifactPath,
    sha256: descriptor.sha256,
    sizeBytes: descriptor.sizeBytes,
    capabilityMetadataPath,
    capabilityMetadataSha256: sha256(descriptorBytes),
  };
  const indexBody = { schemaVersion: COMPONENT_INDEX_SCHEMA, components: [componentEntry] };
  const componentIndex = { ...indexBody, sha256: sha256(canonicalJson(indexBody)) };
  const indexBytes = Buffer.from(canonicalJson(componentIndex));
  for (const relative of [
    'indexes/components.json',
    'agents/component-index.json',
    'mobile/component-index.json',
  ]) {
    atomicWrite(path.join(staging, ...relative.split('/')), indexBytes);
  }
  return {
    ...componentEntry,
    indexPath: 'indexes/components.json',
    indexSha256: sha256(indexBytes),
  };
}

function stagePluginMetadata(source, staging, expectedRelease, ingest) {
  let found = false;
  for (const [platform, manifest] of [
    ['claude', '.claude-plugin/plugin.json'],
    ['codex', '.codex-plugin/plugin.json'],
  ]) {
    const repositoryPath = path.join(
      source,
      'dist/plugins',
      platform,
      'prometheus-skill-pack',
      manifest
    );
    const packagedPath = path.join(source, manifest);
    const selected = fs.existsSync(repositoryPath)
      ? repositoryPath
      : fs.existsSync(packagedPath)
        ? packagedPath
        : null;
    if (!selected) continue;
    const metadata = JSON.parse(fs.readFileSync(selected, 'utf8'));
    if (metadata.version !== expectedRelease) {
      fail(
        `${platform} plugin version ${metadata.version ?? 'missing'} differs from contract ${expectedRelease}`
      );
    }
    const destination = path.join(staging, manifest);
    if (!fs.existsSync(destination)) copyEntry(selected, destination, true, source, ingest);
    found = true;
  }
  if (!found) fail('source payload has no Claude or Codex plugin metadata');
}

function targetPayloads(skills, executionComponent) {
  const digest = sha256(canonicalJson(skills));
  return TARGETS.map(target => ({
    target,
    mode: COPY_TARGETS.has(target) ? 'copy' : 'symlink',
    skillCount: skills.length,
    skillsSha256: digest,
    executionComponentSha256: executionComponent.sha256,
    executionCapabilitySha256: executionComponent.capabilityMetadataSha256,
    executionComponentIndexSha256: executionComponent.indexSha256,
  }));
}

function verifyExecutionComponent(generationPath, manifest) {
  const component = manifest.executionComponent;
  if (
    component?.world !== 'prometheus:component@0.1.0' ||
    !/^[a-f0-9]{64}$/.test(component?.sha256 ?? '') ||
    !Number.isSafeInteger(component?.sizeBytes) ||
    !/^[a-f0-9]{64}$/.test(component?.capabilityMetadataSha256 ?? '') ||
    !/^[a-f0-9]{64}$/.test(component?.indexSha256 ?? '')
  ) {
    fail('generation execution component receipt is invalid');
  }
  const safeFile = relative => {
    const normalized = path.posix.normalize(relative ?? '');
    if (
      !normalized ||
      normalized !== relative ||
      normalized.startsWith('../') ||
      path.isAbsolute(normalized)
    ) {
      fail('generation execution component path is unsafe');
    }
    return path.join(generationPath, ...normalized.split('/'));
  };
  const artifactBytes = fs.readFileSync(safeFile(component.artifactPath));
  if (sha256(artifactBytes) !== component.sha256 || artifactBytes.length !== component.sizeBytes) {
    fail('generation execution component artifact is invalid');
  }
  const descriptorBytes = fs.readFileSync(safeFile(component.capabilityMetadataPath));
  if (sha256(descriptorBytes) !== component.capabilityMetadataSha256) {
    fail('generation execution capability metadata is invalid');
  }
  const descriptor = JSON.parse(descriptorBytes);
  if (
    descriptor.schemaVersion !== 1 ||
    descriptor.release !== manifest.sourceVersion ||
    !/^[a-z0-9][a-z0-9-]*$/.test(component.componentId ?? '') ||
    descriptor.componentId !== component.componentId ||
    descriptor.world !== component.world ||
    descriptor.sha256 !== component.sha256 ||
    descriptor.sizeBytes !== component.sizeBytes ||
    canonicalJson(descriptor.capabilities?.hostImports) !== canonicalJson(EXEC_HOST_IMPORTS) ||
    canonicalJson(descriptor.capabilities?.wasiAdapterImports) !== canonicalJson(EXEC_WASI_IMPORTS)
  ) {
    fail('generation execution capability metadata does not match its receipt');
  }
  if (descriptor.sourcePath !== component.artifactPath) {
    fail('generation skill component path and execution receipt diverge');
  }
  const indexPaths = [
    component.indexPath,
    'agents/component-index.json',
    'mobile/component-index.json',
  ];
  const indexBytes = fs.readFileSync(safeFile(indexPaths[0]));
  if (
    sha256(indexBytes) !== component.indexSha256 ||
    !indexBytes.equals(fs.readFileSync(safeFile(indexPaths[1]))) ||
    !indexBytes.equals(fs.readFileSync(safeFile(indexPaths[2])))
  ) {
    fail('host/generated-agent/mobile execution component indexes diverge');
  }
  const index = JSON.parse(indexBytes);
  const indexBody = { schemaVersion: index.schemaVersion, components: index.components };
  const expectedEntry = {
    componentId: component.componentId,
    world: component.world,
    artifactPath: component.artifactPath,
    sha256: component.sha256,
    sizeBytes: component.sizeBytes,
    capabilityMetadataPath: component.capabilityMetadataPath,
    capabilityMetadataSha256: component.capabilityMetadataSha256,
  };
  if (
    index.schemaVersion !== COMPONENT_INDEX_SCHEMA ||
    index.sha256 !== sha256(canonicalJson(indexBody)) ||
    canonicalJson(index.components) !== canonicalJson([expectedEntry])
  ) {
    fail('generation execution component index is invalid');
  }
}

/**
 * Verify the release manifest that fixes the bundle identity.
 *
 * `executableOf(path)` returns the recorded executable intent for a runtime file
 * or null when this caller has no authority to check it. The schema-2 release
 * manifest records that boolean instead of a mode string, for the same reason
 * the generation manifest does: a mode is a property of the host that wrote the
 * file, and folding it into the bundle identity made the identity host-dependent
 * and umask-dependent at once.
 *
 * Schema-1 release manifests are still accepted, under their original rules --
 * which include the full mode comparison. That comparison is only meaningful on
 * a volume with POSIX mode semantics, and rather than skip it there (which would
 * silently weaken the check for an old generation) this refuses to verify one on
 * a volume that cannot represent the mode it recorded.
 */
/**
 * The subset of a manifest that IS the generation identity.
 *
 * Identical field list for both schema versions; what differs is the
 * canonicalization used to digest it, and what the `files` entries contain.
 */
/**
 * Executable intent for a payload path, read from the signed manifest.
 *
 * Returns null for a schema-1 manifest, which records a mode rather than a
 * normalized bit; schema-1 callers keep their original mode comparison.
 */
function manifestExecutableLookup(manifest) {
  if (manifest.schemaVersion === 1) {
    const modes = new Map(manifest.files.map(entry => [entry.path, entry.mode]));
    return relative => {
      const mode = modes.get(relative);
      return mode === undefined ? null : (Number.parseInt(mode, 8) & 0o111) !== 0;
    };
  }
  const index = new Map(manifest.files.map(entry => [entry.path, entry]));
  return relative => {
    const entry = index.get(relative);
    return entry && entry.type === 'file' ? Boolean(entry.executable) : null;
  };
}

function generationIdentity(manifest) {
  return {
    schemaVersion: manifest.schemaVersion,
    sourceVersion: manifest.sourceVersion,
    signerKeyId: manifest.signerKeyId,
    bundleId: manifest.bundleId,
    hookRuntime: manifest.hookRuntime,
    sourceProvenance: manifest.sourceProvenance,
    skillIndex: manifest.skillIndex,
    executionComponent: manifest.executionComponent,
    files: manifest.files,
    targetPayloads: manifest.targetPayloads,
  };
}

/**
 * Schema 1 digests the pretty-printed sorted-key form it was signed with, so
 * every generation already on disk keeps its name. Schema 2 digests RFC 8785,
 * which a non-JavaScript verifier can reproduce.
 */
function generationDigest(manifest) {
  const identity = generationIdentity(manifest);
  return manifest.schemaVersion === 1 ? sha256(canonicalJson(identity)) : sha256(jcs(identity));
}

function verifyReleaseManifest(payloadRoot, expectedBundle = null, executableOf = () => null) {
  const manifestPath = path.join(payloadRoot, 'shared/harnesses/generated/release-manifest.json');
  if (!fs.existsSync(manifestPath)) fail(`release manifest is missing: ${manifestPath}`);
  const release = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  // The identity is reconstructed field by field rather than hashing the file,
  // so a manifest carrying an extra unhashed field cannot pass. That means this
  // list has to track the generator exactly: schema 2 added the dispatcher
  // interpreter, and schema 1 never had it.
  const identity = {
    schemaVersion: release.schemaVersion,
    sourceVersion: release.sourceVersion,
    contractSchemaVersion: release.contractSchemaVersion,
    dispatcherAbi: release.dispatcherAbi,
    ...(release.schemaVersion === 1
      ? {}
      : { dispatcherInterpreter: release.dispatcherInterpreter }),
    contractSha256: release.contractSha256,
    runtimeFiles: release.runtimeFiles,
  };
  const bundleId = sha256(canonicalJson(identity));
  if (release.bundleId !== bundleId) fail('release manifest bundle identity mismatch');
  if (expectedBundle && bundleId !== expectedBundle) {
    fail(`release bundle ${bundleId} does not match expected ${expectedBundle}`);
  }
  if (
    release.dispatcherAbi !== 'hook-runtime-v1' ||
    !Array.isArray(release.runtimeFiles) ||
    release.runtimeFiles.length === 0
  ) {
    fail('release manifest hook runtime contract is invalid');
  }
  if (![1, RELEASE_MANIFEST_SCHEMA_VERSION].includes(release.schemaVersion)) {
    fail(`unsupported release manifest schema: ${release.schemaVersion ?? 'missing'}`);
  }
  const legacyModes = release.schemaVersion === 1;
  const seen = new Set();
  for (const entry of release.runtimeFiles) {
    const normalized = path.posix.normalize(entry.path ?? '');
    if (
      !normalized ||
      normalized !== entry.path ||
      normalized.startsWith('../') ||
      path.isAbsolute(normalized) ||
      seen.has(normalized)
    ) {
      fail(`unsafe or duplicate release payload path: ${entry.path}`);
    }
    seen.add(normalized);
    const absolute = path.join(payloadRoot, ...normalized.split('/'));
    const stat = fs.lstatSync(absolute, { throwIfNoEntry: false });
    const actualHash = stat?.isFile() ? sha256(fs.readFileSync(absolute)) : 'missing';
    if (!stat?.isFile() || actualHash !== entry.sha256) {
      fail(
        `release payload verification failed for ${normalized}: ` +
          `expected sha256=${entry.sha256}; actual sha256=${actualHash}; ` +
          `payloadRoot=${payloadRoot}; manifest=${manifestPath}`
      );
    }
    if (legacyModes) {
      if (!CAPABILITIES.posixModes) {
        fail(
          `release manifest schema ${release.schemaVersion} records permission modes, which cannot ` +
            `be verified on a volume without POSIX mode semantics: ${manifestPath}`
        );
      }
      const actualMode = modeString(stat.mode);
      if (actualMode !== entry.mode) {
        fail(
          `release payload mode verification failed for ${normalized}: ` +
            `expected mode=${entry.mode}; actual mode=${actualMode}; manifest=${manifestPath}`
        );
      }
      continue;
    }
    if (typeof entry.executable !== 'boolean') {
      fail(`release payload entry records no executable bit: ${normalized}`);
    }
    const recorded = executableOf(normalized);
    if (recorded !== null && recorded !== entry.executable) {
      fail(
        `release payload executable bit disagrees with the payload manifest for ${normalized}: ` +
          `release manifest says ${entry.executable}, payload says ${recorded}`
      );
    }
    if (CAPABILITIES.executableBit) {
      const observed = (stat.mode & 0o100) !== 0;
      if (observed !== entry.executable) {
        fail(
          `release payload executable bit verification failed for ${normalized}: ` +
            `expected ${entry.executable}; observed ${observed}; manifest=${manifestPath}`
        );
      }
    }
  }
  return release;
}

function stageReleaseRuntimeSupport(source, staging, ingest) {
  const file = path.join(source, 'shared/harnesses/generated/release-manifest.json');
  const release = JSON.parse(fs.readFileSync(file, 'utf8'));
  for (const entry of release.runtimeFiles ?? []) {
    const relative = path.posix.normalize(entry.path ?? '');
    if (!relative.startsWith('skills/')) continue;
    const destination = path.join(staging, ...relative.split('/'));
    if (!fs.existsSync(destination))
      copyEntry(path.join(source, ...relative.split('/')), destination, true, source, ingest);
  }
}

function verifyGeneration(
  generationPath,
  expectedName = path.basename(generationPath),
  trustStorePath = path.join(
    path.dirname(path.dirname(generationPath)),
    'trust/allowed-signers.json'
  )
) {
  const manifestPath = path.join(generationPath, 'manifest.json');
  if (!fs.existsSync(manifestPath)) fail(`generation has no manifest: ${generationPath}`);
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  if (![1, MANIFEST_SCHEMA_VERSION].includes(manifest.schemaVersion)) {
    fail(`unsupported generation manifest schema: ${manifest.schemaVersion ?? 'missing'}`);
  }
  const digest = generationDigest(manifest);
  if (manifest.generation !== digest || expectedName !== digest)
    fail(`generation identity mismatch: ${generationPath}`);
  const signaturePath = path.join(generationPath, MANIFEST_SIGNATURE);
  if (!fs.existsSync(signaturePath)) fail(`generation signature is missing: ${generationPath}`);
  const signerKeyId = verifySignedValue(
    manifest,
    JSON.parse(fs.readFileSync(signaturePath, 'utf8')),
    trustStorePath
  );
  if (manifest.signerKeyId !== signerKeyId) fail('manifest signer does not match its signature');
  if (!Array.isArray(manifest.targetPayloads) || manifest.targetPayloads.length !== TARGETS.length)
    fail('generation does not certify all 14 targets');
  verifyExecutionComponent(generationPath, manifest);
  for (let index = 0; index < TARGETS.length; index += 1) {
    const payload = manifest.targetPayloads[index];
    const wantedMode = COPY_TARGETS.has(TARGETS[index]) ? 'copy' : 'symlink';
    if (
      payload?.target !== TARGETS[index] ||
      payload?.mode !== wantedMode ||
      payload?.executionComponentSha256 !== manifest.executionComponent.sha256 ||
      payload?.executionCapabilitySha256 !== manifest.executionComponent.capabilityMetadataSha256 ||
      payload?.executionComponentIndexSha256 !== manifest.executionComponent.indexSha256
    ) {
      fail('generation target payload matrix does not match the canonical 14 targets');
    }
  }
  if (manifest.schemaVersion === 1) {
    if (!CAPABILITIES.posixModes) {
      fail(
        `a schema-1 generation records permission modes and cannot be verified on a volume ` +
          `without POSIX mode semantics: ${generationPath}`
      );
    }
    if (canonicalJson(collectManifestFilesV1(generationPath)) !== canonicalJson(manifest.files)) {
      fail(`generation contains unmanifested or missing payload files: ${generationPath}`);
    }
  } else if (
    !entriesEqual(
      collectPayloadEntries(generationPath, manifestIntentLookup(manifest.files), CAPABILITIES),
      manifest.files
    )
  ) {
    fail(`generation contains unmanifested or missing payload files: ${generationPath}`);
  }
  const indexPath = path.join(generationPath, 'indexes/skills.json');
  const mobilePath = path.join(generationPath, 'mobile/skill-index.json');
  const agentPath = path.join(generationPath, 'agents/skill-index.json');
  const parityPath = path.join(generationPath, 'mobile/parity.json');
  if (
    !fs.existsSync(indexPath) ||
    !fs.existsSync(mobilePath) ||
    !fs.existsSync(agentPath) ||
    !fs.existsSync(parityPath)
  ) {
    fail('generation omits a host, generated-agent, or mobile skill-index projection');
  }
  const indexBytes = fs.readFileSync(indexPath);
  if (
    !indexBytes.equals(fs.readFileSync(mobilePath)) ||
    !indexBytes.equals(fs.readFileSync(agentPath))
  ) {
    fail('host/generated-agent/mobile skill indexes diverge');
  }
  const index = JSON.parse(indexBytes);
  const indexBody = { schemaVersion: index.schemaVersion, entries: index.entries };
  if (
    index.schemaVersion !== SKILL_INDEX_SCHEMA ||
    index.sha256 !== sha256(canonicalJson(indexBody)) ||
    manifest.skillIndex?.sha256 !== index.sha256 ||
    manifest.skillIndex?.entryCount !== index.entries.length
  ) {
    fail('generation skill index receipt is invalid');
  }
  const parity = JSON.parse(fs.readFileSync(parityPath, 'utf8'));
  if (
    parity.schemaVersion !== 1 ||
    parity.skillIndexSchema !== SKILL_INDEX_SCHEMA ||
    parity.skillIndexSha256 !== index.sha256 ||
    parity.entryCount !== index.entries.length
  ) {
    fail('mobile skill-index parity receipt is invalid');
  }
  if (manifest.schemaVersion === 1) {
    for (const entry of manifest.files) {
      const absolute = path.join(generationPath, ...entry.path.split('/'));
      const stat = fs.lstatSync(absolute);
      const bytes = stat.isSymbolicLink()
        ? Buffer.from(fs.readlinkSync(absolute))
        : fs.readFileSync(absolute);
      const type = stat.isSymbolicLink() ? 'symlink' : 'file';
      if (
        type !== entry.type ||
        sha256(bytes) !== entry.sha256 ||
        bytes.length !== entry.size ||
        modeString(stat.mode) !== entry.mode
      ) {
        fail(`generation verification failed for ${entry.path}`);
      }
      if (
        stat.isSymbolicLink() &&
        !isWithin(generationPath, path.resolve(path.dirname(absolute), fs.readlinkSync(absolute)))
      ) {
        fail(`generation symlink escapes the immutable payload: ${entry.path}`);
      }
    }
  }
  // Schema 2 verified every entry's bytes, type, realization, and link
  // containment inside collectPayloadEntries above; repeating a mode comparison
  // here is exactly the host dependence that was removed.

  // Executability comes from the manifest, never from the filesystem. On a
  // volume that cannot represent an executable bit, `stat().mode & 0o111` is
  // zero for every file including the dispatcher, so the old gate rejected an
  // otherwise valid generation.
  const executableOf = manifestExecutableLookup(manifest);
  for (const script of REQUIRED_SCRIPTS) {
    const relative = `shared/scripts/${script}`;
    const absolute = path.join(generationPath, ...relative.split('/'));
    if (!fs.existsSync(absolute)) fail(`required script is missing: ${script}`);
    if (executableOf(relative) !== true) {
      fail(`required script is not recorded as executable: ${script}`);
    }
  }
  const release = verifyReleaseManifest(generationPath, manifest.bundleId, executableOf);
  const dispatcher = release.runtimeFiles.find(
    entry => entry.path === 'shared/scripts/generated/hook-dispatch-v1.sh'
  );
  const runner = release.runtimeFiles.find(
    entry => entry.path === 'shared/scripts/hook-runtime-v1.sh'
  );
  if (
    manifest.hookRuntime?.abi !== 'hook-runtime-v1' ||
    manifest.hookRuntime?.bundleId !== manifest.bundleId ||
    manifest.hookRuntime?.dispatcherPath !== 'shared/scripts/generated/hook-dispatch-v1.sh' ||
    manifest.hookRuntime?.dispatcherSha256 !== dispatcher?.sha256 ||
    manifest.hookRuntime?.runnerSha256 !== runner?.sha256
  ) {
    fail('generation hook runtime receipt is invalid');
  }
  // Schema-1 generations predate the recorded interpreter and keep launching by
  // shebang; schema 2 must name it, and it must agree with the release identity.
  if (manifest.schemaVersion !== 1) {
    if (manifest.hookRuntime.dispatcherInterpreter !== release.dispatcherInterpreter) {
      fail('generation dispatcher interpreter does not match the release identity');
    }
    if (manifest.hookRuntime.dispatcherInterpreter !== 'bash') {
      fail('generation dispatcher interpreter is not allowlisted');
    }
  }
  if (
    !manifest.files.some(
      entry => /(^|\/)tests?\//.test(entry.path) || /(^|\/)fixtures?\//.test(entry.path)
    )
  ) {
    fail('generation contains no regression fixture or test');
  }
  return manifest;
}

/**
 * The active target for `current` or `previous`.
 *
 * The pointer FILE is consulted first because it is the record of truth. The
 * link is read only when no pointer file exists, which is exactly a store
 * written by an installer that predates them -- reading it keeps such a store
 * activatable across the upgrade instead of invalidating it.
 *
 * A junction's readlink returns an ABSOLUTE substitute name where a POSIX
 * symlink returns the relative text it was created with, so the legacy value is
 * reduced back to `generations/<id>` rather than returned raw.
 */
function currentTarget(pluginRoot, name) {
  const pointer = readPointer(pluginRoot, name);
  if (pointer) return pointer;
  const link = path.join(pluginRoot, name);
  const stat = fs.lstatSync(link, { throwIfNoEntry: false });
  if (!stat?.isSymbolicLink()) return null;
  const resolved = path.resolve(path.dirname(link), fs.readlinkSync(link));
  if (!isWithin(path.join(pluginRoot, 'generations'), resolved)) return null;
  const generation = path.basename(resolved);
  const legacy = `generations/${generation}`;
  return POINTER_PATTERN.test(legacy) ? legacy : null;
}

// Collisions recorded during a run, reported together at the end.
//
// WHY THIS EXISTS (2026-08-13). When something the installer does not own holds
// a skill's canonical name, placement silently diverts to `prometheus-<name>`
// and `targetDestination()` then re-derives the SAME fallback — so
// `verifyTargets()` validates the renamed path and the run prints
// "Verified immutable generation installed to all supported user targets."
//
// The install succeeds by its own definition while the skill is unreachable at
// the name every tool searches. An operator shipped believing all skills were
// available; a Codex session blocked because `deep-research` was not where it
// looked. Declining to clobber a foreign file is correct. Doing so while
// reporting success is not.
//
// Renaming is now recorded and reported, and the process exits non-zero.
const COLLISIONS = [];

function recordCollision(targetRoot, skill, occupant) {
  // Label the target by its path relative to $HOME (".claude/skills"), not by
  // basename — every target's basename is "skills".
  const target = path.relative(os.homedir(), targetRoot) || targetRoot;
  let detail = 'unknown';
  try {
    const st = fs.lstatSync(occupant);
    if (st.isSymbolicLink()) detail = `symlink -> ${fs.readlinkSync(occupant)}`;
    else if (st.isDirectory())
      detail = fs.existsSync(path.join(occupant, '.git'))
        ? 'foreign git checkout'
        : 'unowned directory';
    else detail = 'file';
    detail += `, mtime ${st.mtime.toISOString().slice(0, 10)}`;
  } catch {
    /* occupant vanished between checks — report what we know */
  }
  COLLISIONS.push({ target, skill: skill.name, path: occupant, detail });
}

// Fail the run if any skill could not be placed at its canonical name.
//
// `--allow-fallback` permits the rename for a genuine third-party conflict, but
// still exits non-zero: a renamed skill is unreachable at the name tools search,
// so the run must never read as success.
function assertNoCollisions(allowFallback) {
  if (COLLISIONS.length === 0) return;
  const label = allowFallback ? 'RENAMED (--allow-fallback)' : 'COLLISION';
  process.stderr.write(
    `\ninstall-plugin-generation: ${COLLISIONS.length} skill(s) could not be installed at their canonical name.\n` +
      `These skills are on disk under a "prometheus-" prefix and are NOT reachable\n` +
      `at the name tools search for.\n\n`
  );
  for (const c of COLLISIONS) {
    process.stderr.write(`  ${label}  ~/${c.target}/${c.skill}\n`);
    process.stderr.write(`            occupied by: ${c.detail}\n`);
    process.stderr.write(`            installed as: prometheus-${c.skill}\n`);
  }
  process.stderr.write(
    `\nRemediation — move each occupant aside, then reinstall:\n` +
      COLLISIONS.map(c => `  mv ~/${c.target}/${c.skill}{,.foreign-$(date +%Y%m%d)}`).join('\n') +
      `\n  node scripts/install.js --scope user\n\n`
  );
  fail(`${COLLISIONS.length} skill(s) not installed at canonical name`);
}

function isManagedSkillLink(destination, pluginRoot, skill) {
  const resolved = path.resolve(path.dirname(destination), fs.readlinkSync(destination));
  if (isWithin(pluginRoot, resolved)) return true;
  const legacySuffix = path.join('skills', skill.relative);
  if (!resolved.endsWith(`${path.sep}${legacySuffix}`)) return false;
  const skillFile = path.join(resolved, 'SKILL.md');
  return fs.existsSync(skillFile) && readSkillName(skillFile) === skill.name;
}

function installLinkTarget(targetRoot, skill, pluginRoot) {
  ensureDirectory(targetRoot);
  let destination = path.join(targetRoot, skill.name);
  const managedTarget = path.join(pluginRoot, 'current/skills', skill.relative);
  const existing = fs.lstatSync(destination, { throwIfNoEntry: false });
  if (
    existing &&
    !(existing.isSymbolicLink() && isManagedSkillLink(destination, pluginRoot, skill))
  ) {
    recordCollision(targetRoot, skill, destination);
    destination = path.join(targetRoot, `prometheus-${skill.name}`);
  }
  const fallback = fs.lstatSync(destination, { throwIfNoEntry: false });
  if (
    fallback &&
    !(fallback.isSymbolicLink() && isManagedSkillLink(destination, pluginRoot, skill))
  )
    fail(`skill target collision: ${destination}`);
  // Link by ABSOLUTE path.
  //
  // This previously stored `path.relative(dirname(destination), managedTarget)`.
  // That is only correct when the platform skills dir sits at a fixed depth
  // below $HOME, and it does not: ~/.opencode/skills is TWO levels deep, so the
  // computed "../../.prometheus/plugins/..." resolved against a SIBLING of $HOME
  // (e.g. ~/.TOOLS/.prometheus/...) instead of $HOME itself. Every one of those
  // links dangled.
  //
  // OpenCode validates each {file:...} command reference at startup and refuses
  // to boot on the first miss, so 145 dangling links did not degrade OpenCode —
  // they stopped it from starting at all:
  //   Error: Configuration is invalid ... bad file reference
  //
  // An absolute target is correct at any depth. The generation store lives under
  // $HOME on every supported platform, so there is no portability argument for
  // keeping these relative.
  //
  // The link is created with an EXPLICIT type. `fs.symlinkSync` with no type
  // autodetects, selects 'dir' for a directory target, and raises EPERM without
  // Developer Mode -- which is what made every one of these fail on Windows.
  // Where a directory symlink is unavailable the ladder falls to a junction,
  // which needs no privilege and which both `isSymbolicLink()` and `test -L`
  // report as a link, so nothing downstream has to know which rung was used.
  removeEntry(destination);
  materializeLink({
    linkPath: destination,
    target: managedTarget,
    capabilities: CAPABILITIES,
    degradedCopy: 'fail',
    kind: 'directory',
  });
}

function isManagedCopy(destination, target) {
  if (fs.existsSync(path.join(destination, '.prometheus-generation'))) return true;
  if (target === '.codex/skills' && fs.existsSync(path.join(destination, '.prometheus-pack')))
    return true;
  if (target === '.minimax/skills') {
    try {
      return (
        JSON.parse(fs.readFileSync(path.join(destination, '_meta.json'), 'utf8')).platform ===
        'minimax'
      );
    } catch {
      return false;
    }
  }
  return false;
}

function copySkill(source, targetRoot, target, skill, generation, projection) {
  ensureDirectory(targetRoot);
  let destination = path.join(targetRoot, skill.name);
  if (fs.existsSync(destination) && !isManagedCopy(destination, target)) {
    recordCollision(targetRoot, skill, destination);
    destination = path.join(targetRoot, `prometheus-${skill.name}`);
  }
  if (fs.existsSync(destination) && !isManagedCopy(destination, target)) {
    fail(`skill target collision: ${destination}`);
  }
  const temporary = path.join(targetRoot, `.${path.basename(destination)}.${process.pid}.tmp`);
  fs.rmSync(temporary, { recursive: true, force: true });
  // The immutable generation was fsynced before activation. Copy-based
  // platform projections are replaceable caches, so fsync only their receipt
  // and parent-directory rename instead of flushing thousands of duplicate files.
  copyEntry(source, temporary, false, source, projection(temporary));
  if (target === '.minimax/skills') {
    atomicWrite(
      path.join(temporary, '_meta.json'),
      canonicalJson({ platform: 'minimax', name: skill.name, generation })
    );
  }
  atomicWrite(path.join(temporary, '.prometheus-generation'), `${generation}\n`);
  const prior = `${destination}.${process.pid}.old`;
  fs.rmSync(prior, { recursive: true, force: true });
  if (fs.existsSync(destination)) fs.renameSync(destination, prior);
  fs.renameSync(temporary, destination);
  fs.rmSync(prior, { recursive: true, force: true });
  syncDirectory(targetRoot);
}

/**
 * True when this target must be materialized as a managed copy.
 *
 * Two targets are ALWAYS copies because the harness requires it. The rest are
 * links, unless the probe says this volume offers no directory link primitive
 * at all -- in which case a copy is the only remaining way to place a skill,
 * and the machinery for it (receipt file, `isManagedCopy`, copy verification,
 * uninstall) already exists because two targets have always used it.
 */
function targetIsCopy(target) {
  return COPY_TARGETS.has(target) || CAPABILITIES.directoryLinkStrategy === 'copy';
}

function installTargets(
  home,
  pluginRoot,
  generationPath,
  generation,
  skills,
  targets = TARGETS,
  manifest = null
) {
  // Copy-mode projections carry executable intent from the signed manifest, so
  // a projection onto a volume without an executable bit records the same
  // intent the generation does instead of silently losing it.
  const projection = destinationRoot =>
    projectionContext({ generationPath, destinationRoot, manifest });
  for (const target of targets) {
    const targetRoot = path.join(home, ...target.split('/'));
    for (const skill of skills) {
      if (targetIsCopy(target))
        copySkill(
          path.join(generationPath, 'skills', skill.relative),
          targetRoot,
          target,
          skill,
          generation,
          projection
        );
      else installLinkTarget(targetRoot, skill, pluginRoot);
    }
  }
}

function targetDestination(targetRoot, target, skill, pluginRoot) {
  const primary = path.join(targetRoot, skill.name);
  const existing = fs.lstatSync(primary, { throwIfNoEntry: false });
  if (!existing) return primary;
  if (existing.isSymbolicLink()) {
    const resolved = path.resolve(path.dirname(primary), fs.readlinkSync(primary));
    if (isWithin(pluginRoot, resolved)) return primary;
  }
  if (isManagedCopy(primary, target)) return primary;
  return path.join(targetRoot, `prometheus-${skill.name}`);
}

function verifyTargets(home, pluginRoot, generationPath, generation, skills, targets = TARGETS) {
  for (const target of targets) {
    const targetRoot = path.join(home, ...target.split('/'));
    for (const skill of skills) {
      const destination = targetDestination(targetRoot, target, skill, pluginRoot);
      if (targetIsCopy(target)) {
        const receipt = path.join(destination, '.prometheus-generation');
        const sourceSkill = path.join(generationPath, 'skills', skill.relative, 'SKILL.md');
        const targetSkill = path.join(destination, 'SKILL.md');
        const minimaxMetadataValid =
          target !== '.minimax/skills' ||
          JSON.parse(fs.readFileSync(path.join(destination, '_meta.json'), 'utf8')).platform ===
            'minimax';
        if (
          !fs.existsSync(receipt) ||
          fs.readFileSync(receipt, 'utf8').trim() !== generation ||
          !fs.existsSync(targetSkill) ||
          !minimaxMetadataValid ||
          sha256(fs.readFileSync(sourceSkill)) !== sha256(fs.readFileSync(targetSkill))
        ) {
          fail(`copy target validation failed: ${target}/${skill.name}`);
        }
      } else {
        const stat = fs.lstatSync(destination, { throwIfNoEntry: false });
        const wanted = path.join(pluginRoot, 'current/skills', skill.relative);
        // Unchanged in strength. A junction satisfies both halves: libuv sets
        // S_IFLNK for any reparse point on lstat, and its readlink returns the
        // absolute substitute name, which resolves to the same place a relative
        // symlink would.
        if (
          !stat?.isSymbolicLink() ||
          path.resolve(path.dirname(destination), fs.readlinkSync(destination)) !== wanted
        ) {
          fail(`symlink target validation failed: ${target}/${skill.name}`);
        }
      }
    }
  }
}

function receiptFile(pluginRoot, generation, target) {
  return path.join(
    pluginRoot,
    'receipts',
    generation,
    `${target.replaceAll('/', '__').replace(/^\./, '')}.json`
  );
}

function targetReceipt(manifest, targetPayload) {
  return {
    schemaVersion: 1,
    generation: manifest.generation,
    signerKeyId: manifest.signerKeyId,
    target: targetPayload.target,
    mode: targetPayload.mode,
    skillCount: targetPayload.skillCount,
    skillsSha256: targetPayload.skillsSha256,
    skillIndexSha256: manifest.skillIndex.sha256,
    executionComponentSha256: targetPayload.executionComponentSha256,
    executionCapabilitySha256: targetPayload.executionCapabilitySha256,
    executionComponentIndexSha256: targetPayload.executionComponentIndexSha256,
    status: 'verified',
  };
}

function writeTargetReceipts(pluginRoot, manifest, identity, targets = TARGETS) {
  const selected = new Set(targets);
  for (const targetPayload of manifest.targetPayloads.filter(entry => selected.has(entry.target))) {
    const body = targetReceipt(manifest, targetPayload);
    atomicWrite(
      receiptFile(pluginRoot, manifest.generation, targetPayload.target),
      canonicalJson({ body, signature: signValue(body, identity) }),
      0o600
    );
  }
}

function verifyTargetReceipts(pluginRoot, manifest, trustStorePath, targets = TARGETS) {
  if (manifest.targetPayloads.length !== TARGETS.length)
    fail('target receipt matrix is incomplete');
  const selected = new Set(targets);
  for (const targetPayload of manifest.targetPayloads.filter(entry => selected.has(entry.target))) {
    const file = receiptFile(pluginRoot, manifest.generation, targetPayload.target);
    if (!fs.existsSync(file)) fail(`target receipt is missing: ${targetPayload.target}`);
    const receipt = JSON.parse(fs.readFileSync(file, 'utf8'));
    const expected = targetReceipt(manifest, targetPayload);
    if (canonicalJson(receipt.body) !== canonicalJson(expected)) {
      fail(`target receipt body mismatch: ${targetPayload.target}`);
    }
    const receiptSigner = verifySignedValue(receipt.body, receipt.signature, trustStorePath);
    if (receiptSigner !== manifest.signerKeyId) {
      fail(`target receipt signer differs from generation signer: ${targetPayload.target}`);
    }
  }
}

// Stable projections are FILE links for the scripts and helpers and a DIRECTORY
// link for `lib`. A volume with no file-symlink primitive degrades the files to
// content copies -- they stay runnable, and `createStableDispatchers` re-runs on
// every install and rollback, which is the only time `current` moves.
function createStableDispatchers(pluginRoot) {
  const degradations = [];
  const project = (name, target, kind) => {
    const outcome = replaceConvenienceLink(pluginRoot, `stable/${name}`, target, 'contents', kind);
    if (outcome.realized !== 'symlink') {
      degradations.push({ path: `stable/${name}`, realized: outcome.realized });
    }
  };
  for (const script of [...STABLE_SCRIPTS, ...STABLE_HELPERS]) {
    project(script, `../current/shared/scripts/${script}`, 'file');
  }
  for (const directory of STABLE_DIRECTORIES) {
    project(directory, `../current/shared/scripts/${directory}`, 'directory');
  }
  project('skill-index.json', '../current/indexes/skills.json', 'file');
  return degradations;
}

/**
 * Assert one stable projection reaches the bytes it is supposed to reach.
 *
 * A link is required whenever the host can make one, so the assertion is
 * unchanged on every link-capable volume. Where the host cannot -- probed, not
 * assumed -- a content copy is accepted and its BYTES are compared against the
 * target, which is a stronger check than the link identity it replaces.
 */
function verifyProjection(pluginRoot, projected, expected, kind) {
  const stat = fs.lstatSync(projected, { throwIfNoEntry: false });
  if (!stat) fail(`stable projection is missing: ${projected}`);
  const linkable =
    kind === 'directory'
      ? CAPABILITIES.symlinkDirectory || CAPABILITIES.junction
      : CAPABILITIES.symlinkFile;
  if (stat.isSymbolicLink()) {
    // A junction reports an absolute substitute name; a symlink reports the
    // relative text. Compare where they land, not how they are spelled.
    const resolved = path.resolve(path.dirname(projected), fs.readlinkSync(projected));
    if (resolved !== path.resolve(expected)) {
      fail(`stable projection resolves elsewhere: ${projected}`);
    }
    const resolvedStat = fs.statSync(projected, { throwIfNoEntry: false });
    if (kind === 'directory' ? !resolvedStat?.isDirectory() : !resolvedStat?.isFile()) {
      fail(`stable projection has the wrong kind: ${projected}`);
    }
    return;
  }
  if (linkable) fail(`stable projection is not a link on a link-capable volume: ${projected}`);
  if (kind === 'directory') fail(`stable directory projection is not a link: ${projected}`);
  if (!stat.isFile()) fail(`stable projection is not a file: ${projected}`);
  if (!fs.readFileSync(projected).equals(fs.readFileSync(expected))) {
    fail(`degraded stable projection does not match its target: ${projected}`);
  }
}

/**
 * Executability comes from the signed manifest, never from the filesystem.
 *
 * `stat().mode & 0o111` is zero for every file on a volume that cannot record a
 * permission bit, so the old check rejected an otherwise valid projection of a
 * script that git records as 100755.
 */
function verifyStableDispatchers(pluginRoot, manifest) {
  const stable = path.join(pluginRoot, 'stable');
  const executableOf = manifestExecutableLookup(manifest);
  for (const script of [...STABLE_SCRIPTS, ...STABLE_HELPERS]) {
    verifyProjection(
      pluginRoot,
      path.join(stable, script),
      path.join(pluginRoot, 'current/shared/scripts', script),
      'file'
    );
    if (executableOf(`shared/scripts/${script}`) !== true) {
      fail(`stable script is not recorded as executable: ${script}`);
    }
  }
  for (const directory of STABLE_DIRECTORIES) {
    verifyProjection(
      pluginRoot,
      path.join(stable, directory),
      path.join(pluginRoot, 'current/shared/scripts', directory),
      'directory'
    );
  }
  verifyProjection(
    pluginRoot,
    path.join(stable, 'skill-index.json'),
    path.join(pluginRoot, 'current/indexes/skills.json'),
    'file'
  );
}

/**
 * Resolve a bundle to its generation directory, pointer file first.
 *
 * The containment check is the one assertion that must not move: whichever
 * mechanism named the generation, the result has to live inside the generation
 * store. `isWithin` reduces a verbatim `\\?\C:\...` spelling to the ordinary
 * one on both sides first, so the guard fires on a real escape and not on a
 * path that merely came back from a different API.
 */
function resolveBundleIndex(pluginRoot, bundleId, label) {
  const generations = path.join(pluginRoot, 'generations');
  const name = `bundles/${bundleId}`;
  const pointer = readPointer(pluginRoot, name);
  const link = path.join(pluginRoot, name);
  const stat = fs.lstatSync(link, { throwIfNoEntry: false });
  let resolved = null;
  if (pointer) {
    resolved = path.join(pluginRoot, ...pointer.split('/'));
    // The link is a convenience now. Its absence is normal on a volume with no
    // directory link primitive, and a non-link occupying its name is worth
    // reporting but is not an activation failure.
    if (stat && !stat.isSymbolicLink()) {
      process.stderr.write(
        `install-plugin-generation: ${label} ${bundleId} is present but is not a link\n`
      );
    }
  } else if (stat?.isSymbolicLink()) {
    resolved = path.resolve(path.dirname(link), fs.readlinkSync(link));
  } else {
    fail(`${label} is missing: ${bundleId}`);
  }
  if (!isWithin(generations, resolved)) fail(`${label} escapes generations: ${bundleId}`);
  const realized = fs.realpathSync(resolved);
  if (!isWithin(generations, realized)) fail(`${label} escapes generations: ${bundleId}`);
  return realized;
}

function verifyHookRuntime(
  pluginRoot,
  manifest,
  trustStorePath = path.join(pluginRoot, 'trust/allowed-signers.json')
) {
  const runner = path.join(pluginRoot, 'runtime/v1/run-hook');
  const runnerStat = fs.lstatSync(runner, { throwIfNoEntry: false });
  if (
    !runnerStat?.isFile() ||
    sha256(fs.readFileSync(runner)) !== manifest.hookRuntime.runnerSha256
  ) {
    fail('stable hook runtime v1 is missing or invalid');
  }
  if (manifestExecutableLookup(manifest)('shared/scripts/hook-runtime-v1.sh') !== true) {
    fail('stable hook runtime v1 is not recorded as executable');
  }
  const resolved = resolveBundleIndex(pluginRoot, manifest.bundleId, 'bundle index');
  const indexed = verifyGeneration(resolved, path.basename(resolved), trustStorePath);
  if (
    indexed.bundleId !== manifest.bundleId ||
    indexed.hookRuntime.dispatcherSha256 !== manifest.hookRuntime.dispatcherSha256
  ) {
    fail(`bundle index collision: ${manifest.bundleId}`);
  }
}

function validateBundleIndex(pluginRoot, generationPath, manifest, trustStorePath) {
  if (!isWithin(path.join(pluginRoot, 'generations'), generationPath)) {
    fail(`bundle generation escapes immutable storage: ${manifest.bundleId}`);
  }
  const name = `bundles/${manifest.bundleId}`;
  if (
    !readPointer(pluginRoot, name) &&
    !fs.lstatSync(path.join(pluginRoot, name), { throwIfNoEntry: false })
  ) {
    return;
  }
  const resolved = resolveBundleIndex(pluginRoot, manifest.bundleId, 'bundle index');
  const indexed = verifyGeneration(resolved, path.basename(resolved), trustStorePath);
  if (
    indexed.bundleId !== manifest.bundleId ||
    indexed.hookRuntime.dispatcherSha256 !== manifest.hookRuntime.dispatcherSha256
  ) {
    fail(`bundle identity collision: ${manifest.bundleId}`);
  }
}

/**
 * Place the compiled hook dispatcher for THIS machine, if the payload has one.
 *
 * The target is chosen by EXECUTION, not by `process.platform` + `process.arch`.
 * Those two answer "what was Node built for", which is not the same question as
 * "what machine code will run here": an arm64 Windows host runs an x64 binary
 * under emulation, and a musl container reports `linux` while refusing a glibc
 * build. Running a candidate's `--version` asks the operating system the actual
 * question, and costs one process per candidate once per install.
 *
 * The candidate is probed AT ITS INSTALLED PATH rather than inside the
 * generation. `CreateProcess` still enforces MAX_PATH on the executable it is
 * given -- a generation directory is a 64-character digest nested under the
 * store, which took a real payload here to 326 characters and made every probe
 * fail with ENOENT even though the bytes were sound and ran from a short path.
 * `runtime/v1/` is short and fixed, and it is where the binary actually runs,
 * so probing there tests the thing that matters instead of a copy of it.
 *
 * Absence is not an error. `hook-entry.mjs` falls back to the shell runtime, so
 * a payload with no binary for this target is slower, not broken.
 */
function installHookDispatcher(pluginRoot, generationPath, manifest) {
  const binRoot = path.join(generationPath, 'bin');
  if (!fs.existsSync(binRoot)) return null;
  const executableOf = manifestExecutableLookup(manifest);
  for (const target of fs.readdirSync(binRoot).sort()) {
    for (const name of ['prometheus-hook', 'prometheus-hook.exe']) {
      const relative = `bin/${target}/${name}`;
      const candidate = path.join(generationPath, ...relative.split('/'));
      if (!fs.existsSync(candidate)) continue;
      // Only a manifested entry the signed manifest calls executable is ever run.
      if (executableOf(relative) !== true) continue;
      const installed = path.join(pluginRoot, 'runtime/v1', name);
      atomicWrite(installed, fs.readFileSync(candidate), 0o755);
      const probe = spawnSync(installed, ['--version'], { encoding: 'utf8', shell: false });
      if (!probe.error && probe.status === 0) {
        return { target, name, version: probe.stdout.trim() };
      }
      // Wrong architecture, wrong libc, or an unusable image: leave nothing
      // behind that `hook-entry.mjs` would later try to exec.
      fs.rmSync(installed, { force: true });
    }
  }
  return null;
}

function installHookRuntime(pluginRoot, generationPath, manifest, trustStorePath) {
  validateBundleIndex(pluginRoot, generationPath, manifest, trustStorePath);
  const runnerSource = path.join(generationPath, 'shared/scripts/hook-runtime-v1.sh');
  atomicWrite(path.join(pluginRoot, 'runtime/v1/run-hook'), fs.readFileSync(runnerSource), 0o755);
  installHookDispatcher(pluginRoot, generationPath, manifest);
  const name = `bundles/${manifest.bundleId}`;
  const target = `generations/${manifest.generation}`;
  if (readPointer(pluginRoot, name) !== target) {
    setActivationPointer(pluginRoot, name, target, `../generations/${manifest.generation}`);
  }
  verifyHookRuntime(pluginRoot, manifest, trustStorePath);
}

function uninstall(home, pluginRoot, targets = TARGETS, removePluginRoot = true) {
  for (const target of targets) {
    const targetRoot = path.join(home, ...target.split('/'));
    if (!fs.existsSync(targetRoot)) continue;
    for (const name of fs.readdirSync(targetRoot)) {
      const destination = path.join(targetRoot, name);
      const stat = fs.lstatSync(destination, { throwIfNoEntry: false });
      if (
        stat?.isSymbolicLink() &&
        isWithin(pluginRoot, path.resolve(targetRoot, fs.readlinkSync(destination)))
      ) {
        fs.unlinkSync(destination);
      } else if (stat?.isDirectory() && isManagedCopy(destination, target)) {
        fs.rmSync(destination, { recursive: true, force: true });
      }
    }
    syncDirectory(targetRoot);
  }
  if (removePluginRoot && fs.existsSync(pluginRoot)) {
    fs.rmSync(pluginRoot, { recursive: true, force: true });
    syncDirectory(path.dirname(pluginRoot));
  }
  return 'uninstalled';
}

function verifyActive(
  pluginRoot,
  trustStorePath = path.join(pluginRoot, 'trust/allowed-signers.json'),
  contract = null,
  targets = TARGETS
) {
  const target = currentTarget(pluginRoot, 'current');
  if (!target) fail('no active plugin generation');
  const resolved = path.resolve(pluginRoot, target);
  if (!isWithin(path.join(pluginRoot, 'generations'), resolved))
    fail('active pointer escapes generations directory');
  const manifest = verifyGeneration(resolved, path.basename(resolved), trustStorePath);
  if (contract) assertMinimumActiveVersion(manifest.sourceVersion, contract, 'active generation');
  // The pointer file is authoritative, but the stable projections resolve
  // THROUGH the convenience link. If a swap was interrupted after the pointer
  // moved and before the link was recreated, say so -- otherwise the first
  // projection to be read fails with a bare ENOENT naming an arbitrary script.
  const currentLink = path.join(pluginRoot, 'current');
  if (!fs.existsSync(path.join(currentLink, 'manifest.json'))) {
    fail(
      'the active generation pointer is set but its convenience link is missing or broken; ' +
        'an interrupted swap is completed by the next install or rollback'
    );
  }
  verifyStableDispatchers(pluginRoot, manifest);
  verifyHookRuntime(pluginRoot, manifest, trustStorePath);
  verifyTargetReceipts(pluginRoot, manifest, trustStorePath, targets);
  return manifest;
}

function rollback(pluginRoot, home, trustStorePath, contract, targets = TARGETS) {
  return withStoreLock(pluginRoot, () =>
    rollbackLocked(pluginRoot, home, trustStorePath, contract, targets)
  );
}

function rollbackLocked(pluginRoot, home, trustStorePath, contract, targets) {
  recoverPendingLinks(pluginRoot);
  const active = currentTarget(pluginRoot, 'current');
  const previous = currentTarget(pluginRoot, 'previous');
  if (!active || !previous) fail('rollback requires current and previous generations');
  const generationPath = path.resolve(pluginRoot, previous);
  if (!isWithin(path.join(pluginRoot, 'generations'), generationPath))
    fail('previous pointer escapes generations directory');
  const manifest = verifyGeneration(generationPath, path.basename(previous), trustStorePath);
  assertMinimumActiveVersion(manifest.sourceVersion, contract, 'rollback generation');
  const skills = collectSkills(path.join(generationPath, 'skills'));
  validateBundleIndex(pluginRoot, generationPath, manifest, trustStorePath);
  installTargets(home, pluginRoot, generationPath, manifest.generation, skills, targets, manifest);
  verifyTargets(home, pluginRoot, generationPath, manifest.generation, skills, targets);
  assertNoCollisions(false);
  installHookRuntime(pluginRoot, generationPath, manifest, trustStorePath);
  setActivationPointer(pluginRoot, 'current', previous, previous);
  setActivationPointer(pluginRoot, 'previous', active, active);
  createStableDispatchers(pluginRoot);
  verifyTargetReceipts(pluginRoot, manifest, trustStorePath, targets);
  return path.basename(previous);
}

/**
 * Refuse to install hooks whose executable this host cannot resolve.
 *
 * Exec-form hooks name a bare executable that the HARNESS resolves through
 * PATH. That is not a new dependency class for this pack -- `.mcp.json` already
 * spawns `npx` the same way -- but the failure mode is much worse. If the name
 * does not resolve, the harness's spawn fails before `hook-entry.mjs` executes
 * a single line, so there is no JSON envelope, no error code, and no hint: all
 * 31 hooks simply stop running, silently.
 *
 * The scenario this exists for is macOS. A GUI-launched application inherits
 * launchd's environment rather than a shell's, and `nvm`, `fnm`, and `asdf` all
 * put `node` somewhere only a shell knows about. A Mac that installed the pack
 * from a terminal can still fail to dispatch a single hook when the harness is
 * started from the Dock.
 *
 * Resolvability is tested, not behaviour: only `error` is inspected, because a
 * non-zero exit still proves the executable was found and started.
 */
function assertHookExecutablesResolvable(source) {
  const configured = path.join(source, 'hooks/hooks.json');
  if (!fs.existsSync(configured)) return;
  const config = JSON.parse(fs.readFileSync(configured, 'utf8'));
  const executables = new Set();
  for (const groups of Object.values(config.hooks ?? {})) {
    for (const group of groups) {
      for (const hook of group.hooks ?? []) {
        // Only exec form resolves through PATH. A shell-form entry is the
        // shell's problem, and an absolute path is resolved by the path itself.
        if (!Array.isArray(hook.args) || !hook.command) continue;
        if (path.isAbsolute(hook.command)) continue;
        executables.add(hook.command);
      }
    }
  }
  for (const executable of executables) {
    if (!spawnSync(executable, ['--version'], { shell: false }).error) continue;
    fail(
      [
        `the generated hooks spawn ${JSON.stringify(executable)}, which cannot be resolved here`,
        '  every hook would fail before any of this pack runs, with no error of its own',
        `  detail: ${executable} is not on PATH for this process`,
        '  remediation: put it on PATH for the environment the harness starts in.',
        "    macOS launches from the Dock inherit launchd's environment, not a shell's,",
        '    so a version manager such as nvm, fnm, or asdf needs either a system-wide',
        '    install or `launchctl setenv PATH` for the harness to see it.',
      ].join('\n')
    );
  }
}

function install(args) {
  const source = args.sourceRoot;
  const contract = readSkillSystem(source);
  assertMinimumActiveVersion(contract.releaseVersion, contract, 'source release');
  const targetDefinitions = targetsById(contract, args.targets);
  const selectedTargets = targetDefinitions.map(target => target.path);
  if (canonicalJson(contract.targets.map(target => target.path)) !== canonicalJson(TARGETS)) {
    fail('distribution contract target paths diverge from the signed target matrix');
  }
  const distributionSkills = collectDistributionSkills(source, contract);
  for (const required of [
    'skills',
    'shared/scripts',
    'shared/harnesses/generated/release-manifest.json',
    'hooks',
    'skill-system.json',
  ]) {
    if (!fs.existsSync(path.join(source, required))) fail(`source payload is missing ${required}`);
  }
  // The ingest oracle is git's index, which records mode 100755/100644 for every
  // tracked path regardless of the host that checked it out. It is the only
  // portable authority for the executable bit: a Windows checkout carries none,
  // and a POSIX checkout carries one the local umask perturbs.
  const oracle = readIngestOracle(source);
  const sourceExecutable = relative => {
    const recorded = oracle?.get(relative);
    if (recorded) return recorded.type === 'file' && recorded.executable;
    if (CAPABILITIES.executableBit) {
      const stat = fs.statSync(path.join(source, ...relative.split('/')), {
        throwIfNoEntry: false,
      });
      return Boolean(stat) && (stat.mode & 0o100) !== 0;
    }
    return null;
  };
  for (const script of REQUIRED_SCRIPTS) {
    const relative = `shared/scripts/${script}`;
    if (!fs.existsSync(path.join(source, ...relative.split('/')))) {
      fail(`required script is missing: ${script}`);
    }
    const executable = sourceExecutable(relative);
    if (executable === null) {
      fail(
        `no portable executable-bit authority for ${relative}: this volume cannot observe an ` +
          'executable bit and the source is not a git checkout'
      );
    }
    if (!executable) fail(`required script is not executable: ${script}`);
  }

  assertHookExecutablesResolvable(source);

  const sourceBefore = sourceProvenance(source);
  validateSourceProvenance(sourceBefore, args, 'before staging');

  const generations = path.join(args.pluginRoot, 'generations');
  ensureDirectory(generations);
  const signingIdentity = ensureSigningIdentity(args.signingKey, args.trustStore);
  const staging = path.join(
    generations,
    `.staging-${process.pid}-${crypto.randomBytes(6).toString('hex')}`
  );
  ensureDirectory(staging);
  const ingest = ingestContext({ repoRoot: source, destinationRoot: staging, oracle });
  try {
    ensureDirectory(path.join(staging, 'skills'));
    for (const skill of distributionSkills) {
      copyEntry(skill.source, path.join(staging, 'skills', skill.name), true, source, ingest);
    }
    for (const root of PAYLOAD_ROOTS) {
      const absolute = path.join(source, root);
      // Pass `source` (the repo root) explicitly: each PAYLOAD_ROOTS entry is
      // copied from source/<root>, so letting repoRoot default to the subtree
      // would make exclusion paths relative to e.g. <repo>/skills and the
      // skills/imported/** evidence match would never fire.
      if (fs.existsSync(absolute))
        copyEntry(absolute, path.join(staging, root), true, source, ingest);
    }
    stagePluginMetadata(source, staging, contract.releaseVersion, ingest);
    stageReleaseRuntimeSupport(source, staging, ingest);
    const executionComponent = stageExecutionComponent(
      source,
      staging,
      contract.releaseVersion,
      ingest
    );
    const skills = collectSkills(path.join(staging, 'skills'));
    if (skills.length === 0) fail('generation contains no installable skills');
    const skillIndex = buildSkillIndex(skills);
    atomicWrite(path.join(staging, 'indexes/skills.json'), canonicalJson(skillIndex));
    atomicWrite(path.join(staging, 'mobile/skill-index.json'), canonicalJson(skillIndex));
    atomicWrite(path.join(staging, 'agents/skill-index.json'), canonicalJson(skillIndex));
    atomicWrite(
      path.join(staging, 'mobile/parity.json'),
      canonicalJson({
        schemaVersion: 1,
        skillIndexSchema: SKILL_INDEX_SCHEMA,
        skillIndexSha256: skillIndex.sha256,
        entryCount: skillIndex.entries.length,
      })
    );
    const release = verifyReleaseManifest(staging, args.expectedBundle, relative => {
      const intent = ingest.intents.get(relative);
      return intent && intent.type === 'file' ? Boolean(intent.executable) : null;
    });
    if (release.sourceVersion !== contract.releaseVersion) {
      fail(
        `release manifest version ${release.sourceVersion} differs from contract ${contract.releaseVersion}`
      );
    }
    const sourceAfter = sourceProvenance(source);
    validateSourceProvenance(sourceAfter, args, 'after staging');
    if (
      sourceAfter.sourceCommit !== sourceBefore.sourceCommit ||
      sourceAfter.sourceTreeState !== sourceBefore.sourceTreeState ||
      !sameExternalSources(sourceAfter.externalSources, sourceBefore.externalSources)
    ) {
      fail(`source provenance changed while staging payload: ${source}`);
    }
    const dispatcher = release.runtimeFiles.find(
      entry => entry.path === 'shared/scripts/generated/hook-dispatch-v1.sh'
    );
    const runner = release.runtimeFiles.find(
      entry => entry.path === 'shared/scripts/hook-runtime-v1.sh'
    );
    if (!dispatcher || !runner) fail('release manifest omits hook runtime payloads');
    const hookRuntime = {
      abi: release.dispatcherAbi,
      bundleId: release.bundleId,
      dispatcherPath: dispatcher.path,
      dispatcherSha256: dispatcher.sha256,
      // The runtime launches the dispatcher by this interpreter rather than by
      // a filesystem executable bit, so it has to be part of what is signed.
      dispatcherInterpreter: release.dispatcherInterpreter,
      runnerSha256: runner.sha256,
    };
    // Not every staged entry passes through `copyEntry`. The installer writes
    // skill indexes, parity and component receipts directly, and creating any of
    // those materializes intermediate DIRECTORIES -- `.agents` exists only
    // because `.agents/plugins` was copied into it. So the default has to depend
    // on what is actually on disk: a directory nobody recorded is structure, and
    // a file nobody recorded is one this installer wrote, which is never
    // executable.
    const intentOf = relative => {
      const recorded = ingest.intents.get(relative);
      if (recorded) return recorded;
      const stat = fs.lstatSync(path.join(staging, ...relative.split('/')), {
        throwIfNoEntry: false,
      });
      if (stat?.isDirectory() && !stat.isSymbolicLink()) return { type: 'directory' };
      return { type: 'file', executable: false };
    };
    // Those same implicit directories never had a mode applied from a manifest
    // entry, so on POSIX they would carry whatever the umask allowed -- the
    // exact dependence schema 2 exists to remove. Directories only: every file
    // was moded when it was copied or written.
    normalizeDirectoryModes(staging);
    const identity = {
      schemaVersion: MANIFEST_SCHEMA_VERSION,
      sourceVersion: contract.releaseVersion,
      signerKeyId: signingIdentity.keyId,
      bundleId: release.bundleId,
      hookRuntime,
      sourceProvenance: sourceAfter,
      skillIndex: { sha256: skillIndex.sha256, entryCount: skillIndex.entries.length },
      executionComponent,
      files: collectPayloadEntries(staging, intentOf, CAPABILITIES),
      targetPayloads: targetPayloads(skills, executionComponent),
    };
    const generation = sha256(jcs(identity));
    const manifest = { ...identity, generation };
    atomicWrite(path.join(staging, 'manifest.json'), canonicalJson(manifest));
    atomicWrite(
      path.join(staging, MANIFEST_SIGNATURE),
      canonicalJson(signValue(manifest, signingIdentity)),
      0o600
    );
    // The materialization record is written AFTER identity is fixed and is
    // excluded from the entry list, so a host that had to degrade a link reports
    // an honest local account without moving the generation hash. Verification
    // never reads it: collectPayloadEntries accepts either realization of a link
    // entry on that entry's own evidence, so an edited record cannot authorize
    // anything.
    atomicWrite(
      path.join(staging, MATERIALIZATION_RECORD),
      canonicalJson({
        schemaVersion: MATERIALIZATION_SCHEMA_VERSION,
        generation,
        capabilities: {
          symlinkFile: CAPABILITIES.symlinkFile,
          symlinkDirectory: CAPABILITIES.symlinkDirectory,
          junction: CAPABILITIES.junction,
          hardlink: CAPABILITIES.hardlink,
          executableBit: CAPABILITIES.executableBit,
          posixModes: CAPABILITIES.posixModes,
        },
        directoryLinkStrategy: CAPABILITIES.directoryLinkStrategy,
        fileLinkStrategy: CAPABILITIES.fileLinkStrategy,
        executableBitAuthority: oracle ? 'git-index' : 'filesystem',
        degradations: [...ingest.degradations].sort((left, right) =>
          left.path < right.path ? -1 : left.path > right.path ? 1 : 0
        ),
      })
    );
    verifyGeneration(staging, generation, args.trustStore);

    const generationPath = path.join(generations, generation);
    if (fs.existsSync(generationPath)) {
      verifyGeneration(generationPath, generation, args.trustStore);
      fs.rmSync(staging, { recursive: true, force: true });
    } else {
      fs.renameSync(staging, generationPath);
      syncDirectory(generations);
    }

    // Everything from here mutates the shared store: pointers, convenience
    // links, and the platform targets. It runs under the same mutex the shell
    // bootstrap uses, and any link swap interrupted by a previous run is
    // completed first so recovery never races a live swap.
    return withStoreLock(args.pluginRoot, () => {
      recoverPendingLinks(args.pluginRoot);
      validateBundleIndex(args.pluginRoot, generationPath, manifest, args.trustStore);
      installTargets(
        args.home,
        args.pluginRoot,
        generationPath,
        generation,
        skills,
        selectedTargets,
        manifest
      );
      verifyTargets(
        args.home,
        args.pluginRoot,
        generationPath,
        generation,
        skills,
        selectedTargets
      );
      assertNoCollisions(Boolean(args.allowFallback));
      writeTargetReceipts(args.pluginRoot, manifest, signingIdentity, selectedTargets);
      installHookRuntime(args.pluginRoot, generationPath, manifest, args.trustStore);
      const active = currentTarget(args.pluginRoot, 'current');
      if (active !== `generations/${generation}`) {
        if (active) setActivationPointer(args.pluginRoot, 'previous', active, active);
        setActivationPointer(
          args.pluginRoot,
          'current',
          `generations/${generation}`,
          `generations/${generation}`
        );
      }
      createStableDispatchers(args.pluginRoot);
      verifyActive(args.pluginRoot, args.trustStore, contract, selectedTargets);
      return generation;
    });
  } catch (error) {
    fs.rmSync(staging, { recursive: true, force: true });
    throw error;
  }
}

function copyTargetReferencesGeneration(home, generation) {
  for (const target of COPY_TARGETS) {
    const targetRoot = path.join(home, ...target.split('/'));
    if (!fs.existsSync(targetRoot)) continue;
    for (const name of fs.readdirSync(targetRoot)) {
      const marker = path.join(targetRoot, name, '.prometheus-generation');
      if (fs.existsSync(marker) && fs.readFileSync(marker, 'utf8').trim() === generation) {
        return `${target}/${name}`;
      }
    }
  }
  return null;
}

/**
 * Identity-only verification, for deciding whether a generation may be RETIRED.
 *
 * Deliberately weaker than verifyGeneration(): it proves the manifest belongs to
 * this directory and has not been edited, but says nothing about who produced it
 * or whether it satisfies the current payload schema. That is sufficient to
 * retire a generation and never sufficient to activate one, so this must only
 * ever be called from the prune path.
 */
function verifyGenerationIdentity(generationPath, expectedName) {
  const manifestPath = path.join(generationPath, 'manifest.json');
  if (!fs.existsSync(manifestPath)) fail(`generation has no manifest: ${generationPath}`);
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  if (![1, MANIFEST_SCHEMA_VERSION].includes(manifest.schemaVersion)) {
    fail(`unsupported generation manifest schema: ${manifest.schemaVersion ?? 'missing'}`);
  }
  const digest = generationDigest(manifest);
  if (manifest.generation !== digest || expectedName !== digest)
    fail(`generation identity mismatch: ${generationPath}`);
  if (typeof manifest.sourceVersion !== 'string' || manifest.sourceVersion.length === 0)
    fail(`generation has no sourceVersion: ${generationPath}`);
  return manifest;
}

function pruneObsoleteGenerations(args, contract) {
  const generationsRoot = path.join(args.pluginRoot, 'generations');
  if (!fs.existsSync(generationsRoot))
    return { retired: [], minimumActiveVersion: contract.minimumActiveVersion };
  const active = path.basename(currentTarget(args.pluginRoot, 'current') ?? '');
  const previous = path.basename(currentTarget(args.pluginRoot, 'previous') ?? '');
  const retired = [];
  const reapedStaging = [];
  for (const generation of fs.readdirSync(generationsRoot).sort()) {
    // Reap abandoned staging directories.
    //
    // Staging dirs are named `.staging-<pid>-<random>` and are renamed into
    // place on success, so one that still exists is a partial install whose
    // process died before completing. Nothing ever removed them: this loop used
    // to skip every `.staging-` entry unconditionally, so they accumulated
    // silently (one here had been abandoned since 2026-08-05).
    //
    // Reap only when the owning pid is gone. A staging dir belonging to a LIVE
    // process is a concurrent install and must be left alone -- deleting it
    // would corrupt that run. An unparseable name is left alone too.
    if (generation.startsWith('.staging-')) {
      const owner = Number.parseInt(generation.split('-')[1] ?? '', 10);
      if (!Number.isInteger(owner) || owner <= 0) continue;
      let alive = true;
      try {
        process.kill(owner, 0);
      } catch (error) {
        alive = error.code === 'EPERM';
      }
      if (alive) continue;
      if (!args.dryRun) {
        fs.rmSync(path.join(generationsRoot, generation), { recursive: true, force: true });
      }
      reapedStaging.push(generation);
      continue;
    }
    const generationPath = path.join(generationsRoot, generation);
    if (!fs.lstatSync(generationPath).isDirectory()) continue;
    // Verify identity first, and only identity.
    //
    // This loop used to call the full verifyGeneration() on EVERY generation
    // before deciding whether it was even a prune candidate, which broke the
    // prune twice over:
    //
    //   1. Unsigned generations abort it. Those predate manifest signing and are
    //      by construction the oldest ones -- exactly what the prune exists to
    //      remove. So the prune aborted on its own targets and could never
    //      delete anything.
    //   2. Generations predating the executionComponent field abort it too, and
    //      12 of them here are current-version generations the prune would have
    //      skipped a line later anyway. They were being held to today's schema
    //      for no reason at all.
    //
    // Full verification answers "may this be activated?". Deletion asks a
    // different and weaker question, and the identity digest answers it: it
    // proves the manifest describes this directory and has not been edited.
    // Everything that makes deletion safe is checked below -- not current, not
    // previous, not referenced by an installed target, receipts individually
    // signature-verified before removal. Something that cannot be activated and
    // is referenced by nothing is garbage, and declining to collect it is not a
    // security property.
    const manifest = verifyGenerationIdentity(generationPath, generation);
    if (compareVersions(manifest.sourceVersion, contract.minimumActiveVersion) >= 0) continue;
    if (generation === active || generation === previous) {
      fail(
        `obsolete generation is selected by ${generation === active ? 'current' : 'previous'}: ${generation}`
      );
    }
    const targetReference = copyTargetReferencesGeneration(args.home, generation);
    if (targetReference)
      fail(`obsolete generation is referenced by installed target ${targetReference}`);

    const receiptRoot = path.join(args.pluginRoot, 'receipts', generation);
    if (fs.existsSync(receiptRoot)) {
      for (const file of fs.readdirSync(receiptRoot).sort()) {
        if (!file.endsWith('.json')) continue;
        const receipt = JSON.parse(fs.readFileSync(path.join(receiptRoot, file), 'utf8'));
        if (receipt.body?.generation !== generation)
          fail(`obsolete receipt generation mismatch: ${file}`);
        verifySignedValue(receipt.body, receipt.signature, args.trustStore);
      }
      if (!args.dryRun) fs.rmSync(receiptRoot, { recursive: true, force: true });
    }

    const bundlesRoot = path.join(args.pluginRoot, 'bundles');
    if (fs.existsSync(bundlesRoot)) {
      for (const bundle of fs.readdirSync(bundlesRoot).sort()) {
        const link = path.join(bundlesRoot, bundle);
        const stat = fs.lstatSync(link, { throwIfNoEntry: false });
        if (!stat?.isSymbolicLink()) continue;
        const resolved = path.resolve(bundlesRoot, fs.readlinkSync(link));
        if (resolved === generationPath && !args.dryRun) fs.unlinkSync(link);
      }
    }
    if (!args.dryRun) fs.rmSync(generationPath, { recursive: true, force: true });
    retired.push({
      generation,
      sourceVersion: manifest.sourceVersion,
      bundleId: manifest.bundleId,
    });
  }
  return {
    retired,
    reapedStaging,
    minimumActiveVersion: contract.minimumActiveVersion,
    dryRun: args.dryRun,
  };
}

/**
 * Exposed so the identity and signature rules can be exercised by a fixture
 * without fabricating a whole generation store. Nothing outside
 * `scripts/tests/` may import this.
 */
export const __testing = {
  assertHookExecutablesResolvable,
  breadcrumbFile,
  installHookDispatcher,
  canonicalJson,
  readPointer,
  recoverPendingLinks,
  replaceConvenienceLink,
  setActivationPointer,
  withStoreLock,
  writePointer,
  generationDigest,
  generationIdentity,
  jcs,
  manifestExecutableLookup,
  signValue,
  signaturePayload,
  verifySignedValue,
  MANIFEST_SCHEMA_VERSION,
  RELEASE_MANIFEST_SCHEMA_VERSION,
  SIGNATURE_SCHEMA_VERSION,
  setCapabilities(value) {
    CAPABILITIES = value;
  },
};

// `import.meta.main` (Node 24.2+) separates "run as a program" from "imported
// by a fixture". On a runtime that does not define it the value is `undefined`,
// and this DEFAULTS TO RUNNING: an install that silently did nothing would be
// far worse than an import with side effects.
if (import.meta.main !== false) main();

function main() {
  const args = parseArgs(process.argv.slice(2));
  assertSafeRoot(args.pluginRoot, args.home);
  try {
    // Probe before anything else touches the store. The probe runs in the
    // generation store root because capability varies by VOLUME: a store on a
    // removable or network volume can lack primitives that the temporary
    // directory on C: or /tmp reports as present.
    //
    // The cache is keyed on the installer's own bytes rather than on the release
    // version, so an installer that probes a primitive its predecessor never
    // measured cannot read a stale record as "unsupported".
    CAPABILITIES = loadCapabilities({
      storeRoot: path.join(args.pluginRoot, 'generations'),
      installerVersion: sha256(fs.readFileSync(fileURLToPath(import.meta.url))),
      cacheFile: path.join(args.home, '.prometheus/capabilities.json'),
    });
    const contract = readSkillSystem(args.sourceRoot);
    const selectedTargets = targetsById(contract, args.targets).map(target => target.path);
    let generation;
    if (args.verify)
      generation = verifyActive(
        args.pluginRoot,
        args.trustStore,
        contract,
        selectedTargets
      ).generation;
    else if (args.rollback)
      generation = rollback(args.pluginRoot, args.home, args.trustStore, contract, selectedTargets);
    else if (args.uninstall) {
      generation = uninstall(
        args.home,
        args.pluginRoot,
        selectedTargets,
        selectedTargets.length === TARGETS.length
      );
    } else if (args.pruneObsolete)
      generation = JSON.stringify(pruneObsoleteGenerations(args, contract));
    else generation = install(args);
    process.stdout.write(`${generation}\n`);
  } catch (error) {
    process.stderr.write(`install-plugin-generation: ${error.message}\n`);
    process.exitCode = 1;
  }
}
