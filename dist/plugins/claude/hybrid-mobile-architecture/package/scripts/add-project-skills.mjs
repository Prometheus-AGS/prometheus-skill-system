// TJ-ARCH-MOB-001 compliant
import { main } from './portable/platform.mjs';
import { installProjectSkills } from './portable/skills.mjs';
await main(async () => {
    if (process.argv.length > 3 || process.argv[2]?.startsWith('--'))
        throw new Error('Usage: node scripts/add-project-skills.mjs [project-root]');
    await installProjectSkills(process.argv[2] ?? '.');
});
