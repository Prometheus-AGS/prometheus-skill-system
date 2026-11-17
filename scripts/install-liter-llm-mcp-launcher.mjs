#!/usr/bin/env node
// install-liter-llm-mcp-launcher.mjs — install the liter-llm MCP launcher and
// move harness MCP entries that still use the old pack-written form onto it.
//
// The launcher (shared/scripts/liter-llm-mcp-launch.sh) is installed as
// <home>/.local/bin/liter-llm-mcp. It lives there, beside the liter-llm binary
// it starts, and not in the repository checkout or a plugin generation, because
// harness configs outlive both: a checkout moves, a worktree is deleted, and
// `current` can be rolled back to a generation that predates the launcher.
//
// Migration is deliberately narrow. An entry is rewritten only when it is
// exactly what the pack used to write:
//     liter-llm mcp --transport stdio [--config <the default config path>]
// with no environment block and no other settings. Anything else (another
// binary path, another config path, extra arguments, an env block) is a user's
// own entry and is reported, never changed.
//
// Usage:
//   node scripts/install-liter-llm-mcp-launcher.mjs [--home <dir>] [--dry-run] [--check] [--json]
//     --dry-run  report what would change; write nothing; exit 0
//     --check    report what would change; write nothing; exit 1 if anything would
//
// Exit: 0 done (or nothing to do) · 1 a step failed, or --check found pending work

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SERVER = 'liter-llm';
const LAUNCHER_NAME = 'liter-llm-mcp';
const LAUNCHER_MARKER = '# liter-llm-mcp-launch.sh';
const DEFAULT_CONFIG_SUFFIX = '/.config/liter-llm/liter-llm-proxy.toml';
const OLD_ARGS_PREFIX = ['mcp', '--transport', 'stdio'];
const LAUNCHER_MODE = 0o755;
const CLAUDE_SHAPE_KEYS = new Set(['command', 'args', 'env', 'type', 'description', 'enabled']);
const OPENCODE_SHAPE_KEYS = new Set(['command', 'environment', 'type', 'enabled']);
const TOML_TABLE = /^\[(mcp_servers|mcp\.servers)\.(?:liter-llm|"liter-llm")\]\s*(?:#.*)?$/;
const TOML_SUBTABLE = /^\[(mcp_servers|mcp\.servers)\.(?:liter-llm|"liter-llm")\./;

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const launcherSource = path.resolve(scriptDir, '../shared/scripts/liter-llm-mcp-launch.sh');

function parseArgs(argv) {
  const options = { home: os.homedir(), dryRun: false, check: false, json: false };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--home') {
      const value = argv[index + 1];
      if (!value) throw new Error('--home needs a directory');
      options.home = path.resolve(value);
      index += 1;
    } else if (arg === '--dry-run') options.dryRun = true;
    else if (arg === '--check') options.check = true;
    else if (arg === '--json') options.json = true;
    else throw new Error(`unknown argument: ${arg}`);
  }
  return options;
}

export function launcherPath(home) {
  return path.join(home, '.local/bin', LAUNCHER_NAME);
}

// A nonempty inherited CODEX_HOME wins over the --home fallback, as in every
// other Codex installer path.
function codexHome(home) {
  return process.env.CODEX_HOME ? path.resolve(process.env.CODEX_HOME) : path.join(home, '.codex');
}

function configTargets(home) {
  const json = (relative, key, shape = 'claude') => ({
    kind: 'json',
    file: path.join(home, relative),
    key,
    shape,
  });
  const toml = file => ({ kind: 'toml', file });
  return [
    json('.claude/mcp.json', 'mcpServers'),
    json('.claude/mcp_servers.json', 'mcpServers'),
    json('.opencode/opencode.json', 'mcp', 'opencode'),
    json('.config/opencode/config.json', 'mcp_servers'),
    json('.minimax/mcp/mcp.json', 'mcpServers'),
    json('.cursor/mcp.json', 'mcpServers'),
    json('.codeium/windsurf/mcp/mcp.json', 'mcpServers'),
    toml(path.join(codexHome(home), 'config.toml')),
    toml(path.join(home, '.kimi-code/config.toml')),
    toml(path.join(home, '.config/codex/config.toml')),
  ];
}

function isDefaultConfigPath(value, home) {
  return [home, '${HOME}', '$HOME', '~'].some(prefix => value === prefix + DEFAULT_CONFIG_SUFFIX);
}

// argv is [command, ...args]. True only for the exact command line the pack wrote.
export function isPackWrittenInvocation(argv, home) {
  if (!Array.isArray(argv) || argv.some(value => typeof value !== 'string')) return false;
  const [command, ...args] = argv;
  if (command !== SERVER) return false;
  if (!OLD_ARGS_PREFIX.every((value, index) => args[index] === value)) return false;
  if (args.length === OLD_ARGS_PREFIX.length) return true;
  return (
    args.length === OLD_ARGS_PREFIX.length + 2 &&
    args[3] === '--config' &&
    isDefaultConfigPath(args[4], home)
  );
}

function isEmptyObject(value) {
  return (
    value === undefined ||
    (value !== null && typeof value === 'object' && Object.keys(value).length === 0)
  );
}

function withoutKey(entry, removed, replacements) {
  return Object.fromEntries(
    Object.entries(entry)
      .filter(([key]) => key !== removed)
      .map(([key, value]) => [key, key in replacements ? replacements[key] : value])
  );
}

// Returns { state: 'current' | 'migrate' | 'customised', entry? }.
function classifyJsonEntry(entry, shape, home) {
  const launcher = launcherPath(home);
  if (entry === null || typeof entry !== 'object' || Array.isArray(entry))
    return { state: 'customised' };
  if (shape === 'opencode') {
    if (Array.isArray(entry.command) && entry.command.length === 1 && entry.command[0] === launcher)
      return { state: 'current' };
    const plain =
      Object.keys(entry).every(key => OPENCODE_SHAPE_KEYS.has(key)) &&
      isEmptyObject(entry.environment);
    if (!plain || !isPackWrittenInvocation(entry.command, home)) return { state: 'customised' };
    return { state: 'migrate', entry: { ...entry, command: [launcher] } };
  }
  const noArgs = entry.args === undefined || (Array.isArray(entry.args) && entry.args.length === 0);
  if (entry.command === launcher && noArgs) return { state: 'current' };
  const plain =
    Object.keys(entry).every(key => CLAUDE_SHAPE_KEYS.has(key)) && isEmptyObject(entry.env);
  const argv = [entry.command, ...(Array.isArray(entry.args) ? entry.args : [])];
  if (!plain || !isPackWrittenInvocation(argv, home)) return { state: 'customised' };
  return { state: 'migrate', entry: withoutKey(entry, 'args', { command: launcher }) };
}

function planJson(target, home) {
  const text = fs.readFileSync(target.file, 'utf8');
  let config;
  try {
    config = JSON.parse(text);
  } catch {
    return { state: 'unreadable', detail: 'not valid JSON; left alone' };
  }
  const entry = config?.[target.key]?.[SERVER];
  if (entry === undefined) return { state: 'absent' };
  const verdict = classifyJsonEntry(entry, target.shape, home);
  if (verdict.state !== 'migrate') return verdict;
  const next = { ...config, [target.key]: { ...config[target.key], [SERVER]: verdict.entry } };
  return { state: 'migrate', text: `${JSON.stringify(next, null, 2)}\n` };
}

function stripTomlComment(line) {
  let quote = '';
  for (let index = 0; index < line.length; index += 1) {
    const char = line[index];
    if (quote) {
      if (char === '\\' && quote === '"') index += 1;
      else if (char === quote) quote = '';
    } else if (char === '"' || char === "'") quote = char;
    else if (char === '#') return line.slice(0, index);
  }
  return line;
}

// The two value forms the pack wrote: a basic string and an array of basic strings.
function parseTomlValue(raw) {
  const value = raw.trim();
  try {
    if (value.startsWith('"')) return JSON.parse(value);
    if (value.startsWith('[')) return JSON.parse(value.replace(/,\s*\]$/, ']'));
  } catch {
    return undefined;
  }
  return undefined;
}

// Reads `key = value` statements from a table body; arrays may span lines.
// Returns undefined when the body holds anything this reader does not expect.
function parseTomlBody(lines) {
  const statements = {};
  let pending = '';
  for (const line of lines) {
    const content = stripTomlComment(line).trim();
    if (!content && !pending) continue;
    pending = pending ? `${pending} ${content}` : content;
    const match = pending.match(/^([A-Za-z0-9_-]+)\s*=\s*(.*)$/);
    if (!match) return undefined;
    if (match[2].startsWith('[') && !match[2].endsWith(']')) continue;
    const value = parseTomlValue(match[2]);
    if (value === undefined || match[1] in statements) return undefined;
    statements[match[1]] = value;
    pending = '';
  }
  return pending ? undefined : statements;
}

function planToml(target, home) {
  const launcher = launcherPath(home);
  const text = fs.readFileSync(target.file, 'utf8');
  const lines = text.split('\n');
  const start = lines.findIndex(line => TOML_TABLE.test(line.trim()));
  if (start === -1) return { state: 'absent' };
  if (lines.some(line => TOML_SUBTABLE.test(line.trim()))) return { state: 'customised' };
  const next = lines.findIndex((line, index) => index > start && line.trimStart().startsWith('['));
  const end = next === -1 ? lines.length : next;
  const body = lines.slice(start + 1, end);
  const statements = parseTomlBody(body);
  if (!statements) return { state: 'customised' };
  const keys = Object.keys(statements);
  if (keys.length === 1 && statements.command === launcher) return { state: 'current' };
  const onlyPackKeys = keys.every(key => key === 'command' || key === 'args');
  const argv = [statements.command, ...(Array.isArray(statements.args) ? statements.args : [])];
  if (!onlyPackKeys || !isPackWrittenInvocation(argv, home)) return { state: 'customised' };
  const trailing = body.length - body.findLastIndex(line => line.trim() !== '') - 1;
  const replacement = [
    '# Launcher installed by the Prometheus skill pack: it sources',
    '# ~/.prometheus/kbd/secrets.env, then runs `liter-llm mcp --transport stdio --config ...`.',
    `command = ${JSON.stringify(launcher)}`,
    ...Array(trailing).fill(''),
  ];
  const nextLines = [...lines.slice(0, start + 1), ...replacement, ...lines.slice(end)];
  return { state: 'migrate', text: nextLines.join('\n') };
}

function timestamp() {
  return new Date().toISOString().replace(/[-:T]/g, '').slice(0, 14);
}

// Same directory, then rename: a reader never sees a half-written config, and
// the original mode is kept (these files are often 0600).
function replaceFile(file, text, mode) {
  const temporary = `${file}.tmp.${process.pid}`;
  fs.writeFileSync(temporary, text, { mode });
  fs.chmodSync(temporary, mode);
  fs.renameSync(temporary, file);
}

function migrateTarget(target, home, write) {
  if (!fs.existsSync(target.file)) return { file: target.file, state: 'no-file' };
  const plan = target.kind === 'json' ? planJson(target, home) : planToml(target, home);
  if (plan.state !== 'migrate')
    return { file: target.file, state: plan.state, detail: plan.detail };
  if (!write) return { file: target.file, state: 'would-migrate' };
  const mode = fs.statSync(target.file).mode & 0o777;
  const backup = `${target.file}.bak.${timestamp()}`;
  fs.copyFileSync(target.file, backup);
  fs.chmodSync(backup, mode);
  replaceFile(target.file, plan.text, mode);
  return { file: target.file, state: 'migrated', backup };
}

function installLauncher(home, write) {
  const destination = launcherPath(home);
  const wanted = fs.readFileSync(launcherSource);
  const present = fs.existsSync(destination);
  if (present) {
    const installed = fs.readFileSync(destination);
    const executable = (fs.statSync(destination).mode & 0o111) !== 0;
    if (installed.equals(wanted) && executable) return { path: destination, state: 'current' };
    // Never replace a file of the same name that this pack did not write.
    if (!installed.includes(LAUNCHER_MARKER))
      return {
        path: destination,
        state: 'foreign',
        detail: 'a different file already has this name; left alone',
      };
  }
  if (!write) return { path: destination, state: present ? 'would-update' : 'would-install' };
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  replaceFile(destination, wanted, LAUNCHER_MODE);
  return { path: destination, state: present ? 'updated' : 'installed' };
}

function run(options) {
  const write = !options.dryRun && !options.check;
  const launcher = installLauncher(options.home, write);
  // Entries are moved only once the launcher they will name really exists.
  const launcherReady = ['current', 'installed', 'updated'].includes(launcher.state);
  const mayMigrate = write ? launcherReady : launcher.state !== 'foreign';
  const configs = mayMigrate
    ? configTargets(options.home).map(target => {
        try {
          return migrateTarget(target, options.home, write);
        } catch (error) {
          return { file: target.file, state: 'failed', detail: error.message };
        }
      })
    : [];
  const failed = launcher.state === 'foreign' || configs.some(result => result.state === 'failed');
  const pending =
    launcher.state.startsWith('would-') || configs.some(result => result.state === 'would-migrate');
  return { launcher, configs, failed, pending };
}

const CONFIG_LABELS = {
  migrated: 'moved onto the launcher',
  'would-migrate': 'would move onto the launcher',
  current: 'already uses the launcher',
  customised: 'left alone (not the pack-written entry)',
  unreadable: 'left alone',
  failed: 'FAILED',
};

function report(result) {
  const lines = [`liter-llm MCP launcher: ${result.launcher.state} (${result.launcher.path})`];
  if (result.launcher.detail) lines.push(`  ${result.launcher.detail}`);
  for (const config of result.configs) {
    const label = CONFIG_LABELS[config.state];
    if (!label) continue;
    const detail = config.detail && config.state !== 'unreadable' ? `: ${config.detail}` : '';
    const suffix = config.backup ? ` (backup ${config.backup})` : '';
    lines.push(`  ${config.file}: ${label}${detail}${suffix}`);
  }
  return lines.join('\n');
}

function main() {
  let options;
  try {
    options = parseArgs(process.argv.slice(2));
  } catch (error) {
    process.stderr.write(`install-liter-llm-mcp-launcher: ${error.message}\n`);
    return 2;
  }
  let result;
  try {
    result = run(options);
  } catch (error) {
    process.stderr.write(`install-liter-llm-mcp-launcher: ${error.message}\n`);
    return 1;
  }
  process.stdout.write(
    options.json ? `${JSON.stringify(result, null, 2)}\n` : `${report(result)}\n`
  );
  if (result.failed) return 1;
  return options.check && result.pending ? 1 : 0;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.exitCode = main();
}
