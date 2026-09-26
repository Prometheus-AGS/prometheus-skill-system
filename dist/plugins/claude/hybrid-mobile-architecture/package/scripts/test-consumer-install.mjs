// TJ-ARCH-MOB-001 compliant
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { existsSync, realpathSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, relative } from 'node:path';
import { json, main, object, packageRoot, run } from './portable/platform.mjs';
const required = ['connected-skill-packages', 'claude-hooks-reliability', 'realtime-skill-refiner', 'tauri-tray-app', 'launchagent-supervisor', 'auto-skill-package-integration'];
await main(async () => {
    const args = process.argv.slice(2);
    let ref = 'HEAD', keep = false;
    for (let index = 0; index < args.length; index++) {
        if (args[index] === '--keep')
            keep = true;
        else if (args[index] === '--ref' && args[index + 1])
            ref = args[++index];
        else
            throw new Error('Usage: node scripts/test-consumer-install.mjs [--ref ref] [--keep]');
    }
    const work = await mkdtemp(join(tmpdir(), 'builder-consumer-')), clone = join(work, 'package');
    try {
        const reuse = process.env.CONSUMER_INSTALL_REUSE_CLONE;
        if (reuse) {
            if (realpathSync(reuse) === realpathSync(packageRoot))
                throw new Error('Reuse clone cannot be the source checkout');
            run('git', ['-C', reuse, 'rev-parse', '--git-dir'], { capture: true });
            if (run('git', ['-C', reuse, 'status', '--porcelain'], { capture: true }).stdout.trim())
                throw new Error('Reuse clone must be pristine');
            run('git', ['clone', '--quiet', '--no-local', reuse, clone]);
        }
        else
            run('git', ['clone', '--quiet', '--no-local', packageRoot, clone]);
        run('git', ['-C', clone, 'checkout', '--quiet', ref]);
        const relationship = relative(realpathSync(packageRoot), realpathSync(clone));
        if (!relationship || !relationship.startsWith('..'))
            throw new Error('Clone is not isolated from source checkout');
        const fixture = process.env.CONSUMER_INSTALL_FIXTURE;
        if (fixture === 'undeclared-skill') {
            await mkdir(join(clone, 'skills/rogue-undeclared-skill'));
            await writeFile(join(clone, 'skills/rogue-undeclared-skill/SKILL.md'), '---\nname: rogue-undeclared-skill\ndescription: Undeclared\n---\n');
        }
        else if (fixture === 'missing-plugin-json')
            await rm(join(clone, 'plugin.json'));
        else if (fixture === 'unresolvable-skill')
            await rm(join(clone, 'skills/tauri-tray-app'), { recursive: true });
        else if (fixture === 'frontmatter-name-mismatch') {
            const file = join(clone, 'skills/tauri-tray-app/SKILL.md');
            await writeFile(file, (await readFile(file, 'utf8')).replace(/^name: tauri-tray-app$/m, 'name: wrong-name'));
        }
        else if (fixture === 'missing-harness-mirror')
            await rm(join(clone, '.claude/skills/realtime-skill-refiner'), { recursive: true });
        else if (fixture)
            throw new Error(`Unknown consumer fixture: ${fixture}`);
        const verifier = join(clone, 'scripts/verify-skill-manifest.mjs');
        if (!existsSync(verifier))
            throw new Error('Committed package lacks portable manifest verifier; source working-tree tests are not release proof');
        // No skip-contract success: every reported pass executes the shipped verifier.
        run(process.execPath, [verifier, clone]);
        const registry = json(join(clone, 'builder.manifest.json')), distribution = object(registry.distribution), skills = Array.isArray(registry.skills) ? registry.skills.map(String) : [];
        for (const name of required)
            if (!skills.includes(name))
                throw new Error(`Required skill is not registered: ${name}`);
        for (const harness of ['.claude', '.agents'])
            for (const name of skills)
                if (!existsSync(join(clone, harness, 'skills', name, 'SKILL.md')))
                    throw new Error(`Missing consumer mirror: ${harness}/${name}`);
        // Exercise real copied project installation from the committed payload.
        run(process.execPath, [join(clone, 'scripts/add-project-skills.mjs'), join(work, 'application')]);
        process.stdout.write(`Committed consumer install verified (${String(distribution.packageSkill)}, ${skills.length} skills).\n`);
    }
    finally {
        if (keep)
            process.stdout.write(`Consumer clone retained: ${work}\n`);
        else
            await rm(work, { recursive: true, force: true });
    }
});
