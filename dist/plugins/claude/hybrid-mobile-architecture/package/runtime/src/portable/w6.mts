// TJ-ARCH-MOB-001 compliant
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
export function checkW6(root: string): string[] {
  const specFile = 'docs/03-hooks-reliability.md', skillFile = 'skills/claude-hooks-reliability/SKILL.md', archFile = 'docs/05-hma-pmp-companion-architecture.md';
  const spec = readFileSync(join(root, specFile), 'utf8'), skill = readFileSync(join(root, skillFile), 'utf8'), arch = readFileSync(join(root, archFile), 'utf8');
  const topics = ['inline .bash -c[^|]*fragile', 'sha', 'subagent', 'stdout', 'leak', 'sessionstart', 'structured', 'userpromptsubmit', 'interpreter|unfixable'];
  const remedies = ['extracted scripts', 'compile .run-hook', 'cache', 'process-group', 'regex-anchored', 'exec 2', 'ndjson|structured', 'rust binary', 'tighten'];
  const errors: string[] = [];
  for (const [file, text] of [[specFile, spec], [skillFile, skill]]) {
    const headings = [...text.matchAll(/^### W6\.(\d+) (.*)$/gm)];
    if (headings.map(h => h[1]).join(',') !== '1,2,3,4,5,6,7,8,9') errors.push(`${file}: expected ordered distinct W6.1-W6.9 sections`);
    topics.forEach((topic, index) => {
      const heading = headings.find(h => Number(h[1]) === index + 1)?.[2] ?? '';
      if (!new RegExp(topic, 'i').test(heading)) errors.push(`${file}: W6.${index + 1} should be the '${topic}' weakness`);
      if (topics.filter(candidate => new RegExp(candidate, 'i').test(heading)).length > 1) errors.push(`${file}: ambiguous W6.${index + 1} topic`);
    });
  }
  if (/^\| W6\.\d+ \|/m.test(arch)) errors.push('docs/05 remedy table must use R6.x');
  const citations = [...spec.matchAll(/Fix \(R6\.(\d+)\)/g)].map(m => m[1]);
  if (new Set(citations).size !== citations.length) errors.push('R6.x cited by more than one weakness');
  for (const number of citations) if (!new RegExp(`^\\| R6\\.${number} \\|`, 'm').test(arch)) errors.push(`R6.${number}: no such row`);
  remedies.forEach((remedy, index) => {
    const row = arch.match(new RegExp(`^\\| R6\\.${index + 1} \\|.*$`, 'm'))?.[0] ?? '';
    if (!new RegExp(remedy, 'i').test(row)) errors.push(`R6.${index + 1} should be the '${remedy}' remedy`);
  });
  return errors;
}
