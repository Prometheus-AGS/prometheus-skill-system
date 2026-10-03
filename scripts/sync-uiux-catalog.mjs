import { mkdir, readFile, writeFile, lstat } from 'node:fs/promises';
import { resolve, dirname, join, relative, isAbsolute } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';

// Offline distribution generator. Copies the approved portable subset only.
const source = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const miniIndex = process.argv.indexOf('--mini');
if (miniIndex < 0 || !process.argv[miniIndex + 1]) throw new Error('Usage: node scripts/sync-uiux-catalog.mjs --mini <mini-checkout>');
const mini = resolve(process.argv[miniIndex + 1]);
const lock = JSON.parse(await readFile(join(source, 'skills/ui-ux/catalog.lock.json'), 'utf8'));
const within = (base, path) => {
  const target = resolve(base, path);
  const rel = relative(base, target);
  if (rel.startsWith('..') || isAbsolute(rel)) throw new Error(`Catalog path escapes distribution root: ${path}`);
  return target;
};
const writes = [];
for (const entry of lock.skills) {
  if (!entry.mini.path) continue;
  const src = within(source, entry.full.path);
  const dest = within(mini, entry.mini.path);
  const provenance = !entry.assets.some(asset => asset.path === 'PROVENANCE.md') && await lstat(join(src, 'PROVENANCE.md')).catch(() => undefined);
  for (const asset of [...entry.assets, ...(provenance ? [{ path: 'PROVENANCE.md' }] : [])]) {
    const from = within(src, asset.path);
    if (!(await lstat(from)).isFile()) throw new Error(`Only regular assets may be distributed: ${from}`);
    const bytes = await readFile(from);
    if (asset.sha256 && createHash('sha256').update(bytes).digest('hex') !== asset.sha256) throw new Error(`Asset differs from pinned catalog: ${from}`);
    writes.push([within(dest, asset.path), bytes]);
  }
}
writes.push([join(mini, 'references/ui-ux/catalog.lock.json'), await readFile(join(source, 'skills/ui-ux/catalog.lock.json'))]);
writes.push([join(mini, 'references/ui-ux/COMPATIBILITY.md'), await readFile(join(source, 'skills/ui-ux/COMPATIBILITY.md'))]);
writes.push([join(mini, 'skills/prometheus-ui-ux/references/catalog.lock.json'), await readFile(join(source, 'skills/ui-ux/catalog.lock.json'))]);
// Preflight destination ancestry before making any writes: source-controlled
// catalog paths cannot be used to follow an existing link outside the project.
for (const [target] of writes) {
  let current = target;
  while (current !== mini) {
    const stat = await lstat(current).catch(error => { if (error.code === 'ENOENT') return undefined; throw error; });
    if (stat?.isSymbolicLink()) throw new Error(`Linked distribution target is unsupported: ${current}`);
    current = dirname(current);
  }
}
for (const [target, bytes] of writes) {
  await mkdir(dirname(target), { recursive: true });
  await writeFile(target, bytes);
}
console.log(JSON.stringify({ operation: 'sync-uiux-catalog', portableSkills: lock.skills.filter(x => x.mini.path).length, files: writes.length }));
