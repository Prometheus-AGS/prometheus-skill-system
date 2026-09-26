// TJ-ARCH-MOB-001 compliant
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { json, object, writeJson } from './platform.mjs';
export function settingsFile(root) {
    if (!existsSync(root))
        throw new Error(`Not a directory: ${root}`);
    return [join(root, '.claude/settings.json'), join(root, 'hooks/hooks.json')].find(existsSync);
}
export function inspect(doc) {
    const findings = [];
    for (const [event, rows] of Object.entries(object(doc.hooks ?? doc))) {
        if (!Array.isArray(rows)) {
            findings.push(`Invalid hook event array: ${event}`);
            continue;
        }
        for (const row of rows) {
            const entry = object(row), matcher = entry.matcher;
            if (matcher === undefined)
                findings.push(`W6.8: ${event} is missing a matcher`);
            if (event === 'SessionStart' && (matcher === '*' || matcher === undefined))
                findings.push(`W6.6: ${event} has an unbounded matcher`);
            if (event === 'SubagentStop' && typeof matcher === 'string' && matcher !== '*' && !(matcher.startsWith('^') && matcher.endsWith('$')))
                findings.push(`W6.3: ${event} matcher is not anchored`);
            for (const value of Array.isArray(entry.hooks) ? entry.hooks : []) {
                const hook = object(value), command = String(hook.command ?? '');
                if (/\b(?:ba)?sh -c/.test(command) && command.length > 120)
                    findings.push(`W6.1: ${event} has a long inline shell body`);
                if (command === 'node' && !Array.isArray(hook.args))
                    findings.push(`${event}: exec-form Node hook is missing args`);
            }
        }
    }
    return findings;
}
export async function normalize(file, dryRun) {
    const doc = json(file), container = object(doc.hooks ?? doc);
    for (const [event, rows] of Object.entries(container))
        if (Array.isArray(rows))
            for (const row of rows) {
                const entry = object(row);
                if (entry.matcher === undefined)
                    entry.matcher = event === 'SessionStart' ? 'claude-code' : '*';
                if (event === 'SessionStart' && entry.matcher === '*')
                    entry.matcher = 'claude-code';
                if (event === 'SubagentStop' && typeof entry.matcher === 'string' && entry.matcher !== '*' && !(entry.matcher.startsWith('^') && entry.matcher.endsWith('$')))
                    entry.matcher = `^(${entry.matcher.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})$`;
            }
    if (!dryRun)
        await writeJson(file, doc);
    for (const issue of inspect(doc))
        process.stderr.write(`Review: ${issue}\n`);
}
export function inspectFiles(root) {
    const issues = [], directory = join(root, '.claude/hooks');
    if (existsSync(directory))
        for (const name of readdirSync(directory)) {
            if (!/\.(?:sh|py|mjs)$/.test(name))
                continue;
            const source = readFileSync(join(directory, name), 'utf8');
            if (name.endsWith('.sh') && !source.includes('exec 2>'))
                issues.push(`W6.4: ${name} does not redirect stderr`);
            if (name.endsWith('.py') && /^[^#\n]*print\(\s*f?["']/m.test(source))
                issues.push(`W6.4: ${name} prints unstructured stdout`);
            if (name.endsWith('.mjs') && /\bconsole\.log\(/.test(source))
                issues.push(`W6.4: ${name} uses console.log; reserve stdout for protocol JSON`);
        }
    const runner = [join(root, 'shared/scripts/run-hook'), join(root, 'scripts/run-hook')].find(existsSync);
    if (runner) {
        const source = readFileSync(runner, 'utf8');
        if (/shasum|sha256sum/.test(source) && !/cache|CACHE/.test(source))
            issues.push('W6.2: dispatcher hashes have no cache window');
        if (!/setsid|setpgid/.test(source))
            issues.push('W6.5: runner lacks process-group cleanup');
        if (!source.includes('hooks.ndjson'))
            issues.push('W6.7: runner lacks structured invocation log');
    }
    return issues;
}
