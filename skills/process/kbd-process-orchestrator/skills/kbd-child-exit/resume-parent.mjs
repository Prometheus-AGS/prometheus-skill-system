import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';

const phasePath = JSON.parse(readFileSync(0, 'utf8'));
if (!Array.isArray(phasePath) || phasePath.length < 2 ||
    !phasePath.every(id => typeof id === 'string' && id.length > 0)) {
  throw new Error('Child exit requires a canonical path containing the parent and child');
}

const parent = phasePath.at(-2);
const ancestors = phasePath.slice(0, -2).flatMap(id => ['--ancestor', id]);
const result = spawnSync('prometheus', [
  'kbd', '--path', '.', 'phase', 'activate',
  '--command-id', `phase-exit:${process.argv[2]}`,
  '--id', parent, ...ancestors, '--exact-next-work', '/kbd-status',
], { shell: false, stdio: ['ignore', 'inherit', 'inherit'] });
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
