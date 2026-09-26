// TJ-ARCH-MOB-001 compliant
import { join } from 'node:path';
import { commandPath, packageRoot, run } from './platform.mjs';
export function builder(args) {
    const result = process.env.KNOWME_BUILDER_FORCE_SOURCE !== '1' && commandPath('knowme-builder')
        ? run('knowme-builder', args, { allowFailure: true })
        : run('cargo', ['run', '--quiet', '--manifest-path', join(packageRoot, 'tools/knowme-builder/Cargo.toml'), '--', ...args], { allowFailure: true });
    process.exitCode = result.status;
}
export function scaffold(profile) {
    const args = process.argv.slice(2);
    if (args.length !== 1 || args[0].startsWith('-'))
        throw new Error('Expected exactly one destination. Use knowme-builder new for typed options; legacy options are not ignored.');
    builder(['new', args[0], '--profile', profile, '--mode', 'runnable']);
}
