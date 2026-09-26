import { resolve } from 'node:path';
import { randomUUID } from 'node:crypto';
import { assert, main, run } from './common.mjs';
await main(() => {
    assert(process.argv.length === 3 || process.argv.length === 4, 'node scripts/verify-flutter-ios-restart.mjs <flutter-project> [existing-device-id]', 2);
    const project = resolve(process.argv[2]);
    const existingDevice = process.argv[3];
    if (existingDevice) {
        const base = ['test', '--no-uninstall', 'integration_test/notes_test.dart', '-d', existingDevice];
        run('flutter', base, { cwd: project });
        run('flutter', [...base, '--dart-define=VERIFY_RESTART=true'], { cwd: project });
        process.stdout.write(`PASS: Flutter UI -> Rust FFI -> SQLite -> application relaunch recovery on device ${existingDevice}\n`);
        return;
    }
    assert(process.platform === 'darwin', 'managed iOS simulator verification requires macOS', 2);
    const inventory = JSON.parse(run('xcrun', ['simctl', 'list', '--json'], { capture: true }));
    const device = inventory.devicetypes
        ?.filter(item => item.identifier && /^iPhone /.test(item.name ?? ''))
        .at(-1);
    const runtime = inventory.runtimes
        ?.filter(item => item.identifier && item.isAvailable !== false && /^iOS /.test(item.name ?? ''))
        .at(-1);
    assert(Boolean(device?.identifier), 'No available iPhone simulator device type');
    assert(Boolean(runtime?.identifier), 'No available iOS simulator runtime');
    const name = `knowme-builder-cert-${randomUUID()}`;
    const id = run('xcrun', ['simctl', 'create', name, device.identifier, runtime.identifier], { capture: true }).trim();
    assert(Boolean(id), 'simctl did not return a device identifier');
    try {
        run('xcrun', ['simctl', 'boot', id]);
        run('xcrun', ['simctl', 'bootstatus', id, '-b']);
        const base = ['test', '--no-uninstall', 'integration_test/notes_test.dart', '-d', id];
        run('flutter', base, { cwd: project });
        run('flutter', [...base, '--dart-define=VERIFY_RESTART=true'], { cwd: project });
        process.stdout.write(`PASS: Flutter UI -> Rust FFI -> SQLite -> application relaunch recovery on simulator ${id}\n`);
    }
    finally {
        run('xcrun', ['simctl', 'shutdown', id], { allowFailure: true });
        run('xcrun', ['simctl', 'delete', id], { allowFailure: true });
    }
});
