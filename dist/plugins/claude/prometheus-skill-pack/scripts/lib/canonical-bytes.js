/**
 * The bytes git stores for a file, independent of how this host checked it out.
 *
 * `core.autocrlf=true` (the Git for Windows default) rewrites LF blobs to CRLF
 * at checkout, so a Windows working tree and a POSIX one hold different bytes
 * for the same commit. Anything that copies or hashes working-tree bytes into a
 * committed artifact -- dist/ -- then produces host-specific output: a Windows
 * rebuild rewrites hundreds of files that no one edited, and the committed copy
 * reads as stale there.
 *
 * canonicalBytes() substitutes the index blob only when it can prove the file
 * on disk IS that blob with CRLF line endings: the object id of the disk bytes
 * with CRLF folded to LF equals the id the index records. Every other file --
 * an uncommitted edit, an untracked file, a blob that is itself stored with
 * CRLF, a file outside any repository -- comes back exactly as it is on disk,
 * so a build still reflects unstaged work.
 *
 * Files without a carriage return cannot have been converted and are returned
 * without consulting git, so a POSIX checkout pays nothing.
 */

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const CARRIAGE_RETURN = 13;
const LINE_FEED = 10;

// directory -> repository root (or null); repository root -> index state (or null)
const owners = new Map();
const repositories = new Map();

function git(root, args) {
  const result = spawnSync('git', args, {
    cwd: root,
    encoding: 'buffer',
    maxBuffer: 256 * 1024 * 1024,
  });
  return result.error || result.status !== 0 ? null : result.stdout;
}

// A submodule's `.git` is a file and a superproject's a directory; either marks
// the repository whose index describes the files beneath it.
function repositoryRoot(directory) {
  if (owners.has(directory)) return owners.get(directory);
  let root = null;
  if (fs.existsSync(path.join(directory, '.git'))) root = directory;
  else {
    const parent = path.dirname(directory);
    if (parent !== directory) root = repositoryRoot(parent);
  }
  owners.set(directory, root);
  return root;
}

// One `ls-files` per repository, never per file: dist/ holds thousands of
// paths, and a pathspec list that long exceeds the Windows command line.
function repository(root) {
  if (repositories.has(root)) return repositories.get(root);
  let state = null;
  const listing = git(root, ['ls-files', '-s', '-z']);
  if (listing) {
    const format = git(root, ['rev-parse', '--show-object-format']);
    const index = new Map();
    for (const record of listing.toString('utf8').split('\0')) {
      const tab = record.indexOf('\t');
      if (tab < 0) continue;
      const [, id, stage] = record.slice(0, tab).split(' ');
      if (stage === '0') index.set(record.slice(tab + 1), id);
    }
    // Only git that predates SHA-256 support lacks the flag, and it is SHA-1 only.
    state = { algorithm: format ? format.toString().trim() : 'sha1', index };
  }
  repositories.set(root, state);
  return state;
}

function objectId(algorithm, bytes) {
  return crypto.createHash(algorithm).update(`blob ${bytes.length}\0`).update(bytes).digest('hex');
}

function foldCrlf(bytes) {
  const folded = Buffer.allocUnsafe(bytes.length);
  let length = 0;
  for (let index = 0; index < bytes.length; index += 1) {
    if (bytes[index] === CARRIAGE_RETURN && bytes[index + 1] === LINE_FEED) continue;
    folded[length] = bytes[index];
    length += 1;
  }
  return folded.subarray(0, length);
}

/** The bytes of `file` as git stores them, or as they are on disk when git cannot vouch otherwise. */
export function canonicalBytes(file) {
  const bytes = fs.readFileSync(file);
  if (!bytes.includes(CARRIAGE_RETURN)) return bytes;
  const absolute = path.resolve(file);
  const root = repositoryRoot(path.dirname(absolute));
  if (!root) return bytes;
  const state = repository(root);
  const id = state?.index.get(path.relative(root, absolute).split(path.sep).join('/'));
  if (!id || objectId(state.algorithm, bytes) === id) return bytes;
  const folded = foldCrlf(bytes);
  return objectId(state.algorithm, folded) === id ? folded : bytes;
}
