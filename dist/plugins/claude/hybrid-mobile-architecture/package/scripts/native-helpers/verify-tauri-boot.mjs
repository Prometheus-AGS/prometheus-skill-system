import { spawn, spawnSync } from 'node:child_process';
import { closeSync, existsSync, mkdirSync, openSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { setTimeout } from 'node:timers/promises';
import { assert, main, text } from './common.mjs';
export async function stopTree(child) {
    if (!child.pid)
        return;
    if (process.platform === 'win32') {
        spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], { stdio: 'ignore', shell: false });
        return;
    }
    try {
        process.kill(-child.pid, 'SIGTERM');
    }
    catch {
        return;
    }
    for (let i = 0; i < 5; i++) {
        await setTimeout(1000);
        try {
            process.kill(-child.pid, 0);
        }
        catch {
            return;
        }
    }
    try {
        process.kill(-child.pid, 'SIGKILL');
    }
    catch { /* Already exited. */ }
}
await main(async () => {
    assert(process.argv.length >= 4 && process.argv.length <= 5, 'node scripts/verify-tauri-boot.mjs <binary> <empty-app-data-dir> [timeout-seconds]', 2);
    const binary = resolve(process.argv[2]), data = resolve(process.argv[3]), seconds = Number(process.argv[4] ?? 240);
    assert(existsSync(binary) && statSync(binary).isFile(), `Tauri binary is not a file: ${binary}`, 2);
    assert(!existsSync(data), `App-data proof directory must not already exist: ${data}`, 2);
    assert(Number.isFinite(seconds) && seconds > 0, 'timeout must be positive', 2);
    mkdirSync(data, { recursive: true });
    const log = join(data, 'tauri-process.log'), diagnostics = join(data, 'diagnostics/desktop.log'), handle = openSync(log, 'w');
    const child = spawn(binary, [], { env: { ...process.env, APP_DATA_DIR: data, GEN_UI_APP_DATA_DIR: data, RUST_LOG: 'info' }, stdio: ['ignore', handle, handle], detached: process.platform !== 'win32', shell: false });
    closeSync(handle);
    let startupError;
    child.on('error', error => { startupError = error; });
    const interrupt = () => { void stopTree(child).finally(() => process.exit(130)); };
    process.once('SIGINT', interrupt);
    process.once('SIGTERM', interrupt);
    try {
        const deadline = Date.now() + seconds * 1000;
        while (Date.now() < deadline) {
            assert(!startupError, `Tauri launch failed: ${startupError?.message}`);
            assert(child.exitCode === null && child.signalCode === null, 'Tauri exited before reaching ready state');
            const output = existsSync(diagnostics) ? text(diagnostics) : '';
            const legacyReady = ['desktop migrations ready', 'seed load ready', 'sync ready in local-only mode'].every(signal => output.includes(signal)) && ['config-db', 'memory-db', 'model-cache/fastembed'].every(dir => existsSync(join(data, dir)) && statSync(join(data, dir)).isDirectory());
            const notesReady = existsSync(join(data, 'notes.sqlite3')) && statSync(join(data, 'notes.sqlite3')).isFile();
            if (legacyReady || notesReady) {
                console.log(`Tauri boot proof passed\nmode=${notesReady ? 'notes-baseline' : 'legacy-runtime'}\ndiagnostics=${existsSync(diagnostics) ? diagnostics : 'not-emitted'}\napp_data=${data}`);
                return;
            }
            await setTimeout(1000);
        }
        assert(false, `Tauri did not reach ready within ${seconds}s`);
    }
    catch (error) {
        for (const path of [log, diagnostics])
            if (existsSync(path))
                console.error(text(path).split(/\r?\n/).slice(-200).join('\n'));
        throw error;
    }
    finally {
        process.off('SIGINT', interrupt);
        process.off('SIGTERM', interrupt);
        await stopTree(child);
    }
});
