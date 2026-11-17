/**
 * Empirical filesystem-capability probing for the plugin generation store.
 *
 * WHY THIS IS PROBED AND NEVER INFERRED
 *
 * Every capability this module reports varies by VOLUME, not by platform:
 *
 *   - Symlink creation on Windows needs SeCreateSymbolicLinkPrivilege, granted
 *     by Developer Mode or elevation. Two Windows hosts, same build, differ.
 *   - Junctions need no privilege at all and work where symlinks do not, which
 *     is why a `process.platform === 'win32'` check cannot answer "can I make a
 *     directory link here".
 *   - Hardlinks and the executable bit are absent on FAT/exFAT and present on
 *     NTFS and ext4, on the SAME host.
 *
 * The probe therefore runs INSIDE the generation store root. Probing
 * `os.tmpdir()` and applying the answer to the store is the classic version of
 * this bug: on Windows `%TEMP%` is nearly always on C:, so a store on a
 * removable or network volume inherits an answer measured somewhere else.
 *
 * TWO MECHANICAL WIN32 CONSTRAINTS ARE ENCODED HERE
 *
 *   1. `fs.symlinkSync` MUST be called with an explicit `type`. With no type,
 *      libuv autodetects, selects 'dir' for a directory target, and raises
 *      EPERM without Developer Mode — including on hosts where a 'file' symlink
 *      would have been fine.
 *   2. A junction's target MUST be absolute. A relative substitute name is
 *      accepted by the call and produces a reparse point that resolves against
 *      the wrong base.
 *
 * On POSIX the `type` argument is ignored, so the 'junction' probe there simply
 * creates an ordinary symlink. `directoryLinkStrategy` prefers 'symlink'
 * whenever symlinks work, so POSIX never selects the junction rung; the raw
 * `junction` field is recorded for the materialization record, not for dispatch.
 */

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

export const CAPABILITY_SCHEMA_VERSION = 1;

/** Fields that make up a probe result. Ordered for a stable, readable record. */
const PROBE_FIELDS = [
  'symlinkFile',
  'symlinkDirectory',
  'junction',
  'hardlink',
  'executableBit',
  'posixModes',
];

function attempt(action) {
  try {
    action();
    return { ok: true, code: null };
  } catch (error) {
    return { ok: false, code: error.code ?? error.message };
  }
}

function removeQuietly(target) {
  try {
    fs.rmSync(target, { recursive: true, force: true, maxRetries: 3 });
  } catch {
    /* a probe artifact that outlives the probe is noise, never a failure */
  }
}

/**
 * Measure filesystem primitives inside `storeRoot`.
 *
 * Throws only when the store root itself cannot host the probe; an individual
 * primitive that fails is a recorded absence, not an error.
 */
export function probeFilesystemCapabilities(storeRoot) {
  const root = path.resolve(storeRoot);
  fs.mkdirSync(root, { recursive: true });
  const probeRoot = path.join(
    root,
    `.capability-probe-${process.pid}-${crypto.randomBytes(6).toString('hex')}`
  );
  try {
    fs.mkdirSync(probeRoot);
  } catch (error) {
    const failure = new Error(`generation store root cannot host a capability probe: ${root}`);
    failure.code = 'STORE_ROOT_UNWRITABLE';
    failure.cause = error;
    throw failure;
  }

  const results = { probeRoot };
  const failures = {};
  try {
    const targetDirectory = path.join(probeRoot, 'target-directory');
    const targetFile = path.join(probeRoot, 'target-file');
    fs.mkdirSync(targetDirectory);
    fs.writeFileSync(targetFile, 'probe');

    const linkProbes = [
      ['symlinkFile', () => fs.symlinkSync(targetFile, path.join(probeRoot, 'link-file'), 'file')],
      [
        'symlinkDirectory',
        () => fs.symlinkSync(targetDirectory, path.join(probeRoot, 'link-directory'), 'dir'),
      ],
      // Absolute target: a junction with a relative substitute name resolves
      // against the wrong base.
      [
        'junction',
        () => fs.symlinkSync(targetDirectory, path.join(probeRoot, 'link-junction'), 'junction'),
      ],
      ['hardlink', () => fs.linkSync(targetFile, path.join(probeRoot, 'link-hard'))],
    ];
    for (const [field, action] of linkProbes) {
      const outcome = attempt(action);
      results[field] = outcome.ok;
      if (!outcome.ok) failures[field] = outcome.code;
    }

    // A link primitive counts only when the result is observable as a link.
    // A call that "succeeds" and leaves a plain file behind is not support.
    for (const [field, name] of [
      ['symlinkFile', 'link-file'],
      ['symlinkDirectory', 'link-directory'],
      ['junction', 'link-junction'],
    ]) {
      if (!results[field]) continue;
      const stat = fs.lstatSync(path.join(probeRoot, name), { throwIfNoEntry: false });
      if (!stat?.isSymbolicLink()) {
        results[field] = false;
        failures[field] = 'NOT_OBSERVABLE_AS_LINK';
      }
    }

    // Executable bit: set it, observe it, clear it, observe it gone. Both halves
    // are required. libuv derives st_mode on Windows from FILE_ATTRIBUTE_READONLY
    // alone, so chmod appears to succeed and the observed mode never moves.
    const modeProbe = path.join(probeRoot, 'mode-probe');
    fs.writeFileSync(modeProbe, 'probe');
    const executable = attempt(() => {
      fs.chmodSync(modeProbe, 0o755);
      if ((fs.statSync(modeProbe).mode & 0o100) === 0) throw new Error('EXECUTE_BIT_NOT_SET');
      fs.chmodSync(modeProbe, 0o644);
      if ((fs.statSync(modeProbe).mode & 0o100) !== 0) throw new Error('EXECUTE_BIT_NOT_CLEARED');
    });
    results.executableBit = executable.ok;
    if (!executable.ok) failures.executableBit = executable.code;

    // Full POSIX permission semantics, which is a strictly stronger claim than
    // the executable bit and is what the owner-only key predicate dispatches on.
    const posix = attempt(() => {
      fs.chmodSync(modeProbe, 0o600);
      const observed = fs.statSync(modeProbe).mode & 0o777;
      if (observed !== 0o600) throw new Error(`OBSERVED_${observed.toString(8)}`);
    });
    results.posixModes = posix.ok;
    if (!posix.ok) failures.posixModes = posix.code;
  } finally {
    removeQuietly(probeRoot);
  }

  const record = {
    schemaVersion: CAPABILITY_SCHEMA_VERSION,
    storeRoot: root,
    probeRoot,
  };
  for (const field of PROBE_FIELDS) record[field] = Boolean(results[field]);
  record.directoryLinkStrategy = record.symlinkDirectory
    ? 'symlink'
    : record.junction
      ? 'junction'
      : 'copy';
  record.fileLinkStrategy = record.symlinkFile ? 'symlink' : 'copy';
  record.unsupported = failures;
  return record;
}

/**
 * Is there a POSIX shell, and can it hash?
 *
 * Every cold-path script in this pack is a shell script, and the hook runtime
 * itself is one. A Windows host without git-bash or WSL has neither, and the
 * failure that produces is an opaque interpreter error from whatever tried to
 * run the script. Probing for it lets the caller name the missing dependency
 * instead.
 *
 * The hashing tool is probed alongside, because the runtime verifies the
 * dispatcher's digest before executing it and the tool that does so is not the
 * same everywhere: `sha256sum` is coreutils, `shasum` is a Perl script that
 * ships with git-bash, and a host can have one, both, or neither.
 *
 * This is NOT cached. A shell can be installed or removed between two installer
 * runs without the installer changing, and one spawn is cheaper than being
 * wrong about it.
 */
export function probeShell() {
  for (const command of ['bash', 'sh']) {
    const probe = spawnSync(command, ['-c', 'exit 0'], { encoding: 'utf8', shell: false });
    if (probe.error || probe.status !== 0) continue;
    const digest = ['sha256sum', 'shasum'].find(tool => {
      const check = spawnSync(command, ['-c', `command -v ${tool} >/dev/null 2>&1`], {
        encoding: 'utf8',
        shell: false,
      });
      return !check.error && check.status === 0;
    });
    return { available: true, command, digestTool: digest ?? null };
  }
  return { available: false, command: null, digestTool: null };
}

function readCache(cacheFile) {
  try {
    return JSON.parse(fs.readFileSync(cacheFile, 'utf8'));
  } catch {
    return null;
  }
}

/**
 * Probe once per (installer version, store root) and reuse the answer.
 *
 * The cache is discarded when the installer version changes, because a new
 * installer may probe primitives the old one never measured; a record missing a
 * field would otherwise read as "unsupported" forever.
 */
export function loadCapabilities({ storeRoot, installerVersion, cacheFile, force = false }) {
  const root = path.resolve(storeRoot);
  if (!installerVersion) throw new Error('capability cache requires an installer version');
  if (!force) {
    const cached = readCache(cacheFile);
    if (
      cached &&
      cached.schemaVersion === CAPABILITY_SCHEMA_VERSION &&
      cached.installerVersion === installerVersion &&
      cached.storeRoot === root &&
      PROBE_FIELDS.every(field => typeof cached[field] === 'boolean')
    ) {
      return { ...cached, shell: probeShell(), source: 'cache' };
    }
  }
  const probed = { ...probeFilesystemCapabilities(root), installerVersion };
  try {
    fs.mkdirSync(path.dirname(cacheFile), { recursive: true });
    const temporary = `${cacheFile}.${process.pid}.tmp`;
    fs.writeFileSync(temporary, `${JSON.stringify(probed, null, 2)}\n`);
    fs.renameSync(temporary, cacheFile);
  } catch {
    /* an unwritable cache costs a re-probe, never a failed install */
  }
  return { ...probed, shell: probeShell(), source: 'probe' };
}

/**
 * Materialize an intended symlink at `linkPath`, descending symlink → junction
 * → copy, and report which rung was actually used.
 *
 * The copy rung writes the LINK TARGET TEXT, not the target's bytes. That is
 * what keeps a degraded host on the same generation identity: the manifest
 * hashes the recorded target text, so an on-disk file holding exactly that text
 * still verifies byte-for-byte. Copying the target's CONTENT instead would make
 * the entry unverifiable against its own hash and would force verification to
 * trust the (deliberately unhashed) materialization record.
 *
 * It is also what git already does on this host: `core.symlinks=false` checks a
 * symlink out as a plain file holding its target path.
 *
 * `degradedCopy` selects what the copy rung writes:
 *
 *   'target-text'  the link's target path, for PAYLOAD entries, so the bytes
 *                  still hash to the manifest's recorded value.
 *   'contents'     the target file's bytes, for ACTIVATION links, which have to
 *                  keep working as the thing they point at. A directory target
 *                  raises NO_DIRECTORY_LINK_PRIMITIVE instead of being copied.
 *   'fail'         raise rather than degrade at all.
 *
 * `kind` states whether the link stands for a directory or a file. Pass it
 * whenever the target may not exist yet; omit it only for payload entries,
 * whose targets are always already staged.
 *
 * `allowJunction` must be FALSE for anything inside a payload, and the reason is
 * not a preference. A junction's substitute name is ABSOLUTE -- there is no
 * relative form -- so a junction created inside a staging directory keeps
 * pointing at the STAGING path after that directory is renamed into
 * `generations/<id>`. A relative POSIX symlink survives the same rename because
 * it never named the parent. So the payload ladder is symlink then copy, and
 * the junction rung belongs only to activation links, which live at a stable
 * path and are re-created on every install and rollback.
 */
export function materializeLink({
  linkPath,
  target,
  capabilities,
  degradedCopy = 'target-text',
  kind = null,
  allowJunction = true,
}) {
  const absoluteTarget = path.resolve(path.dirname(linkPath), target);
  // `kind` is required whenever the target may not exist yet. Activation links
  // are created BEFORE the pointer they lead through is moved, so the target is
  // routinely absent at this moment; inferring the kind from a failed stat would
  // pick 'file', and a file symlink is exactly the rung that needs a privilege
  // Windows does not grant by default. A junction may be created against a
  // target that does not exist and resolves once it does.
  const wantsDirectory =
    kind === null
      ? Boolean(fs.statSync(absoluteTarget, { throwIfNoEntry: false })?.isDirectory())
      : kind === 'directory';
  fs.mkdirSync(path.dirname(linkPath), { recursive: true });

  const symlinkSupported = wantsDirectory
    ? capabilities.symlinkDirectory
    : capabilities.symlinkFile;
  if (symlinkSupported) {
    const outcome = attempt(() =>
      fs.symlinkSync(target, linkPath, wantsDirectory ? 'dir' : 'file')
    );
    if (outcome.ok) return { realized: 'symlink', intended: 'symlink' };
    // A capability that was probed present and then failed here is a real
    // condition (a race with a policy change, a different volume): fall through
    // to the next rung and record it rather than aborting the install.
    return fallback({
      linkPath,
      target,
      absoluteTarget,
      wantsDirectory,
      capabilities,
      degradedCopy,
      allowJunction,
      reason: outcome.code,
    });
  }
  return fallback({
    linkPath,
    target,
    absoluteTarget,
    wantsDirectory,
    capabilities,
    degradedCopy,
    allowJunction,
    reason: wantsDirectory ? 'NO_DIRECTORY_SYMLINK' : 'NO_FILE_SYMLINK',
  });
}

function fallback({
  linkPath,
  target,
  absoluteTarget,
  wantsDirectory,
  capabilities,
  degradedCopy,
  allowJunction,
  reason,
}) {
  if (wantsDirectory && allowJunction && capabilities.junction) {
    // Junction substitute names must be absolute.
    const outcome = attempt(() => fs.symlinkSync(absoluteTarget, linkPath, 'junction'));
    if (outcome.ok) return { realized: 'junction', intended: 'symlink', reason };
  }
  if (degradedCopy === 'target-text') {
    fs.writeFileSync(linkPath, target);
    return { realized: 'copy', intended: 'symlink', reason };
  }
  if (degradedCopy === 'contents') {
    // An ACTIVATION link, unlike a payload entry, has to keep working as the
    // thing it points at: a stable dispatcher must still run. So the fallback
    // copies the target's bytes rather than its path text.
    //
    // For a DIRECTORY there is no bounded version of that. Copying a whole
    // generation per pointer would multiply the store, and every copy would go
    // stale the moment the pointer moved -- silently, because nothing would
    // resolve through it any more. A volume that offers neither a symlink nor a
    // junction genuinely cannot express directory indirection, and saying so is
    // better than pretending.
    if (wantsDirectory) {
      const failure = new Error(
        `no directory link primitive available for ${linkPath}: this volume supports neither ` +
          'directory symlinks nor junctions, so an activation pointer cannot be materialized'
      );
      failure.code = 'NO_DIRECTORY_LINK_PRIMITIVE';
      throw failure;
    }
    fs.copyFileSync(absoluteTarget, linkPath);
    return { realized: 'copy', intended: 'symlink', reason };
  }
  const failure = new Error(`no link primitive available for ${linkPath}`);
  failure.code = 'NO_LINK_PRIMITIVE';
  throw failure;
}
