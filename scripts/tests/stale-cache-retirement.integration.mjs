#!/usr/bin/env node
// Real native CLIs, packaged installers and disk boundaries; no mocked registries.
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const options = Object.fromEntries(process.argv.slice(2).reduce((pairs, arg, i, all) => i % 2 === 0 ? [...pairs, [arg.slice(2), all[i + 1]]] : pairs, []));
for (const key of ['scratch', 'evidence', 'claude-bin', 'codex-bin']) assert(options[key] && path.isAbsolute(options[key]), `absolute --${key} required`);
fs.mkdirSync(options.scratch, { recursive: true });
fs.mkdirSync(options.evidence, { recursive: true });
const scratch = fs.mkdtempSync(path.join(options.scratch, 'retirement-'));
const home = path.join(scratch, 'home'), codexHome = path.join(scratch, 'custom-codex');
for (const dir of [home, codexHome, path.join(scratch, 'tmp')]) fs.mkdirSync(dir);
const env = { PATH: `${path.dirname(process.execPath)}:/usr/bin:/bin:/opt/homebrew/bin`, HOME: home, CODEX_HOME: codexHome, CLAUDE_CONFIG_DIR: path.join(home, '.claude'), TMPDIR: path.join(scratch, 'tmp'), XDG_CONFIG_HOME: path.join(scratch, 'config'), XDG_DATA_HOME: path.join(scratch, 'data'), GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null', LANG: 'C', LC_ALL: 'C', PYTHONDONTWRITEBYTECODE: '1', PROMETHEUS_LEARNING_CORTEX: '0', PROMETHEUS_PROJECT_ID_SKIP_RUNTIME: '1' };
const records = [], cases = [], blocked = [], observations = [];
const hash = input => crypto.createHash('sha256').update(input).digest('hex');
const json = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const pack = path.join(root, 'dist/plugins/claude/prometheus-skill-pack');
const store = path.join(home, '.prometheus/plugins/prometheus-skill-pack');
const version = json(path.join(pack, 'skill-system.json')).releaseVersion;
const name = 'prometheus-skill-pack', id = `${name}@${name}`;

function run(label, binary, args, expected = 0, extraEnv = {}, timeout = 120_000) {
  const result = spawnSync(binary, args, { cwd: scratch, env: { ...env, ...extraEnv }, encoding: 'utf8', timeout, maxBuffer: 32 * 1024 * 1024 });
  const prefix = path.join(options.evidence, String(records.length).padStart(3, '0'));
  fs.writeFileSync(`${prefix}.stdout`, result.stdout || ''); fs.writeFileSync(`${prefix}.stderr`, result.stderr || '');
  records.push({ label, binary, args, environmentOverrides: extraEnv, exitCode: result.status, error: result.error?.message, stdout: `${prefix}.stdout`, stderr: `${prefix}.stderr`, cwd: scratch });
  if (expected !== null) assert.equal(result.status, expected, `${label}: ${result.error?.message || result.stderr || result.stdout}`);
  return result;
}
function snapshot(dir) {
  const entries = [];
  const walk = (file, relative) => {
    const stat = fs.lstatSync(file);
    if (stat.isSymbolicLink()) entries.push([relative, 'link', stat.mode & 0o777, fs.readlinkSync(file)]);
    else if (stat.isDirectory()) { entries.push([relative, 'dir', stat.mode & 0o777]); for (const name of fs.readdirSync(file).sort()) walk(path.join(file, name), `${relative}/${name}`); }
    else entries.push([relative, 'file', stat.mode & 0o777, hash(fs.readFileSync(file))]);
  };
  walk(dir, ''); return hash(JSON.stringify(entries));
}
const installer = (label, verify = false) => run(label, process.execPath, [path.join(root, 'scripts/install-plugin-generation.js'), '--source-root', pack, '--home', home, '--plugin-root', store, '--targets', 'claude', ...(verify ? ['--verify'] : [])]);
const maintenance = (label, apply, expected = 0) => run(label, process.execPath, [path.join(root, 'scripts/retire-stale-plugin-caches.mjs'), '--source-root', pack, '--home', home, '--plugin-root', store, '--targets', 'claude', '--codex-bin', options['codex-bin'], ...(apply ? ['--apply'] : [])], expected);
const claude = (label, ...args) => run(label, options['claude-bin'], ['plugin', ...args]);
const codex = (label, ...args) => run(label, options['codex-bin'], ['plugin', ...args]);
let exitCode = 1;
try {
  const archive = path.join(scratch, 'v1.7.0.tar'), historical = path.join(scratch, 'historical');
  fs.mkdirSync(historical);
  run('archive original v1.7.0', 'git', ['-C', root, 'archive', 'v1.7.0', '-o', archive]);
  run('extract original v1.7.0', 'tar', ['-xf', archive, '-C', historical]);
  claude('register historical native marketplace', 'marketplace', 'add', historical);
  claude('install historical native plugin', 'install', '--scope', 'user', id);
  const registry = path.join(home, '.claude/plugins/installed_plugins.json');
  const oldCache = json(registry).plugins[id][0].installPath;
  assert.equal(json(registry).plugins[id][0].version, '1.7.0');
  assert(fs.existsSync(path.join(oldCache, 'scripts/install-plugin-generation.js')));
  cases.push('historical v1.7.0 cache and registration created by real Claude CLI');
  installer('install current signed generation');
  const currentPointer = fs.readFileSync(path.join(store, 'pointers/current'), 'utf8');
  const runtimePath = path.join(store, 'runtime/v1/run-hook');
  const currentRuntimeHash = hash(fs.readFileSync(runtimePath));
  const oldBundle = json(path.join(oldCache, 'shared/harnesses/generated/release-manifest.json')).bundleId;
  const historicalRun = run('original v1.7.0 bootstrap after current installation', 'bash', [path.join(oldCache, 'shared/scripts/bootstrap-hook-runtime.sh'), '--source-root', oldCache, '--expected-bundle', oldBundle], null, { PROMETHEUS_PLUGIN_ROOT: store });
  if (historicalRun.status === 0 && json(path.join(store, 'current/manifest.json')).sourceVersion === '1.7.0') cases.push('original v1.7.0 bootstrap reproduced stale cache downgrade');
  else if (/external source has no commit gitlink/.test(historicalRun.stderr)) {
    // The archived marketplace contains .gitmodules but Claude omits .git. The
    // original installer requires genuine gitlink provenance. Supply that exact
    // tag through Git's normal environment without editing native payload bytes.
    const beforeProvenance = snapshot(oldCache);
    run('initialize isolated historical provenance', 'git', ['-C', historical, 'init']);
    run('fetch original tag provenance', 'git', ['-C', historical, 'fetch', '--depth', '1', root, 'refs/tags/v1.7.0']);
    run('restore exact historical index without changing payload', 'git', ['-C', historical, 'reset', '--mixed', 'FETCH_HEAD']);
    const provenance = { PROMETHEUS_PLUGIN_ROOT: store, GIT_DIR: path.join(historical, '.git'), GIT_WORK_TREE: oldCache };
    const qualified = run('original cached v1.7.0 bootstrap with exact-tag external Git provenance', 'bash', [path.join(oldCache, 'shared/scripts/bootstrap-hook-runtime.sh'), '--source-root', oldCache, '--expected-bundle', oldBundle], null, provenance);
    assert.equal(snapshot(oldCache), beforeProvenance, 'supplying original Git provenance must not edit cached source bytes or modes');
    if (qualified.status === 0 && json(path.join(store, 'current/manifest.json')).sourceVersion === '1.7.0') {
      run('verify historical signed downgrade through original installer', process.execPath, [path.join(oldCache, 'scripts/install-plugin-generation.js'), '--source-root', oldCache, '--home', home, '--plugin-root', store, '--verify'], 0, provenance);
      cases.push('unchanged original cached v1.7.0 bootstrap reproduced verified signed downgrade with explicitly supplied exact-tag Git provenance');
    } else blocked.push(`historical bootstrap with original Git provenance unavailable: exit ${qualified.status}; see evidence log`);
  } else blocked.push(`original v1.7.0 bootstrap reproduction unavailable: exit ${historicalRun.status}; see evidence log`);
  if (json(path.join(store, 'current/manifest.json')).sourceVersion === '1.7.0') {
    const historicalManifest = json(path.join(store, 'current/manifest.json'));
    const expectedCommit = run('record peeled original tag commit', 'git', ['-C', root, 'rev-parse', 'v1.7.0^{}']).stdout.trim();
    assert.equal(historicalManifest.sourceProvenance.sourceCommit, expectedCommit);
    assert.equal(fs.readFileSync(path.join(store, 'pointers/current'), 'utf8'), currentPointer, 'old installer leaves new authoritative pointer untouched');
    assert.notEqual(fs.readlinkSync(path.join(store, 'current')), currentPointer.trim(), 'old installer creates current pointer/link disagreement');
    const historicalRuntimeHash = hash(fs.readFileSync(runtimePath));
    assert.notEqual(historicalRuntimeHash, currentRuntimeHash, 'historical bootstrap overwrites stable runtime bytes');
    observations.push({ historicalDowngrade: { authoritativePointer: currentPointer.trim(), currentLink: fs.readlinkSync(path.join(store, 'current')), historicalSourceVersion: historicalManifest.sourceVersion, sourceCommit: expectedCommit, currentRuntimeHash, historicalRuntimeHash } });
    cases.push('historical bootstrap leaves newer authoritative pointer but overwrites convenience link and stable runtime');
  }
  installer('repair current signed generation after historical attempt');
  assert.equal(fs.readFileSync(path.join(store, 'pointers/current'), 'utf8'), currentPointer);
  assert.equal(fs.readlinkSync(path.join(store, 'current')), currentPointer.trim());
  assert.equal(hash(fs.readFileSync(runtimePath)), currentRuntimeHash);
  const previewBefore = snapshot(home), customBefore = snapshot(codexHome);
  const preview = JSON.parse(maintenance('preview leaves all selected home bytes and modes unchanged', false).stdout);
  assert.equal(snapshot(home), previewBefore); assert.equal(snapshot(codexHome), customBefore);
  assert(preview.candidates.some(candidate => candidate.from === oldCache));
  assert(preview.blockers.some(text => text.includes('registration')));
  cases.push('preview is read-only and reports actual stale native registration');
  const stale = maintenance('apply refuses actual stale native registration', true, 2);
  assert.match(stale.stderr, /registration still names/); assert(fs.existsSync(oldCache));
  cases.push('real verified installer cannot retire a still-registered old cache');

  claude('remove historical marketplace registration', 'marketplace', 'remove', name);
  claude('register current marketplace', 'marketplace', 'add', root);
  claude('refresh native plugin through actual installer', 'install', '--scope', 'user', id);
  assert.equal(json(registry).plugins[id][0].version, version);
  codex('register current Codex marketplace', 'marketplace', 'add', root, '--json');
  codex('install current Codex native plugin', 'add', id, '--json');
  // Historical Codex marketplace metadata is not compatible with current CLI.
  // Copy an untouched actual native cache as stale on-disk state; register using
  // the actual CLI, never fabricated JSON or a replacement list command.
  const oldCodex = path.join(codexHome, 'plugins/cache', name, name, '1.11.3');
  const oldCodexInput = options['legacy-codex-cache'];
  if (oldCodexInput) {
    assert(path.isAbsolute(oldCodexInput)); fs.mkdirSync(path.dirname(oldCodex), { recursive: true });
    fs.cpSync(oldCodexInput, oldCodex, { recursive: true, verbatimSymlinks: true });
  } else blocked.push('Codex stale-cache integration needs --legacy-codex-cache with original version 1.11.3 payload');
  const unrelated = path.join(home, '.claude/plugins/cache/unrelated/plugin/1.0.0');
  fs.mkdirSync(unrelated, { recursive: true }); fs.writeFileSync(path.join(unrelated, 'marker'), 'user-owned');
  const usage = path.join(oldCache, '.in_use'); fs.mkdirSync(usage, { recursive: true });
  fs.writeFileSync(path.join(usage, String(process.pid)), '');
  const inUse = maintenance('live session blocks retirement', true, 2); assert.match(inUse.stderr, /live PIDs/);
  fs.unlinkSync(path.join(usage, String(process.pid)));
  cases.push('live native session marker blocks relocation');
  const oldIdentity = snapshot(oldCache), unrelatedIdentity = snapshot(unrelated), storeIdentity = snapshot(path.join(store, 'generations'));
  const applied = JSON.parse(maintenance('quarantine historical native caches', true).stdout);
  const moved = applied.candidates.find(candidate => candidate.from === oldCache);
  assert(moved); assert(!fs.existsSync(oldCache)); assert.equal(snapshot(moved.to), oldIdentity);
  assert.equal(snapshot(unrelated), unrelatedIdentity); assert.equal(snapshot(path.join(store, 'generations')), storeIdentity);
  assert.equal(json(moved.inventory).status, 'quarantined');
  if (oldCodexInput) assert(applied.candidates.some(candidate => candidate.from === oldCodex));
  cases.push('intact quarantine through real Claude/Codex registration with generations and unrelated plugins preserved');
  const retired = run('original obsolete bootstrap path is no longer executable', 'bash', [path.join(oldCache, 'shared/scripts/bootstrap-hook-runtime.sh'), '--source-root', oldCache, '--expected-bundle', oldBundle], null, { PROMETHEUS_PLUGIN_ROOT: store });
  assert.notEqual(retired.status, 0); assert.equal(fs.readFileSync(path.join(store, 'pointers/current'), 'utf8'), currentPointer);
  cases.push('retired original bootstrap path cannot reactivate historical installer');
  const quarantineBefore = snapshot(path.join(home, '.prometheus/plugin-cache-quarantine'));
  assert.equal(JSON.parse(maintenance('repeat retirement', true).stdout).candidates.length, 0);
  assert.equal(snapshot(path.join(home, '.prometheus/plugin-cache-quarantine')), quarantineBefore);
  cases.push('repeated retirement is idempotent');
  // Simulate interruption after rename but before completing the durable record.
  const pending = json(moved.inventory); pending.status = 'pending'; fs.writeFileSync(moved.inventory, JSON.stringify(pending));
  maintenance('recover interrupted relocation inventory', true);
  assert.equal(json(moved.inventory).status, 'quarantined');
  cases.push('pending relocation reconciles from intact destination');
  const bad = path.join(moved.to, 'unexpected-bytecode.pyc'); fs.writeFileSync(bad, 'contamination');
  const corrupt = maintenance('reject changed quarantined payload', true, 2); assert.match(corrupt.stderr, /quarantined bytes or modes changed/);
  fs.unlinkSync(bad);
  cases.push('quarantine corruption is rejected');
  const contamination = path.join(store, currentPointer.trim(), 'shared/scripts/__pycache__'); fs.mkdirSync(contamination); fs.writeFileSync(path.join(contamination, 'contamination.pyc'), 'not allowed');
  const invalid = maintenance('reject contaminated active signed generation', true, 2); assert.match(invalid.stderr, /installed generation verification failed/);
  fs.rmSync(contamination, { recursive: true });
  installer('final real signed generation verification', true);
  cases.push('contaminated signed generation blocks maintenance');
  exitCode = blocked.length ? 2 : 0;
} catch (error) { records.push({ failure: error.stack }); }
const report = { status: exitCode === 0 ? 'PASS' : exitCode === 2 ? 'BLOCKED' : 'FAIL', scratch, environment: env, cases, blocked, observations, records };
fs.writeFileSync(path.join(options.evidence, 'report.json'), `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({ status: report.status, scratch, cases, blocked, evidence: path.join(options.evidence, 'report.json') }, null, 2));
process.exitCode = exitCode;
