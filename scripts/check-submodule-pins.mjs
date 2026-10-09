#!/usr/bin/env node
// Fails when a submodule gitlink (index) disagrees with the commit recorded for
// the same path in skill-system.json `imports[]`. A release commit once moved the
// gitlink back to an older SHA while the manifest kept the newer one, and the
// installer then built the older tool without any check noticing.
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const contract = JSON.parse(readFileSync(join(root, 'skill-system.json'), 'utf8'));

function gitlinkFor(path) {
  const out = execFileSync('git', ['ls-files', '-s', '--', path], { cwd: root, encoding: 'utf8' });
  const match = out.match(/^160000 ([0-9a-f]{40}) 0\t/m);
  return match ? match[1] : null;
}

const problems = [];
for (const entry of contract.imports ?? []) {
  if (!entry.path || !entry.commit) continue;
  const gitlink = gitlinkFor(entry.path);
  if (!gitlink) continue; // not a submodule
  const recorded = entry.commit.toLowerCase();
  if (!gitlink.startsWith(recorded) && !recorded.startsWith(gitlink)) {
    problems.push(`${entry.path}: gitlink ${gitlink.slice(0, 12)} != skill-system.json commit ${recorded.slice(0, 12)}`);
  }
}

if (problems.length > 0) {
  console.error(`submodule pin drift (${problems.length}):\n  ${problems.join('\n  ')}`);
  process.exit(1);
}
console.log('submodule pins match skill-system.json');
