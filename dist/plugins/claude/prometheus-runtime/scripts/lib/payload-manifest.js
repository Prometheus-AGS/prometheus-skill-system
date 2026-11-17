/**
 * Host-independent payload identity (canonical manifest schema version 2).
 *
 * WHAT CHANGED AND WHY
 *
 * Schema 1 recorded a full permission mode per entry, so the generation hash was
 * a function of the host. That is fatal on Windows, where libuv reports 0o444 or
 * 0o666 and nothing else — but it was ALREADY wrong between Linux hosts, because
 * the recorded mode is the copied file's mode and therefore umask-dependent. Two
 * Linux machines with different umasks produced different identities for the
 * same payload and nobody noticed, because nothing compared them.
 *
 * Schema 2 records what git's tree model and Nix's archive format both record,
 * and for the same reason those two are portable: an entry TYPE and a single
 * normalized executable bit. Modes, timestamps, ownership, security identifiers,
 * file attributes, and access control lists are all excluded from identity, and
 * modes are re-applied FROM the manifest at materialization instead of being
 * observed after the fact.
 *
 * SYMLINKS HASH THEIR RECORDED TARGET, NEVER THEIR REALIZATION
 *
 * This is the property that makes degradation safe. A host with no link
 * primitive writes the link's target text into a plain file; the entry still
 * hashes as a symlink over that same text, so identity is unchanged and the
 * on-disk bytes still verify against the recorded hash. Verification therefore
 * never has to consult the materialization record, which is deliberately
 * unhashed and must not be able to influence a decision.
 *
 * EXECUTABLE INTENT COMES FROM THE INGEST ORACLE, NOT THE DESTINATION
 *
 * A Windows checkout cannot carry an executable bit at all, so observing the
 * staged copy would record `false` for every script and diverge from Linux. The
 * git index records mode 100755/100644 for every tracked path regardless of
 * host, so it is the portable authority. It also settles entry TYPE: git records
 * mode 120000 for a symlink even when `core.symlinks=false` checked it out as a
 * plain file holding its target path.
 */

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

import { jcs } from './jcs.js';

export const MANIFEST_SCHEMA_VERSION = 2;
export const MATERIALIZATION_RECORD = 'materialization.json';
export const MATERIALIZATION_SCHEMA_VERSION = 1;

/** Files that describe a generation and therefore cannot be entries within it. */
export const UNMANIFESTED_ROOT_FILES = Object.freeze([
  'manifest.json',
  'manifest.sig.json',
  MATERIALIZATION_RECORD,
]);

function sha256(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

function toPosix(value) {
  return value.split(path.sep).join('/');
}

function isWithin(parent, candidate) {
  const relative = path.relative(parent, candidate);
  return (
    relative === '' ||
    (!relative.startsWith(`..${path.sep}`) && relative !== '..' && !path.isAbsolute(relative))
  );
}

/**
 * Executable and type intent for every path git tracks under `sourceRoot`.
 *
 * Returns null when `sourceRoot` is not a git checkout. Callers decide whether
 * that is acceptable; on a host that cannot observe an executable bit it is not,
 * because there would then be no authority at all and identity would silently
 * diverge from every other host.
 */
export function readIngestOracle(sourceRoot) {
  const result = spawnSync('git', ['ls-files', '-s', '-z', '--recurse-submodules'], {
    cwd: sourceRoot,
    encoding: 'buffer',
    maxBuffer: 256 * 1024 * 1024,
  });
  if (result.error || result.status !== 0) return null;
  const oracle = new Map();
  for (const record of result.stdout.toString('utf8').split('\0')) {
    if (!record) continue;
    const separator = record.indexOf('\t');
    if (separator < 0) continue;
    const mode = record.slice(0, separator).split(' ')[0];
    const relative = record.slice(separator + 1);
    if (mode === '120000') oracle.set(relative, { type: 'symlink', executable: false });
    else if (mode === '100755') oracle.set(relative, { type: 'file', executable: true });
    else if (mode === '100644') oracle.set(relative, { type: 'file', executable: false });
    // 160000 gitlinks are submodule pointers, not payload entries; the
    // --recurse-submodules listing already supplies the files inside them.
  }
  return oracle.size ? oracle : null;
}

/**
 * Resolve ingest intent for one source path.
 *
 * Authority order is oracle, then filesystem, then failure. The failure is
 * deliberate: a host that can neither consult the oracle nor observe an
 * executable bit would produce a manifest that disagrees with every other host,
 * and a silently divergent identity is worse than a refused install.
 */
export function resolveIngestIntent({ sourcePath, repoRoot, oracle, capabilities }) {
  const stat = fs.lstatSync(sourcePath);
  const relative = toPosix(path.relative(repoRoot, sourcePath));
  const recorded = oracle?.get(relative) ?? null;
  if (stat.isDirectory() && !stat.isSymbolicLink()) return { type: 'directory' };

  if (recorded?.type === 'symlink') {
    // Either a real symlink, or the plain file a `core.symlinks=false` checkout
    // left behind holding the target path.
    const target = stat.isSymbolicLink()
      ? fs.readlinkSync(sourcePath)
      : fs.readFileSync(sourcePath, 'utf8');
    return { type: 'symlink', target };
  }
  if (stat.isSymbolicLink()) {
    return { type: 'symlink', target: fs.readlinkSync(sourcePath) };
  }
  if (!stat.isFile()) {
    const failure = new Error(`unsupported payload entry type: ${sourcePath}`);
    failure.code = 'UNSUPPORTED_ENTRY_TYPE';
    throw failure;
  }
  if (recorded) return { type: 'file', executable: recorded.executable };
  if (capabilities.executableBit) {
    return { type: 'file', executable: (stat.mode & 0o100) !== 0 };
  }
  const failure = new Error(
    `no portable executable-bit authority for ${relative || sourcePath}: this volume cannot ` +
      'observe an executable bit and the source is not a git checkout, so the recorded ' +
      'generation identity would diverge from every other host'
  );
  failure.code = 'NO_EXECUTABLE_AUTHORITY';
  throw failure;
}

/**
 * POSIX modes re-applied FROM the manifest, not copied from the source.
 *
 * This is what removes the schema-1 umask dependence: the destination mode is a
 * pure function of the recorded executable bit, so two hosts with different
 * umasks materialize byte-identical trees. On a volume with no mode semantics
 * there is nothing to apply and the manifest stays the sole authority.
 */
export function applyManifestMode(target, intent, capabilities) {
  if (!capabilities.posixModes) return null;
  const mode = intent.type === 'directory' ? 0o755 : intent.executable ? 0o755 : 0o644;
  fs.chmodSync(target, mode);
  return mode;
}

function fileEntry(relative, absolute, intent) {
  const bytes = fs.readFileSync(absolute);
  return {
    path: relative,
    type: 'file',
    executable: Boolean(intent.executable),
    sha256: sha256(bytes),
    size: bytes.length,
  };
}

function symlinkEntry(relative, target) {
  return {
    path: relative,
    type: 'symlink',
    target,
    sha256: sha256(target),
    size: Buffer.byteLength(target),
  };
}

/**
 * Derive the canonical entry list for a materialized payload.
 *
 * `intentOf(relativePath)` supplies type, executable bit, and link target. At
 * creation it is the ingest oracle; at verification it is the signed manifest.
 * Content hashes and sizes always come from disk, and every entry's REALIZATION
 * is asserted here, so using the manifest as the intent source at verification
 * is not circular.
 */
export function collectPayloadEntries(root, intentOf, capabilities) {
  const entries = [];
  const walk = relativeDirectory => {
    const absoluteDirectory = relativeDirectory
      ? path.join(root, ...relativeDirectory.split('/'))
      : root;
    for (const name of fs.readdirSync(absoluteDirectory).sort()) {
      const relative = relativeDirectory ? `${relativeDirectory}/${name}` : name;
      if (!relativeDirectory && UNMANIFESTED_ROOT_FILES.includes(name)) continue;
      const absolute = path.join(absoluteDirectory, name);
      const stat = fs.lstatSync(absolute);
      const intent = intentOf(relative);

      if (intent?.type === 'symlink') {
        const target = intent.target;
        if (typeof target !== 'string' || target.length === 0) {
          throw new Error(`link entry has no recorded target: ${relative}`);
        }
        if (!isWithin(root, path.resolve(path.dirname(absolute), target))) {
          throw new Error(`payload link escapes the immutable payload: ${relative}`);
        }
        if (stat.isSymbolicLink()) {
          // A junction reports an ABSOLUTE substitute name where a POSIX symlink
          // reports the relative text it was created with. Both are the same
          // indirection, so the comparison is over the resolved location.
          const realized = path.resolve(path.dirname(absolute), fs.readlinkSync(absolute));
          const intended = path.resolve(path.dirname(absolute), target);
          if (realized !== intended) {
            throw new Error(
              `link entry resolves elsewhere: ${relative} ` +
                `(recorded ${target} -> ${intended}; realized -> ${realized})`
            );
          }
        } else if (stat.isFile()) {
          if (fs.readFileSync(absolute, 'utf8') !== target) {
            throw new Error(`degraded link entry does not hold its recorded target: ${relative}`);
          }
        } else {
          throw new Error(`link entry is neither a link nor its degraded copy: ${relative}`);
        }
        entries.push(symlinkEntry(relative, target));
        continue;
      }

      if (stat.isSymbolicLink()) {
        throw new Error(`payload entry is a link but is not recorded as one: ${relative}`);
      }
      if (stat.isDirectory()) {
        if (intent && intent.type !== 'directory') {
          throw new Error(
            `payload entry is a directory but is recorded as ${intent.type}: ${relative}`
          );
        }
        entries.push({ path: relative, type: 'directory' });
        walk(relative);
        continue;
      }
      if (!stat.isFile()) {
        const failure = new Error(`unsupported payload entry type: ${relative}`);
        failure.code = 'UNSUPPORTED_ENTRY_TYPE';
        throw failure;
      }
      if (intent && intent.type !== 'file') {
        throw new Error(`payload entry is a file but is recorded as ${intent.type}: ${relative}`);
      }
      if (!intent) throw new Error(`payload entry is not manifested: ${relative}`);
      if (capabilities.executableBit) {
        const observed = (stat.mode & 0o100) !== 0;
        if (observed !== Boolean(intent.executable)) {
          throw new Error(
            `payload entry executable bit disagrees with the manifest: ${relative} ` +
              `(recorded ${Boolean(intent.executable)}, observed ${observed})`
          );
        }
      }
      entries.push(fileEntry(relative, absolute, intent));
    }
  };
  walk('');
  entries.sort((left, right) => (left.path < right.path ? -1 : left.path > right.path ? 1 : 0));
  return entries;
}

/** Intent lookup backed by a signed manifest's entry list. */
export function manifestIntentLookup(files) {
  const index = new Map(files.map(entry => [entry.path, entry]));
  return relative => index.get(relative) ?? null;
}

/** Entries are equal when their canonical RFC 8785 form is equal. */
export function entriesEqual(left, right) {
  return jcs(left) === jcs(right);
}
