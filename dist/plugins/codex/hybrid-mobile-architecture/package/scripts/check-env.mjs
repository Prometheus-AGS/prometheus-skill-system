// TJ-ARCH-MOB-001 compliant
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { main, packageRoot, commandPath, run, object } from './portable/platform.mjs';
import { requireVersion } from './portable/versions.mjs';
await main(() => {
    const args = process.argv.slice(2);
    if (args.some(arg => !['--install', '--full', '--with-prometheus'].includes(arg)))
        throw new Error('Usage: node scripts/check-env.mjs [--install] [--full] [--with-prometheus]');
    const install = args.includes('--install') || args.includes('--full'), full = args.includes('--full');
    const missing = [], notes = [];
    const output = (name, argv = ['--version']) => commandPath(name) ? run(name, argv, { capture: true, allowFailure: true }) : { status: 127, stdout: '' };
    const version = (text) => text.match(/\d+\.\d+\.\d+(?:-[\w.-]+)?/)?.[0] ?? '';
    const check = (name, expected, remediation, argv = ['--version']) => {
        const valid = () => { const found = output(name, argv); return found.status === 0 && (!expected || version(found.stdout) === expected); };
        if (!valid() && install && remediation) {
            try {
                run(...remediation);
            }
            catch (error) {
                notes.push(error.message);
            }
        }
        if (valid())
            process.stdout.write(`OK ${name}${expected ? ' ' + expected : ''}\n`);
        else
            missing.push(`${name}${expected ? ' (required ' + expected + ')' : ''}`);
    };
    const rust = requireVersion('toolchain', 'rust');
    check('rustc', rust, commandPath('rustup') ? ['rustup', ['toolchain', 'install', rust]] : undefined);
    if (!commandPath('rustup'))
        notes.push('Install the native rustup distribution; no downloaded shell installer is executed.');
    const targets = ['wasm32-unknown-unknown', 'aarch64-linux-android', 'armv7-linux-androideabi', 'x86_64-linux-android', 'aarch64-apple-ios', 'aarch64-apple-ios-sim', 'x86_64-apple-ios', 'x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc'];
    if (commandPath('rustup'))
        for (const target of targets) {
            const installed = () => output('rustup', ['target', 'list', '--installed', '--toolchain', rust]).stdout.split(/\s+/).includes(target);
            if (!installed() && install)
                try {
                    run('rustup', ['target', 'add', '--toolchain', rust, target]);
                }
                catch (error) {
                    notes.push(error.message);
                }
            if (!installed())
                notes.push(`Target unavailable: ${target}`);
        }
    for (const [binary, crate, pin] of [['flutter_rust_bridge_codegen', 'flutter_rust_bridge_codegen', requireVersion('frameworks', 'flutter_rust_bridge')], ['cargo-ndk', 'cargo-ndk', undefined]])
        check(binary, pin, ['cargo', ['install', crate, ...(pin ? ['--version', '=' + pin] : []), '--locked']]);
    check('cargo', requireVersion('frameworks', 'tauri_cli'), ['cargo', ['install', 'tauri-cli', '--version', '=' + requireVersion('frameworks', 'tauri_cli'), '--locked']], ['tauri', '--version']);
    const node = requireVersion('toolchain', 'node');
    if (process.versions.node !== node) {
        missing.push(`Development Node ${node} (consumer scripts only require Node >=22)`);
        notes.push('Use the native Node installer or configured version manager; this checker does not replace its running interpreter.');
    }
    check('bun', requireVersion('toolchain', 'bun'), ['npm', ['install', '-g', 'bun@' + requireVersion('toolchain', 'bun')]]);
    check('pnpm', requireVersion('toolchain', 'pnpm'), ['npm', ['install', '-g', 'pnpm@' + requireVersion('toolchain', 'pnpm')]]);
    check('tsc', requireVersion('toolchain', 'typescript'), ['npm', ['install', '-g', 'typescript@' + requireVersion('toolchain', 'typescript')]]);
    check('openspec', requireVersion('frameworks', 'openspec'), ['npm', ['install', '-g', '@fission-ai/openspec@' + requireVersion('frameworks', 'openspec')]]);
    if (!commandPath('flutter') && install)
        try {
            run(process.execPath, [join(packageRoot, 'scripts/install-flutter.mjs')]);
        }
        catch (error) {
            notes.push(error.message);
        }
    if (commandPath('flutter')) {
        let info = object(JSON.parse(output('flutter', ['--version', '--machine']).stdout || '{}'));
        if (info.channel !== 'beta' && install)
            run('flutter', ['channel', 'beta']);
        if (full && (info.channel !== 'beta' || info.frameworkVersion !== requireVersion('toolchain', 'flutter')))
            run('flutter', ['upgrade']);
        info = object(JSON.parse(output('flutter', ['--version', '--machine']).stdout || '{}'));
        if (info.channel !== 'beta' || info.frameworkVersion !== requireVersion('toolchain', 'flutter'))
            missing.push(`Flutter beta ${requireVersion('toolchain', 'flutter')}`);
        const flutterPath = commandPath('flutter') ?? '', sdk = typeof info.flutterRoot === 'string' ? info.flutterRoot : join(flutterPath, '../..');
        const dart = join(sdk, 'bin/cache/dart-sdk/bin', process.platform === 'win32' ? 'dart.exe' : 'dart');
        if (!existsSync(dart) || run(dart, ['mcp-server', '--help'], { capture: true, allowFailure: true }).status)
            missing.push('Flutter-bundled Dart MCP server');
    }
    else
        missing.push('Flutter SDK');
    if (args.includes('--with-prometheus')) {
        for (const name of ['prometheus', 'forge', 'pk', 'pk-cherry', 'liter-llm'])
            check(name, undefined);
        if (commandPath('prometheus')) {
            const status = output('prometheus', ['doctor', '--json', '--exclude', 'control.kbd-runtime', '--exclude', 'state.kbd-orchestrator', '--exclude', 'control.kbd-rollout', '--exclude', 'service:sovereign-sync']);
            const report = object(JSON.parse(status.stdout || '{}')), contract = String(report.contractVersion ?? object(report.controlPlane).contractVersion ?? '');
            if (status.status !== 0 || Number(contract.split('.')[0]) < 2 || !contract)
                missing.push('Prometheus control plane contract >=2 operational doctor');
        }
        if (output('pk', ['doctor', '--json']).status)
            missing.push('pk operational doctor');
        if (install)
            notes.push('Full control-plane bootstrap is managed by its own package; no full checkout or shell installer is invoked.');
    }
    if (!process.env.ANDROID_HOME && !process.env.ANDROID_SDK_ROOT)
        notes.push('Android SDK not configured; Android builds unverified.');
    if (!process.env.ANDROID_NDK_HOME && !process.env.ANDROID_NDK)
        notes.push('Android NDK not configured; native Android builds unverified.');
    if (process.platform === 'darwin') {
        if (output('xcode-select', ['-p']).status && install)
            run('xcode-select', ['--install']);
        if (output('xcode-select', ['-p']).status)
            missing.push('Xcode command-line tools');
    }
    else
        notes.push('iOS builds require an Apple host.');
    if (process.platform === 'win32')
        notes.push('Rust targets are not linker proof: validate MSVC/Windows SDK and native Windows x64/ARM64 builds separately.');
    if (!commandPath('docker'))
        notes.push('Docker not installed; external service containers unavailable.');
    for (const note of notes)
        process.stderr.write(`NOTE ${note}\n`);
    if (missing.length)
        throw new Error(`Unmet prerequisites: ${missing.join(', ')}`);
    process.stdout.write('Selected tooling checks passed. Native target builds and runtime execution require separate proof.\n');
});
