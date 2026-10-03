import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';

export const MINIMUM_VERSION = '1.14.0';
export const REGISTRY = 'https://registry.npmjs.org';
export const cacheHome = () => path.resolve(process.env.PROMETHEUS_OPENSPEC_HOME || path.join(os.homedir(), '.prometheus', 'openspec'));
export const readJson = file => fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, 'utf8')) : null;

export function writeJson(file, value) {
  fs.mkdirSync(path.dirname(file), { recursive: true, mode: 0o700 });
  const temporary = `${file}.${crypto.randomUUID()}.tmp`;
  fs.writeFileSync(temporary, `${JSON.stringify(value, null, 2)}\n`, { mode: 0o600 });
  fs.renameSync(temporary, file);
}

export function acceptedVersion(version) {
  if (!/^\d+\.\d+\.\d+$/.test(version || '')) throw new Error('OpenSpec version must be an exact stable semantic version');
  const actual = version.split('.').map(Number);
  const minimum = MINIMUM_VERSION.split('.').map(Number);
  for (let i = 0; i < 3; i++) {
    if (actual[i] > minimum[i]) return version;
    if (actual[i] < minimum[i]) throw new Error(`OpenSpec ${version} is below minimum ${MINIMUM_VERSION}`);
  }
  return version;
}

export function findProject(start, requireKbd = true) {
  let cursor = path.resolve(start);
  for (;;) {
    const hasKbd = fs.existsSync(path.join(cursor, '.prometheus', 'project.json')) ||
      fs.existsSync(path.join(cursor, '.kbd-orchestrator', 'project.json')) ||
      fs.existsSync(path.join(cursor, '.kbd-orchestrator', 'current-waypoint.json'));
    if (fs.existsSync(path.join(cursor, 'openspec')) && (!requireKbd || hasKbd)) return cursor;
    const parent = path.dirname(cursor);
    if (parent === cursor) return null;
    cursor = parent;
  }
}

export function deadline(timeoutMs) {
  const until = Date.now() + timeoutMs;
  return () => {
    const remaining = until - Date.now();
    if (remaining <= 0) throw Object.assign(new Error('OpenSpec operation exceeded its time budget'), { status: 'pending' });
    return remaining;
  };
}

export function acquireLock(home, operation, project) {
  fs.mkdirSync(home, { recursive: true, mode: 0o700 });
  const file = path.join(home, 'operation.lock');
  let fd;
  try { fd = fs.openSync(file, 'wx', 0o600); }
  catch (error) {
    if (error.code !== 'EEXIST') throw error;
    throw Object.assign(new Error(`OpenSpec operation is locked: ${file}. Inspect the owner; remove only after confirming it has stopped.`), { status: 'contended' });
  }
  fs.writeFileSync(fd, JSON.stringify({ pid: process.pid, operation, project, startedAt: new Date().toISOString() }));
  fs.closeSync(fd);
  return () => fs.unlinkSync(file);
}

export function receipt(home, body) {
  const value = { schemaVersion: 1, time: new Date().toISOString(), ...body };
  const key = crypto.createHash('sha256').update(body.project || 'no-project').digest('hex').slice(0, 24);
  const dir = path.join(home, 'receipts', key);
  const file = path.join(dir, `${Date.now()}-${crypto.randomUUID()}.json`);
  writeJson(file, value);
  writeJson(path.join(dir, 'latest.json'), { ...value, receipt: file });
  return { ...value, receipt: file };
}
