// TJ-ARCH-MOB-001 compliant
import { cp, rm, mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { json, main, packageRoot } from './portable/platform.mjs';
import { tree, equal, harnesses } from './portable/skills.mjs';
await main(async () => {
  const args = process.argv.slice(2);
  if (args.some(arg => arg !== '--check')) throw new Error('Usage: node scripts/sync-harness-skills.mjs [--check]');
  const skills = json(join(packageRoot, 'builder.manifest.json')).skills;
  if (!Array.isArray(skills) || !skills.length) throw new Error('Missing canonical skills');
  const drift: string[] = [];
  for (const targetRoot of ['templates/project-skills', ...harnesses.map(h => `${h}/skills`)]) for (const name of skills) {
    if (typeof name !== 'string' || !/^[a-z0-9-]+$/.test(name)) throw new Error('Unsafe canonical skill name');
    const source = join(packageRoot, 'skills', name), target = join(packageRoot, targetRoot, name);
    const content = await tree(source); if (!content.size) throw new Error(`Missing canonical skill: ${name}`);
    if (equal(content, await tree(target))) continue;
    drift.push(`${targetRoot}/${name}`);
    if (!args.includes('--check')) { await rm(target, { recursive: true, force: true }); await mkdir(target, { recursive: true }); await cp(source, target, { recursive: true }); }
  }
  if (args.includes('--check') && drift.length) throw new Error(`Skill mirror drift:\n${drift.join('\n')}`);
  process.stdout.write(`Skill mirrors ${args.includes('--check') ? 'verified' : 'synchronized'} (${drift.length} differences).\n`);
});
