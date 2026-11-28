#!/usr/bin/env node
// Thin client for the shared native *offline* inspection command.
// No project/status lookup, prior-art fallback, command execution from the
// inspected documents, or authority. An older native binary is also BLOCKED.
import { spawnSync } from 'node:child_process';

const files = process.argv.slice(2);
if (files.length !== 3) {
  console.error('Usage: node kbd-scoped-admission-inspect.mjs SNAPSHOT.json DISPOSITION.json INSPECTION.json');
} else {
  const result = spawnSync('prometheus', [
    'kbd', 'inspect-scoped-admission',
    '--parent-snapshot=' + files[0],
    '--disposition=' + files[1],
    '--proposal=' + files[2],
  ], { shell: false, encoding: 'utf8', timeout: 30_000, maxBuffer: 8 * 1024 * 1024 });
  // A timeout/output-buffer failure can carry partial stdout; never
  // present that as a complete diagnostic document.
  if (result.error) {
    console.error('Native inspector unavailable or output incomplete:', result.error.message);
  } else {
    if (result.stdout) process.stdout.write(result.stdout);
    if (result.stderr) process.stderr.write(result.stderr);
  }
}
console.error('BLOCKED: offline inspection cannot authorize execution, activation or closure.');
process.exitCode = 2;
