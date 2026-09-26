import { readFile, readdir, writeFile, mkdir } from 'node:fs/promises';
import { resolve, join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { normalizeUiuxFrontmatter, frontmatterAdaptation } from './uiux-frontmatter.mjs';

// Run after producing local runtime artifacts, then run the offline sync.
const full = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const index = process.argv.indexOf('--mini');
const mini = index < 0 ? undefined : resolve(process.argv[index + 1]);
const lockPath = join(full, 'skills/ui-ux/catalog.lock.json');
const lock = JSON.parse(await readFile(lockPath, 'utf8'));
const owned = ['prometheus-ui-ux', 'prometheus-ui-review', 'prometheus-impeccable-core', 'ui-ux-pro-max'];
// Normalize all recorded headers before hashing; imported bodies remain intact.
for (const entry of lock.skills) {
  const skillFile = join(full, entry.full.path, 'SKILL.md');
  const before = await readFile(skillFile, 'utf8');
  const content = normalizeUiuxFrontmatter(before, entry.license, entry.id);
  if (content !== before) await writeFile(skillFile, content);
  const asset = entry.assets.find(item => item.path === 'SKILL.md');
  if (asset) asset.sha256 = createHash('sha256').update(content).digest('hex');
  if (!entry.adaptations.includes(frontmatterAdaptation)) entry.adaptations.push(frontmatterAdaptation);
  const provenanceFile = join(full, entry.full.path, 'PROVENANCE.md');
  const provenance = await readFile(provenanceFile, 'utf8').catch(error => { if (error.code === 'ENOENT') return null; throw error; });
  if (provenance !== null && !provenance.includes(frontmatterAdaptation)) {
    await writeFile(provenanceFile, provenance.replace('## Adaptations\n', `## Adaptations\n\n- ${frontmatterAdaptation}`));
  }
}
async function assets(root, prefix = '') {
  const result = [];
  for (const entry of await readdir(join(root, prefix), { withFileTypes: true })) {
    if (['node_modules', '.git'].includes(entry.name)) continue;
    const path = prefix ? `${prefix}/${entry.name}` : entry.name;
    // The catalog embeds itself in the routing skill. Hashing it would create a
    // recursive digest; the containing package/distribution hashes this copy.
    if (path === 'references/catalog.lock.json') continue;
    if (entry.isSymbolicLink()) throw new Error(`Portable skill contains a link: ${path}`);
    if (entry.isDirectory()) result.push(...await assets(root, path));
    else result.push({ path, sha256: createHash('sha256').update(await readFile(join(root, path))).digest('hex') });
  }
  return result.sort((a, b) => a.path.localeCompare(b.path));
}
for (const id of owned) {
  const path = `skills/ui-ux/${id}`;
  await readFile(join(full, path, 'SKILL.md'));
  const proMax = id === 'ui-ux-pro-max';
  const revision = proMax ? 'dcc40ff5133ef78276117db0cc34e7b83cc8aeba' : 'local';
  const record = {
    id, upstreamName: proMax ? id : null,
    source: proMax ? `https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/tree/${revision}` : `prometheus-skill-pack/${path}`,
    revision, license: id === 'prometheus-impeccable-core' ? 'MIT AND Apache-2.0' : 'MIT',
    full: { path, status: 'bundled' }, mini: { path: `skills/${id}`, status: 'portable-node-runtime' },
    invocation: 'automatic', assets: await assets(join(full, path)),
    adaptations: [proMax ? 'TypeScript 7/Node port with pinned upstream data; consult provenance.json for runtime parity limitations.' : 'Prometheus-owned routing, review or bounded core adaptation; native Impeccable engine parity is not claimed.', frontmatterAdaptation],
  };
  lock.skills = lock.skills.filter(item => item.id !== id);
  lock.skills.push(record);
}
if (!lock.sources.some(item => item.repository === 'https://github.com/nextlevelbuilder/ui-ux-pro-max-skill')) {
  lock.sources.push({ repository: 'https://github.com/nextlevelbuilder/ui-ux-pro-max-skill', revision: 'dcc40ff5133ef78276117db0cc34e7b83cc8aeba', license: 'MIT', evidence: 'LICENSE' });
}
const content = JSON.stringify(lock, null, 2) + '\n';
await writeFile(lockPath, content);
for (const path of [join(full, 'skills/ui-ux/prometheus-ui-ux/references/catalog.lock.json'), ...(mini ? [join(mini, 'references/ui-ux/catalog.lock.json'), join(mini, 'skills/prometheus-ui-ux/references/catalog.lock.json')] : [])]) {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, content);
}
console.log(JSON.stringify({ operation: 'refresh-uiux-local-assets', skills: lock.skills.length }));
