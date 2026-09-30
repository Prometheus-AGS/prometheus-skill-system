import fs from 'node:fs';
import path from 'node:path';
import { randomUUID } from 'node:crypto';
import { spawnSync } from 'node:child_process';

export const readJson = file => JSON.parse(fs.readFileSync(file, 'utf8'));
export const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);

export function parseArgs(argv) {
  const args = { _: [] };
  for (let index = 0; index < argv.length; index++) {
    const token = argv[index];
    if (!token.startsWith('--')) { args._.push(token); continue; }
    const key = token.slice(2);
    if (!['project', 'input', 'recorder'].includes(key) || !argv[index + 1]) throw new Error('Invalid adapter arguments');
    args[key] = argv[++index];
  }
  return args;
}

export function writeJson(file, value) {
  writeAtomic(file, `${JSON.stringify(value, null, 2)}\n`);
}

export function writeAtomic(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  const temporary = path.join(path.dirname(file), `.${path.basename(file)}.${randomUUID()}.tmp`);
  fs.writeFileSync(temporary, content, { flag: 'wx' });
  try {
    for (let attempt = 0; ; attempt++) {
      try { fs.renameSync(temporary, file); break; }
      catch (error) {
        // Windows scanners can hold the destination open during an atomic replacement.
        if (process.platform !== 'win32' || !['EPERM', 'EBUSY', 'EACCES'].includes(error.code) || attempt === 6) throw error;
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 25 * 2 ** attempt);
      }
    }
  } finally { fs.rmSync(temporary, { force: true }); }
}

export function runJson(command, args, cwd, input, timeout) {
  const result = spawnSync(command, args, { cwd, shell: false, encoding: 'utf8', input, timeout, maxBuffer: 16 * 1024 * 1024 });
  // Do not echo child stderr: recorder inputs and process configuration may contain secrets.
  if (result.error || result.status !== 0) throw new Error('Adapter subprocess did not complete successfully');
  return JSON.parse(result.stdout);
}

export function bindingAt(project) {
  const file = path.join(project, '.prometheus', 'cadence-binding.json');
  return fs.existsSync(file) ? readJson(file) : null;
}
