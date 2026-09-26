// TJ-ARCH-MOB-001 compliant
import { createHash } from 'node:crypto';
import { existsSync, lstatSync, mkdirSync, readFileSync, readdirSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, isAbsolute, join, relative, resolve, sep } from 'node:path';
import { main, packageRoot } from './portable/platform.mjs';
import { verifyManifest } from './portable/manifest.mjs';
// Both distributions consume this exact payload. Variant is receipt metadata,
// never a switch that silently drops runtime or transitive template resources.
const roots = ['skills', 'templates', 'assets', 'scripts', 'runtime', 'tools', 'compatibility', 'schemas', 'references', 'deploy', 'ci', '.claude-plugin', '.codex-plugin', '.agents/skills', '.agents/plugins', '.claude/skills', '.claude/hooks', '.claude/commands', '.codex/skills', '.codex/prompts', '.opencode/skills', '.opencode/hooks', '.opencode/plugins', '.opencode/commands', '.kimi/skills', '.kimi-code/skills', '.kimi-code/hooks', '.kimi-code/commands', 'builder.manifest.json', 'plugin.json', 'marketplace.json', 'versions.toml', 'README.md', 'AGENTS.md', 'CLAUDE.md', 'LICENSE'];
const excluded = new Set(['target', 'node_modules', '.git', 'dist', 'build', '.DS_Store']);
const thirdPartyScript = (path) => path.startsWith('assets/templates/baselines/flutter/mobile/rust_builder/cargokit/') ||
    path === 'assets/templates/baselines/flutter/mobile/android/gradlew';
await main(() => {
    const args = process.argv.slice(2);
    let output = '', variant = '';
    for (let index = 0; index < args.length; index++) {
        const key = args[index], value = args[++index];
        if (!value || value.startsWith('--'))
            throw new Error(`Missing value for ${key}`);
        if (key === '--output' && !output)
            output = value;
        else if (key === '--variant' && !variant)
            variant = value;
        else
            throw new Error(`Unknown or repeated option: ${key}`);
    }
    if (!output || !['full', 'mini'].includes(variant))
        throw new Error('Usage: node scripts/stage-skill-package.mjs --output <new-directory> --variant <full|mini>');
    const destination = resolve(output), fromRoot = relative(packageRoot, destination);
    if (!(isAbsolute(fromRoot) || fromRoot === '..' || fromRoot.startsWith(`..${sep}`)))
        throw new Error('Stage outside the source checkout');
    if (existsSync(destination))
        throw new Error('Output already exists; refusing to replace staged or user files');
    const errors = verifyManifest(packageRoot);
    if (errors.length)
        throw new Error(errors.join('\n'));
    const files = new Map(), folded = new Set();
    function collect(path) {
        // This maintainer-only entrypoint needs the source repository's Git and
        // OpenSpec execution state; it is intentionally not a consumer command.
        if (path === 'scripts/dispatch.mjs' || path === 'runtime/src/dispatch.mts')
            return;
        const source = join(packageRoot, path), stat = lstatSync(source);
        if (stat.isSymbolicLink())
            throw new Error(`Package contains a symbolic link: ${path}`);
        if (stat.isDirectory()) {
            for (const name of readdirSync(source).sort())
                if (!excluded.has(name))
                    collect(`${path}/${name}`);
            return;
        }
        if (!stat.isFile())
            throw new Error(`Unsupported package entry: ${path}`);
        const key = path.toLocaleLowerCase('en-US');
        if (folded.has(key))
            throw new Error(`Case-insensitive path collision: ${path}`);
        for (const part of path.split('/'))
            if (/[<>:"\\|?*\x00-\x1f]/.test(part) || /[ .]$/.test(part) || /^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(part))
                throw new Error(`Windows-incompatible path: ${path}`);
        const bytes = readFileSync(source), header = bytes.subarray(0, 160).toString('utf8').split('\n')[0];
        if ((/\.(sh|py)$/i.test(path) || /^#!.*\b(?:ba|da|z|k)?sh\b|^#!.*\bpython[\d.]*\b/.test(header)) && !path.split('/').includes('vendor') && !thirdPartyScript(path))
            throw new Error(`First-party shell/Python payload: ${path}`);
        folded.add(key);
        files.set(path, bytes);
    }
    for (const root of roots)
        if (existsSync(join(packageRoot, root)))
            collect(root);
    const hashes = [...files].sort(([left], [right]) => left.localeCompare(right, 'en')).map(([path, bytes]) => ({ path, sha256: createHash('sha256').update(bytes).digest('hex') }));
    const payloadSha256 = createHash('sha256').update(JSON.stringify(hashes)).digest('hex');
    mkdirSync(dirname(destination), { recursive: true });
    const staging = `${destination}.staging-${process.pid}`;
    mkdirSync(staging);
    try {
        for (const [path, bytes] of files) {
            const target = join(staging, 'package', path);
            mkdirSync(dirname(target), { recursive: true });
            writeFileSync(target, bytes);
        }
        writeFileSync(join(staging, 'receipt.json'), `${JSON.stringify({ schemaVersion: 1, variant, payloadSha256, files: hashes, evidence: 'staged-only; installation, compilation and execution require separate checks' }, null, 2)}\n`);
        // Avoid replacing a destination created since preflight, including Windows.
        if (existsSync(destination))
            throw new Error('Output appeared during staging');
        renameSync(staging, destination);
        process.stdout.write(`${JSON.stringify({ variant, path: destination, payloadSha256, files: hashes.length })}\n`);
    }
    finally {
        rmSync(staging, { force: true, recursive: true });
    }
});
