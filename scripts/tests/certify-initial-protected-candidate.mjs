#!/usr/bin/env node
// Commit actual candidate snapshots only in private scratch, then invoke the
// production committed-state verifier. No source index/config/commit changes.
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const options = new Map(), commands = [], repositories = [];
const names = ['full', 'mini', 'surreal', 'companion'];
const fullBase = 'ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
let scratch, scratchParent, evidence, env, owned = false;
function blocked(message) { throw Object.assign(new Error(message), { blocked: true }); }
function within(root, target) {
  const relative = path.relative(root, target);
  return relative && relative !== '..' && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative);
}
function canonicalPath(value) {
  if (!value || !path.isAbsolute(value)) blocked('all paths must be explicit and absolute');
  let ancestor = path.resolve(value); const missing = [];
  while (!fs.existsSync(ancestor)) { missing.unshift(path.basename(ancestor)); ancestor = path.dirname(ancestor); }
  return path.join(fs.realpathSync(ancestor), ...missing);
}
function run(program, args, cwd, { verifier = false, binary = false } = {}) {
  const result = spawnSync(program, args, { cwd, env, shell: false, encoding: binary ? 'buffer' : 'utf8',
    timeout: 600000, maxBuffer: 64 * 1024 * 1024 });
  const row = { program, args, cwd, exitCode: result.status, error: result.error?.code ?? null,
    stdoutSha256: digest(result.stdout ?? ''), stderrSha256: digest(result.stderr ?? '') };
  commands.push(row);
  if (verifier || result.error || result.status !== 0) {
    const label = verifier ? `protected-${path.basename(cwd)}` : `preparation-${commands.length}-${path.basename(cwd)}`;
    fs.writeFileSync(path.join(evidence, `${label}.stdout`), result.stdout ?? '', { mode: 0o600 });
    fs.writeFileSync(path.join(evidence, `${label}.stderr`), result.stderr ?? '', { mode: 0o600 });
    row.stdout = path.join(evidence, `${label}.stdout`); row.stderr = path.join(evidence, `${label}.stderr`);
  }
  if (result.error) blocked(`required ${program} operation unavailable: ${result.error.code}`);
  if (result.status !== 0 && !verifier)
    blocked(`candidate preparation command failed: ${program}; stderr: ${String(result.stderr ?? '').trim().slice(-8192)}; diagnostic: ${row.stderr}`);
  return { stdout: result.stdout, exitCode: result.status };
}
function git(root, ...args) { return run('git', ['-C', root, ...args], root).stdout; }
const excluded = new Set(['.git', 'node_modules', 'target', '__pycache__', '.cache', '.scratch', '.ssh', '.gnupg']);
const privateFiles = new Set(['.codex/auth.json', '.claude/.credentials.json', '.aws/credentials', '.aws/config']);
function sourcePath(relative) { return !privateFiles.has(relative) && !relative.split('/').some(part => excluded.has(part)); }
function indexLinks(root) {
  const links = new Map();
  for (const record of git(root, 'ls-files', '--stage', '-z').split('\0').filter(Boolean)) {
    const match = record.match(/^(\d+) ([0-9a-f]{40}) (\d)\t([\s\S]+)$/);
    if (!match || match[3] !== '0') blocked(`source has unresolved index entries: ${root}`);
    if (match[1] === '160000') links.set(match[4], match[2]);
  }
  return links;
}
function safeDestination(root, relative) {
  const destination = path.join(root, relative);
  if (!within(root, destination)) blocked(`candidate path escapes snapshot: ${relative}`);
  for (let parent = path.dirname(destination); parent !== root; parent = path.dirname(parent)) {
    if (fs.existsSync(parent) && fs.lstatSync(parent).isSymbolicLink()) blocked(`candidate path follows parent symlink: ${relative}`);
  }
  return destination;
}
function overlay(source, destination, links) {
  const rows = [];
  const paths = [...new Set(git(source, 'ls-files', '--cached', '--others', '--exclude-standard', '-z').split('\0').filter(Boolean))].sort();
  for (const relative of paths) {
    if (!sourcePath(relative) || links.has(relative) || path.join(source, relative) === evidence || within(evidence, path.join(source, relative))) continue;
    const from = path.join(source, relative), to = safeDestination(destination, relative);
    const stat = fs.lstatSync(from, { throwIfNoEntry: false });
    fs.rmSync(to, { force: true, recursive: true });
    if (!stat) { rows.push({ path: relative, deleted: true }); continue; }
    fs.mkdirSync(path.dirname(to), { recursive: true });
    if (stat.isSymbolicLink()) {
      const target = fs.readlinkSync(from);
      if (!within(scratch, path.resolve(path.dirname(to), target))) blocked(`candidate symlink escapes scratch: ${relative}`);
      fs.symlinkSync(target, to);
      rows.push({ path: relative, kind: 'symlink', target, sha256: digest(target) });
    } else if (stat.isFile()) {
      const bytes = fs.readFileSync(from); fs.writeFileSync(to, bytes); fs.chmodSync(to, stat.mode & 0o7777);
      rows.push({ path: relative, kind: 'file', mode: (stat.mode & 0o7777).toString(8), bytes: bytes.length, sha256: digest(bytes) });
    } else blocked(`unsupported candidate entry: ${relative}`);
  }
  // Excluded entries must not remain just because they existed in baseline.
  for (const relative of git(destination, 'ls-files', '-z').split('\0').filter(Boolean)) {
    if (!sourcePath(relative)) fs.rmSync(safeDestination(destination, relative), { recursive: true, force: true });
  }
  return rows;
}
function snapshot(name, source, cloneSource, base) {
  const remote = path.join(scratch, 'remotes', `${name}.git`), destination = path.join(scratch, name);
  run('git', ['clone', '--bare', '--no-hardlinks', '--dissociate', cloneSource, remote], scratch);
  git(remote, 'remote', 'remove', 'origin');
  run('git', ['clone', '--no-hardlinks', '--dissociate', remote, destination], scratch);
  git(destination, 'checkout', '--detach', base);
  for (const [key, value] of [['user.name', 'Private integration evidence'], ['user.email', 'private-evidence@invalid'],
    ['commit.gpgsign', 'false'], ['tag.gpgsign', 'false'], ['core.filemode', 'true'], ['core.autocrlf', 'false']])
    git(destination, 'config', '--local', key, value);
  const links = indexLinks(cloneSource);
  const files = overlay(source, destination, links);
  // Full uses the prepared exact import index, but authored .gitmodules bytes
  // come from source. Scratch-local transport rewrites are not release content.
  for (const [relative, commit] of links) git(destination, 'update-index', '--add', '--cacheinfo', `160000,${commit},${relative}`);
  const gitDirectory = fs.realpathSync(git(destination, 'rev-parse', '--absolute-git-dir').trim());
  const common = fs.realpathSync(path.resolve(destination, git(destination, 'rev-parse', '--git-common-dir').trim()));
  if (!within(scratch, gitDirectory) || !within(scratch, common) || fs.existsSync(path.join(gitDirectory, 'objects/info/alternates')))
    blocked('private candidate still borrows Git administration or objects');
  // Update indexed paths directly: some retained historical files live under
  // currently ignored directories. They must not pass through new-file ignore
  // discovery. Only the separately admitted source inventory supplies additions.
  const indexed = new Set(git(destination, 'ls-files', '-z').split('\0').filter(Boolean));
  const updates = [...indexed].filter(relative => !links.has(relative));
  const additions = [...new Set(files.filter(row => !row.deleted && !indexed.has(row.path)).map(row => row.path))];
  for (let offset = 0; offset < updates.length; offset += 100)
    git(destination, '--literal-pathspecs', 'add', '-u', '--', ...updates.slice(offset, offset + 100));
  for (let offset = 0; offset < additions.length; offset += 100)
    git(destination, '--literal-pathspecs', 'add', '--', ...additions.slice(offset, offset + 100));
  git(destination, 'commit', '--allow-empty', '-m', `Private ${name} candidate for committed protected integrity`);
  const candidate = git(destination, 'rev-parse', 'HEAD').trim(), tree = git(destination, 'rev-parse', 'HEAD^{tree}').trim();
  const input = { source, baseCommit: base, files, gitlinks: [...links].map(([relative, commit]) => ({ path: relative, commit })) };
  const inventory = path.join(evidence, `${name}-input.json`);
  fs.writeFileSync(inventory, `${JSON.stringify(input, null, 2)}\n`, { mode: 0o600 });
  return { name, source, cloneSource, baseCommit: base, candidateCommit: candidate, candidateTree: tree,
    root: destination, inventory, inputSha256: digest(JSON.stringify(input)), inventorySha256: digest(fs.readFileSync(inventory)),
    gitDirectory, gitlinks: input.gitlinks, contentScope: 'actual authored working-tree source and selected gitlinks; nested import byte provenance is recorded by the parent integration coordinator' };
}

let code = 0;
try {
  const allowed = new Set([...names.map(name => `--${name}`), '--prepared-full', '--scratch', '--evidence']);
  for (let i = 2; i < process.argv.length; i += 2) {
    if (!allowed.has(process.argv[i]) || options.has(process.argv[i]) || !process.argv[i + 1]) blocked('use unique supported flags with absolute values');
    options.set(process.argv[i], process.argv[i + 1]);
  }
  const sources = Object.fromEntries(names.map(name => [name, canonicalPath(options.get(`--${name}`))]));
  const prepared = canonicalPath(options.get('--prepared-full'));
  const moduleRoot = fs.realpathSync(path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..'));
  const selected = { full: moduleRoot, mini: path.join(path.dirname(moduleRoot), 'ldd-mini-final'),
    surreal: path.join(path.dirname(moduleRoot), 'ldd-memory-final'), companion: path.join(path.dirname(moduleRoot), 'ldd-companion-final') };
  for (const name of names) if (sources[name] !== fs.realpathSync(selected[name])) blocked(`--${name} differs from the explicitly selected candidate root`);
  scratchParent = canonicalPath(options.get('--scratch')); evidence = canonicalPath(options.get('--evidence'));
  if (!within(scratchParent, prepared)) blocked('--prepared-full must be beneath the coordinator scratch parent');
  if (scratchParent === evidence || within(scratchParent, evidence) || within(evidence, scratchParent))
    blocked('--evidence must persist outside the coordinator scratch tree');
  if (Object.values(sources).some(source => source === scratchParent || within(source, scratchParent) || within(scratchParent, source)))
    blocked('scratch parent must be disjoint from every submitted source');
  if (!fs.existsSync(scratchParent) || !fs.lstatSync(scratchParent).isDirectory()) blocked('coordinator scratch parent is unavailable');
  fs.mkdirSync(evidence, { recursive: true });
  scratch = fs.mkdtempSync(path.join(scratchParent, 'protected-candidate-')); fs.chmodSync(scratch, 0o700); owned = true;
  fs.mkdirSync(path.join(scratch, 'home')); fs.mkdirSync(path.join(scratch, 'remotes'));
  fs.writeFileSync(path.join(scratch, 'gitconfig'), '', { mode: 0o600 });
  env = { PATH: process.env.PATH ?? '', HOME: path.join(scratch, 'home'), GIT_CONFIG_NOSYSTEM: '1',
    GIT_CONFIG_GLOBAL: path.join(scratch, 'gitconfig'), GIT_OPTIONAL_LOCKS: '0', GIT_TERMINAL_PROMPT: '0',
    GIT_ALLOW_PROTOCOL: 'file', GIT_SSH_COMMAND: 'false' };
  const bases = Object.fromEntries(names.map(name => [name, git(sources[name], 'rev-parse', 'HEAD').trim()]));
  if (bases.full !== fullBase) blocked(`full source HEAD must be owner-selected ${fullBase}`);
  if (Object.values(bases).some(commit => !/^[0-9a-f]{40}$/.test(commit))) blocked('every source must have an exact committed HEAD');
  if (git(prepared, 'rev-parse', 'HEAD').trim() !== bases.full) blocked('prepared full baseline differs from selected full source');
  const preparedAdmin = fs.realpathSync(git(prepared, 'rev-parse', '--absolute-git-dir').trim());
  if (!within(prepared, preparedAdmin)) blocked('prepared full must have independent contained Git administration');
  const verifier = path.join(sources.full, 'scripts/verify-protected-tests.mjs');
  if (!fs.existsSync(verifier)) blocked('actual production protected-test verifier unavailable');
  const verifierSha256 = digest(fs.readFileSync(verifier));
  for (const name of names) {
    const row = snapshot(name, sources[name], name === 'full' ? prepared : sources[name], bases[name]);
    repositories.push(row);
    const result = run(process.execPath, [verifier, '--base', row.baseCommit, '--candidate', row.candidateCommit], row.root, { verifier: true });
    row.verifier = { path: verifier, sha256: verifierSha256, exitCode: result.exitCode };
    row.result = result.exitCode === 0 ? 'PASS' : 'FAIL';
    if (result.exitCode !== 0) code = 1;
  }
} catch (error) {
  code = error.blocked || ['ENOENT', 'EACCES', 'ENOSPC'].includes(error.code) ? (code === 1 ? 1 : 2) : 1;
  repositories.push({ name: 'prerequisite', result: code === 2 ? 'BLOCKED' : 'FAIL', diagnosis: error.message });
}
const receipt = { schemaVersion: 1, result: code === 0 ? 'PASS' : code === 1 ? 'FAIL' : 'BLOCKED', exitCode: code,
  scratch, scratchParent, evidence, repositories, commands, signing: 'only private scratch evidence commits disable Git signing; no signing credentials or input config changes',
  limitations: 'Initial committed protected integrity only; private commits are not publication, protected-change approval, recursive import certification or final remote fresh-clone evidence.' };
if (owned && evidence) {
  // Persist the committed identities and complete input inventories before
  // deleting only the fresh child. Preserve the coordinator's other fixtures.
  receipt.cleanup = 'fresh private candidate child removed; external receipts and coordinator fixtures retained';
  fs.writeFileSync(path.join(evidence, 'initial-protected-candidate.json'), `${JSON.stringify(receipt, null, 2)}\n`, { mode: 0o600 });
  fs.rmSync(scratch, { recursive: true, force: true });
}
process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`); process.exitCode = code;
