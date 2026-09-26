import { spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { basename, dirname, extname, isAbsolute, join, relative, resolve } from 'node:path';
import { assert, files, main, repoRoot } from './common.mjs';
import { frontmatter, git, scopes, scopeFiles, sha, worktrees } from './wiki-common.mjs';
interface Source { name: string; branch: string; root?: string; ref?: string }
interface Variant { source: Source; path: string; scope: string; data: Buffer; hash: string; text: string }
interface ScopeReport { wiki?: { source_entries: number; canonical_page_paths: number; mappings: Record<string, unknown>[] }; events?: Record<string, unknown>; other_files?: Record<string, unknown>[] }
interface Manifest { schema_version: number; purpose: string; snapshot_ref: string; sources: {name: string; branch: string}[]; scopes: Record<string, ScopeReport>; redactions: Record<string, unknown>[] }
const snapshot = 'codex/pre-consolidation-main-snapshot', date = '2026-07-17';
const rank = (source: Source) => `${source.name === 'primary' ? '0' : source.name === 'integration' ? '1' : '2'}${source.name}`;
const posix = (path: string) => path.replaceAll('\\', '/');
const write = (path: string, content: string | Uint8Array) => { mkdirSync(dirname(path), { recursive: true }); writeFileSync(path, content); };
const json = (path: string, value: unknown) => write(path, JSON.stringify(value, null, 2) + '\n');
function redact(value: string): { text: string; records: {kind: string; occurrences: number}[] } {
  const rules: [string, RegExp, string][] = [['repository-worktree-path', /\/Users\/gqadonis\/Projects\/hybrid-mobile-architecture-src(?:\/\.claude\/worktrees\/[^\s`"']+)?/g, '$REPO_ROOT'], ['home-directory', /\/Users\/gqadonis(?=\/|\b)/g, '$HOME'], ['aws-access-key', /\bAKIA[0-9A-Z]{16}\b/g, '[REDACTED_SECRET]'], ['github-token', /\bgh[pousr]_[A-Za-z0-9_]{20,}\b/g, '[REDACTED_SECRET]'], ['openai-style-key', /\bsk-[A-Za-z0-9_-]{20,}\b/g, '[REDACTED_SECRET]'], ['bearer-token', /\bBearer\s+[A-Za-z0-9._~+/-]{16,}=*/gi, 'Bearer [REDACTED_SECRET]']];
  const records: {kind: string; occurrences: number}[] = [];
  for (const [kind, regex, replacement] of rules) { let count = 0; value = value.replace(regex, () => { count++; return replacement; }); if (count) records.push({kind, occurrences: count}); }
  return {text: value, records};
}
function materialize(variant: Variant): {data: string | Buffer; records: {kind: string; occurrences: number}[]; binary: boolean} {
  const textExtensions = new Set(['', '.md', '.txt', '.json', '.jsonl', '.yaml', '.yml', '.toml', '.log', '.html', '.css', '.js', '.mjs', '.ts', '.mts', '.rs', '.sql', '.surql', '.svg']);
  if (textExtensions.has(extname(variant.scope).toLowerCase())) try {
    const decoded = new TextDecoder('utf-8', {fatal:true, ignoreBOM:true}).decode(variant.data);
    if (!/[\u0000-\u0008\u000B\u000C\u000E-\u001F]/.test(decoded)) { const result = redact(decoded); return {data:result.text,records:result.records,binary:false}; }
  } catch { /* Non-text bytes are preserved exactly, never replacement-decoded. */ }
  return {data:variant.data,records:[],binary:true};
}
function historyRelative(variant: Variant): string {
  const path = posix(relative('knowledge/wiki', variant.scope));
  assert(!isAbsolute(path) && !path.split('/').some(part=>part==='..' || part==='.') && !path.includes('\\'), `unsafe wiki history path: ${variant.scope}`);
  return `knowledge/wiki/history/worktree-consolidation/${variant.source.name}/${path}`;
}
function variants(root: string, source: Source, scope: string): Variant[] {
  let entries: [string, Buffer][];
  if (source.ref) entries = git(root, 'ls-tree', '-r', '-z', '--name-only', source.ref, '--', scope).split('\0').filter(Boolean).map(path => { const result = spawnSync('git', ['show', `${source.ref}:${path}`], { cwd: root, maxBuffer: 128 * 1024 * 1024, shell: false }); assert(result.status === 0, `git show failed for ${path}`); return [path, result.stdout]; });
  else { const folder = join(source.root!, scope); entries = existsSync(folder) ? scopeFiles(folder).map(path => { assert(lstatSync(path).isFile(), `wiki consolidation requires explicit resolution of symlink or special file: ${path}`); return [posix(relative(source.root!, path)), readFileSync(path)]; }) : []; }
  return entries.map(([path, data]) => {
    const relativePath=posix(relative(scope,path)); let decoded=data.toString('utf8');
    if(relativePath==='events.jsonl'||(relativePath.startsWith('knowledge/wiki/')&&relativePath.endsWith('.md'))) decoded=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(data);
    return {source,path,scope:relativePath,data,hash:sha(data),text:decoded};
  }).filter(v => !v.scope.startsWith('consolidation/worktree-wiki-merge/') && !v.scope.startsWith('knowledge/wiki/history/worktree-consolidation/'));
}
function components(pages: Variant[]): Variant[][] {
  const parent = pages.map((_, i) => i), ids = new Map<string, number>(), names = new Map<string, number>();
  const find = (i: number): number => { while (parent[i] !== i) { parent[i] = parent[parent[i]!]!; i = parent[i]!; } return i; };
  pages.forEach((page, i) => { const name = basename(page.scope), id = frontmatter(page.text).id; for (const [map, key] of [[names, name], [ids, id]] as const) if (key) { const prior = map.get(key); if (prior !== undefined) parent[find(i)] = find(prior); else map.set(key, i); } });
  const grouped = new Map<number, Variant[]>(); pages.forEach((page, i) => { const key = find(i), group = grouped.get(key) ?? []; group.push(page); grouped.set(key, group); }); return [...grouped.values()];
}
function build(scope: string, output: string, input: Variant[], manifest: Manifest): void {
  const report = manifest.scopes[scope]!, wiki = join(output, 'knowledge/wiki'); mkdirSync(wiki, {recursive: true});
  const provenance = new Map<string, {original_sha256: string; record: string; sources: Record<string, unknown>[]; redactions: {kind: string; occurrences: number}[]}>(), byId = new Map<string, Set<string>>();
  for (const file of input.filter(v => v.scope === 'events.jsonl')) file.text.split(/\r?\n/).forEach((line, index) => { if (!line.trim()) return; const hash = sha(line), redacted = redact(line), entry = provenance.get(hash) ?? {original_sha256: hash, record: redacted.text, sources: [], redactions: redacted.records}; entry.sources.push({source: file.source.name, branch: file.source.branch, path: file.path, line: index + 1}); provenance.set(hash, entry); try { const id = String((JSON.parse(line) as Record<string, unknown>).id ?? ''); if (id) { const variants = byId.get(id) ?? new Set(); variants.add(hash); byId.set(id, variants); } } catch { /* Malformed events remain preserved and sorted last. */ } });
  const timestamp = (line: string) => { try { return String((JSON.parse(line) as Record<string, unknown>).timestamp ?? '9999-missing'); } catch { return '9999-invalid'; } };
  const records = [...provenance.values()].sort((a, b) => timestamp(a.record).localeCompare(timestamp(b.record)) || a.original_sha256.localeCompare(b.original_sha256));
  const conflicts = [...byId].filter(([, hashes]) => hashes.size > 1).sort(([a], [b]) => a.localeCompare(b)).map(([id, hashes]) => ({id, variants: [...hashes].map(hash => provenance.get(hash))}));
  write(join(output, 'events.jsonl'), records.map(entry => entry.record.trimEnd() + '\n').join(''));
  const ledger = 'consolidation/worktree-wiki-merge'; json(join(output, ledger, 'event-provenance.json'), records); json(join(output, ledger, 'event-conflicts.json'), conflicts);
  report.events = {unique_records: records.length, conflicting_ids: conflicts.length, provenance: `${ledger}/event-provenance.json`, conflicts: `${ledger}/event-conflicts.json`};
  const pages = input.filter(v => v.scope.startsWith('knowledge/wiki/') && v.scope.endsWith('.md') && !['index.md', 'log.md'].includes(basename(v.scope))), mappings: Record<string, unknown>[] = [];
  for (const group of components(pages)) {
    group.sort((a, b) => rank(a.source).localeCompare(rank(b.source)) || a.scope.localeCompare(b.scope) || a.hash.localeCompare(b.hash));
    const selected = group[0]!, destination = basename(selected.scope), unique = group.filter((variant, index) => group.findIndex(other => other.hash === variant.hash) === index), original = redact(selected.text), history = new Map<string, string>(); let canonical = original.text;
    if (unique.length > 1) {
      canonical = canonical.trimEnd() + '\n\n## Consolidated source variants\n';
      for (const variant of unique) { const redacted = redact(variant.text), path = historyRelative(variant); write(join(output, path), `<!-- source=${variant.source.name}; branch=${variant.source.branch}; original_sha256=${variant.hash} -->\n${redacted.text}`); history.set(variant.hash, path); if (variant.hash !== selected.hash) { const end = redacted.text.startsWith('---\n') ? redacted.text.indexOf('\n---\n', 4) : -1; const body = end >= 0 ? redacted.text.slice(end + 5).trimStart() : redacted.text; canonical += `\n### Variant from \`${variant.source.name}\`\n\nOriginal path: \`${variant.path}\`<br>\nOriginal SHA-256: \`${variant.hash}\`\n\n${body.trimEnd()}\n`; } for (const item of redacted.records) manifest.redactions.push({scope, source: variant.source.name, path: variant.path, original_sha256: variant.hash, ...item}); }
    } else for (const item of original.records) manifest.redactions.push({scope, source: selected.source.name, path: selected.path, original_sha256: selected.hash, ...item});
    write(join(wiki, destination), canonical.trimEnd() + '\n');
    const metadata = frontmatter(selected.text);
    for (const filename of [...new Set(group.map(v => basename(v.scope)))].sort()) if (filename !== destination) write(join(wiki, filename), `---\ntype: Reference\nid: ${metadata.id ?? destination.replace(/\.md$/, '')}-alias-${filename.replace(/\.md$/, '')}\ntitle: Alias for ${metadata.title ?? destination.replace(/\.md$/, '')}\ntimestamp: ${date}T00:00:00Z\nrevision: 1\n---\n\nThis historical filename now resolves to [${metadata.title ?? destination}](/${destination}). Its original variants are retained in the worktree-consolidation history.\n`);
    for (const variant of group) { const fm = frontmatter(variant.text); mappings.push({source: variant.source.name, branch: variant.source.branch, original_path: variant.path, original_sha256: variant.hash, id: fm.id ?? null, revision: fm.revision ?? null, timestamp: fm.timestamp ?? fm.updated_at ?? null, canonical_destination: `knowledge/wiki/${destination}`, history_destination: history.get(variant.hash) ?? null, byte_identical_deduplicated: unique.length === 1}); }
  }
  const logs = new Map<string, Set<string>>();
  for (const variant of input.filter(v => v.scope === 'knowledge/wiki/log.md').sort((a,b) => rank(a.source).localeCompare(rank(b.source)))) { let day = 'Undated'; for (const line of redact(variant.text).text.split(/\r?\n/)) if (line.startsWith('## ')) day = line.slice(3).trim(); else if (line.startsWith('* ')) { const entries = logs.get(day) ?? new Set(); entries.add(line); logs.set(day, entries); } }
  write(join(wiki, 'log.md'), ['# Update Log', '', ...[...logs].sort(([a], [b]) => a.localeCompare(b)).flatMap(([day, entries]) => [`## ${day}`, ...entries, ''])].join('\n').trimEnd() + '\n');
  const categories = new Map<string, [string, string][]>();
  for (const file of readdirSync(wiki).filter(file => file.endsWith('.md') && !['index.md', 'log.md'].includes(file)).sort()) { const fm = frontmatter(readFileSync(join(wiki, file), 'utf8')), category = fm.type ?? 'Reference', group = categories.get(category) ?? []; group.push([fm.title ?? file.replace(/\.md$/, '').replaceAll('-', ' '), file]); categories.set(category, group); }
  write(join(wiki, 'index.md'), ['# Wiki Index', '', ...[...categories].sort(([a],[b])=>a.localeCompare(b)).flatMap(([category, group]) => [`## ${category}`, '', ...group.sort(([a],[b])=>a.toLowerCase().localeCompare(b.toLowerCase())).map(([title,file])=>`* [${title}](/${file})`), ''])].join('\n').trimEnd() + '\n');
  report.wiki = {source_entries: pages.length, canonical_page_paths: readdirSync(wiki).filter(file => file.endsWith('.md')).length, mappings};
  const others = new Map<string, Variant[]>(); for (const variant of input) if (variant.scope !== 'events.jsonl' && !variant.scope.startsWith('knowledge/wiki/') && !variant.scope.startsWith('consolidation/')) { const group = others.get(variant.scope) ?? []; group.push(variant); others.set(variant.scope, group); }
  report.other_files = [];
  for (const [path, group] of [...others].sort(([a],[b])=>a.localeCompare(b))) { group.sort((a,b)=>rank(a.source).localeCompare(rank(b.source))); const selected = group[0]!, redacted = materialize(selected), unique = new Map(group.map(v=>[v.hash,v])), preserved: string[] = []; write(join(output, path), redacted.data); if (unique.size > 1) for (const variant of unique.values()) { const variantData = materialize(variant), history = `${ledger}/files/${variant.source.name}/${path}`; write(join(output, history), variantData.data); preserved.push(history); redacted.records.push(...variantData.records); } report.other_files.push({path, canonical_source: selected.source.name, source_hashes: [...unique.keys()].sort(), preserved_variants: preserved, binary_preserved:redacted.binary}); for (const item of redacted.records) manifest.redactions.push({scope, path, ...item}); }
}
await main(() => {
  const args = process.argv.slice(2); assert(args.length === 0 || (args.length === 2 && args[0] === '--manifest'), 'node scripts/consolidate-prometheus-wikis.mjs [--manifest PATH]', 2);
  const root = repoRoot(), sources: Source[] = [{name:'primary',branch:'main-pre-consolidation',ref:snapshot},{name:'integration',branch:'codex/consolidate-main',root}];
  for (const entry of worktrees(root)) if (resolve(entry.worktree!) !== root) { const name = basename(entry.worktree!); assert(!sources.some(source => source.name === name), `duplicate worktree source name: ${name}`); sources.push({name,branch:(entry.branch??'detached').replace(/^refs\/heads\//,''),root:entry.worktree}); }
  const manifest: Manifest = {schema_version:1,purpose:'Lossless Prometheus wiki and event consolidation',snapshot_ref:snapshot,sources:sources.map(({name,branch})=>({name,branch})),scopes:Object.fromEntries(Object.keys(scopes).map(name=>[name,{}])),redactions:[]};
  const staging = mkdtempSync(join(tmpdir(),'hma-wiki-consolidation-'));
  try {
    // Build and validate every scope before touching the user's wiki.
    for (const [scope, path] of Object.entries(scopes)) build(scope, join(staging, scope), sources.flatMap(source=>variants(root,source,path)), manifest);
    for (const [scope, minimum] of Object.entries({root:238,desktop:20,'src-tauri':3,rust:34})) assert(manifest.scopes[scope]!.wiki!.canonical_page_paths >= minimum, `wiki consolidation proof failed: ${scope}: ${manifest.scopes[scope]!.wiki!.canonical_page_paths} < ${minimum}`);
    for (const [scope,path] of Object.entries(scopes)) {
      const source = join(staging,scope), destination = join(root,path), wiki = join(destination,'knowledge/wiki');
      if (existsSync(wiki)) for (const file of readdirSync(wiki).filter(file=>file.endsWith('.md'))) rmSync(join(wiki,file));
      for (const file of scopeFiles(source)) { const target = join(destination, relative(source, file)); mkdirSync(dirname(target), {recursive:true}); copyFileSync(file,target); }
    }
    const output = resolve(root,args[1]??'.prometheus/consolidation/2026-07-17/wiki-consolidation-manifest.json'); json(output,manifest);
    for (const scope of Object.keys(scopes)) console.log(`${scope} ${manifest.scopes[scope]!.wiki!.canonical_page_paths}`); console.log(output);
  } finally { rmSync(staging,{recursive:true,force:true}); }
});
