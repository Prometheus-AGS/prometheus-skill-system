#!/usr/bin/env node
/**
 * generate-service-manifest.mjs — generate shared/services.manifest.json from
 * the platform service templates.
 *
 * The templates are the source of truth:
 *   shared/launchagents/*.plist              (macOS)
 *   shared/systemd/*.service|.timer|.path    (Linux)
 *
 * The manifest is their machine-readable projection: seam 3 of
 * docs/integration-contract.md. An external supervisor reads it to adopt the
 * pack's services where they already run, without parsing plists or units.
 *
 * Guarantees (contract v1):
 *   - Idempotent: two runs on an unchanged tree produce byte-identical output.
 *   - Drift-checked: `--check` exits non-zero and names the stale entry.
 *
 * Why a hand-rolled plist reader and not a library: the committed plists carry
 * `__PLACEHOLDER__` tokens that are substituted at install time. They are
 * well-formed XML, but they are templates, not installed plists, and the
 * placeholders must survive into the manifest verbatim so a consumer can see
 * what gets substituted. A dependency-free reader over the known key shapes
 * keeps this script runnable with no install step.
 *
 * Usage:
 *   node scripts/generate-service-manifest.mjs           # write the manifest
 *   node scripts/generate-service-manifest.mjs --check   # verify, never write
 */

import { readFileSync, writeFileSync, readdirSync, existsSync } from 'node:fs';
import { join, basename, extname } from 'node:path';

const CONTRACT_VERSION = '1.0.0';
const REPO_ROOT = process.cwd();
const LAUNCHAGENTS = join(REPO_ROOT, 'shared/launchagents');
const SYSTEMD = join(REPO_ROOT, 'shared/systemd');
const OUT = join(REPO_ROOT, 'shared/services.manifest.json');

/* ---------------------------------------------------------------- plists -- */

/** Strip XML comments so commented-out keys never enter the manifest. */
function stripComments(xml) {
  return xml.replace(/<!--[\s\S]*?-->/g, '');
}

/** Body of the top-level <dict>, i.e. everything inside <plist><dict>…</dict>. */
function plistBody(xml) {
  const open = xml.indexOf('<dict>');
  const close = xml.lastIndexOf('</dict>');
  return open === -1 || close === -1 ? '' : xml.slice(open + 6, close);
}

/**
 * Read one top-level key's raw value markup. Nested <dict>/<array> depth is
 * tracked so a key inside EnvironmentVariables is never mistaken for a
 * top-level key.
 */
function rawValue(body, key) {
  const re = new RegExp(`<key>${key}</key>`, 'g');
  let m;
  while ((m = re.exec(body)) !== null) {
    // Depth of container elements before this key: 0 means top level.
    const before = body.slice(0, m.index);
    const opens = (before.match(/<dict>|<array>/g) || []).length;
    const closes = (before.match(/<\/dict>|<\/array>/g) || []).length;
    if (opens !== closes) continue;
    const after = body.slice(m.index + m[0].length);
    const val = after.match(
      /^\s*(<string>[\s\S]*?<\/string>|<integer>[\s\S]*?<\/integer>|<true\s*\/>|<false\s*\/>|<array>[\s\S]*?<\/array>|<dict>[\s\S]*?<\/dict>)/
    );
    return val ? val[1] : null;
  }
  return null;
}

function plistString(body, key) {
  const raw = rawValue(body, key);
  const m = raw && raw.match(/^<string>([\s\S]*?)<\/string>$/);
  return m ? m[1] : null;
}

function plistInteger(body, key) {
  const raw = rawValue(body, key);
  const m = raw && raw.match(/^<integer>([\s\S]*?)<\/integer>$/);
  return m ? Number.parseInt(m[1], 10) : null;
}

function plistBool(body, key) {
  const raw = rawValue(body, key);
  if (raw === null) return null;
  if (/^<true\s*\/>$/.test(raw)) return true;
  if (/^<false\s*\/>$/.test(raw)) return false;
  return null;
}

function plistStringArray(body, key) {
  const raw = rawValue(body, key);
  if (!raw || !raw.startsWith('<array>')) return null;
  return [...raw.matchAll(/<string>([\s\S]*?)<\/string>/g)].map((m) => m[1]);
}

/**
 * KeepAlive is either <true/> or a dictionary of conditions
 * (SuccessfulExit/Crashed). Both shapes are reported as-is: the dictionary form
 * is what the launchagent-supervisor rules require, and a consumer must be able
 * to tell them apart.
 */
function plistKeepAlive(body) {
  const raw = rawValue(body, 'KeepAlive');
  if (raw === null) return null;
  if (/^<true\s*\/>$/.test(raw)) return true;
  if (/^<false\s*\/>$/.test(raw)) return false;
  if (raw.startsWith('<dict>')) {
    const out = {};
    for (const m of raw.matchAll(
      /<key>([A-Za-z]+)<\/key>\s*(<true\s*\/>|<false\s*\/>)/g
    )) {
      out[m[1]] = /^<true/.test(m[2]);
    }
    return out;
  }
  return null;
}

/** Port or socket a program binds, read from its own arguments. */
function bindingFromArgs(args) {
  if (!args) return null;
  for (const a of args) {
    const port = a.match(/(?:--port[= ]|:)(\d{2,5})\b/);
    if (port) return { kind: 'tcp', value: Number.parseInt(port[1], 10) };
    if (a.includes('.sock')) return { kind: 'unix', value: a };
  }
  return null;
}

function readPlist(file) {
  const xml = stripComments(readFileSync(file, 'utf8'));
  const body = plistBody(xml);
  const label = plistString(body, 'Label') || basename(file, '.plist');
  const args = plistStringArray(body, 'ProgramArguments') || [];
  return {
    label,
    source: `shared/launchagents/${basename(file)}`,
    program: args[0] ?? null,
    args: args.slice(1),
    binding: bindingFromArgs(args),
    schedule: {
      runAtLoad: plistBool(body, 'RunAtLoad'),
      startInterval: plistInteger(body, 'StartInterval'),
      watchPaths: plistStringArray(body, 'WatchPaths'),
    },
    restart: {
      keepAlive: plistKeepAlive(body),
      throttleInterval: plistInteger(body, 'ThrottleInterval'),
      processType: plistString(body, 'ProcessType'),
    },
    logs: {
      stdout: plistString(body, 'StandardOutPath'),
      stderr: plistString(body, 'StandardErrorPath'),
    },
    workingDirectory: plistString(body, 'WorkingDirectory'),
  };
}

/* --------------------------------------------------------------- systemd -- */

function iniSections(text) {
  const out = {};
  let section = null;
  for (const line of text.split('\n')) {
    const t = line.trim();
    if (!t || t.startsWith('#') || t.startsWith(';')) continue;
    const sec = t.match(/^\[([A-Za-z]+)\]$/);
    if (sec) {
      section = sec[1];
      out[section] ??= {};
      continue;
    }
    const kv = t.match(/^([A-Za-z][A-Za-z0-9]*)=(.*)$/);
    if (!kv || !section) continue;
    // systemd allows repeated keys (Environment=, ExecStartPre=); keep all.
    if (out[section][kv[1]] === undefined) out[section][kv[1]] = kv[2];
    else if (Array.isArray(out[section][kv[1]])) out[section][kv[1]].push(kv[2]);
    else out[section][kv[1]] = [out[section][kv[1]], kv[2]];
  }
  return out;
}

function readSystemd(file) {
  const ini = iniSections(readFileSync(file, 'utf8'));
  const ext = extname(file); // .service | .timer | .path
  const name = basename(file);
  const label = basename(file, ext);
  const exec = ini.Service?.ExecStart ?? null;
  const args = typeof exec === 'string' ? exec.split(/\s+/) : [];
  return {
    label,
    kind: ext.slice(1),
    source: `shared/systemd/${name}`,
    program: args[0] ?? null,
    args: args.slice(1),
    binding: bindingFromArgs(args),
    restart:
      ini.Service === undefined
        ? null
        : {
            restart: ini.Service.Restart ?? null,
            restartSec: ini.Service.RestartSec
              ? Number.parseInt(ini.Service.RestartSec, 10)
              : null,
            type: ini.Service.Type ?? null,
          },
    timer:
      ini.Timer === undefined
        ? null
        : {
            onBootSec: ini.Timer.OnBootSec ?? null,
            onUnitActiveSec: ini.Timer.OnUnitActiveSec ?? null,
            persistent: ini.Timer.Persistent === 'true',
            unit: ini.Timer.Unit ?? null,
          },
    watch:
      ini.Path === undefined
        ? null
        : {
            pathModified: ini.Path.PathModified ?? null,
            unit: ini.Path.Unit ?? null,
          },
    description: ini.Unit?.Description ?? null,
  };
}

/* ---------------------------------------------------------------- build --- */

function listFiles(dir, filter) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir).filter(filter).sort().map((f) => join(dir, f));
}

/** Deterministic key order everywhere; sorted inputs; no timestamps. */
function build() {
  const launchd = listFiles(LAUNCHAGENTS, (f) => f.endsWith('.plist')).map(readPlist);
  const systemd = listFiles(SYSTEMD, (f) =>
    ['.service', '.timer', '.path'].includes(extname(f))
  ).map(readSystemd);

  // One entry per service label, carrying whichever platform sources exist.
  const labels = [...new Set([...launchd, ...systemd].map((e) => e.label))].sort();
  const services = labels.map((label) => {
    const mac = launchd.find((e) => e.label === label) ?? null;
    const units = systemd.filter((e) => e.label === label);
    const primary = mac ?? units.find((u) => u.kind === 'service') ?? units[0];
    return {
      label,
      program: primary?.program ?? null,
      args: primary?.args ?? [],
      binding: primary?.binding ?? null,
      platforms: {
        darwin: mac
          ? {
              source: mac.source,
              schedule: mac.schedule,
              restart: mac.restart,
              logs: mac.logs,
              workingDirectory: mac.workingDirectory,
            }
          : null,
        linux: units.length
          ? units.map((u) => ({
              source: u.source,
              kind: u.kind,
              restart: u.restart,
              timer: u.timer,
              watch: u.watch,
              description: u.description,
            }))
          : null,
      },
    };
  });

  return {
    $schema: 'https://prometheus-ags.dev/schemas/services.manifest.json',
    contractVersion: CONTRACT_VERSION,
    generatedBy: 'scripts/generate-service-manifest.mjs',
    note:
      'GENERATED — do not hand-edit. Regenerate after any change to ' +
      'shared/launchagents/*.plist or shared/systemd/* (constraint C-01). ' +
      'Placeholders such as __PROMETHEUS_ROOT__ are substituted at install time.',
    services,
  };
}

/* ----------------------------------------------------------------- main --- */

const check = process.argv.includes('--check');
const serialized = JSON.stringify(build(), null, 2) + '\n';

if (check) {
  if (!existsSync(OUT)) {
    console.error(
      '[services-manifest] MISSING shared/services.manifest.json — run: node scripts/generate-service-manifest.mjs'
    );
    process.exit(1);
  }
  const current = readFileSync(OUT, 'utf8');
  if (current === serialized) {
    console.log('[services-manifest] up to date');
    process.exit(0);
  }
  // Name the stale entries rather than printing a whole-file diff.
  let stale = [];
  try {
    const a = JSON.parse(current).services ?? [];
    const b = JSON.parse(serialized).services ?? [];
    const byLabel = (list) => Object.fromEntries(list.map((s) => [s.label, JSON.stringify(s)]));
    const A = byLabel(a);
    const B = byLabel(b);
    stale = [...new Set([...Object.keys(A), ...Object.keys(B)])]
      .filter((l) => A[l] !== B[l])
      .sort();
  } catch {
    stale = ['<manifest is not parseable JSON>'];
  }
  console.error('[services-manifest] DRIFT — regenerate: node scripts/generate-service-manifest.mjs');
  for (const label of stale) console.error(`  stale: ${label}`);
  if (stale.length === 0) console.error('  stale: <metadata or ordering differs>');
  process.exit(1);
}

writeFileSync(OUT, serialized);
console.log(
  `[services-manifest] wrote shared/services.manifest.json (${JSON.parse(serialized).services.length} services)`
);
