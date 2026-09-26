import { existsSync, lstatSync, mkdirSync, readlinkSync, readFileSync, writeFileSync } from 'node:fs';
import { basename, dirname, join, relative, resolve } from 'node:path';
import { assert, files, main, repoRoot } from './common.mjs';
import { fileHash, frontmatter, git, scopes, scopeFiles, worktrees } from './wiki-common.mjs';
await main(() => {
  assert(process.argv.length === 3, 'node scripts/worktree-consolidation-inventory.mjs OUTPUT', 2);
  const root = repoRoot();
  const records = worktrees(root).map(raw => {
    const worktree = raw.worktree!; const parts = git(worktree, 'status', '--porcelain=v1', '-z', '--untracked-files=all').split('\0'), dirty: Record<string, unknown>[] = [];
    for (let i = 0; parts[i]; i++) { const line = parts[i]!, status = line.slice(0, 2), path = line.slice(3), entry: Record<string, unknown> = { status, path }; if (/[RC]/.test(status)) entry.source_path = parts[++i]; const candidate = join(worktree, path); try { const stat = lstatSync(candidate); if (stat.isSymbolicLink()) entry.symlink_target = readlinkSync(candidate); else if (stat.isFile()) { entry.sha256 = fileHash(candidate); entry.size = stat.size; } } catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error; } dirty.push(entry); }
    const inventory = Object.fromEntries(Object.entries(scopes).map(([name, rel]) => { const scope = join(worktree, rel), present = existsSync(scope); const entries = present ? scopeFiles(scope).map(path => { const scopePath = relative(scope, path).replaceAll('\\', '/'); const entry: Record<string, unknown> = { path: relative(worktree, path).replaceAll('\\', '/'), scope_path: scopePath, sha256: lstatSync(path).isSymbolicLink() ? null : fileHash(path), size: lstatSync(path).size }; if (lstatSync(path).isSymbolicLink()) entry.symlink_target = readlinkSync(path); else if (path.endsWith('.md')) { const fm = frontmatter(readFileSync(path, 'utf8')); delete fm.type; delete fm.title; if (Object.keys(fm).length) entry.frontmatter = fm; } return entry; }) : []; return [name, { exists: present, file_count: entries.length, wiki_markdown_count: entries.filter(entry => String(entry.scope_path).startsWith('knowledge/wiki/') && String(entry.scope_path).endsWith('.md')).length, files: entries }]; }));
    return { name: resolve(worktree) === root ? 'primary' : basename(worktree), path: resolve(worktree) === root ? '$REPO_ROOT' : `$REPO_ROOT/${relative(root, worktree).replaceAll('\\', '/')}`, head: raw.HEAD ?? '', branch: (raw.branch ?? '').replace(/^refs\/heads\//, ''), locked: raw.locked ?? null, dirty_files: dirty, prometheus_scopes: inventory };
  });
  const branches = git(root, 'for-each-ref', '--format=%(refname)|%(objectname)|%(upstream)|%(upstream:track)', 'refs/heads', 'refs/remotes').trim().split('\n').filter(Boolean).map(line => { const [ref, sha, upstream, tracking] = line.split('|'); return { ref, sha, upstream, tracking }; });
  const manifest = { schema_version: 1, purpose: 'Pre-consolidation loss-prevention inventory', repository: '$REPO_ROOT', baseline_main: git(root, 'rev-parse', 'main').trim(), branches, worktrees: records };
  const output = resolve(root, process.argv[2]!); mkdirSync(dirname(output), { recursive: true }); writeFileSync(output, JSON.stringify(manifest, null, 2) + '\n'); console.log(output);
});
