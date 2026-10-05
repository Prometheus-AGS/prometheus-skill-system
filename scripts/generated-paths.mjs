#!/usr/bin/env node
// Prints the authoritative set of generated paths, one per line (directories end
// in '/'). Derived from the generators themselves: each declares its outputs and
// prints them for --list-outputs. Shared by check:distribution and
// scripts/rebase-regenerate.sh so neither keeps a second copy of the list.
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scripts = path.dirname(fileURLToPath(import.meta.url));
const GENERATORS = ['generate-harness-adapters.js', 'generate-skill-system-distribution.js'];

export function generatedPaths() {
  const paths = new Set();
  for (const generator of GENERATORS) {
    const result = spawnSync(process.execPath, [path.join(scripts, generator), '--list-outputs'], {
      encoding: 'utf8',
    });
    if (result.status !== 0)
      throw new Error(`${generator} --list-outputs failed: ${result.stderr || result.status}`);
    for (const line of result.stdout.split('\n')) if (line) paths.add(line);
  }
  return [...paths].sort();
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    process.stdout.write(`${generatedPaths().join('\n')}\n`);
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exit(1);
  }
}
