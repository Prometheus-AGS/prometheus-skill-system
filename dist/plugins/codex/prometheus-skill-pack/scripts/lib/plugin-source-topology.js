/**
 * Where each client's marketplace entry for this pack points, and whether that
 * location is safe to depend on.
 *
 * WHY THIS EXISTS
 * A plugin whose marketplace `directory` source is deleted fails every one of its hooks
 * on every turn ("Plugin directory does not exist"), in every session that loaded it,
 * and nothing warns beforehand. The incident this was written for: a topic-branch git
 * worktree was registered as the source and removed when its branch merged.
 *
 * The hazard is a TOPIC-BRANCH checkout (it lives only as long as its branch), not a
 * linked worktree as such: a dedicated release-line worktree is a stable source and is
 * only noted. This module is the single authority; the installer scripts and the Rust
 * `prometheus doctor` both call it through scripts/check-plugin-source.js.
 */
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

export const MARKETPLACE = 'prometheus-skill-pack';
const PLUGIN_ID = `${MARKETPLACE}@${MARKETPLACE}`;

/** A branch a durable plugin source may track. A detached HEAD (a tag checkout) is fine. */
export function isReleaseLineBranch(branch) {
  if (!branch) return true;
  return /^(main|master)$/.test(branch) || /(^|\/)main$/.test(branch) || /^release\//.test(branch);
}

function readJson(file) {
  try {
    return JSON.parse(fs.readFileSync(file, 'utf8'));
  } catch {
    return null;
  }
}

/** Distinguishes a file that is simply absent from one that exists but cannot be read or parsed. */
function readJsonStrict(file) {
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch (error) {
    return error?.code === 'ENOENT' ? { state: 'absent' } : { state: 'error', error };
  }
  try {
    return { state: 'ok', value: JSON.parse(text) };
  } catch (error) {
    return { state: 'error', error };
  }
}

/**
 * Every directory source the clients have registered for this pack's marketplace.
 *
 * Registry files are untrusted input, so validation happens HERE, once: every entry that
 * leaves this function is either `{ client, origin, path: <non-empty string> }` or
 * `{ client, origin, path: null, unreadable: true }`. Nothing downstream has to defend
 * against a malformed registration, and no malformed registration can hide another
 * client's source.
 */
export function readRegisteredSources({ home = os.homedir() } = {}) {
  const sources = [];
  const unreadable = (client, origin) => sources.push({ client, origin, path: null, unreadable: true });
  const accept = (client, origin, candidate) => {
    if (typeof candidate === 'string' && candidate.trim() !== '') sources.push({ client, origin, path: candidate });
    else unreadable(client, origin);
  };

  const knownFile = readJsonStrict(path.join(home, '.claude/plugins/known_marketplaces.json'));
  if (knownFile.state === 'error') unreadable('claude', 'known_marketplaces.json');
  if (knownFile.state === 'ok') {
    const entry = knownFile.value?.[MARKETPLACE];
    if (entry?.source?.source === 'directory') {
      // A directory source must carry a usable path; anything else is a malformed registration.
      accept('claude', 'known_marketplaces.json', entry.source.path);
    } else if (typeof entry?.installLocation === 'string' && entry.installLocation.trim() !== '' && entry?.source?.source === undefined) {
      // Older registrations record only where the marketplace was installed.
      accept('claude', 'known_marketplaces.json', entry.installLocation);
    }
    // Any other source type (for example github) has no directory to check: nothing registered.
  }

  const settingsFile = readJsonStrict(path.join(home, '.claude/settings.json'));
  if (settingsFile.state === 'error') unreadable('claude', 'settings.json');
  if (settingsFile.state === 'ok') {
    const source = settingsFile.value?.extraKnownMarketplaces?.[MARKETPLACE]?.source;
    if (source?.source === 'directory') accept('claude', 'settings.json', source.path);
  }

  let toml = null;
  try {
    toml = fs.readFileSync(path.join(home, '.codex/config.toml'), 'utf8');
  } catch (error) {
    // Only a missing file means "nothing registered"; EACCES, EISDIR and the like do not.
    if (error?.code !== 'ENOENT') unreadable('codex', 'config.toml');
  }
  if (toml !== null) {
    const registration = readCodexRegistration(toml);
    if (registration.path) accept('codex', 'config.toml', registration.path);
    // Never silently ignore a registration we could not read: that would hide both a
    // missing source and the installer guard.
    else if (registration.mentioned) unreadable('codex', 'config.toml');
  }
  return sources;
}

/** A TOML string value: "basic" (with the common escapes) or 'literal'. */
function tomlString(raw) {
  const value = raw.trim();
  if (value.startsWith("'")) return /^'([^']*)'/.exec(value)?.[1] ?? null;
  if (!value.startsWith('"')) return null;
  const match = /^"((?:[^"\\]|\\.)*)"/.exec(value);
  if (!match) return null;
  // TOML basic-string escapes: \b \t \n \f \r \" \\ and \uXXXX / \UXXXXXXXX.
  const simple = { b: '\b', t: '\t', n: '\n', f: '\f', r: '\r', '"': '"', '\\': '\\' };
  try {
    return match[1].replace(/\\(?:([btnfr"\\])|u([0-9a-fA-F]{4})|U([0-9a-fA-F]{8}))/g, (_all, one, u4, u8) => {
      if (one) return simple[one];
      return String.fromCodePoint(parseInt(u4 ?? u8, 16));
    });
  } catch {
    return null; // an invalid code point is an unreadable value, not a path
  }
}

const KEY = `(?:"${MARKETPLACE}"|'${MARKETPLACE}'|${MARKETPLACE})`;

/**
 * Codex's `[marketplaces.<name>]` registration. Accepts the table header with a bare,
 * double-quoted or single-quoted key, either string form for values, and dotted keys.
 * `mentioned` is true when the file refers to the marketplace at all, so an unreadable
 * registration is reported instead of vanishing.
 */
function readCodexRegistration(toml) {
  // Presence is detected broadly on purpose: any non-comment line that names the marketplace in a
  // file that has a `marketplaces` section counts as a registration, so an unsupported spelling
  // (an inline table under `[marketplaces]`, say) is reported as unreadable, never as absent.
  const body = toml.split(/\r?\n/).filter(line => !/^\s*#/.test(line)).join('\n');
  const mentioned = /marketplaces/.test(body) && body.includes(MARKETPLACE);
  const values = {};
  let inTable = false;
  let atRoot = true; // dotted keys below only count at the document root, before any [table] header
  const header = new RegExp(`^\\s*\\[\\s*marketplaces\\s*\\.\\s*${KEY}\\s*\\]\\s*(?:#.*)?$`);
  const dotted = new RegExp(`^\\s*marketplaces\\s*\\.\\s*${KEY}\\s*\\.\\s*(source_type|source)\\s*=\\s*(.+)$`);
  for (const line of toml.split(/\r?\n/)) {
    if (/^\s*\[/.test(line)) {
      inTable = header.test(line);
      atRoot = false;
      continue;
    }
    // `marketplaces.<name>.source = ...` inside some other [table] defines that table's own key.
    const direct = atRoot ? dotted.exec(line) : null;
    if (direct) values[direct[1]] = tomlString(direct[2]);
    else if (inTable) {
      const pair = /^\s*(source_type|source)\s*=\s*(.+)$/.exec(line);
      if (pair) values[pair[1]] = tomlString(pair[2]);
    }
  }
  return {
    mentioned,
    path: values.source_type === 'local' && values.source ? values.source : null,
  };
}

// Read-only on purpose: `git status` refreshes and WRITES the index unless optional locks
// are off, and the doctor must not modify the checkout it inspects.
function runGit(cwd, args) {
  const run = spawnSync('git', ['--no-optional-locks', ...args], {
    cwd,
    encoding: 'utf8',
    env: { ...process.env, GIT_OPTIONAL_LOCKS: '0' },
  });
  return { status: run.status, stdout: (run.stdout ?? '').trim(), stderr: (run.stderr ?? '').trim(), error: run.error };
}

const firstLine = text => (text ?? '').split('\n')[0];

/**
 * Facts about a checkout. `run` is injectable for tests.
 *
 * The BRANCH is the decisive fact (it is what the installer guard acts on), so a probe that
 * establishes it must succeed or this throws: a failure is never read as "release branch" or
 * "not a checkout". The auxiliary facts (worktree link, dirtiness) degrade independently into
 * `partial`, so one failed probe cannot hide an established topic branch. `repo: false` is
 * returned only when git itself says the directory is not a repository.
 */
export function gitFacts(dir, run = runGit) {
  const inside = run(dir, ['rev-parse', '--git-dir']);
  if (inside.error) throw new Error(`git could not be run: ${inside.error.message}`);
  if (inside.status !== 0) {
    if (/not a git repository/i.test(inside.stderr)) return { repo: false };
    throw new Error(`git rev-parse --git-dir failed${inside.stderr ? `: ${firstLine(inside.stderr)}` : ''}`);
  }
  // `symbolic-ref --quiet` exits 1 with no output on a detached HEAD (expected); any other failure is an error.
  const head = run(dir, ['symbolic-ref', '--quiet', '--short', 'HEAD']);
  if (head.status !== 0 && head.status !== 1) throw new Error(`git symbolic-ref failed${head.stderr ? `: ${firstLine(head.stderr)}` : ''}`);
  const branch = head.status === 0 ? head.stdout : null;
  const partial = [];
  const probe = (args, what) => {
    const result = run(dir, args);
    if (result.status === 0) return result.stdout;
    partial.push(`git ${what} failed${result.stderr ? `: ${firstLine(result.stderr)}` : ''}`);
    return null;
  };
  const gitDir = probe(['rev-parse', '--absolute-git-dir'], 'rev-parse --absolute-git-dir');
  const common = probe(['rev-parse', '--path-format=absolute', '--git-common-dir'], 'rev-parse --git-common-dir');
  // Tracked changes and untracked, non-ignored files both mean the install is not what is committed.
  const status = probe(['status', '--porcelain'], 'status');
  return {
    repo: true,
    linkedWorktree: gitDir && common ? path.resolve(gitDir) !== path.resolve(common) : null,
    branch,
    dirty: status === null ? null : status !== '',
    partial,
  };
}

function finding(code, severity, entry, message, extra = {}) {
  return { code, severity, client: entry?.client, path: entry?.path, message, ...extra };
}

/**
 * Classify registered sources. `facts` is injectable so callers can supply their own git
 * probe. status: skip (nothing registered) | ok | warn | fail (a source is missing).
 */
/** Canonical form for comparing sources: symlink aliases of one checkout are one source. */
export function canonicalPath(p) {
  try {
    return fs.realpathSync(p);
  } catch {
    return path.resolve(p);
  }
}

export function evaluateSources(entries, { facts = gitFacts } = {}) {
  if (entries.length === 0) return { status: 'skip', findings: [], sources: [] };
  const findings = [];
  const remedy = 'Point the marketplace at a durable release-line checkout, then restart running sessions.';

  for (const entry of entries.filter(e => !e.path)) {
    findings.push(
      finding(
        'REGISTRATION_UNREADABLE',
        'warn',
        null,
        `the ${entry.client} registration (${entry.origin}) could not be read, so its marketplace source cannot be checked.`,
        { client: entry.client, origin: entry.origin }
      )
    );
  }

  // One registry may list the same source twice (known_marketplaces.json and settings.json).
  const readable = entries.filter(entry => entry.path);
  const unique = [...new Map(readable.map(entry => [`${entry.client}|${canonicalPath(entry.path)}`, entry])).values()];

  // One entry's failure must never hide another's: every entry is inspected on its own.
  const inspect = entry => {
    // Only "it is not there" is a missing source. A path that cannot be examined (EACCES, ELOOP...)
    // is an inspection failure, which stays advisory: the source may well exist.
    try {
      fs.statSync(entry.path);
    } catch (error) {
      if (error?.code === 'ENOENT' || error?.code === 'ENOTDIR') {
        findings.push(
          finding('SOURCE_MISSING', 'fail', entry, `${entry.client} marketplace source ${entry.path} does not exist; every hook of this plugin fails until it is restored. ${remedy}`)
        );
      } else {
        findings.push(finding('INSPECTION_FAILED', 'warn', entry, `${entry.client} marketplace source ${entry.path} could not be examined (${error?.code ?? error}), so it cannot be checked.`));
      }
      return;
    }
    const git = facts(entry.path);
    if (!git.repo) {
      findings.push(finding('NOT_A_CHECKOUT', 'warn', entry, `${entry.client} marketplace source ${entry.path} is not a git checkout, so its version cannot be verified.`));
      return;
    }
    if (!isReleaseLineBranch(git.branch)) {
      findings.push(
        finding('TOPIC_BRANCH', 'warn', entry, `${entry.client} marketplace source ${entry.path} is on topic branch '${git.branch}'; if that branch or worktree is removed, every hook of this plugin fails. ${remedy}`, { branch: git.branch })
      );
    }
    if (git.partial?.length) {
      findings.push(finding('INSPECTION_FAILED', 'warn', entry, `${entry.client} marketplace source ${entry.path} was only partly inspected: ${git.partial.join('; ')}.`));
    }
    if (git.dirty) {
      findings.push(finding('SOURCE_DIRTY', 'warn', entry, `${entry.client} marketplace source ${entry.path} has uncommitted or untracked changes, so what is installed is not what is committed.`));
    }
    if (git.linkedWorktree) {
      findings.push(finding('LINKED_WORKTREE', 'info', entry, `${entry.client} marketplace source ${entry.path} is a linked git worktree: removing the worktree removes the plugin source.`));
    }
  };
  for (const entry of unique) {
    try {
      inspect(entry);
    } catch (error) {
      findings.push(finding('INSPECTION_FAILED', 'warn', entry, `${entry.client} marketplace source ${entry.path} could not be inspected: ${error?.message ?? error}`));
    }
  }

  const distinct = [...new Set(readable.map(entry => canonicalPath(entry.path)))];
  if (distinct.length > 1) {
    findings.push(
      finding('SOURCES_DISAGREE', 'warn', null, `registered sources disagree: ${readable.map(e => `${e.client}=${e.path}`).join(', ')}. Clients would install different checkouts.`)
    );
  }
  const status = findings.some(f => f.severity === 'fail') ? 'fail' : findings.some(f => f.severity === 'warn') ? 'warn' : 'ok';
  return { status, findings, sources: entries };
}

function pidAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error?.code === 'EPERM';
  }
}

/**
 * The installed native cache against the active generation, and live sessions still
 * running on superseded cache versions. Informational: a session on an old version
 * keeps working while its bundle is registered; it is reported so it can be restarted.
 */
export function nativeCacheSkew({ home = os.homedir() } = {}) {
  const installed = readJson(path.join(home, '.claude/plugins/installed_plugins.json'))?.plugins?.[PLUGIN_ID]?.[0];
  const installedVersion = installed?.version ?? null;
  const generation = readJson(path.join(home, '.prometheus/plugins/prometheus-skill-pack/current/skill-system.json'));
  const activeGeneration = generation?.releaseVersion ?? null;
  const findings = [];
  if (installedVersion && activeGeneration && installedVersion !== activeGeneration) {
    findings.push({
      code: 'CACHE_BEHIND_GENERATION',
      severity: 'warn',
      message: `the installed Claude plugin is ${installedVersion} but the active generation is ${activeGeneration}; refresh the native plugin installs.`,
    });
  }
  const cacheRoot = path.join(home, '.claude/plugins/cache', MARKETPLACE, MARKETPLACE);
  // An absent directory is simply empty; any other failure (EACCES, ENOTDIR...) makes the
  // inspection incomplete and must be said so rather than read as "no live sessions".
  const unreadableDirs = [];
  const list = dir => {
    try {
      return fs.readdirSync(dir);
    } catch (error) {
      if (error?.code !== 'ENOENT') unreadableDirs.push(`${dir} (${error?.code ?? 'unreadable'})`);
      return [];
    }
  };
  if (installedVersion) {
    for (const version of list(cacheRoot)) {
      if (version === installedVersion) continue;
      const inUse = path.join(cacheRoot, version, '.in_use');
      const pids = list(inUse).map(Number).filter(pid => Number.isInteger(pid) && pid > 0 && pidAlive(pid)).sort((a, b) => a - b);
      if (pids.length > 0) {
        findings.push({
          code: 'LIVE_SESSIONS_ON_SUPERSEDED',
          severity: 'info',
          version,
          pids,
          message: `${pids.length} live process(es) (${pids.join(', ')}) still use superseded plugin version ${version}; restart those sessions to move them to ${installedVersion}.`,
        });
      }
    }
  }
  if (unreadableDirs.length > 0) {
    findings.push({
      code: 'SKEW_UNREADABLE',
      severity: 'info',
      message: `the native plugin cache could not be fully inspected; could not read: ${unreadableDirs.join(', ')}.`,
    });
  }
  return { installedVersion, activeGeneration, findings };
}
