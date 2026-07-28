#!/usr/bin/env node

import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';

const root = resolve(new URL('..', import.meta.url).pathname);
const roots = ['scripts', 'shared/scripts', 'skills/process', '.opencode'];
const projectionNames = /(?:progress\.json|current-waypoint\.json|position\.json|tasks\.md)/i;
const directDestination =
  /(?:^|[^0-9])>{1,2}\s*[^;&|]*(?:progress\.json|current-waypoint\.json|position\.json|tasks\.md)|\b(?:mv|cp|install)\b[^\n]*(?:progress\.json|current-waypoint\.json|position\.json|tasks\.md)/i;
const programmaticWrite =
  /(?:writeFileSync|writeFile|Bun\.write|Deno\.writeTextFile|fs::write|File::create|OpenOptions::new)[^;\n]*(?:progress\.json|current-waypoint\.json|position\.json|tasks\.md)/i;
const runtimeGuard = /kbd_runtime_authoritative|generatedBy[^]*kbd-runtime/;
const legacyBoundary = /kbd_projection_write_guard/;
const offenders = [];
const extensions = new Set(['.sh', '.js', '.mjs', '.cjs', '.ts', '.py', '.rs']);

function walk(directory) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      if (entry.name !== 'tests' && entry.name !== 'fixtures') walk(path);
      continue;
    }
    if (!entry.isFile() || !extensions.has(path.slice(path.lastIndexOf('.')))) continue;
    const source = readFileSync(path, 'utf8');
    const directProjectionWrite = source
      .split('\n')
      .some(
        line =>
          projectionNames.test(line) &&
          (directDestination.test(line) || programmaticWrite.test(line)) &&
          !/^\s*#/.test(line)
      );
    if (directProjectionWrite && !runtimeGuard.test(source) && !legacyBoundary.test(source)) {
      offenders.push(relative(root, path));
    }
  }
}

for (const directory of roots.map(path => join(root, path))) {
  if (statSync(directory).isDirectory()) walk(directory);
}

if (offenders.length > 0) {
  console.error('Direct KBD compatibility writers without a runtime boundary:');
  for (const offender of offenders.sort()) console.error(`  ${offender}`);
  process.exit(1);
}

console.log('KBD direct-writer gate: all compatibility writers are runtime-bounded');
