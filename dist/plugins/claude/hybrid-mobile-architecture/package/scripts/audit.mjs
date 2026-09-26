// TJ-ARCH-MOB-001 compliant
import { existsSync, readdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { main, packageRoot, run } from './portable/platform.mjs';
import { audit } from './portable/audit.mjs';
await main(() => {
    const args = process.argv.slice(2), mode = args[0] ?? 'flutter';
    let root = resolve(args[1] ?? '.');
    if (args.length > 2 || !['flutter', 'tauri', 'web', 'rust', 'rust-workspace', 'doc-consistency', 'generator-purity', 'tray-templates', 'all'].includes(mode)) {
        process.exitCode = 2;
        process.stderr.write('Unknown audit mode or extra arguments\n');
        return;
    }
    if (mode === 'tray-templates') {
        process.exitCode = run(process.execPath, [join(packageRoot, 'scripts/verify-tray-templates.mjs')], { allowFailure: true }).status;
        return;
    }
    const results = [];
    if (mode === 'all') {
        if (!existsSync(root))
            throw new Error(`Missing project root: ${root}`);
        if (!existsSync(join(root, 'mobile')) && !existsSync(join(root, 'desktop'))) {
            const children = readdirSync(root, { withFileTypes: true }).filter(e => e.isDirectory() && (existsSync(join(root, e.name, 'mobile')) || existsSync(join(root, e.name, 'desktop'))));
            if (children.length === 1)
                root = join(root, children[0].name);
        }
        for (const [sub, platform] of [['mobile', 'flutter'], ['desktop', 'tauri'], ['rust/gen_ui_core', 'rust'], ['rust', 'rust-workspace']])
            if (existsSync(join(root, sub)))
                results.push([platform, audit(platform, join(root, sub))]);
        if (existsSync(join(root, 'web')) || existsSync(join(root, 'server')))
            results.push(['web', audit('web', root)]);
        if (!results.length)
            throw new Error('No auditable Flutter/Tauri/monolithic Rust surfaces found; refusing a vacuous all pass');
        for (const global of ['doc-consistency', 'generator-purity'])
            results.push([global, audit(global, packageRoot)]);
    }
    else
        results.push([mode, audit(mode, root)]);
    for (const [name, result] of results) {
        for (const warning of result.warnings)
            process.stderr.write(`WARN ${name}: ${warning}\n`);
        for (const failure of result.failures)
            process.stderr.write(`FAIL ${name}: ${failure}\n`);
        process.stdout.write(`${name}: ${result.checked} checks, ${result.failures.length} failures, ${result.warnings.length} warnings. Static audit is not build/run proof.\n`);
    }
    process.exitCode = results.some(([, result]) => result.failures.length) ? 1 : 0;
});
