// Isolation for harness eval runs (vlbv-r4-portability T2, AUTH-ISO-1, SAFE-1).
//
// Every run gets a scratch root outside the repository holding the project, a
// scratch HOME and the harness's relocated config home. Only the minimum auth
// material is copied in (owner decision D-R4-03), config files are reduced to
// provider/model settings (no MCP servers, hooks or plugins), and the whole
// scratch root is deleted after the run. Real harness state is hashed before
// and after; it is only ever read.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { ROOT, hashPath, run, sha256 } from './common.mjs';

const HOME = os.homedir();

/**
 * Real harness files the runner reads or must leave untouched.
 * ~/.claude.json and Claude's keychain items are excluded: the interactive Claude
 * Code session that drives a boundary run rewrites them continuously, so they
 * cannot attribute a change to the evals (B5 attempt 2b false alarm). Eval
 * sessions never read them: they run with a scratch CLAUDE_CONFIG_DIR and the
 * CLAUDE_CODE_OAUTH_TOKEN environment variable.
 */
export const REAL_STATE = [
  '.claude/settings.json', '.claude/.credentials.json',
  '.codex/auth.json', '.codex/config.toml',
  '.local/share/opencode/auth.json', '.config/opencode/opencode.json',
  '.kimi-code/config.toml', '.kimi-code/credentials', '.kimi-code/oauth', '.kimi-code/device_id', '.kimi-code/region',
  '.qwen/settings.json', '.qwen/oauth_creds.json',
].map(rel => path.join(HOME, rel));

/** File names that identify copied auth material (leftover search). */
export const AUTH_NAMES = ['auth.json', 'credentials', 'oauth', 'oauth_creds.json', '.credentials.json', 'device_id'];

/** Keychain items that belong to the evaluated harnesses (matched on attributes). */
const KEYCHAIN_MATCH = /codex|openai|opencode|kimi|moonshot|qwen|dashscope/i;

/**
 * Hash of the harness-related login-keychain items (attributes only; never
 * secret values). Unrelated items are excluded so other apps' keychain activity
 * during a long run is not reported as a harness state change.
 */
export function keychainListingHash() {
  if (process.platform !== 'darwin') return 'not-darwin';
  const out = run('security', ['dump-keychain'], { timeoutMs: 30_000 });
  if (out.exit !== 0) return `unavailable:exit-${out.exit}`;
  // dump-keychain without -d prints attributes only. Each item starts with a
  // `keychain:` line; keep whole items whose attributes name a harness.
  const items = out.stdout.split(/^(?=keychain: )/m).filter(item => KEYCHAIN_MATCH.test(item));
  return `${items.length}:${sha256(items.join(''))}`;
}

/** Before/after manifest of real harness state. */
export function realStateManifest() {
  const files = Object.fromEntries(REAL_STATE.map(file => [file.replace(HOME, '~'), hashPath(file)]));
  return { files, keychainListing: keychainListingHash() };
}

/** @returns {string[]} differences between two manifests */
export function compareManifests(before, after) {
  const diffs = [];
  for (const [file, digest] of Object.entries(before.files)) {
    if (after.files[file] !== digest) diffs.push(`changed: ${file}`);
  }
  if (before.keychainListing !== after.keychainListing) diffs.push('changed: login keychain item listing');
  return diffs;
}

/** Create a scratch root with home, project and tmp directories. */
export function createScratch(label) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), `vlbv-eval-${label}-`)));
  if (root.startsWith(ROOT)) throw new Error(`scratch root ${root} is inside the repository`);
  const dirs = { root, home: path.join(root, 'home'), project: path.join(root, 'project'), tmp: path.join(root, 'tmp') };
  for (const dir of [dirs.home, dirs.tmp]) fs.mkdirSync(dir, { recursive: true });
  return dirs;
}

/** Base environment shared by every adapter: scratch HOME and XDG, no control plane, no learning writes. */
export function baseEnv(scratch) {
  return {
    PATH: process.env.PATH,
    LANG: process.env.LANG ?? 'en_US.UTF-8',
    TERM: 'dumb',
    HOME: scratch.home,
    USERPROFILE: scratch.home,
    TMPDIR: scratch.tmp,
    XDG_CONFIG_HOME: path.join(scratch.home, '.config'),
    XDG_DATA_HOME: path.join(scratch.home, '.local/share'),
    XDG_CACHE_HOME: path.join(scratch.home, '.cache'),
    XDG_STATE_HOME: path.join(scratch.home, '.local/state'),
    PROMETHEUS_PLUGIN_ROOT: path.join(scratch.root, 'plugins'),
    PROMETHEUS_DATA_DIR: path.join(scratch.root, 'data'),
    PROMETHEUS_KBD_CONTROL_PLANE: '0',
    PROMETHEUS_CONTROL_ENDPOINT: 'http://127.0.0.1:1',
    PROMETHEUS_LEARNING_CORTEX: '0',
    PROMETHEUS_LEARNING_PK: '0',
    GIT_CONFIG_NOSYSTEM: '1',
    GIT_CONFIG_GLOBAL: os.devNull,
    NO_COLOR: '1',
  };
}

/** Copy one real file or directory into the scratch home when it exists. Returns the copied path or null. */
export function copyAuth(realRel, scratchHome, destRel = realRel) {
  const from = path.join(HOME, realRel);
  if (!fs.existsSync(from)) return null;
  const to = path.join(scratchHome, destRel);
  fs.mkdirSync(path.dirname(to), { recursive: true });
  fs.cpSync(from, to, { recursive: true, dereference: true });
  if (fs.statSync(to).isFile()) fs.chmodSync(to, 0o600);
  return to;
}

/**
 * Keep only top-level keys and the named TOML tables (with their subtables).
 * @param {string} text @param {RegExp} keepKey top-level key filter @param {string[]} keepTables table prefixes
 */
export function filterToml(text, keepKey, keepTables) {
  const out = [];
  let section = null;
  let keep = true;
  for (const line of text.split('\n')) {
    const header = line.match(/^\s*\[\[?\s*([^\]]+?)\s*\]\]?\s*$/);
    if (header) {
      section = header[1].replace(/"/g, '');
      keep = keepTables.some(prefix => section === prefix || section.startsWith(`${prefix}.`));
      if (keep) out.push(line);
      continue;
    }
    if (section === null) {
      if (/^\s*$/.test(line) || /^\s*#/.test(line) || keepKey.test(line)) out.push(line);
    } else if (keep) out.push(line);
  }
  return `${out.join('\n').trim()}\n`;
}

/** Write a reduced copy of a real TOML config into the scratch home. */
export function copyFilteredToml(realRel, scratchHome, keepKey, keepTables) {
  const from = path.join(HOME, realRel);
  if (!fs.existsSync(from)) return null;
  const to = path.join(scratchHome, realRel);
  fs.mkdirSync(path.dirname(to), { recursive: true });
  fs.writeFileSync(to, filterToml(fs.readFileSync(from, 'utf8'), keepKey, keepTables), { mode: 0o600 });
  return to;
}

/** Write a reduced copy of a real JSON config keeping only the named top-level keys. */
export function copyFilteredJson(realRel, scratchHome, keepKeys) {
  const from = path.join(HOME, realRel);
  if (!fs.existsSync(from)) return null;
  let parsed;
  try { parsed = JSON.parse(fs.readFileSync(from, 'utf8')); } catch { return null; }
  const reduced = Object.fromEntries(Object.entries(parsed).filter(([key]) => keepKeys.includes(key)));
  const to = path.join(scratchHome, realRel);
  fs.mkdirSync(path.dirname(to), { recursive: true });
  fs.writeFileSync(to, `${JSON.stringify(reduced, null, 2)}\n`, { mode: 0o600 });
  return to;
}

/** Find auth-named files anywhere under a directory. */
export function findAuthFiles(dir) {
  const hits = [];
  if (!fs.existsSync(dir)) return hits;
  const walk = current => {
    for (const name of fs.readdirSync(current)) {
      const full = path.join(current, name);
      const stat = fs.lstatSync(full);
      if (AUTH_NAMES.includes(name)) hits.push(full);
      if (stat.isDirectory() && !stat.isSymbolicLink()) walk(full);
    }
  };
  walk(dir);
  return hits;
}

/** Remove a scratch root. Returns auth files still present afterwards (expected: none). */
export function destroyScratch(scratch, { keep = false } = {}) {
  if (!keep) fs.rmSync(scratch.root, { recursive: true, force: true });
  return findAuthFiles(scratch.root);
}

/** Auth-named files tracked or untracked in the repository working tree (expected: none new). */
export function repoAuthFiles() {
  const out = run('git', ['status', '--porcelain', '--untracked-files=all'], { cwd: ROOT, timeoutMs: 30_000 });
  return out.stdout.split('\n').map(l => l.slice(3).trim()).filter(Boolean)
    .filter(file => AUTH_NAMES.includes(path.basename(file)));
}
