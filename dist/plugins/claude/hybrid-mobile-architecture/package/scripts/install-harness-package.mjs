// TJ-ARCH-MOB-001 compliant
import { existsSync, readFileSync } from 'node:fs';
import { cp, mkdir, readdir, rm, mkdtemp } from 'node:fs/promises';
import { homedir, tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { json, object, main, packageRoot, writeJson, run, commandPath } from './portable/platform.mjs';
const hash = (file) => createHash('sha256').update(readFileSync(file)).digest('hex');
await main(async () => {
    const args = process.argv.slice(2);
    const values = { harness: 'all', scope: 'user', source: packageRoot, ref: '' };
    const flags = new Set();
    for (let index = 0; index < args.length; index++) {
        const key = args[index].replace(/^--/, '');
        if (['harness', 'scope', 'source', 'ref'].includes(key)) {
            if (!args[index + 1] || args[index + 1].startsWith('--'))
                throw new Error(`Missing value: --${key}`);
            values[key] = args[++index];
        }
        else if (['check', 'uninstall', 'with-cli', 'with-mcp', 'with-prometheus', 'help'].includes(key))
            flags.add(key);
        else
            throw new Error(`Unknown option: ${args[index]}`);
    }
    const harnesses = ['claude-code', 'codex', 'opencode', 'kimi-code', 'minimax-code', 'zed'];
    if (flags.has('help')) {
        process.stdout.write(`node scripts/install-harness-package.mjs [--harness ${harnesses.join('|')}|all] [--scope user|project] [--source git-url-or-path] [--ref ref] [--check] [--uninstall] [--with-cli] [--with-mcp]\n`);
        return;
    }
    if (!['all', ...harnesses].includes(values.harness) || !['user', 'project'].includes(values.scope))
        throw new Error('Invalid harness or scope');
    if (flags.has('with-prometheus'))
        throw new Error('--with-prometheus is not portable yet. Install the selected full or mini control plane separately; no shell bootstrap is invoked.');
    const check = flags.has('check'), project = values.scope === 'project';
    const selected = values.harness === 'all' ? harnesses : [values.harness];
    if (values.ref && selected.includes('claude-code'))
        throw new Error('--ref is not supported by the Claude marketplace adapter; choose a pinned source URL or omit Claude.');
    const manifest = json(join(packageRoot, 'builder.manifest.json')), pkg = object(manifest.package);
    const packageId = String(pkg.id), marketplace = 'knowme-builder', identity = `${packageId}@${marketplace}`;
    const stateRoot = project ? resolve('.knowme-builder') : join(process.env.XDG_STATE_HOME ?? (process.platform === 'win32' ? process.env.LOCALAPPDATA ?? join(homedir(), 'AppData/Local') : join(homedir(), '.local/state')), 'knowme-builder');
    const receiptPath = join(stateRoot, project ? 'harness-install.json' : 'install.json');
    const old = existsSync(receiptPath) ? json(receiptPath) : {};
    if (old.package && old.package !== packageId)
        throw new Error('Install receipt belongs to another package');
    const receipt = { ...old, schemaVersion: 2, package: packageId, version: pkg.version, source: values.source, ref: values.ref, scope: values.scope,
        harnesses: [...new Set([...(Array.isArray(old.harnesses) ? old.harnesses : []), ...selected])],
        marketplacesAdded: object(old.marketplacesAdded), pluginsAdded: object(old.pluginsAdded), skillsAdded: object(old.skillsAdded), mcpAdded: object(old.mcpAdded), ownedFiles: object(old.ownedFiles) };
    const save = async () => { if (!check)
        await writeJson(receiptPath, receipt); };
    const execute = (command, argv) => { if (check)
        process.stdout.write(`${JSON.stringify([command, ...argv])}\n`);
    else
        run(command, argv); };
    const probe = (command, argv) => commandPath(command) ? run(command, argv, { capture: true, allowFailure: true }) : { status: 127, stdout: '' };
    const parse = (value) => { try {
        return JSON.parse(value);
    }
    catch {
        return undefined;
    } };
    const ownedFiles = object(receipt.ownedFiles);
    const copyOwned = async (source, target) => {
        const expected = hash(source), prior = ownedFiles[target];
        if (existsSync(target) && hash(target) !== expected && (!prior || hash(target) !== prior))
            throw new Error(`User file differs; refusing overwrite: ${target}`);
        if (check) {
            process.stdout.write(`copy ${source} -> ${target}\n`);
            return;
        }
        const shouldOwn = !existsSync(target) || Boolean(prior);
        await mkdir(dirname(target), { recursive: true });
        await cp(source, target);
        if (shouldOwn)
            ownedFiles[target] = expected;
        await save();
    };
    const opencodeRoot = project ? resolve('.opencode') : join(process.env.XDG_CONFIG_HOME ?? join(homedir(), '.config'), 'opencode');
    const opencodeConfig = join(opencodeRoot, 'opencode.json');
    const mcp = object(receipt.mcpAdded);
    const mcpMark = (harness) => { mcp[harness] = object(mcp[harness]); return mcp[harness]; };
    const publicSkills = [packageId, ...(Array.isArray(manifest.skills) ? manifest.skills.map(String) : [])];
    if (flags.has('uninstall')) {
        if (!old.package)
            throw new Error(`No installation receipt: ${receiptPath}`);
        // Receipts v1 did not track copied-file hashes. Do not guess ownership.
        if (old.schemaVersion !== 2)
            throw new Error('Legacy installation receipt requires migration before portable uninstall; no files removed.');
        for (const [file, digest] of Object.entries(ownedFiles))
            if (existsSync(file) && hash(file) !== digest)
                throw new Error(`Installed file was edited; preserving it: ${file}`);
        // Preflight external MCP state before removing any plugin or file.
        for (const key of ['claude', 'codex'])
            for (const name of ['dart', 'shadcn'])
                if (object(mcp[key])[name]) {
                    const current = probe(key, ['mcp', 'get', name]);
                    const saved = object(object(old.mcpSnapshots)[key])[name];
                    if (typeof saved !== 'string' || current.status !== 0 || current.stdout !== saved)
                        throw new Error('MCP entry changed or cannot be verified; preserving ' + key + '/' + name);
                }
        if (existsSync(opencodeConfig)) {
            const current = object(json(opencodeConfig).mcp);
            for (const [name, value] of Object.entries(object(old.opencodeMcpEntries)))
                if (JSON.stringify(current[name]) !== JSON.stringify(value))
                    throw new Error('MCP configuration was edited; preserving ' + name);
        }
        const marketplaces = object(receipt.marketplacesAdded), plugins = object(receipt.pluginsAdded), skills = object(receipt.skillsAdded);
        for (const harness of Array.isArray(old.harnesses) ? old.harnesses.map(String) : []) {
            if (harness === 'claude-code') {
                if (plugins.claude)
                    execute('claude', ['plugin', 'uninstall', identity, '--scope', values.scope]);
                if (marketplaces.claude)
                    execute('claude', ['plugin', 'marketplace', 'remove', marketplace]);
            }
            if (harness === 'codex' && !project) {
                if (plugins.codex)
                    execute('codex', ['plugin', 'remove', identity]);
                if (marketplaces.codex)
                    execute('codex', ['plugin', 'marketplace', 'remove', marketplace]);
            }
            // Skill files are removed only by their per-file receipts below.
            const key = harness === 'claude-code' ? 'claude' : harness;
            if (['claude', 'codex'].includes(key))
                for (const name of ['dart', 'shadcn'])
                    if (object(mcp[key])[name])
                        execute(key, ['mcp', 'remove', ...(key === 'claude' ? ['--scope', values.scope] : []), name]);
        }
        if (existsSync(opencodeConfig)) {
            const config = json(opencodeConfig), entries = object(config.mcp), expected = object(old.opencodeMcpEntries);
            for (const [name, value] of Object.entries(expected)) {
                if (JSON.stringify(entries[name]) !== JSON.stringify(value))
                    throw new Error(`MCP configuration was edited; preserving ${name}`);
                delete entries[name];
            }
            if (!check && Object.keys(expected).length)
                await writeJson(opencodeConfig, { ...config, mcp: entries });
        }
        for (const file of Object.keys(ownedFiles)) {
            if (check)
                process.stdout.write(`remove owned file ${file}\n`);
            else
                await rm(file, { force: true });
        }
        if (!check)
            await rm(receiptPath, { force: true });
        process.stdout.write(check ? 'Uninstall check complete; no changes.\n' : 'Receipt-owned installation removed.\n');
        return;
    }
    let portableRoot = packageRoot, clonedSource;
    const copyHarness = (harness) => ['opencode', 'kimi-code', 'minimax-code', 'zed'].includes(harness)
        || harness === 'codex' && project;
    if (selected.some(copyHarness) && resolve(values.source) !== packageRoot) {
        if (existsSync(values.source))
            portableRoot = resolve(values.source);
        else if (check)
            process.stdout.write('Remote source content validation will occur on install: ' + values.source + '\n');
        else {
            clonedSource = await mkdtemp(join(tmpdir(), 'builder-source-'));
            portableRoot = join(clonedSource, 'package');
            const source = /^[^/:]+\/[^/]+$/.test(values.source) ? 'https://github.com/' + values.source + '.git' : values.source;
            try {
                run('git', ['clone', '--quiet', '--no-local', source, portableRoot]);
                if (values.ref)
                    run('git', ['-C', portableRoot, 'checkout', '--quiet', values.ref]);
            }
            catch (error) {
                await rm(clonedSource, { recursive: true, force: true });
                throw error;
            }
        }
    }
    else if (values.ref && selected.some(copyHarness))
        throw new Error('--ref requires a remote source for portable skill copies; local working trees are not silently checked out.');
    if (!clonedSource && values.ref && portableRoot !== packageRoot && existsSync(values.source))
        throw new Error('--ref cannot mutate a local source checkout; supply a remote source.');
    try {
        const sourceManifest = json(join(portableRoot, 'builder.manifest.json'));
        if (object(sourceManifest.package).id !== packageId || object(sourceManifest.package).version !== pkg.version)
            throw new Error('Source package identity/version differs; use that release own installer.');
        const sourceSkills = [String(object(sourceManifest.distribution).packageSkill), ...(Array.isArray(sourceManifest.skills) ? sourceManifest.skills.map(String) : [])];
        const skillSourceRoot = String(object(sourceManifest.distribution).skillSourceRoot ?? 'skills');
        if (skillSourceRoot !== 'skills')
            throw new Error('Unsupported external skill source root');
        const walk = async (directory) => {
            const files = [];
            for (const entry of await readdir(directory, { withFileTypes: true })) {
                if (entry.isSymbolicLink())
                    throw new Error('Cannot install symlink: ' + join(directory, entry.name));
                if (entry.isDirectory())
                    for (const child of await walk(join(directory, entry.name)))
                        files.push(join(entry.name, child));
                else
                    files.push(entry.name);
            }
            return files;
        };
        const skillCopies = [];
        const minimaxData = process.env.MINIMAX_DATA_DIR ?? process.env.MAVIS_DATA_DIR ?? join(homedir(), '.minimax');
        for (const harness of selected)
            if (copyHarness(harness)) {
                const genericRoot = join(project ? process.cwd() : homedir(), '.agents/skills');
                const roots = harness === 'kimi-code'
                    ? [genericRoot, join(project ? process.cwd() : homedir(), '.kimi-code/skills')]
                    : project ? [genericRoot] : harness === 'opencode'
                        ? [genericRoot, join(opencodeRoot, 'skills')]
                        : harness === 'minimax-code'
                            ? [join(minimaxData, 'skills')]
                            : [genericRoot];
                for (const name of sourceSkills) {
                    if (!/^[a-z0-9-]+$/.test(name))
                        throw new Error('Unsafe skill name in source package');
                    const source = join(portableRoot, skillSourceRoot, name);
                    for (const file of await walk(source))
                        for (const targetRoot of roots) {
                            const target = join(targetRoot, name, file);
                            if (!skillCopies.some(([, existing]) => existing === target))
                                skillCopies.push([join(source, file), target]);
                        }
                }
            }
        // A differing preexisting skill aborts the whole plan before any mutation.
        for (const [source, target] of skillCopies)
            if (existsSync(target) && hash(target) !== hash(source) && hash(target) !== ownedFiles[target])
                throw new Error('User skill differs; refusing overwrite: ' + target);
        for (const harness of selected) {
            const key = harness === 'claude-code' ? 'claude' : harness;
            if (key === 'claude' || key === 'codex' && !project) {
                if (!commandPath(key) && !check)
                    throw new Error(`Required executable not found: ${key}`);
                const list = probe(key, ['plugin', 'marketplace', 'list', '--json']);
                const listing = parse(list.stdout);
                const marketRows = key === 'claude' ? listing : object(listing).marketplaces;
                const pluginResult = probe(key, ['plugin', 'list', '--json']);
                const installed = parse(pluginResult.stdout);
                const rows = key === 'claude' ? installed : object(installed).installed;
                if (!check && (list.status !== 0 || !Array.isArray(marketRows) || pluginResult.status !== 0 || !Array.isArray(rows)))
                    throw new Error('Cannot establish marketplace/plugin ownership for ' + key + '; inventory failed or returned invalid JSON.');
                const present = Array.isArray(marketRows) && marketRows.some(entry => object(entry).name === marketplace);
                if (present && values.ref)
                    throw new Error('Cannot enforce --ref on an existing marketplace; register the requested revision explicitly before installation.');
                if (present && args.includes('--source')) {
                    const existing = object(marketRows.find(entry => object(entry).name === marketplace));
                    const origin = typeof existing.repo === 'string' ? existing.repo : typeof existing.source === 'string' ? existing.source : object(existing.source).url ?? object(existing.source).repo;
                    if (origin !== values.source)
                        throw new Error('Cannot verify requested --source against existing marketplace origin; preserving its registration.');
                }
                const hadPlugin = Array.isArray(rows) && rows.some(row => key === 'claude' ? row.id === identity && row.scope === values.scope : row.pluginId === identity);
                if (present)
                    execute(key, ['plugin', 'marketplace', key === 'claude' ? 'update' : 'upgrade', marketplace]);
                else {
                    execute(key, ['plugin', 'marketplace', 'add', values.source, ...(key === 'claude' ? ['--scope', values.scope] : values.ref ? ['--ref', values.ref] : [])]);
                    object(receipt.marketplacesAdded)[key] = true;
                    await save();
                }
                execute(key, ['plugin', key === 'claude' ? 'install' : 'add', identity, ...(key === 'claude' ? ['--scope', values.scope] : [])]);
                if (!hadPlugin)
                    object(receipt.pluginsAdded)[key] = true;
                await save();
            }
            else {
                // Skill files are staged from the verified source and owned individually.
                if (harness === 'codex')
                    for (const name of await readdir(join(packageRoot, '.codex/prompts')))
                        if (name.startsWith('knowme-builder-') && name.endsWith('.md'))
                            await copyOwned(join(packageRoot, '.codex/prompts', name), resolve('.codex/prompts', name));
                if (harness === 'opencode') {
                    await copyOwned(join(packageRoot, '.opencode/plugins/knowme-builder.mjs'), join(opencodeRoot, 'plugins/knowme-builder.mjs'));
                    await copyOwned(join(packageRoot, 'templates/activation-manifest.json'), join(opencodeRoot, 'knowme-builder/activation-manifest.json'));
                    for (const name of await readdir(join(packageRoot, '.opencode/commands')))
                        if (name.startsWith('knowme-builder-') && name.endsWith('.md'))
                            await copyOwned(join(packageRoot, '.opencode/commands', name), join(opencodeRoot, 'commands', name));
                }
            }
            if (flags.has('with-mcp') && (key === 'claude' || key === 'codex' && !project))
                for (const name of ['dart', 'shadcn']) {
                    if (probe(key, ['mcp', 'get', name]).status === 0)
                        continue;
                    const command = name === 'dart' ? ['dart', 'mcp-server', '--force-roots-fallback'] : ['npx', 'shadcn@latest', 'mcp'];
                    execute(key, ['mcp', 'add', ...(key === 'claude' ? ['--scope', values.scope] : []), name, '--', ...command]);
                    mcpMark(key)[name] = true;
                    if (!check) {
                        const snapshots = object(receipt.mcpSnapshots), byHarness = object(snapshots[key]);
                        const captured = probe(key, ['mcp', 'get', name]);
                        if (captured.status === 0)
                            byHarness[name] = captured.stdout;
                        snapshots[key] = byHarness;
                        receipt.mcpSnapshots = snapshots;
                    }
                    await save();
                }
        }
        for (const [source, target] of skillCopies)
            await copyOwned(source, target);
        if (flags.has('with-mcp') && selected.includes('opencode')) {
            const config = existsSync(opencodeConfig) ? json(opencodeConfig) : {}, entries = object(config.mcp), owned = object(receipt.opencodeMcpEntries);
            for (const [name, command] of Object.entries({ 'dart-mcp-server': ['dart', 'mcp-server', '--force-roots-fallback'], shadcn: ['npx', 'shadcn@latest', 'mcp'] }))
                if (!entries[name]) {
                    entries[name] = { type: 'local', command, enabled: true };
                    owned[name] = entries[name];
                }
            if (!check)
                await writeJson(opencodeConfig, { ...config, mcp: entries });
            receipt.opencodeMcpEntries = owned;
            await save();
        }
        if (flags.has('with-cli')) {
            execute('cargo', ['build', '--release', '--locked', '--manifest-path', join(packageRoot, 'tools/knowme-builder/Cargo.toml'), '--target-dir', join(packageRoot, 'tools/knowme-builder/target')]);
            const name = `knowme-builder${process.platform === 'win32' ? '.exe' : ''}`;
            if (check)
                process.stdout.write(`install native ${name} to cargo bin\n`);
            else
                await copyOwned(join(packageRoot, 'tools/knowme-builder/target/release', name), join(process.env.CARGO_HOME ?? join(homedir(), '.cargo'), 'bin', name));
        }
        await save();
        process.stdout.write(check ? 'Check complete; no host state changed.\n' : `Installed KnowMe Builder ${pkg.version}; receipt: ${receiptPath}\n`);
    }
    finally {
        if (clonedSource)
            await rm(clonedSource, { recursive: true, force: true });
    }
});
