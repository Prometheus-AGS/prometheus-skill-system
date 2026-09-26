// TJ-ARCH-MOB-001 compliant
import { readFileSync, existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
// Self-contained so copied hooks have no dependency on the source checkout.
export function activate(payload, manifest) {
    if (!payload || typeof payload !== 'object' || !manifest || typeof manifest !== 'object')
        return;
    const prompt = String(payload.prompt ?? '').toLowerCase();
    const skills = manifest.skills;
    if (!Array.isArray(skills))
        return;
    const hits = skills.filter((skill) => skill && typeof skill.name === 'string' && Array.isArray(skill.terms) &&
        skill.terms.some((term) => typeof term === 'string' && prompt.includes(term.toLowerCase()))).map(skill => skill.name);
    if (!hits.length)
        return;
    return { hookSpecificOutput: { hookEventName: 'UserPromptSubmit', additionalContext: ['KnowMe Builder skills relevant to this prompt; invoke only those whose contract actually applies:', ...hits.map(name => `  - ${name}`)].join('\n') } };
}
try {
    if (!process.stdin.isTTY) {
        const payload = JSON.parse(readFileSync(0, 'utf8'));
        const candidates = [process.env.KNOWME_BUILDER_ACTIVATION_MANIFEST, resolve('.knowme-builder/activation-manifest.json')];
        let directory = dirname(fileURLToPath(import.meta.url));
        for (;;) {
            candidates.push(join(directory, 'templates/activation-manifest.json'));
            const parent = dirname(directory);
            if (parent === directory)
                break;
            directory = parent;
        }
        const file = candidates.find((value) => Boolean(value && existsSync(value)));
        if (file) {
            const output = activate(payload, JSON.parse(readFileSync(file, 'utf8')));
            if (output)
                process.stdout.write(`${JSON.stringify(output)}\n`);
        }
    }
}
catch { /* Advisory hooks never block the user's prompt. */ }
