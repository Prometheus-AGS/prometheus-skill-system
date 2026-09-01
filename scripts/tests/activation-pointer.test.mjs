/**
 * Fixtures for the activation primitives: typed link ladder, pointer-file
 * activation, recoverable swaps, and the verbatim-prefix guard.
 *
 * The shell half runs the REAL `shared/scripts/hook-runtime-v1.sh` against a
 * fabricated plugin store. That script is the thing every hook on every session
 * goes through, so it is exercised as a program, not paraphrased.
 */

import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { materializeLink, probeFilesystemCapabilities } from '../lib/capabilities.js';
import { POINTER_PATTERN, isWithin, stripVerbatimPrefix } from '../lib/store-paths.js';
import { __testing } from '../install-plugin-generation.js';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const results = [];
const skipped = [];
const check = (name, body) => {
  body();
  results.push(name);
};
const skip = (name, why) => skipped.push(`${name} -- ${why}`);

const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-activation-'));
const capabilities = probeFilesystemCapabilities(workspace);
const sha256 = value => crypto.createHash('sha256').update(value).digest('hex');

// ---------------------------------------------------------------------------
// The verbatim prefix
// ---------------------------------------------------------------------------

check('a verbatim candidate is contained by an ordinary parent', () => {
  assert.equal(isWithin('C:\\store', '\\\\?\\C:\\store\\generations\\abc'), true);
  assert.equal(isWithin('\\\\?\\C:\\store', 'C:\\store\\generations\\abc'), true);
  assert.equal(isWithin('\\\\?\\C:\\store', '\\\\?\\C:\\store'), true);
});

check('normalization does not widen the guard', () => {
  // The whole risk of stripping a prefix is that it turns a real escape into an
  // accepted path. It must not: an escape is an escape in either spelling.
  assert.equal(isWithin('C:\\store\\generations', '\\\\?\\C:\\store\\other\\x'), false);
  assert.equal(isWithin('C:\\store\\generations', '\\\\?\\D:\\store\\generations\\x'), false);
  assert.equal(isWithin('\\\\?\\C:\\store\\generations', 'C:\\store'), false);
  assert.equal(isWithin('/srv/store/generations', '/srv/store/other'), false);
});

check('the UNC verbatim form reduces to its ordinary spelling', () => {
  assert.equal(stripVerbatimPrefix('\\\\?\\UNC\\server\\share\\x'), '\\\\server\\share\\x');
  assert.equal(stripVerbatimPrefix('\\\\?\\C:\\x'), 'C:\\x');
  assert.equal(stripVerbatimPrefix('/srv/store'), '/srv/store');
  assert.equal(stripVerbatimPrefix('C:\\x'), 'C:\\x');
});

check('an activation pointer may only name a generation', () => {
  assert.equal(POINTER_PATTERN.test(`generations/${'a'.repeat(64)}`), true);
  for (const bad of [
    'generations/../../etc/passwd',
    `generations/${'a'.repeat(63)}`,
    `generations/${'A'.repeat(64)}`,
    `/abs/generations/${'a'.repeat(64)}`,
    `generations/${'a'.repeat(64)}/x`,
    '',
  ]) {
    assert.equal(POINTER_PATTERN.test(bad), false, `accepted ${JSON.stringify(bad)}`);
  }
});

// ---------------------------------------------------------------------------
// The real hook runtime, against a fabricated store
// ---------------------------------------------------------------------------

/**
 * Build the smallest plugin store `hook-runtime-v1.sh` will accept.
 *
 * The runtime reads the manifest with awk and never parses JSON, so the store
 * only needs the fields it greps for plus a dispatcher whose hash matches.
 */
function fabricateStore(name, { pointerTarget = null, makeLink = true } = {}) {
  const root = path.join(workspace, name);
  const generations = path.join(root, 'generations');
  const dispatcherRelative = 'shared/scripts/generated/hook-dispatch-v1.sh';
  const dispatcherBody = '#!/usr/bin/env bash\nprintf \'dispatched %s\\n\' "$*"\n';
  const dispatcherSha = sha256(dispatcherBody);

  // The generation directory is named by its own identity, as the runtime
  // requires; the value itself is arbitrary for this fixture.
  const generation = sha256(`${name}-generation`);
  const bundleId = sha256(`${name}-bundle`);
  const generationRoot = path.join(generations, generation);
  fs.mkdirSync(path.join(generationRoot, path.dirname(dispatcherRelative)), { recursive: true });
  fs.writeFileSync(path.join(generationRoot, ...dispatcherRelative.split('/')), dispatcherBody);
  fs.chmodSync(path.join(generationRoot, ...dispatcherRelative.split('/')), 0o755);
  fs.writeFileSync(
    path.join(generationRoot, 'manifest.json'),
    `${JSON.stringify(
      {
        abi: 'hook-runtime-v1',
        bundleId,
        dispatcherPath: dispatcherRelative,
        dispatcherSha256: dispatcherSha,
        // The runtime launches the dispatcher by the interpreter its receipt
        // names, so a fabricated store has to carry one too.
        dispatcherInterpreter: 'bash',
        generation,
      },
      null,
      2
    )}\n`
  );

  fs.mkdirSync(path.join(root, 'pointers/bundles'), { recursive: true });
  fs.writeFileSync(
    path.join(root, 'pointers/bundles', bundleId),
    `${pointerTarget ?? `generations/${generation}`}\n`
  );

  if (makeLink && capabilities.directoryLinkStrategy !== 'copy') {
    fs.mkdirSync(path.join(root, 'bundles'), { recursive: true });
    const linkPath = path.join(root, 'bundles', bundleId);
    if (capabilities.symlinkDirectory) fs.symlinkSync(`../generations/${generation}`, linkPath, 'dir');
    else fs.symlinkSync(generationRoot, linkPath, 'junction');
  }
  return { root, generation, bundleId };
}

function runRuntime(store, bundleId, extraEnv = {}) {
  return spawnSync(
    'bash',
    [path.join(repoRoot, 'shared/scripts/hook-runtime-v1.sh'), '--bundle', bundleId, '--resolve-only'],
    {
      encoding: 'utf8',
      env: { ...process.env, PROMETHEUS_PLUGIN_ROOT: store, ...extraEnv },
    }
  );
}

const bashAvailable = spawnSync('bash', ['-c', 'exit 0']).status === 0;
if (!bashAvailable) {
  skip('hook runtime resolution', 'no POSIX shell on this host');
} else {
  const good = fabricateStore('runtime-good');

  check('the runtime resolves the generation through the pointer file', () => {
    const result = runRuntime(good.root, good.bundleId);
    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    const payload = JSON.parse(result.stdout);
    assert.equal(payload.status, 'ok');
    assert.equal(payload.bundle, good.bundleId);
    assert.equal(payload.generation, good.generation);
  });

  check('the pointer file, not the link, is authoritative', () => {
    // Remove the convenience link entirely. Resolution must still succeed,
    // which is what demotes the `-L` check from a gate to an advisory.
    const link = path.join(good.root, 'bundles', good.bundleId);
    if (fs.existsSync(link)) fs.rmSync(link, { recursive: true, force: true });
    const result = runRuntime(good.root, good.bundleId);
    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    assert.equal(JSON.parse(result.stdout).generation, good.generation);
  });

  check('a pointer escaping the generation store is refused', () => {
    const escaping = fabricateStore('runtime-escaping', {
      pointerTarget: '../../../etc',
      makeLink: false,
    });
    const result = runRuntime(escaping.root, escaping.bundleId);
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /INVALID_POINTER|ESCAPING_BUNDLE/);
  });

  if (capabilities.directoryLinkStrategy !== 'copy') {
    check('a well-formed pointer that RESOLVES outside the store is refused', () => {
      // The regex above rejects a traversal spelled into the pointer text. This
      // is the case it cannot see: the pointer names a syntactically valid
      // generation whose directory is itself an indirection out of the store.
      // Only the resolve-then-compare containment check catches it, so removing
      // that check would leave this open.
      const outside = path.join(workspace, 'outside-the-store');
      fs.mkdirSync(outside, { recursive: true });
      fs.writeFileSync(path.join(outside, 'manifest.json'), '{}');
      const escaping = fabricateStore('runtime-resolved-escape', { makeLink: false });
      const generationPath = path.join(escaping.root, 'generations', escaping.generation);
      fs.rmSync(generationPath, { recursive: true, force: true });
      if (capabilities.symlinkDirectory) fs.symlinkSync(outside, generationPath, 'dir');
      else fs.symlinkSync(outside, generationPath, 'junction');
      const result = runRuntime(escaping.root, escaping.bundleId);
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, /ESCAPING_BUNDLE/);
    });
  } else {
    skip('resolved pointer escape', 'no directory link primitive to build the escape with');
  }

  check('a pointer naming something other than a generation is refused', () => {
    const bogus = fabricateStore('runtime-bogus', {
      pointerTarget: 'generations/not-a-sha256',
      makeLink: false,
    });
    const result = runRuntime(bogus.root, bogus.bundleId);
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /INVALID_POINTER/);
  });

  check('a missing pointer and missing link report NOT_ACTIVATED', () => {
    const orphan = fabricateStore('runtime-orphan', { makeLink: false });
    fs.rmSync(path.join(orphan.root, 'pointers/bundles', orphan.bundleId));
    const result = runRuntime(orphan.root, orphan.bundleId);
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /NOT_ACTIVATED/);
  });

  if (capabilities.junction && !capabilities.symlinkDirectory) {
    check('a plugin root reached through a junction resolves and stays contained', () => {
      // The realistic Windows shape. `cd` through a junction and `pwd -P` both
      // work because msys2-runtime reports a junction with a drive-letter
      // substitute name as a POSIX symlink, so the containment check sees two
      // fully resolved operands in the same spelling.
      const alias = path.join(workspace, 'store-alias');
      fs.symlinkSync(path.resolve(good.root), alias, 'junction');
      const result = runRuntime(alias, good.bundleId);
      assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
      assert.equal(JSON.parse(result.stdout).generation, good.generation);
    });
  } else {
    skip('junction-rooted plugin root', 'this volume creates directory symlinks, not junctions');
  }

  // The verbatim spelling is deliberately NOT exercised through the shell.
  // msys2 cannot represent it at all -- `cygpath -u '\\?\C:\Users\x'` yields
  // `/c/?/C:/Users/x` and `cd` into it fails -- so a fixture asserting that the
  // shell handles a verbatim root would be asserting something the platform
  // cannot do. `strip_verbatim` in the runtime is a cheap guard for the
  // `//?/` form should any resolver ever emit it; the spelling that genuinely
  // bites is Rust's `fs::canonicalize`, covered by `isWithin` above.
  skip(
    'verbatim plugin root through the shell',
    'msys2 cannot represent a verbatim path; the Node and Rust surfaces carry that guard'
  );

  check('the legacy link-only store still resolves', () => {
    // A store written by an installer that predates the pointer file has only
    // the link. Verification must not invalidate it.
    if (capabilities.directoryLinkStrategy === 'copy') {
      throw new Error('unreachable: no directory link primitive');
    }
    const legacy = fabricateStore('runtime-legacy');
    fs.rmSync(path.join(legacy.root, 'pointers'), { recursive: true, force: true });
    const result = runRuntime(legacy.root, legacy.bundleId);
    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    assert.equal(JSON.parse(result.stdout).generation, legacy.generation);
  });
}

// ---------------------------------------------------------------------------
// Swapping a pointer, and recovering a swap that was interrupted
// ---------------------------------------------------------------------------

__testing.setCapabilities(capabilities);

/** A store root with two generation directories to point at. */
function twoGenerationStore(name) {
  const root = path.join(workspace, name);
  const first = 'a'.repeat(64);
  const second = 'b'.repeat(64);
  for (const generation of [first, second]) {
    fs.mkdirSync(path.join(root, 'generations', generation), { recursive: true });
    fs.writeFileSync(path.join(root, 'generations', generation, 'marker'), generation);
  }
  return { root, first, second };
}

check('a pointer swap moves the file and the link together', () => {
  const store = twoGenerationStore('swap');
  __testing.setActivationPointer(
    store.root,
    'current',
    `generations/${store.first}`,
    `generations/${store.first}`
  );
  assert.equal(__testing.readPointer(store.root, 'current'), `generations/${store.first}`);
  assert.equal(
    fs.readFileSync(path.join(store.root, 'current', 'marker'), 'utf8'),
    store.first,
    'the convenience link must reach the generation the pointer names'
  );

  // The second swap replaces a link that already exists. On Windows this is the
  // case a rename cannot do at all: MoveFileExW refuses when the destination is
  // a directory, and a junction carries the directory attribute.
  __testing.setActivationPointer(
    store.root,
    'current',
    `generations/${store.second}`,
    `generations/${store.second}`
  );
  assert.equal(__testing.readPointer(store.root, 'current'), `generations/${store.second}`);
  assert.equal(fs.readFileSync(path.join(store.root, 'current', 'marker'), 'utf8'), store.second);
  assert.equal(
    fs.existsSync(path.join(store.root, 'pointers/.pending')) &&
      fs.readdirSync(path.join(store.root, 'pointers/.pending')).length,
    0,
    'a completed swap leaves no breadcrumb'
  );
});

check('a swap interrupted between unlink and create is completed on the next run', () => {
  const store = twoGenerationStore('interrupted');
  __testing.setActivationPointer(
    store.root,
    'current',
    `generations/${store.first}`,
    `generations/${store.first}`
  );

  // Reproduce the exact window: the pointer file has already moved, the
  // breadcrumb is on disk, and the process died before the link was recreated.
  __testing.writePointer(store.root, 'current', `generations/${store.second}`);
  fs.writeFileSync(
    __testing.breadcrumbFile(store.root, 'current'),
    `${JSON.stringify({
      schemaVersion: 1,
      link: 'current',
      target: `generations/${store.second}`,
      degradedCopy: 'contents',
      kind: 'directory',
      pid: process.pid,
    })}\n`
  );
  fs.rmSync(path.join(store.root, 'current'), { recursive: true, force: true });
  assert.equal(fs.existsSync(path.join(store.root, 'current')), false);

  const recovered = __testing.recoverPendingLinks(store.root);
  assert.deepEqual(recovered, ['current']);
  assert.equal(fs.readFileSync(path.join(store.root, 'current', 'marker'), 'utf8'), store.second);
  assert.equal(fs.readdirSync(path.join(store.root, 'pointers/.pending')).length, 0);
});

check('the store mutex is exclusive, is reentrant by declaration, and breaks a dead holder', () => {
  const store = twoGenerationStore('lock');
  const lock = path.join(store.root, '.bootstrap-lock');

  assert.equal(
    __testing.withStoreLock(store.root, () => fs.existsSync(lock)),
    true,
    'the lock must exist while it is held'
  );
  assert.equal(fs.existsSync(lock), false, 'the lock must be released afterwards');

  // A lock left behind by a process that no longer exists is broken rather than
  // waited out. Pid 1 is not a plausible holder on any host this runs on, and
  // an unparseable holder is treated the same way.
  fs.mkdirSync(lock, { recursive: true });
  fs.writeFileSync(path.join(lock, 'pid'), '999999999\n');
  assert.equal(__testing.withStoreLock(store.root, () => 'ran'), 'ran');

  // The shell bootstrap acquires this same lock and then invokes the installer;
  // re-acquiring it in the child would deadlock against its own parent.
  fs.mkdirSync(lock, { recursive: true });
  fs.writeFileSync(path.join(lock, 'pid'), `${process.pid}\n`);
  const previous = process.env.PROMETHEUS_STORE_LOCK_HELD;
  process.env.PROMETHEUS_STORE_LOCK_HELD = '1';
  try {
    assert.equal(__testing.withStoreLock(store.root, () => 'ran'), 'ran');
    assert.equal(fs.existsSync(lock), true, 'a declared-held lock must not be released by the child');
  } finally {
    if (previous === undefined) delete process.env.PROMETHEUS_STORE_LOCK_HELD;
    else process.env.PROMETHEUS_STORE_LOCK_HELD = previous;
    fs.rmSync(lock, { recursive: true, force: true });
  }
});

// ---------------------------------------------------------------------------
// Why the junction rung belongs to activation and not to a payload
// ---------------------------------------------------------------------------

/** Materialize `alias -> target` inside `<root>/staged`, then rename it. */
function stageThenRename(name, allowJunction) {
  const root = path.join(workspace, name);
  const staged = path.join(root, 'staged');
  fs.mkdirSync(path.join(staged, 'target'), { recursive: true });
  fs.writeFileSync(path.join(staged, 'target', 'inner.txt'), 'inner\n');
  const outcome = materializeLink({
    linkPath: path.join(staged, 'alias'),
    target: 'target',
    capabilities,
    allowJunction,
  });
  const settled = path.join(root, 'generations-0000');
  fs.renameSync(staged, settled);
  return { settled, outcome };
}

if (capabilities.junction && !capabilities.symlinkDirectory) {
  check('a junction inside a staged payload does NOT survive the rename', () => {
    // This is not a hypothetical. A junction's substitute name is ABSOLUTE --
    // there is no relative form -- so one created inside `.staging-<pid>-<rand>`
    // keeps pointing at that directory after it is renamed to
    // `generations/<id>`, and the entry then resolves to a path that no longer
    // exists. This check exists to keep the reason recorded.
    const { settled, outcome } = stageThenRename('rename-junction', true);
    assert.equal(outcome.realized, 'junction');
    const alias = path.join(settled, 'alias');
    assert.equal(fs.lstatSync(alias).isSymbolicLink(), true, 'it is still a link');
    assert.equal(
      fs.existsSync(path.join(alias, 'inner.txt')),
      false,
      'and it now leads nowhere, because it named the staging path'
    );
  });

  check('the payload ladder skips the junction and survives the rename', () => {
    const { settled, outcome } = stageThenRename('rename-payload', false);
    assert.equal(outcome.realized, 'copy');
    const alias = path.join(settled, 'alias');
    assert.equal(
      fs.readFileSync(alias, 'utf8'),
      'target',
      'the degraded entry holds its recorded target and still verifies against its own hash'
    );
  });
} else if (capabilities.symlinkDirectory) {
  check('a relative symlink survives the rename, which is why POSIX never noticed', () => {
    const { settled, outcome } = stageThenRename('rename-symlink', false);
    assert.equal(outcome.realized, 'symlink');
    assert.equal(
      fs.readFileSync(path.join(settled, 'alias', 'inner.txt'), 'utf8'),
      'inner\n'
    );
  });
} else {
  skip('staging rename', 'no link primitive on this volume');
}

check('a pointer that does not name a generation is refused on write and on read', () => {
  const store = twoGenerationStore('guard');
  assert.throws(() => __testing.writePointer(store.root, 'current', '../../etc'));
  fs.mkdirSync(path.join(store.root, 'pointers'), { recursive: true });
  fs.writeFileSync(path.join(store.root, 'pointers/current'), '../../etc\n');
  assert.throws(() => __testing.readPointer(store.root, 'current'));
});

fs.rmSync(workspace, { recursive: true, force: true });

process.stdout.write(`activation-pointer: ${results.length} checks passed\n`);
for (const name of results) process.stdout.write(`  ok   ${name}\n`);
for (const note of skipped) process.stdout.write(`  SKIP ${note}\n`);
