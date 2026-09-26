// TJ-ARCH-MOB-001 compliant
import { existsSync, lstatSync, readFileSync, readdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { main, packageRoot } from './portable/platform.mjs';
await main(() => {
    if (process.argv.length > 3)
        throw new Error('Usage: node scripts/check-portable-runtime.mjs [package-root]');
    const root = process.argv[2] ? resolve(process.argv[2]) : packageRoot;
    if (!existsSync(join(root, 'builder.manifest.json')))
        throw new Error('Not a Builder package root');
    const excluded = new Set(['.git', 'node_modules', 'target', 'vendor', 'compass-out', 'dist', 'build', '.worktrees', 'worktrees']);
    const thirdPartyScript = (path) => path.startsWith('assets/templates/baselines/flutter/mobile/rust_builder/cargokit/') ||
        path === 'assets/templates/baselines/flutter/mobile/android/gradlew';
    const violations = [];
    let checked = 0;
    function walk(directory, prefix = '') {
        for (const name of readdirSync(directory).sort()) {
            if (excluded.has(name))
                continue;
            const path = join(directory, name), relative = prefix ? `${prefix}/${name}` : name;
            const stat = lstatSync(path);
            if (stat.isSymbolicLink())
                continue; // Symlink payload safety belongs to the staging gate.
            if (stat.isDirectory())
                walk(path, relative);
            else if (stat.isFile()) {
                checked++;
                const header = readFileSync(path).subarray(0, 160).toString('utf8').split('\n')[0];
                if ((/\.(sh|py)$/i.test(name) || /^#!.*\b(?:ba|da|z|k)?sh\b|^#!.*\bpython[\d.]*\b/.test(header)) && !thirdPartyScript(relative))
                    violations.push(relative);
            }
        }
    }
    walk(root);
    if (violations.length)
        throw new Error(`First-party shell/Python files remain:\n${violations.join('\n')}`);
    process.stdout.write(`Portable runtime file gate passed (${checked} files; vendor internals excluded).\n`);
});
