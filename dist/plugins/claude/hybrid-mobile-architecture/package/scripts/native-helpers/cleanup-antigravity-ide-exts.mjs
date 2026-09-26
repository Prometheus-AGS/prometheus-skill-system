import { existsSync, lstatSync, readdirSync, rmSync } from 'node:fs';
import { homedir } from 'node:os';
import { join, resolve } from 'node:path';
import { assert, main, text } from './common.mjs';
await main(() => {
    const args = process.argv.slice(2);
    let root = join(homedir(), '.antigravity-ide/extensions'), apply = false;
    for (let i = 0; i < args.length; i++) {
        if (args[i] === '--apply')
            apply = true;
        else if (args[i] === '--dry-run')
            apply = false;
        else if (args[i] === '--root') {
            assert(args[i + 1], '--root requires a path');
            root = resolve(args[++i]);
        }
        else {
            assert(args[i] === '--help', `unknown option ${args[i]}`);
            console.log('node cleanup_antigravity_ide_exts.mjs [--root DIR] [--dry-run|--apply]\nDefault: preview only. --apply removes older duplicate extension versions.');
            return;
        }
    }
    assert(existsSync(root) && lstatSync(root).isDirectory(), `extension root missing or symlink: ${root}`);
    const groups = new Map(), skipped = [];
    for (const dir of readdirSync(root, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
        if (!dir.isDirectory())
            continue;
        const path = join(root, dir.name);
        let id, version;
        try {
            const pkg = JSON.parse(text(join(path, 'package.json')));
            if (typeof pkg.publisher === 'string' && typeof pkg.name === 'string')
                id = `${pkg.publisher}.${pkg.name}`;
            if (typeof pkg.version === 'string')
                version = pkg.version;
        }
        catch { /* Preserve directory-name fallback for missing/broken package metadata. */ }
        const fallback = /^(.*)-(\d[^/]*)$/.exec(dir.name);
        if (fallback) {
            id ??= fallback[1];
            version ??= fallback[2];
        }
        if (!id || !version) {
            skipped.push(dir.name);
            continue;
        }
        const list = groups.get(id) ?? [];
        list.push({ path, directory: dir.name, version, obsolete: existsSync(join(path, '.obsolete')) });
        groups.set(id, list);
    }
    const key = (version) => version.trim().replace(/^[vV]+/, '').split('.').map(part => Number(/^\d+/.exec(part)?.[0] ?? 0));
    const size = (dir) => readdirSync(dir, { withFileTypes: true }).reduce((sum, entry) => { const path = join(dir, entry.name); return sum + (entry.isDirectory() ? size(path) : lstatSync(path).size); }, 0);
    let freed = 0;
    for (const [id, group] of [...groups].sort(([a], [b]) => a.localeCompare(b))) {
        if (group.length < 2)
            continue;
        group.sort((a, b) => { const x = key(a.version), y = key(b.version); for (let i = 0; i < Math.max(x.length, y.length, 3); i++)
            if ((x[i] ?? 0) !== (y[i] ?? 0))
                return (x[i] ?? 0) - (y[i] ?? 0); return 0; });
        for (const entry of group.slice(0, -1)) {
            const bytes = size(entry.path);
            let status = 'WOULD DELETE';
            if (apply)
                try {
                    rmSync(entry.path, { recursive: true });
                    status = 'deleted';
                    freed += bytes;
                }
                catch (error) {
                    status = `FAILED: ${error}`;
                    process.exitCode = 1;
                }
            console.log(`${id} v${entry.version} | ${entry.directory} | ${(bytes / 1024 ** 2).toFixed(1)} MB | ${status}${entry.obsolete ? ' [had .obsolete]' : ''}`);
        }
        const newest = group.at(-1);
        console.log(`${id} v${newest.version} | ${newest.directory} | KEPT (newest)`);
    }
    if (skipped.length)
        console.log(`Dirs skipped (no parsable id/version):\n${skipped.join('\n')}`);
    console.log(`TOTAL FREED: ${(freed / 1024 ** 2).toFixed(1)} MB (${(freed / 1024 ** 3).toFixed(2)} GB)${apply ? '' : ' — dry run; no files removed'}`);
});
