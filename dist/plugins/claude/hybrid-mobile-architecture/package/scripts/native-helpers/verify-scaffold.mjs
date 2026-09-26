import { existsSync, mkdirSync, mkdtempSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parse } from 'smol-toml';
import { assert, files, main, run, text } from './common.mjs';
await main(() => {
    const mode = process.argv[2] ?? 'verify';
    assert(['verify', '--update'].includes(mode), 'usage: node scripts/verify-scaffold.mjs [--update]');
    const root = resolve(dirname(fileURLToPath(import.meta.url)), '..'), work = mkdtempSync(join(tmpdir(), 'hma-scaffold-')), project = join(work, 'proj');
    const manifest = join(root, 'ci/expected-tree.txt');
    try {
        mkdirSync(project);
        run(process.execPath, [join(root, 'scripts/scaffold-rust-core.mjs'), project], { capture: true, env: { ...process.env, KNOWME_BUILDER_FORCE_SOURCE: process.env.KNOWME_BUILDER_FORCE_SOURCE ?? '1' } });
        run(process.execPath, [join(root, 'scripts/add-project-skills.mjs'), project], { capture: true });
        const paths = files(project).map(path => relative(project, path).replaceAll('\\', '/')).filter(path => !/^rust\/(vendor|target)\//.test(path) && !path.endsWith('.lock')).sort();
        const metadata = text(join(project, '.knowme-builder/project.toml')).split(/\r?\n/).filter(line => /^(profile|builderVersion|requiredPrometheusContract|generationMode) =/.test(line)).sort();
        const actual = ['# Structural manifest of the scaffold output.', '# Regenerate: node scripts/verify-scaffold.mjs --update', '# Paths and key manifest values only — never file contents.', '', '## paths', ...paths, '', '## builder metadata', ...metadata, ''].join('\n');
        if (mode === '--update') {
            mkdirSync(dirname(manifest), { recursive: true });
            writeFileSync(manifest, actual);
            console.log('manifest updated: ci/expected-tree.txt; review the intentional structural diff before committing');
            return;
        }
        assert(existsSync(manifest), 'manifest missing; run node scripts/verify-scaffold.mjs --update');
        const expected = text(manifest);
        if (expected !== actual) {
            const expectedLines = new Set(expected.split('\n')), actualLines = new Set(actual.split('\n'));
            console.error([...expectedLines].filter(line => !actualLines.has(line)).map(line => `- ${line}`).concat([...actualLines].filter(line => !expectedLines.has(line)).map(line => `+ ${line}`)).slice(0, 60).join('\n'));
            assert(false, 'scaffold output changed; update ci/expected-tree.txt only after reviewing an intentional change');
        }
        for (const subtree of ['rust', 'scripts'])
            if (existsSync(join(project, subtree)))
                for (const file of files(join(project, subtree)))
                    assert(!/__APP_[A-Z_]+__|__ENV_PREFIX__|@[A-Z_]+_VERSION@|@EMBEDDING_DIM@/.test(text(file)), `unsubstituted placeholders in emitted code: ${file}`);
        const count = (path) => readdirSync(path, { withFileTypes: true }).filter(entry => entry.isDirectory() && existsSync(join(path, entry.name, 'SKILL.md'))).length;
        const expectedCount = count(join(root, 'templates/project-skills'));
        assert(expectedCount > 0, 'derived 0 companion skills; refusing vacuous assertion');
        for (const harness of ['.claude', '.codex', '.opencode', '.kimi', '.agents', '.kimi-code']) {
            const skills = join(project, harness, 'skills');
            assert(count(skills) === expectedCount, `${harness} companion skill count mismatch; expected ${expectedCount}`);
            assert(!existsSync(join(skills, 'hybrid-mobile-architecture')), `${harness} incorrectly contains package routing skill`);
        }
        let tomls = 0;
        for (const file of files(join(project, 'rust')))
            if (file.endsWith('.toml') && !relative(project, file).split(/[\\/]/).some(part => ['vendor', 'target'].includes(part))) {
                parse(text(file));
                tomls++;
            }
        console.log(`✓ ${tomls} TOML manifests parse\n✓ scaffold verification passed`);
    }
    finally {
        rmSync(work, { recursive: true, force: true });
    }
});
