// TJ-ARCH-MOB-001 compliant
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { mkdir, rename, writeFile, rm } from 'node:fs/promises';
import { dirname, delimiter, join, resolve, extname } from 'node:path';
import { spawnSync } from 'node:child_process';
import { randomUUID } from 'node:crypto';
import { fileURLToPath } from 'node:url';
export class ProcessFailure extends Error {
    exitCode;
    constructor(message, exitCode) {
        super(message);
        this.exitCode = exitCode;
    }
}
export function object(value) {
    if (!value || typeof value !== 'object' || Array.isArray(value))
        return {};
    return value;
}
export const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
export function json(file) { return object(JSON.parse(readFileSync(file, 'utf8'))); }
export async function atomicWrite(file, content) {
    await mkdir(dirname(file), { recursive: true });
    const temporary = `${file}.${randomUUID()}.tmp`;
    try {
        await writeFile(temporary, content, { mode: 0o600 });
        for (let attempt = 0;; attempt++) {
            try {
                await rename(temporary, file);
                break;
            }
            catch (error) {
                if (attempt >= 4 || !['EPERM', 'EBUSY'].includes(error.code ?? ''))
                    throw error;
                await new Promise(r => setTimeout(r, 25 * (attempt + 1)));
            }
        }
    }
    finally {
        await rm(temporary, { force: true });
    }
}
export async function writeJson(file, data) { await atomicWrite(file, `${JSON.stringify(data, null, 2)}\n`); }
export function commandPath(name) {
    const directories = name.includes('/') || name.includes('\\') ? [''] : (process.env.PATH ?? '').split(delimiter);
    const extensions = process.platform === 'win32' && !extname(name) ? ['.exe', '.com', '.cmd', '.bat', ''] : [''];
    for (const directory of directories)
        for (const suffix of extensions) {
            const candidate = directory ? join(directory, name + suffix) : resolve(name + suffix);
            if (existsSync(candidate) && statSync(candidate).isFile())
                return candidate;
        }
    return undefined;
}
// npm-generated batch wrappers cannot be executed with shell:false on Windows.
// Resolve their package's declared JS bin; never parse/run arbitrary batch text.
export function invocation(name, args) {
    const executable = commandPath(name);
    if (!executable)
        throw new Error(`Required executable not found: ${name}`);
    if (!/\.(cmd|bat)$/i.test(executable))
        return { command: executable, args };
    // Flutter's vendor batch wrappers ultimately invoke the cached native Dart
    // executable and Flutter snapshot. Use that native boundary directly once
    // the SDK has been bootstrapped; never interpret user text as batch syntax.
    if (/(?:^|[/\\])(flutter|dart)\.bat$/i.test(executable)) {
        const dart = join(dirname(executable), 'cache/dart-sdk/bin/dart.exe');
        const snapshot = join(dirname(executable), 'cache/flutter_tools.snapshot');
        if (existsSync(dart) && (/dart\.bat$/i.test(executable) || existsSync(snapshot)))
            return { command: dart, args: /flutter\.bat$/i.test(executable) ? [snapshot, ...args] : args };
        throw new Error('Flutter SDK cache is not bootstrapped; use the vendor SDK setup before portable invocation.');
    }
    const roots = [join(dirname(executable), 'node_modules'), resolve(dirname(executable), '..')];
    const binName = name.replace(/.*[/\\]/, '').replace(/\.(cmd|bat)$/i, '');
    for (const root of roots) {
        if (!existsSync(root))
            continue;
        const packages = readdirSync(root).flatMap(entry => entry.startsWith('@') && statSync(join(root, entry)).isDirectory()
            ? readdirSync(join(root, entry)).map(child => join(root, entry, child)) : [join(root, entry)]);
        for (const directory of packages) {
            const manifest = join(directory, 'package.json');
            if (!existsSync(manifest))
                continue;
            const pkg = json(manifest);
            const bin = typeof pkg.bin === 'string' && String(pkg.name).split('/').at(-1) === binName ? pkg.bin : object(pkg.bin)[binName];
            if (typeof bin !== 'string')
                continue;
            const entry = resolve(directory, bin);
            if (!existsSync(entry))
                continue;
            const head = readFileSync(entry, 'utf8').slice(0, 120);
            if (/\.(?:mjs|cjs|js)$/.test(entry) || /#![^\n]*\bnode\b/.test(head))
                return { command: process.execPath, args: [entry, ...args] };
        }
    }
    throw new Error(`Cannot launch batch-only tool ${name} without a shell. Install its native executable or Node package entrypoint.`);
}
export function run(name, args, options = {}) {
    const command = invocation(name, args);
    const result = spawnSync(command.command, command.args, {
        cwd: options.cwd, env: options.env, encoding: 'utf8', shell: false,
        stdio: options.capture ? ['ignore', 'pipe', 'pipe'] : 'inherit',
    });
    if (result.error)
        throw result.error;
    const status = result.status ?? 1;
    if (status && !options.allowFailure)
        throw new ProcessFailure(`${name} exited ${status}${result.stderr ? `: ${result.stderr.trim()}` : ''}`, status);
    return { status, stdout: result.stdout ?? '', stderr: result.stderr ?? '' };
}
export async function main(action) {
    try {
        await action();
    }
    catch (error) {
        process.stderr.write(`${error.message}\n`);
        process.exitCode = error instanceof ProcessFailure ? error.exitCode : 1;
    }
}
