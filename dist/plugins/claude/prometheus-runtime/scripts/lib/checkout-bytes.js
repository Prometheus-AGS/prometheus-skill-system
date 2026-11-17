/**
 * Detect working-tree bytes that are not the bytes git stores.
 *
 * The release manifest pins every runtime file by the sha256 of its bytes on
 * disk. Where git converted those bytes at checkout -- `core.autocrlf=true`
 * under `* text=auto`, an `eol=crlf` rule, any smudge filter -- that hash is a
 * property of the host, not of the commit. Commit 49c296aa shipped exactly that:
 * five runtime hashes equal to sha256(LF blob converted to CRLF), so release
 * payload verification failed on every LF checkout.
 *
 * A file passes when its bytes are exactly what git stores for it: they equal
 * the index blob, or -- for an uncommitted edit or a new file -- git's clean
 * filter leaves them unchanged. Everything else is a converted checkout.
 */

import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';

function git(root, args, input) {
  return spawnSync('git', ['-c', 'core.safecrlf=false', ...args], {
    cwd: root,
    // A string `input` would be encoded with `encoding`, and 'buffer' is not one.
    input: input === undefined ? undefined : Buffer.from(input, 'utf8'),
    encoding: 'buffer',
    maxBuffer: 256 * 1024 * 1024,
    env: { ...process.env, GIT_LITERAL_PATHSPECS: '1' },
  });
}

function countCarriageReturns(bytes) {
  let count = 0;
  for (const byte of bytes) if (byte === 13) count += 1;
  return count;
}

/**
 * Files among `files` ({ path, bytes }, paths relative to `root` with forward
 * slashes) whose bytes git would not store verbatim.
 *
 * Returns [] when `root` is not inside a git work tree: conversion is something
 * a git checkout does, so without one there is nothing to detect.
 */
export function checkoutConversions(root, files) {
  const inside = git(root, ['rev-parse', '--is-inside-work-tree']);
  if (inside.error || inside.status !== 0 || inside.stdout.toString().trim() !== 'true') return [];
  const format = git(root, ['rev-parse', '--show-object-format']);
  // Only git that predates SHA-256 support lacks the flag, and it is SHA-1 only.
  const algorithm = format.status === 0 ? format.stdout.toString().trim() : 'sha1';
  const objectId = bytes =>
    crypto.createHash(algorithm).update(`blob ${bytes.length}\0`).update(bytes).digest('hex');

  const listing = git(root, ['ls-files', '-s', '-z', '--', ...files.map(file => file.path)]);
  if (listing.error || listing.status !== 0) {
    return files.map(file => ({ path: file.path, reason: 'git ls-files failed' }));
  }
  const indexed = new Map();
  for (const record of listing.stdout.toString('utf8').split('\0')) {
    const separator = record.indexOf('\t');
    if (separator < 0) continue;
    const [, id, stage] = record.slice(0, separator).split(' ');
    if (stage === '0') indexed.set(record.slice(separator + 1), id);
  }

  // Nearly every file is byte-identical to its index blob; only the rest need
  // git to say what it would store for them.
  const suspects = files
    .map(file => ({ ...file, id: objectId(file.bytes) }))
    .filter(file => indexed.get(file.path) !== file.id);
  if (suspects.length === 0) return [];
  // --stdin-paths hashes each file through the clean filter its attributes select.
  const hashed = git(
    root,
    ['hash-object', '--stdin-paths'],
    `${suspects.map(file => file.path).join('\n')}\n`
  );
  if (hashed.error || hashed.status !== 0) {
    return suspects.map(file => ({ path: file.path, reason: 'git hash-object failed' }));
  }
  const stored = hashed.stdout.toString().trim().split(/\r?\n/);

  const conversions = [];
  suspects.forEach((file, index) => {
    if (stored[index] === file.id) return; // an uncommitted edit, stored verbatim
    const carriageReturns = countCarriageReturns(file.bytes);
    const origin =
      stored[index] === indexed.get(file.path)
        ? 'it is unmodified, so the checkout converted it'
        : 'git would rewrite them on commit';
    conversions.push({
      path: file.path,
      reason: `working-tree bytes are not the bytes git stores (${carriageReturns} CR byte(s) on disk; ${origin})`,
    });
  });
  return conversions;
}

/** Failure lines for `conversions`, ending with the one remedy that applies. */
export function describeCheckoutConversions(conversions) {
  if (conversions.length === 0) return [];
  const paths = conversions.map(conversion => conversion.path);
  return [
    ...conversions.map(conversion => `${conversion.path}: ${conversion.reason}`),
    "Hashing these would pin this host's line-ending conversion into bundleId. Re-check them " +
      "out under the repository's eol rules: delete them, then run " +
      `git checkout -- ${paths.join(' ')}`,
  ];
}
