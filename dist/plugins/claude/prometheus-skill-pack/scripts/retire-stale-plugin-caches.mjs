#!/usr/bin/env node

// Native cache retirement is deliberately separate from generation activation.
// Old caches carry their own installers: upgrading the store cannot patch them.
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { isWithin, POINTER_PATTERN, resolveCodexHome } from './lib/store-paths.js';

const NAME = 'prometheus-skill-pack';
const ID = `${NAME}@${NAME}`;
const SCHEMA = 'prometheus-native-cache-retirement-v1';
const digest = value => crypto.createHash('sha256').update(value).digest('hex');
const readJson = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const present = file => Boolean(fs.lstatSync(file, { throwIfNoEntry: false }));
const fail = message => {
  throw new Error(message);
};

function options(argv) {
  if (argv.includes('--preview') && argv.includes('--apply'))
    fail('--preview and --apply are mutually exclusive');
  const args = {
    apply: false,
    targets: 'all',
    sourceRoot: path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..'),
    codexBin: process.env.PROMETHEUS_CODEX_BIN || 'codex',
  };
  const values = {
    '--home': 'home',
    '--plugin-root': 'pluginRoot',
    '--source-root': 'sourceRoot',
    '--quarantine-root': 'quarantineRoot',
    '--targets': 'targets',
    '--codex-bin': 'codexBin',
  };
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === '--apply') args.apply = true;
    else if (argv[i] === '--preview') continue;
    else if (values[argv[i]]) {
      const key = values[argv[i]];
      if (!argv[i + 1] || argv[i + 1].startsWith('--')) fail(`missing value for ${argv[i]}`);
      args[key] = argv[++i];
    } else fail(`unknown argument: ${argv[i]}`);
  }
  if (!args.home || !args.pluginRoot)
    fail(
      '--home and --plugin-root are required; preview is the default, --apply performs relocation'
    );
  for (const key of ['home', 'pluginRoot', 'sourceRoot']) args[key] = path.resolve(args[key]);
  args.codexHome = resolveCodexHome(args.home);
  args.quarantineRoot = path.resolve(
    args.quarantineRoot || path.join(args.home, '.prometheus/plugin-cache-quarantine')
  );
  args.caches = [
    {
      client: 'claude',
      root: path.join(args.home, '.claude/plugins/cache', NAME, NAME),
      loader: path.join(args.home, '.claude/plugins'),
    },
    {
      client: 'codex',
      root: path.join(args.codexHome, 'plugins/cache', NAME, NAME),
      loader: path.join(args.codexHome, 'plugins'),
    },
  ];
  return args;
}

// Resolve existing ancestors, including symlinks, without creating anything.
function canonical(file) {
  if (present(file)) return fs.realpathSync(file);
  const parent = path.dirname(file);
  return parent === file ? file : path.join(canonical(parent), path.basename(file));
}

function safePaths(args) {
  const quarantine = canonical(args.quarantineRoot);
  if (quarantine === path.parse(quarantine).root || quarantine === canonical(args.home))
    fail('unsafe quarantine root');
  for (const root of [args.pluginRoot, ...args.caches.map(cache => cache.loader)]) {
    const actual = canonical(root);
    if (isWithin(actual, quarantine) || isWithin(quarantine, actual))
      fail(`quarantine must be separate from loader and generation paths: ${root}`);
  }
}

function active(args) {
  const pointer = fs.readFileSync(path.join(args.pluginRoot, 'pointers/current'), 'utf8').trim();
  if (!POINTER_PATTERN.test(pointer))
    fail('current pointer is missing or invalid; repair activation before cache retirement');
  const generation = path.join(args.pluginRoot, pointer);
  if (!isWithin(canonical(path.join(args.pluginRoot, 'generations')), canonical(generation)))
    fail('current generation escapes store');
  const manifest = readJson(path.join(generation, 'manifest.json'));
  plainVersion(manifest.sourceVersion);
  return { generation: pointer, version: manifest.sourceVersion };
}

// Do not guess the ordering of prereleases or unrelated cache directory names.
function plainVersion(version) {
  if (typeof version !== 'string' || !/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(version))
    fail(`cannot safely order cache version: ${version}`);
  return version.split('.').map(BigInt);
}

function older(left, right) {
  const a = plainVersion(left),
    b = plainVersion(right);
  for (let i = 0; i < 3; i += 1) if (a[i] !== b[i]) return a[i] < b[i];
  return false;
}

function liveSessions(root) {
  const dir = path.join(root, '.in_use');
  if (!present(dir)) return [];
  return fs
    .readdirSync(dir)
    .filter(name => /^[1-9]\d*$/.test(name))
    .map(Number)
    .filter(pid => {
      try {
        process.kill(pid, 0);
        return true;
      } catch (error) {
        return error.code === 'EPERM';
      }
    })
    .sort((a, b) => a - b);
}

function treeIdentity(root) {
  const entries = [];
  function walk(relative) {
    const file = path.join(root, relative),
      stat = fs.lstatSync(file);
    const entry = { path: relative.split(path.sep).join('/'), mode: stat.mode & 0o777 };
    if (stat.isSymbolicLink())
      entries.push({ ...entry, type: 'link', target: fs.readlinkSync(file) });
    else if (stat.isDirectory()) {
      entries.push({ ...entry, type: 'directory' });
      for (const name of fs.readdirSync(file).sort()) walk(path.join(relative, name));
    } else if (stat.isFile())
      entries.push({ ...entry, type: 'file', sha256: digest(fs.readFileSync(file)) });
    else fail(`unsupported cache entry: ${file}`);
  }
  walk('');
  return digest(JSON.stringify(entries));
}

function inspect(args, current) {
  const candidates = [],
    blockers = [],
    preserved = [];
  for (const cache of args.caches) {
    if (!present(cache.root)) continue;
    if (fs.lstatSync(cache.root).isSymbolicLink())
      fail(`cache root must be a real directory: ${cache.root}`);
    if (canonical(cache.root) !== path.join(canonical(cache.loader), 'cache', NAME, NAME))
      fail(`cache ancestors redirect outside the native cache: ${cache.root}`);
    if (isWithin(canonical(args.pluginRoot), canonical(cache.root)))
      fail(`native cache overlaps the generation store: ${cache.root}`);
    for (const version of fs.readdirSync(cache.root).sort()) {
      const from = path.join(cache.root, version);
      if (!/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(version)) {
        preserved.push({ path: from, reason: 'unrecognized version; inspect manually' });
        continue;
      }
      if (!older(version, current.version)) {
        preserved.push({ path: from, reason: 'current or newer version' });
        continue;
      }
      const stat = fs.lstatSync(from);
      if (!stat.isDirectory() || stat.isSymbolicLink()) {
        blockers.push(`older cache is not a real directory: ${from}`);
        continue;
      }
      const plugin = readJson(path.join(from, `.${cache.client}-plugin/plugin.json`));
      if (plugin.name !== NAME || plugin.version !== version) {
        blockers.push(`cache identity disagrees with directory: ${from}`);
        continue;
      }
      const pids = liveSessions(from);
      if (pids.length)
        blockers.push(`restart sessions using ${from}: live PIDs ${pids.join(', ')}`);
      const sha256 = treeIdentity(from);
      const key = `${cache.client}-${version}-${digest(`${canonical(from)}\n${sha256}`)}`;
      candidates.push({
        client: cache.client,
        version,
        from,
        to: path.join(args.quarantineRoot, key),
        sha256,
        inventory: path.join(args.quarantineRoot, `${key}.json`),
        livePids: pids,
      });
    }
  }
  return { candidates, blockers, preserved };
}

function checkClaude(args, current, candidates) {
  if (!candidates.some(candidate => candidate.client === 'claude')) return;
  const registryFile = path.join(args.home, '.claude/plugins/installed_plugins.json');
  if (!present(registryFile))
    fail('Claude registration is absent; refresh the native plugin before retiring its caches');
  const rows = readJson(registryFile)?.plugins?.[ID];
  if (!Array.isArray(rows) || !rows.length)
    fail(`Claude registration missing for ${ID}; refresh through the supported installer`);
  for (const row of rows) {
    if (
      row.version !== current.version ||
      typeof row.installPath !== 'string' ||
      !path.isAbsolute(row.installPath)
    )
      fail(
        `Claude registration still names ${row.version ?? 'unknown'}; refresh every registered scope to ${current.version}`
      );
    const plugin = readJson(path.join(row.installPath, '.claude-plugin/plugin.json'));
    if (plugin.name !== NAME || plugin.version !== current.version)
      fail(`Claude registered payload does not match active release: ${row.installPath}`);
    if (
      candidates.some(candidate => isWithin(canonical(candidate.from), canonical(row.installPath)))
    )
      fail(`Claude still registers an obsolete cache: ${row.installPath}`);
  }
}

function checkCodex(args, current, candidates) {
  if (!candidates.some(candidate => candidate.client === 'codex')) return;
  const result = spawnSync(args.codexBin, ['plugin', 'list', '--json'], {
    encoding: 'utf8',
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
    env: { ...process.env, HOME: args.home, CODEX_HOME: args.codexHome },
  });
  if (result.error || result.status !== 0)
    fail(
      `cannot inspect Codex registration through plugin list: ${result.error?.message || result.stderr}`
    );
  const rows = JSON.parse(result.stdout)?.installed;
  if (!Array.isArray(rows)) fail('unrecognized Codex plugin list response');
  const matches = rows.filter(row => row.pluginId === ID && row.installed);
  if (!matches.length)
    fail(`Codex registration missing for ${ID}; refresh through the supported installer`);
  for (const row of matches) {
    if (row.version !== current.version || !row.enabled)
      fail(`Codex must register and enable release ${current.version} before cache retirement`);
    for (const location of [row.installPath, row.source?.path].filter(Boolean)) {
      if (!path.isAbsolute(location))
        fail(`Codex returned a non-absolute plugin path: ${location}`);
      if (candidates.some(candidate => isWithin(canonical(candidate.from), canonical(location))))
        fail(`Codex still registers an obsolete cache: ${location}`);
    }
  }
}

function syncDirectory(dir) {
  let fd;
  try {
    fd = fs.openSync(dir, 'r');
    fs.fsyncSync(fd);
  } catch (error) {
    if (!['EINVAL', 'ENOTSUP', 'EISDIR', 'EPERM', 'EACCES'].includes(error.code)) throw error;
  } finally {
    if (fd !== undefined) fs.closeSync(fd);
  }
}

function save(file, record) {
  const temp = `${file}.${process.pid}.tmp`;
  const fd = fs.openSync(temp, 'wx', 0o600);
  try {
    fs.writeFileSync(fd, `${JSON.stringify(record, null, 2)}\n`);
    fs.fsyncSync(fd);
  } finally {
    fs.closeSync(fd);
  }
  fs.renameSync(temp, file);
  syncDirectory(path.dirname(file));
}

function inventories(args, mutate = false) {
  if (!present(args.quarantineRoot)) return [];
  const records = [];
  for (const name of fs
    .readdirSync(args.quarantineRoot)
    .sort()
    .filter(name => name.endsWith('.json'))) {
    const file = path.join(args.quarantineRoot, name),
      record = readJson(file);
    if (record.schema !== SCHEMA) continue;
    const cache = args.caches.find(cache => cache.client === record.client);
    if (
      !cache ||
      path.dirname(record.from) !== cache.root ||
      path.dirname(record.to) !== args.quarantineRoot ||
      record.inventory !== file
    )
      fail(`invalid relocation inventory: ${file}`);
    if (present(record.to)) {
      if (present(record.from)) fail(`both original and quarantined cache exist; inspect ${file}`);
      if (treeIdentity(record.to) !== record.sha256)
        fail(`quarantined bytes or modes changed: ${file}`);
      if (mutate && record.status !== 'quarantined')
        save(file, { ...record, status: 'quarantined' });
    } else if (record.status === 'quarantined' || !present(record.from))
      fail(`relocation inventory has missing payload: ${file}`);
    records.push({
      inventory: file,
      from: record.from,
      to: record.to,
      status: present(record.to) ? 'quarantined' : 'pending',
    });
  }
  return records;
}

function apply(args) {
  // Share the activation lock; never remove somebody else's or a stale lock here.
  const lock = path.join(args.pluginRoot, '.bootstrap-lock');
  try {
    fs.mkdirSync(lock);
  } catch (error) {
    if (error.code === 'EEXIST')
      fail(`generation store is locked: ${lock}; retry after its owner exits`);
    throw error;
  }
  try {
    fs.writeFileSync(path.join(lock, 'pid'), `${process.pid}\n`);
    const result = spawnSync(
      process.execPath,
      [
        path.join(args.sourceRoot, 'scripts/install-plugin-generation.js'),
        '--verify',
        '--source-root',
        args.sourceRoot,
        '--home',
        args.home,
        '--plugin-root',
        args.pluginRoot,
        '--targets',
        args.targets,
      ],
      {
        encoding: 'utf8',
        timeout: 120_000,
        maxBuffer: 16 * 1024 * 1024,
        env: {
          ...process.env,
          HOME: args.home,
          CODEX_HOME: args.codexHome,
          PROMETHEUS_STORE_LOCK_HELD: '1',
        },
      }
    );
    if (result.error || result.status !== 0)
      fail(
        `installed generation verification failed; no cache relocated: ${result.error?.message || result.stderr}`
      );
    const current = active(args),
      report = inspect(args, current);
    if (report.blockers.length) fail(report.blockers.join('\n'));
    checkClaude(args, current, report.candidates);
    checkCodex(args, current, report.candidates);
    inventories(args, true);
    for (const candidate of report.candidates) {
      if (liveSessions(candidate.from).length || treeIdentity(candidate.from) !== candidate.sha256)
        fail(`cache changed during retirement; retry after quiescing sessions: ${candidate.from}`);
      if (present(candidate.to)) fail(`quarantine destination already exists: ${candidate.to}`);
      fs.mkdirSync(args.quarantineRoot, { recursive: true, mode: 0o700 });
      const record = {
        schema: SCHEMA,
        ...candidate,
        active: current,
        status: 'pending',
        createdAt: new Date().toISOString(),
      };
      if (present(candidate.inventory)) {
        const prior = readJson(candidate.inventory);
        if (
          prior.from !== record.from ||
          prior.to !== record.to ||
          prior.sha256 !== record.sha256 ||
          prior.status !== 'pending'
        )
          fail(`conflicting relocation inventory: ${candidate.inventory}`);
      } else save(candidate.inventory, record);
      try {
        fs.renameSync(candidate.from, candidate.to);
      } catch (error) {
        if (error.code === 'EXDEV')
          fail(
            'quarantine must be on the same filesystem as the cache; choose --quarantine-root outside loader/store paths on that filesystem'
          );
        throw error;
      }
      syncDirectory(path.dirname(candidate.from));
      syncDirectory(args.quarantineRoot);
      save(candidate.inventory, { ...record, status: 'quarantined' });
    }
    return {
      mode: 'apply',
      generationVerified: true,
      active: current,
      ...report,
      relocations: inventories(args),
      restartRequired: report.candidates.length > 0,
      note: 'Start fresh Claude and Codex sessions before installed acceptance. PID markers cannot identify every loaded session.',
    };
  } finally {
    fs.rmSync(lock, { recursive: true });
  }
}

try {
  const args = options(process.argv.slice(2));
  safePaths(args);
  let report;
  if (args.apply) report = apply(args);
  else {
    const current = active(args),
      inspection = inspect(args, current);
    try {
      checkClaude(args, current, inspection.candidates);
    } catch (error) {
      inspection.blockers.push(error.message);
    }
    report = {
      mode: 'preview',
      generationVerified: false,
      active: current,
      ...inspection,
      relocations: inventories(args),
      codexRegistration: inspection.candidates.some(candidate => candidate.client === 'codex')
        ? 'checked through supported CLI only during apply'
        : 'not required',
      note: 'Preview writes nothing. Refresh native registrations and quiesce affected sessions before --apply; generation verification runs during apply.',
    };
  }
  process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
} catch (error) {
  process.stderr.write(`retire-stale-plugin-caches: BLOCKED: ${error.message}\n`);
  process.exitCode = 2;
}
