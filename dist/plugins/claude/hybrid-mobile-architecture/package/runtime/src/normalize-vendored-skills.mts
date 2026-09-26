// TJ-ARCH-MOB-001 compliant
import { readFile, readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { packageRoot, main, atomicWrite, object } from './portable/platform.mjs';
await main(async () => {
  const args = process.argv.slice(2);
  if (args.some(arg => arg !== '--check')) throw new Error('Usage: node scripts/normalize-vendored-skills.mjs [--check]');
  const source = await readFile(join(packageRoot, 'scripts/check-skill-contracts.mjs'), 'utf8');
  const literal = source.match(/export const INTERNAL_SKILL_HARNESSES = (\{[\s\S]*?\n\});/);
  if (!literal) throw new Error('Missing INTERNAL_SKILL_HARNESSES contract');
  const table = object(JSON.parse(literal[1].replace(/,(\s*[}\]])/g, '$1')));
  let changed = 0, scanned = 0;
  for (const [harness, prefixes] of Object.entries(table)) {
    const directory = join(packageRoot, harness, 'skills');
    const entries = await readdir(directory, { withFileTypes: true });
    for (const [prefix, count] of Object.entries(object(prefixes))) {
      const names = entries.filter(entry => entry.isDirectory() && entry.name.startsWith(prefix));
      if (names.length !== count) throw new Error(`${harness}: expected ${count} ${prefix} skills, found ${names.length}`);
      for (const name of names) {
        const file = join(directory, name.name, 'SKILL.md'), text = (await readFile(file, 'utf8')).replaceAll('\r\n', '\n');
        const fm = text.match(/^---\n([\s\S]*?)\n---(?:\n|$)/); if (!fm) throw new Error(`Missing frontmatter: ${file}`);
        let head = `${fm[1]}\n`.replace(/^  internal:.*\n/gm, '');
        if (/^metadata:\n/m.test(head)) head = head.replace(/^metadata:\n((?:  .+\n)*)/m, '$&  internal: true\n');
        else head += 'metadata:\n  internal: true\n';
        const next = `---\n${head}---\n${text.slice(fm[0].length)}`;
        scanned++; if (next === text) continue; changed++;
        if (!args.includes('--check')) await atomicWrite(file, next);
      }
    }
  }
  if (args.includes('--check') && changed) throw new Error(`${changed}/${scanned} vendored mirrors need normalization`);
  process.stdout.write(`Normalized ${changed}/${scanned} vendored mirrors.\n`);
});
