// TJ-ARCH-MOB-001 compliant
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { isAbsolute, join, relative, resolve, win32 } from 'node:path';
import { json, object } from './platform.mjs';
export function verifyManifest(root) {
    const errors = [];
    const read = (path) => { try {
        return json(join(root, path));
    }
    catch {
        errors.push(`Invalid or missing manifest: ${path}`);
        return {};
    } };
    const market = read('.claude-plugin/marketplace.json'), plugin = read('plugin.json'), registry = read('builder.manifest.json');
    for (const [name, value] of [['marketplace', market], ['plugin', plugin]])
        for (const field of ['name', 'version'])
            if (typeof value[field] !== 'string' || !value[field])
                errors.push(`${name}: missing ${field}`);
    if (!Array.isArray(market.plugins) || !market.plugins.length)
        errors.push('Marketplace declares no plugins');
    if (market.version !== plugin.version)
        errors.push('Marketplace/plugin version disagreement');
    if (existsSync(join(root, 'marketplace.json'))) {
        const descriptor = read('marketplace.json'), version = object(descriptor.skill).version ?? descriptor.version;
        if (version && version !== market.version)
            errors.push('Root marketplace version disagreement');
    }
    const distribution = object(registry.distribution), skillRoot = String(distribution.skillSourceRoot ?? 'skills');
    const sourceRoot = resolve(root, skillRoot), rel = relative(root, sourceRoot);
    if (isAbsolute(skillRoot) || win32.isAbsolute(skillRoot) || rel.startsWith('..'))
        errors.push('Skill root escapes package');
    else {
        const names = [distribution.packageSkill, ...(Array.isArray(registry.skills) ? registry.skills : [])];
        for (const name of names) {
            if (typeof name !== 'string' || !/^[a-z0-9-]+$/.test(name)) {
                errors.push(`Unsafe skill name: ${String(name)}`);
                continue;
            }
            const file = join(sourceRoot, name, 'SKILL.md');
            if (!existsSync(file)) {
                errors.push(`Declared skill missing: ${name}`);
                continue;
            }
            const frontmatter = readFileSync(file, 'utf8').match(/^---\r?\n([\s\S]*?)\r?\n---/);
            if (frontmatter?.[1].match(/^name:\s*(.+)$/m)?.[1].trim() !== name)
                errors.push(`Frontmatter name disagrees: ${name}`);
        }
        if (existsSync(sourceRoot))
            for (const entry of readdirSync(sourceRoot, { withFileTypes: true }))
                if (entry.isDirectory() && !names.includes(entry.name))
                    errors.push(`Undeclared skill: ${entry.name}`);
    }
    for (const file of ['.claude-plugin/marketplace.json', 'marketplace.json', 'plugin.json', 'builder.manifest.json'])
        if (existsSync(join(root, file))) {
            if (/"(?:\/(?:Users|home|opt|usr|var)\/|~\/|[A-Za-z]:\\\\|\\\\\\\\)/.test(readFileSync(join(root, file), 'utf8')))
                errors.push(`Absolute install path in ${file}`);
        }
    return errors;
}
