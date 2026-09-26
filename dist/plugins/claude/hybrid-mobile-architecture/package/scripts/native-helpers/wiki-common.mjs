import { createHash } from 'node:crypto';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { run } from './common.mjs';
export const scopes = { root: '.prometheus', desktop: 'apps/knowme-poc/desktop/.prometheus', 'src-tauri': 'apps/knowme-poc/desktop/src-tauri/.prometheus', rust: 'apps/knowme-poc/rust/.prometheus' };
export const sha = (data) => createHash('sha256').update(data).digest('hex');
export const fileHash = (path) => sha(readFileSync(path));
export function scopeFiles(root) {
    return readdirSync(root, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name)).flatMap(entry => entry.isDirectory() ? scopeFiles(join(root, entry.name)) : [join(root, entry.name)]);
}
export const git = (root, ...args) => run('git', args, { cwd: root, capture: true });
export function frontmatter(text) {
    if (!text.startsWith('---\n'))
        return {};
    const end = text.indexOf('\n---\n', 4);
    if (end < 0)
        return {};
    return Object.fromEntries(text.slice(4, end).split('\n').flatMap(line => { const match = /^(type|id|title|revision|timestamp|created_at|updated_at):\s*(.*?)\s*$/.exec(line); return match ? [[match[1], match[2].replace(/^["']+|["']+$/g, '')]] : []; }));
}
export function worktrees(root) {
    // NUL porcelain avoids Git quotePath escaping Unicode or whitespace in paths.
    return git(root, 'worktree', 'list', '--porcelain', '-z').split('\0\0').filter(Boolean).map(block => Object.fromEntries(block.split('\0').filter(Boolean).map(line => { const at = line.indexOf(' '); return at < 0 ? [line, ''] : [line.slice(0, at), line.slice(at + 1)]; })));
}
