// TJ-ARCH-MOB-001 compliant
import { join } from 'node:path';
import { main, packageRoot, run } from './portable/platform.mjs';
await main(() => {
    const args = process.argv.slice(2);
    if (args.some(arg => !['--fast', '--list'].includes(arg)))
        throw new Error('Usage: node scripts/run-all-gates.mjs [--fast|--list]');
    const gates = [
        ...['check-builder-authority', 'check-skill-contracts', 'check-prometheus-boundary', 'check-runtime-security', 'check-git-url-discovery', 'check-w6-mapping', 'sync-harness-skills', 'sync-skill-resources', 'verify-skill-manifest', 'verify-hooks-reliability', 'normalize-vendored-skills', 'test-opencode-plugin'].map(name => ({ name, steps: [[process.execPath, join(packageRoot, 'scripts', name + '.mjs'), ...(['sync-harness-skills', 'sync-skill-resources', 'normalize-vendored-skills'].includes(name) ? ['--check'] : name === 'check-builder-authority' ? ['--release'] : [])]] })),
        ...['doc-consistency', 'generator-purity'].map(mode => ({ name: 'audit:' + mode, steps: [[process.execPath, join(packageRoot, 'scripts/audit.mjs'), mode]] })),
        { name: 'opencode-plugin-syntax', steps: [[process.execPath, '--check', join(packageRoot, '.opencode/plugins/knowme-builder.mjs')]] },
        { name: 'generated-drift', steps: ['generate-builder-manifests', 'generate-skill-metadata', 'generate-skill-evals', 'generate-command-contract'].map(name => [process.execPath, join(packageRoot, 'scripts', name + '.mjs'), '--check']) },
        ...['test-harness-installer', 'verify-scaffold', 'verify-tray-templates', 'test-consumer-install'].map(name => ({ name, slow: true, steps: [[process.execPath, join(packageRoot, 'scripts', name + '.mjs')]] })),
    ];
    if (args.includes('--list')) {
        process.stdout.write(gates.map(g => `${g.name}\t${g.slow ? 'slow' : 'fast'}`).join('\n') + '\n');
        return;
    }
    let failed = 0, skipped = 0;
    for (const gate of gates) {
        if (gate.slow && args.includes('--fast')) {
            skipped++;
            process.stdout.write(`${gate.name}: SKIP\n`);
            continue;
        }
        try {
            for (const [command, ...argv] of gate.steps)
                run(command, argv, { cwd: packageRoot });
            process.stdout.write(`${gate.name}: PASS\n`);
        }
        catch (error) {
            failed++;
            process.stderr.write(`${gate.name}: FAIL ${error.message}\n`);
        }
    }
    process.exitCode = failed ? 1 : skipped ? 2 : 0;
    process.stdout.write(`${failed ? 'FAIL' : skipped ? 'PARTIAL' : 'PASS'}: ${gates.length - skipped - failed} passed, ${failed} failed, ${skipped} skipped.\n`);
});
