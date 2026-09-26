// TJ-ARCH-MOB-001 compliant
import { mkdtemp, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { main, packageRoot, run } from '../portable/platform.mjs';
await main(async () => {
    if (process.argv.length !== 2)
        throw new Error('No arguments accepted');
    const shared = await mkdtemp(join(tmpdir(), 'consumer-negative-'));
    try {
        run('git', ['clone', '--quiet', '--no-local', packageRoot, join(shared, 'pristine')]);
        const env = { ...process.env, CONSUMER_INSTALL_REUSE_CLONE: join(shared, 'pristine'), CONSUMER_INSTALL_SKIP_CONTRACT: '' };
        const proof = join(packageRoot, 'scripts/test-consumer-install.mjs');
        const control = run(process.execPath, [proof], { capture: true, allowFailure: true, env: { ...env, CONSUMER_INSTALL_FIXTURE: '' } });
        if (control.status || !control.stdout.includes('Committed consumer install verified'))
            throw new Error('Positive committed-package control failed; negative fixtures would be meaningless. Commit/release verification is still pending.');
        const fixtures = [['undeclared-skill', 'Undeclared skill'], ['missing-plugin-json', 'Invalid or missing manifest: plugin.json'], ['unresolvable-skill', 'Declared skill missing'], ['frontmatter-name-mismatch', 'Frontmatter name disagrees'], ['missing-harness-mirror', 'Missing consumer mirror']];
        for (const [fixture, expected] of fixtures) {
            const result = run(process.execPath, [proof], { capture: true, allowFailure: true, env: { ...env, CONSUMER_INSTALL_FIXTURE: fixture } });
            if (result.status !== 1 || !(result.stdout + result.stderr).includes(expected))
                throw new Error(`Fixture ${fixture} did not identify ${expected} (exit ${result.status})`);
        }
        process.stdout.write('Committed consumer positive control and five negative fixtures passed.\n');
    }
    finally {
        await rm(shared, { recursive: true, force: true });
    }
});
