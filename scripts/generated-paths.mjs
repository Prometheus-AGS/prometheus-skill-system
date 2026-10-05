#!/usr/bin/env node
// Declarations are the ownership policy, not evidence of what was generated.
// Materialization observes filesystem operations and walks the resulting tree;
// rebase classification consumes this same policy without a second root list.
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scripts = path.dirname(fileURLToPath(import.meta.url));
const repository = path.resolve(scripts, '..');
const GENERATORS = ['generate-harness-adapters.js', 'generate-skill-system-distribution.js'];

export function normalizeGeneratedPaths(values) {
  const declarations = [...new Set(values)].sort();
  for (const value of declarations) {
    const relative = value.replace(/\/$/, '');
    if (!relative || relative === '.' || /[\\\r\n\0]/.test(value) || path.posix.isAbsolute(value) ||
        path.posix.normalize(relative) !== relative || relative === '..' || relative.startsWith('../')) {
      throw new Error(`invalid generated ownership path: ${JSON.stringify(value)}`);
    }
  }
  return declarations;
}

export function generatedPaths() {
  const paths = [];
  for (const generator of GENERATORS) {
    const result = spawnSync(process.execPath, [path.join(scripts, generator), '--list-outputs'], {
      encoding: 'utf8',
    });
    if (result.status !== 0)
      throw new Error(`${generator} --list-outputs failed: ${result.stderr || result.status}`);
    paths.push(...result.stdout.split('\n').filter(Boolean));
  }
  return normalizeGeneratedPaths(paths);
}

function git(root, args) {
  const result = spawnSync('git', ['-C', root, ...args], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
  if (result.status !== 0)
    throw new Error(`git ${args[0]} failed: ${result.error?.message || result.stderr || result.status}`);
  return result.stdout;
}

// Index modes, including every conflict stage, describe gitlinks even when the
// checkout directory is empty or absent. HEAD also covers a staged deletion.
export function gitlinkPaths(root = repository, { required = false } = {}) {
  // Source archives can still generate/check payloads. Never accidentally read
  // an enclosing repository's index when this source has no Git metadata.
  if (!fs.existsSync(path.join(root, '.git'))) {
    if (required) throw new Error(`Git metadata is unavailable for conflict classification: ${root}`);
    return [];
  }
  const links = new Set();
  for (const entry of git(root, ['ls-files', '--stage', '-z']).split('\0')) {
    const tab = entry.indexOf('\t');
    if (entry.startsWith('160000 ') && tab !== -1) links.add(entry.slice(tab + 1));
  }
  const head = spawnSync('git', ['-C', root, 'rev-parse', '--verify', 'HEAD'], { encoding: 'utf8' });
  if (head.status === 0) {
    for (const entry of git(root, ['ls-tree', '-r', '-z', 'HEAD']).split('\0')) {
      const tab = entry.indexOf('\t');
      if (entry.startsWith('160000 ') && tab !== -1) links.add(entry.slice(tab + 1));
    }
  }
  return [...links].sort();
}

export function generatedPathOwner(relative, declarations, gitlinks = []) {
  if (gitlinks.some(link => relative === link || relative.startsWith(`${link}/`))) return null;
  return declarations.find(entry => entry.endsWith('/')
    ? relative === entry.slice(0, -1) || relative.startsWith(entry)
    : relative === entry) ?? null;
}

function scaffold(relative, declarations) {
  return relative === '' || declarations.some(entry => entry.startsWith(`${relative}/`));
}

export function assertNoOwnedGitlinks(declarations, links) {
  for (const link of links) {
    if (declarations.some(entry => generatedPathOwner(link, [entry]) ||
        entry.replace(/\/$/, '').startsWith(`${link}/`))) {
      throw new Error(`generated ownership overlaps a source gitlink: ${link}`);
    }
  }
}

// Walk all materialized entries, not only the declared roots. Symlinks are
// entries; their targets are never traversed into another tree or submodule.
export function materializedOutputs(root) {
  const entries = [];
  function visit(directory, relative = '') {
    for (const name of fs.readdirSync(directory).sort()) {
      const child = relative ? `${relative}/${name}` : name;
      const absolute = path.join(directory, name);
      const stat = fs.lstatSync(absolute);
      const kind = stat.isDirectory() ? 'directory' : stat.isSymbolicLink() ? 'symlink' : 'file';
      entries.push({ path: child, kind });
      if (kind === 'directory') visit(absolute, child);
    }
  }
  visit(root);
  return entries;
}

export function assertMaterializedOwnership(root, declarations, operations, consumerDeclarations = declarations,
  gitlinks = []) {
  const outputs = materializedOutputs(root);
  const observed = new Map(outputs.map(entry => [entry.path, entry]));
  for (const entry of outputs) {
    if (entry.kind === 'directory' && scaffold(entry.path, declarations)) continue;
    if (!generatedPathOwner(entry.path, declarations))
      throw new Error(`materialized output outside generated ownership: ${entry.path}`);
    if (!generatedPathOwner(entry.path, consumerDeclarations, gitlinks))
      throw new Error(`rebase classification omits materialized generated output: ${entry.path}`);
  }
  for (const operation of operations) {
    if (operation.scaffold && scaffold(operation.path, declarations)) continue;
    if (!generatedPathOwner(operation.path, declarations))
      throw new Error(`${operation.operation} outside generated ownership: ${operation.path}`);
    if (!generatedPathOwner(operation.path, consumerDeclarations, gitlinks))
      throw new Error(`rebase classification omits generated operation: ${operation.path}`);
  }
  for (const declaration of declarations) {
    const relative = declaration.replace(/\/$/, '');
    const entry = observed.get(relative);
    const requiredKind = declaration.endsWith('/') ? 'directory' : 'file';
    if (!entry || entry.kind !== requiredKind || (requiredKind === 'directory' &&
        !outputs.some(output => output.path.startsWith(declaration) && output.kind !== 'directory'))) {
      throw new Error(`required generated output missing from materialization: ${declaration}`);
    }
  }
  return outputs;
}

// Observe synchronous mutation primitives used by this generator, including a
// direct fs write added outside copy()/write(). Refuse before an escaping write
// can touch the source checkout. These bindings are restored even on failure.
export function observeMaterialization(root, declarations, materialize) {
  const operations = [];
  const originals = new Map();
  const targets = {
    writeFileSync: [0], appendFileSync: [0], mkdirSync: [0], chmodSync: [0],
    symlinkSync: [1], copyFileSync: [1], linkSync: [1], renameSync: [0, 1],
    rmSync: [0], unlinkSync: [0], rmdirSync: [0],
  };
  function record(operation, destination) {
    if (typeof destination !== 'string' && !Buffer.isBuffer(destination) && !(destination instanceof URL))
      throw new Error(`generated ${operation} requires an observable destination path`);
    const absolute = path.resolve(destination instanceof URL ? fileURLToPath(destination) : String(destination));
    const relative = path.relative(root, absolute).split(path.sep).join('/');
    if (relative === '..' || relative.startsWith('../') || path.isAbsolute(relative))
      throw new Error(`generated ${operation} escapes materialization root: ${absolute}`);
    const isScaffold = operation === 'mkdirSync';
    if (!(isScaffold && scaffold(relative, declarations)) && !generatedPathOwner(relative, declarations))
      throw new Error(`generated ${operation} outside declared ownership: ${relative}`);
    // A copied symlink may legitimately point elsewhere, but no later mutation
    // may follow it. Check parents and targets for operations that dereference.
    const followsTarget = ['writeFileSync', 'appendFileSync', 'copyFileSync', 'chmodSync',
      'mkdirSync', 'openSync'].includes(operation);
    let ancestor = followsTarget ? absolute : path.dirname(absolute);
    while (ancestor !== root && ancestor.startsWith(`${root}${path.sep}`)) {
      try {
        if (fs.lstatSync(ancestor).isSymbolicLink())
          throw new Error(`generated ${operation} follows a symlink: ${relative}`);
      } catch (error) {
        if (!['ENOENT', 'ENOTDIR'].includes(error.code)) throw error;
      }
      ancestor = path.dirname(ancestor);
    }
    operations.push({ operation, path: relative, scaffold: isScaffold });
  }
  try {
    for (const [operation, indices] of Object.entries(targets)) {
      const original = fs[operation];
      originals.set(operation, original);
      fs[operation] = (...args) => {
        for (const index of indices) record(operation, args[index]);
        return original.apply(fs, args);
      };
    }
    // writeFileSync accepts descriptors, but the generator uses path writes.
    // Track an explicit writable open as well, so it cannot bypass ownership.
    const open = fs.openSync;
    originals.set('openSync', open);
    fs.openSync = (destination, flags, ...args) => {
      if (typeof flags === 'number' ? (flags & (fs.constants.O_WRONLY | fs.constants.O_RDWR |
          fs.constants.O_CREAT | fs.constants.O_TRUNC | fs.constants.O_APPEND)) !== 0 : /[wa+]/.test(flags))
        record('openSync', destination);
      return open.call(fs, destination, flags, ...args);
    };
    materialize();
    return operations;
  } finally {
    for (const [operation, original] of originals) fs[operation] = original;
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const declarations = generatedPaths();
    if (process.argv.includes('--classify-conflicts')) {
      const links = gitlinkPaths(repository, { required: true });
      assertNoOwnedGitlinks(declarations, links);
      const conflicts = git(repository, ['diff', '--name-only', '--diff-filter=U', '-z'])
        .split('\0').filter(Boolean).sort();
      const outside = conflicts.filter(relative => !generatedPathOwner(relative, declarations, links));
      if (outside.length) {
        process.stderr.write('rebase-regenerate: conflicts outside generated ownership; resolve by hand (nothing was changed):\n');
        for (const relative of outside) {
          const kind = links.some(link => relative === link || relative.startsWith(`${link}/`)) ? 'gitlink' : 'source';
          process.stderr.write(`  ${JSON.stringify(relative)} (${kind})\n`);
        }
        process.exitCode = 1;
      }
    } else {
      process.stdout.write(`${declarations.join('\n')}\n`);
    }
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = process.argv.includes('--classify-conflicts') ? 2 : 1;
  }
}
