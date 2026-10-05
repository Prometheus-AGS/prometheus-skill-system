#!/usr/bin/env node
// Offline integration source preparation, never a remote-installability receipt.
// Clone object ownership and source overlay remain wholly inside caller scratch.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';

const args = new Map();
const commands = [], repositories = [], overlays = [];
let scratch, output, gitEnv;
function blocked(message) { throw Object.assign(new Error(message), { blocked: true }); }
function contained(root, target) {
  const relative = path.relative(root, target);
  return relative !== '' && relative !== '..' && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative);
}
function absolute(flag, required = true) {
  const value = args.get(flag);
  if (!value && !required) return null;
  if (!value || !path.isAbsolute(value)) blocked(`${flag} requires an explicit absolute path`);
  // Resolve existing ancestors before containment checks (macOS /tmp is a
  // symlink). A lexical check must not admit a scratch path inside the input.
  let ancestor = path.resolve(value);
  const missing = [];
  while (!fs.existsSync(ancestor)) {
    missing.unshift(path.basename(ancestor));
    const parent = path.dirname(ancestor);
    if (parent === ancestor) blocked(`cannot resolve ${flag} ancestor`);
    ancestor = parent;
  }
  return path.join(fs.realpathSync(ancestor), ...missing);
}
function run(program, argv, options = {}) {
  const result = spawnSync(program, argv, { encoding: 'utf8', shell: false, timeout: 600000,
    maxBuffer: 64 * 1024 * 1024, ...options });
  commands.push({ program, args: argv, exitCode: result.status, error: result.error?.code ?? null });
  if (result.error || result.status !== 0) blocked(`${program} preparation failed (${result.status}): ${result.stderr ?? result.error?.message}`);
  return result.stdout;
}
function sourceGit(source, argv) { return run('git', ['-C', source, ...argv], { env: gitEnv }); }
const ignoredSegments = new Set(['.git', 'node_modules', 'target', '__pycache__', '.cache', '.scratch', '.ssh', '.gnupg']);
const privateState = new Set(['.codex/auth.json', '.claude/.credentials.json', '.aws/credentials', '.aws/config']);
function isSource(relative) {
  return !privateState.has(relative) && !relative.split('/').some(segment => ignoredSegments.has(segment));
}
function gitlinks(source) {
  const links = new Map();
  for (const record of sourceGit(source, ['ls-files', '--stage', '-z']).split('\0').filter(Boolean)) {
    const match = record.match(/^(\d+) ([0-9a-f]+) (\d)\t([\s\S]+)$/);
    if (!match || match[3] !== '0') blocked(`unmerged source index in ${source}`);
    if (match[1] === '160000') links.set(match[4], match[2]);
  }
  return links;
}
function overlay(source, destination, links) {
  const files = sourceGit(source, ['ls-files', '--cached', '--others', '--exclude-standard', '-z'])
    .split('\0').filter(Boolean).sort();
  for (const relative of new Set(files)) {
    if (!isSource(relative) || links.has(relative)) continue;
    const from = path.join(source, relative), to = path.join(destination, relative);
    if (!contained(destination, to)) blocked(`source path escapes checkout: ${relative}`);
    // Never follow a cloned symlink when writing an overlaid child.
    for (let parent = path.dirname(to); parent !== destination; parent = path.dirname(parent)) {
      if (fs.existsSync(parent) && fs.lstatSync(parent).isSymbolicLink()) blocked(`overlay parent is a symlink: ${relative}`);
    }
    const stat = fs.lstatSync(from, { throwIfNoEntry: false });
    fs.rmSync(to, { force: true, recursive: true });
    if (!stat) { overlays.push({ checkout: destination, path: relative, deleted: true }); continue; }
    fs.mkdirSync(path.dirname(to), { recursive: true });
    if (stat.isSymbolicLink()) {
      const target = fs.readlinkSync(from);
      const resolved = path.resolve(path.dirname(to), target);
      if (!contained(scratch, resolved)) blocked(`source symlink would escape scratch: ${relative}`);
      fs.symlinkSync(target, to);
      overlays.push({ checkout: destination, path: relative, kind: 'symlink', target });
    } else if (stat.isFile()) {
      fs.copyFileSync(from, to);
      fs.chmodSync(to, stat.mode & 0o7777);
      overlays.push({ checkout: destination, path: relative, kind: 'file',
        sha256: crypto.createHash('sha256').update(fs.readFileSync(from)).digest('hex'), mode: (stat.mode & 0o7777).toString(8) });
    } else blocked(`unsupported source entry: ${relative}`);
  }
}
function prepareRepository(source, destination, expectedHead, env, depth = 0) {
  if (depth > 16) blocked('nested source gitlink depth exceeds preparation bound');
  const head = sourceGit(source, ['rev-parse', 'HEAD']).trim();
  if (expectedHead && head !== expectedHead) blocked(`initialized gitlink ${source} HEAD ${head} differs from declared ${expectedHead}`);
  const remote = path.join(scratch, 'remotes', `${repositories.length}.git`);
  fs.mkdirSync(path.dirname(remote), { recursive: true });
  run('git', ['clone', '--bare', '--no-hardlinks', '--dissociate', source, remote], { env });
  // Bare source initially points back to the input checkout. Remove that pointer;
  // no later fetch may consult input state or credentials.
  run('git', ['-C', remote, 'remote', 'remove', 'origin'], { env });
  fs.rmSync(destination, { recursive: true, force: true });
  run('git', ['clone', '--no-hardlinks', '--dissociate', remote, destination], { env });
  run('git', ['-C', destination, 'checkout', '--detach', head], { env });
  for (const [key, value] of [['user.name', 'Local integration candidate'], ['user.email', 'local-integration@invalid'], ['commit.gpgsign', 'false'], ['tag.gpgsign', 'false'], ['protocol.file.allow', 'always']])
    run('git', ['-C', destination, 'config', '--local', key, value], { env });
  const links = gitlinks(source);
  repositories.push({ source, destination, baselineHead: head, remote, gitDirectory: path.join(destination, '.git') });
  overlay(source, destination, links);
  // Excluded tracked entries must not survive through the baseline clone.
  for (const relative of sourceGit(destination, ['ls-files', '-z']).split('\0').filter(Boolean)) {
    if (!isSource(relative)) fs.rmSync(path.join(destination, relative), { force: true, recursive: true });
  }
  const declared = new Map();
  const contractFile = path.join(source, 'skill-system.json');
  if (fs.existsSync(contractFile)) {
    for (const item of JSON.parse(fs.readFileSync(contractFile, 'utf8')).imports ?? []) declared.set(item.path, item.commit);
  }
  for (const [relative, indexHead] of links) {
    const from = path.join(source, relative), to = path.join(destination, relative);
    if (!fs.existsSync(path.join(from, '.git'))) blocked(`required source gitlink is uninitialized: ${from}`);
    const expected = declared.get(relative) ?? indexHead;
    prepareRepository(from, to, expected, env, depth + 1);
    const modules = sourceGit(destination, ['config', '-f', '.gitmodules', '--get-regexp', '^submodule\\..*\\.path$']);
    const entry = modules.split('\n').find(line => line.slice(line.indexOf(' ') + 1) === relative);
    if (!entry) blocked(`gitlink lacks .gitmodules mapping: ${relative}`);
    const key = entry.slice(0, entry.indexOf(' ')).replace(/\.path$/, '.url');
    const childRemote = repositories.find(record => record.destination === to).remote;
    run('git', ['-C', destination, 'config', '-f', '.gitmodules', key, childRemote], { env });
    run('git', ['-C', destination, 'config', '--local', key, childRemote], { env });
    run('git', ['-C', destination, 'update-index', '--add', '--cacheinfo', `160000,${expected},${relative}`], { env });
  }
  const gitDirectory = sourceGit(destination, ['rev-parse', '--absolute-git-dir']).trim();
  const common = sourceGit(destination, ['rev-parse', '--git-common-dir']).trim();
  if (!contained(scratch, fs.realpathSync(gitDirectory)) || !contained(scratch, path.resolve(destination, common)))
    blocked(`prepared Git administrative directory escapes scratch: ${destination}`);
  if (fs.existsSync(path.join(gitDirectory, 'objects/info/alternates')))
    blocked(`prepared repository still borrows external objects: ${destination}`);
}

try {
  const allowed = new Set(['--scratch', '--full', '--output', '--legacy-output']);
  for (let i = 2; i < process.argv.length; i += 2) {
    if (!allowed.has(process.argv[i]) || args.has(process.argv[i]) || !process.argv[i + 1]) blocked('invalid or duplicate preparation arguments');
    args.set(process.argv[i], process.argv[i + 1]);
  }
  scratch = absolute('--scratch');
  const full = fs.realpathSync(absolute('--full'));
  output = absolute('--output');
  let legacy = absolute('--legacy-output', false);
  if (!contained(scratch, output) || (legacy && (!contained(scratch, legacy) || contained(output, legacy) || contained(legacy, output) || output === legacy)))
    blocked('output and distinct legacy output must be contained under scratch');
  const reserved = new Set(['home', 'remotes', 'gitconfig', 'legacy-payload.tar', 'preparation-receipt.json']);
  if ([output, legacy].filter(Boolean).some(file => reserved.has(path.relative(scratch, file).split(path.sep)[0])))
    blocked('output cannot occupy preparer-owned home, remotes or receipt paths');
  if (scratch === full || contained(full, scratch) || contained(scratch, full)) blocked('scratch and source checkout must be disjoint');
  // An empty caller-owned root prevents adoption of an existing home/source tree.
  if (fs.existsSync(scratch) && (fs.lstatSync(scratch).isSymbolicLink() || fs.readdirSync(scratch).length))
    blocked('--scratch must be a new or empty directory');
  fs.mkdirSync(scratch, { recursive: true, mode: 0o700 });
  scratch = fs.realpathSync(scratch);
  output = path.join(scratch, path.relative(absolute('--scratch'), output));
  if (legacy) legacy = path.join(scratch, path.relative(absolute('--scratch'), legacy));
  const home = path.join(scratch, 'home'); fs.mkdirSync(home, { mode: 0o700 });
  const env = gitEnv = { PATH: process.env.PATH, HOME: home, GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: path.join(scratch, 'gitconfig'),
    GIT_OPTIONAL_LOCKS: '0',
    GIT_TERMINAL_PROMPT: '0', GIT_ALLOW_PROTOCOL: 'file', GIT_SSH_COMMAND: 'false' };
  fs.writeFileSync(env.GIT_CONFIG_GLOBAL, '[protocol "file"]\n\tallow = always\n');
  prepareRepository(full, output, null, env);
  if (legacy) {
    const baseline = 'ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61';
    const archive = path.join(scratch, 'legacy-payload.tar');
    const prefix = 'dist/plugins/claude/prometheus-skill-pack/';
    run('git', ['-C', full, 'archive', '--format=tar', `--output=${archive}`, baseline, prefix], { env });
    fs.mkdirSync(legacy, { recursive: true });
    // Git archives have no hardlinks. Refuse an escaping tracked symlink before
    // tar can follow it while extracting a later member.
    const tree = sourceGit(full, ['ls-tree', '-r', '-z', baseline, '--', prefix]);
    for (const record of tree.split('\0').filter(Boolean)) {
      const match = record.match(/^(\d+) blob ([0-9a-f]+)\t([\s\S]+)$/);
      if (!match || match[1] !== '120000') continue;
      const relative = match[3].slice(prefix.length);
      const target = sourceGit(full, ['cat-file', '-p', match[2]]);
      if (!contained(legacy, path.resolve(legacy, path.dirname(relative), target)))
        blocked(`committed legacy symlink escapes payload: ${relative}`);
    }
    run('tar', ['-xf', archive, '-C', legacy, '--strip-components=4'], { env });
    if (!fs.existsSync(path.join(legacy, 'shared/scripts/sessionstart-learning.sh')))
      blocked('actual committed baseline archive lacks required historical hook payload');
  }
  const receipt = { schemaVersion: 1, result: 'prepared', acceptance: 'offline local graph only; not final remote installability',
    scratch, source: full, installSource: output, legacyPayload: legacy, legacyCommit: legacy ? 'ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61' : null,
    repositories, overlays, commands, dependencies: 'node_modules/target/caches/auth excluded; no borrowed dependency tree',
    remaining: 'Parent final memory commit/owner merge and exact mini SHA approval plus final remote fresh-clone gate remain required.' };
  fs.writeFileSync(path.join(scratch, 'preparation-receipt.json'), `${JSON.stringify(receipt, null, 2)}\n`, { mode: 0o600 });
  process.stdout.write(`${JSON.stringify({ installSource: output, legacyPayload: legacy, receipt: path.join(scratch, 'preparation-receipt.json') })}\n`);
} catch (error) {
  const unavailable = error.blocked || ['ENOENT', 'EACCES', 'ENOSPC'].includes(error.code);
  process.stderr.write(`${JSON.stringify({ result: unavailable ? 'BLOCKED' : 'FAIL', diagnosis: error.message, commands })}\n`);
  process.exitCode = unavailable ? 2 : 1;
}
