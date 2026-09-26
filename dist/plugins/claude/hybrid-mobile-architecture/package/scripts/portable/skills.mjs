// TJ-ARCH-MOB-001 compliant
import { cp, readdir, readFile, mkdir } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { packageRoot, json, object, writeJson } from './platform.mjs';
export const harnesses = ['.claude', '.codex', '.opencode', '.kimi', '.agents', '.kimi-code'];
export async function tree(directory, prefix = '') {
    const files = new Map();
    if (!existsSync(directory))
        return files;
    for (const entry of await readdir(directory, { withFileTypes: true })) {
        if (entry.isSymbolicLink())
            throw new Error(`Symlinks are not portable payloads: ${join(directory, entry.name)}`);
        if (entry.isDirectory())
            for (const [key, value] of await tree(join(directory, entry.name), `${prefix}${entry.name}/`))
                files.set(key, value);
        else
            files.set(`${prefix}${entry.name}`, (await readFile(join(directory, entry.name))).toString('base64'));
    }
    return files;
}
export function equal(a, b) { return a.size === b.size && [...a].every(([key, value]) => b.get(key) === value); }
export async function installProjectSkills(destination) {
    const root = resolve(destination);
    const manifest = json(join(packageRoot, 'builder.manifest.json'));
    const skills = manifest.skills;
    if (!Array.isArray(skills) || !skills.length || skills.some(name => typeof name !== 'string' || !/^[a-z0-9-]+$/.test(name)))
        throw new Error('Invalid canonical skill list');
    const settingsPath = join(root, '.claude/settings.json');
    const settings = existsSync(settingsPath) ? json(settingsPath) : {};
    const additions = json(join(packageRoot, 'templates/project-skills/settings.hooks.json'));
    // Validate all existing config before copying any payload.
    const hooks = object(settings.hooks);
    for (const [event, entries] of Object.entries(object(additions.hooks))) {
        const current = hooks[event] ?? [];
        if (!Array.isArray(current) || !Array.isArray(entries))
            throw new Error(`Invalid hook entries: ${event}`);
        // Remove only our exact obsolete Python commands, retaining sibling user hooks.
        const preserved = current.map(entry => {
            const value = object(entry);
            if (!Array.isArray(value.hooks))
                return entry;
            return { ...value, hooks: value.hooks.filter(h => !['python3 .claude/hooks/skill-activation.py', 'python3 .claude/hooks/a11y-reminder.py'].includes(String(object(h).command))) };
        }).filter(entry => !Array.isArray(object(entry).hooks) || object(entry).hooks.length > 0);
        hooks[event] = [...new Map([...preserved, ...entries].map(entry => [JSON.stringify(entry), entry])).values()];
    }
    // Never replace an independently edited hook or activation manifest.
    const managedFiles = [
        ...['skill-activation', 'a11y-reminder'].map(hook => [join(packageRoot, 'scripts', hook + '.mjs'), join(root, '.claude/hooks', hook + '.mjs')]),
        [join(packageRoot, 'templates/activation-manifest.json'), join(root, '.knowme-builder/activation-manifest.json')],
    ];
    for (const [source, target] of managedFiles)
        if (existsSync(target) && !(await readFile(source)).equals(await readFile(target)))
            throw new Error('Existing hook metadata differs; refusing overwrite: ' + target);
    for (const harness of harnesses)
        for (const skill of skills) {
            const source = join(packageRoot, 'skills', skill);
            const target = join(root, harness, 'skills', skill);
            if (existsSync(target) && !equal(await tree(source), await tree(target)))
                throw new Error(`Existing skill differs; refusing overwrite: ${target}`);
        }
    for (const harness of harnesses)
        for (const skill of skills)
            await cp(join(packageRoot, 'skills', skill), join(root, harness, 'skills', skill), { recursive: true });
    await mkdir(join(root, '.claude/hooks'), { recursive: true });
    for (const hook of ['skill-activation', 'a11y-reminder'])
        await cp(join(packageRoot, 'scripts', `${hook}.mjs`), join(root, '.claude/hooks', `${hook}.mjs`));
    await mkdir(join(root, '.knowme-builder'), { recursive: true });
    await cp(join(packageRoot, 'templates/activation-manifest.json'), join(root, '.knowme-builder/activation-manifest.json'), { recursive: false });
    await writeJson(settingsPath, { ...settings, hooks });
    process.stdout.write(`Installed ${skills.length} skills for ${harnesses.length} harnesses with portable hooks.\n`);
}
