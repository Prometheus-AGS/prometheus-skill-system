// TJ-ARCH-MOB-001 compliant
import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync, realpathSync, statSync } from 'node:fs';
import { delimiter, dirname, join, resolve } from 'node:path';
export class ToolError extends Error {
    code;
    constructor(message, code = 1) {
        super(message);
        this.code = code;
    }
}
export function run(command, args, options = {}) {
    const result = spawnSync(command, args, { cwd: options.cwd, env: options.env ?? process.env, encoding: 'utf8', stdio: options.capture ? ['ignore', 'pipe', 'pipe'] : 'inherit', shell: false, maxBuffer: 64 * 1024 * 1024 });
    if (result.error)
        throw new ToolError(`required tool failed: ${command}: ${result.error.message}`, 127);
    if (result.status !== 0 && !options.allowFailure)
        throw new ToolError(`${command} failed (${result.status ?? result.signal})${result.stderr ? `: ${result.stderr.trim()}` : ''}`, result.status ?? 1);
    return result.status === 0 ? (result.stdout ?? '') : '';
}
export function executable(name) {
    for (const dir of (process.env.PATH ?? '').split(delimiter)) {
        for (const suffix of process.platform === 'win32' ? ['.exe', ''] : ['']) {
            const candidate = join(dir, name + suffix);
            if (existsSync(candidate) && statSync(candidate).isFile())
                return candidate;
        }
    }
    return undefined;
}
// npm.cmd is a vendor batch launcher. Resolve its JavaScript entrypoint instead
// of invoking cmd.exe or shell:true, so user arguments never become shell code.
export function npm(args, cwd) {
    const candidates = [process.env.npm_execpath, join(dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js'), join(dirname(process.execPath), '../lib/node_modules/npm/bin/npm-cli.js')];
    for (const dir of (process.env.PATH ?? '').split(delimiter)) {
        candidates.push(join(dir, 'node_modules/npm/bin/npm-cli.js'));
        const command = join(dir, 'npm');
        if (existsSync(command)) {
            const real = realpathSync(command);
            if (real.endsWith('npm-cli.js'))
                candidates.push(real);
        }
    }
    const entry = candidates.find((candidate) => Boolean(candidate && candidate.endsWith('npm-cli.js') && existsSync(candidate)));
    if (!entry)
        throw new ToolError('npm JavaScript entrypoint not found; install Node.js with npm or set npm_execpath to npm-cli.js', 127);
    run(process.execPath, [entry, ...args], { cwd });
}
export function files(root) {
    if (!existsSync(root))
        throw new ToolError(`required directory not found: ${root}`);
    return readdirSync(root, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name)).flatMap(entry => entry.isDirectory() ? files(join(root, entry.name)) : entry.isFile() ? [join(root, entry.name)] : []);
}
export function repoRoot() { return resolve(run('git', ['rev-parse', '--show-toplevel'], { capture: true }).trim()); }
export function text(path) { return readFileSync(path, 'utf8'); }
export function assert(condition, message, code = 1) { if (!condition)
    throw new ToolError(message, code); }
export async function main(fn) {
    try {
        await fn();
    }
    catch (error) {
        console.error(error instanceof Error ? error.message : error);
        process.exitCode = error instanceof ToolError ? error.code : 1;
    }
}
