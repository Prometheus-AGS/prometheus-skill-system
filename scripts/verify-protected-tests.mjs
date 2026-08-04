#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { execFileSync, spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const NAMESPACE = 'prometheus-test-change';
const args = process.argv.slice(2);

function option(name, fallback) {
  const index = args.indexOf(name);
  return index === -1 ? fallback : args[index + 1];
}

function git(...gitArgs) {
  return execFileSync('git', gitArgs, { cwd: repository, encoding: 'buffer' });
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  if (value && typeof value === 'object') {
    return `{${Object.keys(value)
      .sort()
      .map(key => `${JSON.stringify(key)}:${canonical(value[key])}`)
      .join(',')}}`;
  }
  return JSON.stringify(value);
}

function protectedPath(path) {
  if (!path) return false;
  if (/(^|\/)tests\/features\/drafts\//.test(path)) return false;
  return (
    /(^|\/)tests\/steps\/.*\.steps\.ts$/.test(path) ||
    /(^|\/)tests\/support\/.*\.ts$/.test(path) ||
    /(^|\/)tests\/features\/.*\.feature$/.test(path)
  );
}

function blob(commit, path) {
  if (!path || !protectedPath(path)) return null;
  const result = spawnSync('git', ['show', `${commit}:${path}`], {
    cwd: repository,
    encoding: 'buffer',
  });
  if (result.status !== 0) return null;
  return createHash('sha256').update(result.stdout).digest('hex');
}

function protectedChanges(base, candidate) {
  const tokens = git('diff', '--raw', '--no-abbrev', '-z', '-M', base, candidate)
    .toString('utf8')
    .split('\0');
  const changes = [];
  for (let index = 0; index < tokens.length; ) {
    const header = tokens[index++];
    if (!header) continue;
    const match = header.match(
      /^:(\d{6}) (\d{6}) ([0-9a-f]+) ([0-9a-f]+) ([A-Z])(\d+)?$/,
    );
    if (!match) throw new Error(`cannot parse git raw-diff header: ${header}`);
    const oldPath = tokens[index++];
    const renamed = match[5] === 'R' || match[5] === 'C';
    const newPath = renamed ? tokens[index++] : oldPath;
    if (!protectedPath(oldPath) && !protectedPath(newPath)) continue;
    changes.push({
      status: match[5],
      oldPath,
      newPath,
      oldMode: match[1],
      newMode: match[2],
      oldSha256: blob(base, oldPath),
      newSha256: blob(candidate, newPath),
    });
  }
  return changes.sort((left, right) =>
    `${left.oldPath}\0${left.newPath}`.localeCompare(`${right.oldPath}\0${right.newPath}`),
  );
}

const repository = execFileSync('git', ['rev-parse', '--show-toplevel'], {
  encoding: 'utf8',
}).trim();
const base = git('rev-parse', option('--base', 'HEAD^')).toString('utf8').trim();
const candidate = git('rev-parse', option('--candidate', 'HEAD')).toString('utf8').trim();
const changes = protectedChanges(base, candidate);

if (args.includes('--template')) {
  const manifest = {
    schemaVersion: '1',
    namespace: NAMESPACE,
    baseCommit: base,
    candidateCommit: candidate,
    changedProtectedPaths: changes,
    reason: option('--reason', 'Explain why the protected test change is intentional.'),
    approver: option('--approver', 'travis@tribehealthsolutions.com'),
    timestamp: option('--timestamp', new Date().toISOString()),
  };
  process.stdout.write(`${canonical(manifest)}\n`);
  process.exit(0);
}

if (changes.length === 0) {
  console.log(
    JSON.stringify({ status: 'ok', baseCommit: base, candidateCommit: candidate, changes: 0 }),
  );
  process.exit(0);
}

const approvalPath = option('--approval');
const allowedSigners = resolve(
  repository,
  option('--allowed-signers', 'config/protected-test-allowed-signers'),
);
if (!approvalPath) {
  console.error(
    `protected-test certification failed: ${changes.length} protected change(s) require an SSH-signed approval manifest`,
  );
  process.exit(1);
}

const approval = resolve(repository, approvalPath);
const raw = readFileSync(approval, 'utf8').trim();
const manifest = JSON.parse(raw);
if (raw !== canonical(manifest)) {
  throw new Error('approval manifest is not canonical JSON');
}
if (
  manifest.schemaVersion !== '1' ||
  manifest.namespace !== NAMESPACE ||
  manifest.baseCommit !== base ||
  manifest.candidateCommit !== candidate ||
  canonical(manifest.changedProtectedPaths) !== canonical(changes) ||
  typeof manifest.reason !== 'string' ||
  manifest.reason.trim().length < 10 ||
  typeof manifest.approver !== 'string' ||
  !Number.isFinite(Date.parse(manifest.timestamp))
) {
  throw new Error('approval manifest does not match the certified Git diff or required metadata');
}

const signature = option('--signature', `${approvalPath}.sig`);
const verified = spawnSync(
  'ssh-keygen',
  [
    '-Y',
    'verify',
    '-f',
    allowedSigners,
    '-I',
    manifest.approver,
    '-n',
    NAMESPACE,
    '-s',
    resolve(repository, signature),
  ],
  { cwd: repository, input: `${raw}\n`, encoding: 'utf8' },
);
if (verified.status !== 0) {
  process.stderr.write(verified.stderr || 'SSH approval signature verification failed\n');
  process.exit(1);
}

console.log(
  JSON.stringify({
    status: 'approved',
    baseCommit: base,
    candidateCommit: candidate,
    changes: changes.length,
    approver: manifest.approver,
  }),
);
