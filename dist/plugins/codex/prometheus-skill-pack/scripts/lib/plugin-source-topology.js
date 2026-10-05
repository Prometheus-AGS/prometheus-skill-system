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

/** Every directory source the clients have registered for this pack's marketplace. */
export function readRegisteredSources({ home = os.homedir() } = {}) {
  const sources = [];
  const known = readJson(path.join(home, '.claude/plugins/known_marketplaces.json'));
  const knownEntry = known?.[MARKETPLACE];
  const knownPath = knownEntry?.source?.source === 'directory' ? knownEntry.source.path : knownEntry?.installLocation;
  if (knownPath) sources.push({ client: 'claude', origin: 'known_marketplaces.json', path: knownPath });

  const settings = readJson(path.join(home, '.claude/settings.json'));
  const settingsSource = settings?.extraKnownMarketplaces?.[MARKETPLACE]?.source;
  if (settingsSource?.source === 'directory' && settingsSource.path) {
    sources.push({ client: 'claude', origin: 'settings.json', path: settingsSource.path });
  }

  try {
    const toml = fs.readFileSync(path.join(home, '.codex/config.toml'), 'utf8');
    const block = new RegExp(`\\[marketplaces\\.${MARKETPLACE}\\]([^\\[]*)`).exec(toml)?.[1] ?? '';
    const local = /source_type\s*=\s*"local"/.test(block);
    const target = /^\s*source\s*=\s*"([^"]+)"/m.exec(block)?.[1];
    if (local && target) sources.push({ client: 'codex', origin: 'config.toml', path: target });
  } catch {
    // no codex config: nothing registered there
  }
  return sources;
}

function gitOut(cwd, args) {
  const run = spawnSync('git', args, { cwd, encoding: 'utf8' });
  return run.status === 0 ? run.stdout.trim() : null;
}

export function gitFacts(dir) {
  if (gitOut(dir, ['rev-parse', '--git-dir']) === null) return { repo: false };
  const gitDir = gitOut(dir, ['rev-parse', '--absolute-git-dir']);
  const common = gitOut(dir, ['rev-parse', '--path-format=absolute', '--git-common-dir']);
  const branch = gitOut(dir, ['symbolic-ref', '--quiet', '--short', 'HEAD']);
  const dirty = (gitOut(dir, ['status', '--porcelain', '--untracked-files=no']) ?? '') !== '';
  return {
    repo: true,
    linkedWorktree: Boolean(gitDir && common && path.resolve(gitDir) !== path.resolve(common)),
    branch,
    dirty,
  };
}

function finding(code, severity, entry, message, extra = {}) {
  return { code, severity, client: entry?.client, path: entry?.path, message, ...extra };
}

/**
 * Classify registered sources. `facts` is injectable so callers can supply their own git
 * probe. status: skip (nothing registered) | ok | warn | fail (a source is missing).
 */
export function evaluateSources(entries, { facts = gitFacts } = {}) {
  if (entries.length === 0) return { status: 'skip', findings: [], sources: [] };
  // One registry may list the same source twice (known_marketplaces.json and settings.json).
  const unique = [...new Map(entries.map(entry => [`${entry.client}|${path.resolve(entry.path)}`, entry])).values()];
  const findings = [];
  const remedy = 'Point the marketplace at a durable release-line checkout, then restart running sessions.';
  for (const entry of unique) {
    if (!fs.existsSync(entry.path)) {
      findings.push(
        finding('SOURCE_MISSING', 'fail', entry, `${entry.client} marketplace source ${entry.path} does not exist; every hook of this plugin fails until it is restored. ${remedy}`)
      );
      continue;
    }
    const git = facts(entry.path);
    if (!git.repo) {
      findings.push(finding('NOT_A_CHECKOUT', 'warn', entry, `${entry.client} marketplace source ${entry.path} is not a git checkout, so its version cannot be verified.`));
      continue;
    }
    if (!isReleaseLineBranch(git.branch)) {
      findings.push(
        finding('TOPIC_BRANCH', 'warn', entry, `${entry.client} marketplace source ${entry.path} is on topic branch '${git.branch}'; if that branch or worktree is removed, every hook of this plugin fails. ${remedy}`, { branch: git.branch })
      );
    }
    if (git.dirty) {
      findings.push(finding('SOURCE_DIRTY', 'warn', entry, `${entry.client} marketplace source ${entry.path} has uncommitted changes, so what is installed is not what is committed.`));
    }
    if (git.linkedWorktree) {
      findings.push(finding('LINKED_WORKTREE', 'info', entry, `${entry.client} marketplace source ${entry.path} is a linked git worktree: removing the worktree removes the plugin source.`));
    }
  }
  const distinct = [...new Set(entries.map(entry => path.resolve(entry.path)))];
  if (distinct.length > 1) {
    findings.push(
      finding('SOURCES_DISAGREE', 'warn', null, `registered sources disagree: ${entries.map(e => `${e.client}=${e.path}`).join(', ')}. Clients would install different checkouts.`)
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
  const list = dir => {
    try {
      return fs.readdirSync(dir);
    } catch {
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
  return { installedVersion, activeGeneration, findings };
}
