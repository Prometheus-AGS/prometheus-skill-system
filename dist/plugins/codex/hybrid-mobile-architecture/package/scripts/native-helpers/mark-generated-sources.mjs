import { existsSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { basename, join, resolve } from 'node:path';
import { assert, main } from './common.mjs';
const marker = '// TJ-ARCH-MOB-001 compliant';
function files(root) {
    if (!existsSync(root))
        return [];
    return readdirSync(root, { withFileTypes: true }).flatMap(entry => {
        const path = join(root, entry.name);
        if (entry.isSymbolicLink())
            return [];
        return entry.isDirectory() ? files(path) : [path];
    });
}
function generatedSources(project) {
    const mobile = basename(project) === 'mobile' ? project : join(project, 'mobile');
    const workspace = basename(project) === 'mobile' ? resolve(project, '..') : project;
    const dart = files(join(mobile, 'lib')).filter(path => path.includes(`${join('lib', 'bridge', 'generated')}`) ||
        path.endsWith('.g.dart') ||
        path.endsWith('.freezed.dart'));
    const rust = files(join(workspace, 'rust', 'gen_ui_ffi', 'src')).filter(path => basename(path).startsWith('frb_generated') && path.endsWith('.rs'));
    return [...dart, ...rust].sort();
}
await main(() => {
    const args = process.argv.slice(2);
    assert(args.length >= 1 && args.length <= 2 && (args.length === 1 || args[1] === '--check'), 'node scripts/mark-generated-sources.mjs <generated-project-root|mobile-dir> [--check]', 2);
    const project = resolve(args[0]);
    const sources = generatedSources(project);
    assert(sources.length > 0, `No generated Flutter/Rust bridge sources found under ${project}`, 2);
    const missing = sources.filter(path => !readFileSync(path, 'utf8').startsWith(`${marker}\n`));
    if (args[1] === '--check') {
        assert(missing.length === 0, `Generated sources missing architecture marker:\n${missing.join('\n')}`);
    }
    else {
        for (const path of missing)
            writeFileSync(path, `${marker}\n${readFileSync(path, 'utf8')}`);
    }
    process.stdout.write(`PASS: ${sources.length} generated sources carry ${marker}\n`);
});
