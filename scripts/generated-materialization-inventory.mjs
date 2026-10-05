#!/usr/bin/env node
// Capture actual bytes and metadata after materialization. Ownership comes from
// the caller's generator declarations; this helper maintains no output root list.
// Capturing is read-only except for an explicitly selected receipt destination.
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { normalizeGeneratedPaths } from './generated-paths.mjs';

const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

export function captureMaterialization(root, declarations, capabilities = null) {
  if (!path.isAbsolute(root)) throw new Error('materialization root must be absolute');
  const resolved = fs.realpathSync(root);
  const ownership = normalizeGeneratedPaths(declarations);
  const entries = new Map();
  function visit(relative) {
    if (entries.has(relative)) return;
    const absolute = path.join(resolved, relative);
    const parent = fs.realpathSync(path.dirname(absolute));
    if (parent !== resolved && !parent.startsWith(`${resolved}${path.sep}`))
      throw new Error(`snapshot path follows an escaping parent symlink: ${relative}`);
    const stat = fs.lstatSync(absolute);
    const kind = stat.isSymbolicLink() ? 'symlink' : stat.isDirectory() ? 'directory' :
      stat.isFile() ? 'file' : 'unsupported';
    if (kind === 'unsupported') throw new Error(`unsupported generated entry: ${relative}`);
    const row = { path: relative, kind, mode: (stat.mode & 0o7777).toString(8) };
    if (kind === 'file') {
      const bytes = fs.readFileSync(absolute);
      row.bytes = bytes.length;
      row.sha256 = hash(bytes);
    } else if (kind === 'symlink') {
      row.target = fs.readlinkSync(absolute);
      row.sha256 = hash(Buffer.from(row.target));
    }
    entries.set(relative, row);
    if (kind === 'directory') {
      for (const name of fs.readdirSync(absolute).sort()) visit(`${relative}/${name}`);
    }
  }
  for (const declaration of ownership) {
    const relative = declaration.replace(/\/$/, '');
    visit(relative);
    if (entries.get(relative).kind !== (declaration.endsWith('/') ? 'directory' : 'file'))
      throw new Error(`generated declaration has wrong materialized kind: ${declaration}`);
  }
  const rows = [...entries.values()].sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0);
  // Raw lstat modes alone do not establish a volume's mode capability. The final
  // integration caller may attach the existing production capability receipt.
  // Never infer mode support from operating-system name or a different volume.
  let posixModes = null;
  if (capabilities) {
    if (!capabilities.storeRoot || !path.isAbsolute(capabilities.storeRoot) ||
        fs.statSync(capabilities.storeRoot).dev !== fs.statSync(resolved).dev)
      throw new Error('mode capability receipt must describe the materialization volume');
    if (typeof capabilities.posixModes !== 'boolean')
      throw new Error('mode capability receipt must explicitly report posixModes');
    posixModes = capabilities.posixModes;
  }
  return { schemaVersion: 1, root: resolved, ownership, posixModes,
    entries: rows, entryCount: rows.length, materializationSha256: hash(JSON.stringify(rows)) };
}

export function compareMaterializations(before, after) {
  if (before.schemaVersion !== 1 || after.schemaVersion !== 1)
    throw new Error('unsupported materialization inventory schema');
  if (JSON.stringify(before.ownership) !== JSON.stringify(after.ownership))
    return { exitCode: 1, diagnosis: 'generated ownership changed between materializations' };
  if (JSON.stringify(before.entries) !== JSON.stringify(after.entries))
    return { exitCode: 1, diagnosis: 'materialized paths, bytes, symlink targets or modes changed' };
  if (before.posixModes !== true || after.posixModes !== true)
    return { exitCode: 2, diagnosis: 'bytes match; exact mode evidence unavailable for this volume' };
  return { exitCode: 0, diagnosis: 'two materializations have identical actual bytes and modes' };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    const allowed = new Set(['--root', '--paths-file', '--capabilities-file', '--output', '--compare']);
    const values = new Map();
    for (let i = 0; i < args.length; i += 2) {
      if (!allowed.has(args[i]) || !args[i + 1] || !path.isAbsolute(args[i + 1]) || values.has(args[i]))
        throw new Error('use unique supported flags with explicit absolute paths');
      values.set(args[i], args[i + 1]);
    }
    if (!values.has('--root') || !values.has('--paths-file'))
      throw new Error('--root and --paths-file are required');
    const declarations = fs.readFileSync(values.get('--paths-file'), 'utf8').split('\n').filter(Boolean);
    const capabilities = values.has('--capabilities-file') ?
      JSON.parse(fs.readFileSync(values.get('--capabilities-file'), 'utf8')) : null;
    const inventory = captureMaterialization(values.get('--root'), declarations, capabilities);
    if (values.has('--output'))
      fs.writeFileSync(values.get('--output'), `${JSON.stringify(inventory, null, 2)}\n`, { mode: 0o600 });
    else process.stdout.write(`${JSON.stringify(inventory, null, 2)}\n`);
    if (values.has('--compare')) {
      const comparison = compareMaterializations(
        JSON.parse(fs.readFileSync(values.get('--compare'), 'utf8')), inventory);
      process.stderr.write(`${comparison.diagnosis}\n`);
      process.exitCode = comparison.exitCode;
    }
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 2;
  }
}
