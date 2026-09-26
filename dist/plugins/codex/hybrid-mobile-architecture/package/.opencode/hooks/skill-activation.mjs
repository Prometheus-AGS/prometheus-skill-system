// TJ-ARCH-MOB-001 compliant
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
try {
    if (!process.stdin.isTTY) {
        const payload = JSON.parse(readFileSync(0, 'utf8'));
        const manifest = JSON.parse(readFileSync(process.env.KNOWME_BUILDER_ACTIVATION_MANIFEST ?? resolve('.knowme-builder/activation-manifest.json'), 'utf8'));
        const prompt = String(payload.prompt ?? payload.message ?? '').toLowerCase();
        const skills = (manifest.skills ?? []).filter(skill => typeof skill?.name === 'string' && Array.isArray(skill.terms) && skill.terms.some(term => typeof term === 'string' && prompt.includes(term.toLowerCase()))).map(skill => skill.name);
        if (skills.length)
            process.stdout.write(`${JSON.stringify({ additionalContext: 'Relevant KnowMe Builder skills (advisory): ' + skills.join(', ') })}\n`);
    }
}
catch { /* Advisory activation is silent for unavailable input or metadata. */ }
