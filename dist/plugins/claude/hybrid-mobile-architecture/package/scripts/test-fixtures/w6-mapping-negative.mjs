// TJ-ARCH-MOB-001 compliant
import { cp, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { main, packageRoot } from '../portable/platform.mjs';
import { checkW6 } from '../portable/w6.mjs';
await main(async () => {
    if (checkW6(packageRoot).length)
        throw new Error('Positive W6 control failed; negative fixtures would be meaningless');
    const fixtures = [
        ['skills/claude-hooks-reliability/SKILL.md', s => s.replace('### W6.4 — Hook stdout', '### W6.X — Hook stdout').replace('### W6.5 — Hooks leak', '### W6.4 — Hooks leak').replace('### W6.X — Hook stdout', '### W6.5 — Hook stdout'), "should be the 'stdout' weakness"],
        ['docs/03-hooks-reliability.md', s => s.replace(/^### W6\.9 .*$/m, '### W6.9 — Something unrelated entirely'), "should be the 'interpreter|unfixable' weakness"],
        ['docs/05-hma-pmp-companion-architecture.md', s => s.replace(/^\| R6\.8 \|/m, '| W6.8 |'), 'must use R6.x'],
        ['docs/03-hooks-reliability.md', s => s.replace('**Fix (R6.7)', '**Fix (R6.6)'), 'R6.x cited by more than one weakness'],
        ['docs/03-hooks-reliability.md', s => s.replace('**Fix (R6.7)', '**Fix (R6.0)'), 'no such row'],
        ['docs/03-hooks-reliability.md', s => s.replace(/^### W6\.1 .*$/m, '### W6.1 — Inline `bash -c` is unfixable long-term'), 'W6.1 should be the'],
        ['docs/05-hma-pmp-companion-architecture.md', s => s.replace(/^\| R6\.4 \|.*$/m, '| R6.4 | Add structured NDJSON log |'), 'R6.4 should be the'],
    ];
    for (const [file, mutate, expected] of fixtures) {
        const work = await mkdtemp(join(tmpdir(), 'w6-negative-'));
        try {
            for (const path of ['docs/03-hooks-reliability.md', 'docs/05-hma-pmp-companion-architecture.md', 'skills/claude-hooks-reliability/SKILL.md'])
                await cp(join(packageRoot, path), join(work, path), { recursive: true });
            const target = join(work, file), before = await readFile(target, 'utf8'), after = mutate(before);
            if (before === after)
                throw new Error(`Fixture failed to mutate ${file}`);
            await writeFile(target, after);
            const errors = checkW6(work);
            if (!errors.some(error => error.includes(expected)))
                throw new Error(`Fixture failed to detect ${expected}: ${errors.join('; ')}`);
        }
        finally {
            await rm(work, { recursive: true, force: true });
        }
    }
    process.stdout.write('W6 mapping positive control and seven negative cases passed.\n');
});
