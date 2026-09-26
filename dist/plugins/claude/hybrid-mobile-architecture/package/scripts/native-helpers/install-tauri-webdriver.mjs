import { appendFileSync, existsSync, readFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { assert, main, repoRoot, run } from './common.mjs';
function value(source, key, pattern) {
    const match = source.match(new RegExp(`^${key}\\s*=\\s*"(${pattern.source})"`, 'm'))?.[1];
    assert(Boolean(match), `versions.toml has no valid ${key}`);
    return match;
}
await main(() => {
    const args = process.argv.slice(2);
    assert(args.length <= 1 && (args.length === 0 || args[0] === '--check'), 'node scripts/install-tauri-webdriver.mjs [--check]', 2);
    const source = readFileSync(join(repoRoot(), 'versions.toml'), 'utf8');
    const driver = value(source, 'tauri_driver', /\d+\.\d+\.\d+(?:[-+][\w.-]+)?/);
    const edgeTool = value(source, 'msedgedriver_tool_rev', /[0-9a-f]{40}/);
    if (args[0] === '--check') {
        process.stdout.write(`tauri-driver=${driver}\nmsedgedriver-tool=${edgeTool}\n`);
        return;
    }
    assert(process.platform === 'win32', 'Tauri WebDriver installation is supported on Windows runners only', 2);
    run('cargo', ['install', 'tauri-driver', '--version', driver, '--locked']);
    run('cargo', ['install', '--git', 'https://github.com/chippers/msedgedriver-tool', '--rev', edgeTool, '--locked']);
    const cargoHome = process.env.CARGO_HOME ?? join(homedir(), '.cargo');
    const installer = join(cargoHome, 'bin', 'msedgedriver-tool.exe');
    assert(existsSync(installer), `msedgedriver-tool binary not found after installation: ${installer}`);
    run(installer, [], { cwd: process.cwd() });
    if (process.env.GITHUB_PATH)
        appendFileSync(process.env.GITHUB_PATH, `${process.cwd()}\n`);
    process.stdout.write(`PASS: installed pinned native Tauri and Edge WebDriver tools\n`);
});
