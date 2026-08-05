#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { execFileSync, spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const args = process.argv.slice(2);
const command = args.shift();
const MAX_GIT_OUTPUT_BYTES = 128 * 1024 * 1024;
const repository = execFileSync('git', ['rev-parse', '--show-toplevel'], {
  encoding: 'utf8',
}).trim();

function option(name, fallback) {
  const index = args.indexOf(name);
  return index === -1 ? fallback : args[index + 1];
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

function git(...gitArgs) {
  return execFileSync('git', gitArgs, {
    cwd: repository,
    encoding: 'buffer',
    maxBuffer: MAX_GIT_OUTPUT_BYTES,
  });
}

function commit(ref) {
  return git('rev-parse', ref).toString('utf8').trim();
}

function cumulative(base, candidate) {
  const patch = git('diff', '--binary', '--full-index', base, candidate);
  return {
    baseCommit: commit(base),
    candidateCommit: commit(candidate),
    diffSha256: createHash('sha256').update(patch).digest('hex'),
    changedFiles: git('diff', '--name-only', '-z', base, candidate)
      .toString('utf8')
      .split('\0')
      .filter(Boolean)
      .sort(),
  };
}

function writeCanonical(path, value) {
  const serialized = `${canonical(value)}\n`;
  if (path) writeFileSync(resolve(repository, path), serialized);
  else process.stdout.write(serialized);
}

function range() {
  const previous = option('--previous');
  const prior = previous ? JSON.parse(readFileSync(resolve(repository, previous), 'utf8')) : null;
  if (prior && prior.status !== 'completed') {
    throw new Error('--previous must name an accepted completed review receipt');
  }
  const base = option('--base', prior?.candidateCommit || 'HEAD^');
  return cumulative(base, option('--candidate', 'HEAD'));
}

if (command === 'pending') {
  writeCanonical(option('--out'), {
    schemaVersion: '1',
    status: 'pending_review',
    ...range(),
    reason: option('--reason', 'The independent judge was unavailable.'),
    timestamp: option('--timestamp', new Date().toISOString()),
  });
  process.exit(0);
}

if (command === 'complete') {
  const findingsPath = option('--findings');
  if (!findingsPath) throw new Error('--findings is required');
  const findingsBytes = readFileSync(resolve(repository, findingsPath));
  const findings = JSON.parse(findingsBytes.toString('utf8'));
  const reviewer = option('--reviewer');
  const model = option('--model');
  if (!reviewer || !model) throw new Error('--reviewer and --model are required');
  const criticalFindings = (findings.findings || []).filter(
    finding => String(finding.severity || '').toUpperCase() === 'CRITICAL',
  ).length;
  if (criticalFindings !== 0) throw new Error('cannot complete a review with CRITICAL findings');
  writeCanonical(option('--out'), {
    schemaVersion: '1',
    status: 'completed',
    ...range(),
    reviewer: {
      kind: 'independent_judge',
      id: reviewer,
      model,
    },
    verdict: 'pass',
    criticalFindings,
    findingsSha256: createHash('sha256').update(findingsBytes).digest('hex'),
    timestamp: option('--timestamp', new Date().toISOString()),
  });
  process.exit(0);
}

if (command === 'waiver-template') {
  writeCanonical(option('--out'), {
    schemaVersion: '1',
    namespace: 'prometheus-review-waiver',
    ...range(),
    reason: option('--reason', 'Explain why final certification may proceed without review.'),
    approver: option('--approver', 'travis@tribehealthsolutions.com'),
    timestamp: option('--timestamp', new Date().toISOString()),
  });
  process.exit(0);
}

if (command !== 'verify') {
  throw new Error('usage: local-review-receipt.mjs pending|complete|waiver-template|verify');
}

const expected = range();
const receiptPath = option('--receipt');
if (receiptPath) {
  const raw = readFileSync(resolve(repository, receiptPath), 'utf8').trim();
  const receipt = JSON.parse(raw);
  if (raw !== canonical(receipt)) throw new Error('review receipt is not canonical JSON');
  if (
    receipt.status === 'completed' &&
    receipt.baseCommit === expected.baseCommit &&
    receipt.candidateCommit === expected.candidateCommit &&
    receipt.diffSha256 === expected.diffSha256 &&
    receipt.criticalFindings === 0 &&
    receipt.verdict === 'pass' &&
    receipt.reviewer?.kind === 'independent_judge' &&
    receipt.reviewer?.id &&
    receipt.reviewer?.model
  ) {
    console.log(JSON.stringify({ status: 'reviewed', ...expected }));
    process.exit(0);
  }
}

const waiverPath = option('--waiver');
if (waiverPath) {
  const raw = readFileSync(resolve(repository, waiverPath), 'utf8').trim();
  const waiver = JSON.parse(raw);
  if (raw !== canonical(waiver)) throw new Error('review waiver is not canonical JSON');
  if (
    waiver.namespace !== 'prometheus-review-waiver' ||
    waiver.baseCommit !== expected.baseCommit ||
    waiver.candidateCommit !== expected.candidateCommit ||
    waiver.diffSha256 !== expected.diffSha256 ||
    typeof waiver.reason !== 'string' ||
    waiver.reason.trim().length < 10 ||
    typeof waiver.approver !== 'string' ||
    !Number.isFinite(Date.parse(waiver.timestamp))
  ) {
    throw new Error('review waiver does not match the cumulative candidate diff');
  }
  const policy = resolve(
    repository,
    option('--allowed-signers', 'config/protected-test-allowed-signers'),
  );
  const signature = resolve(repository, option('--signature', `${waiverPath}.sig`));
  const verified = spawnSync(
    'ssh-keygen',
    [
      '-Y',
      'verify',
      '-f',
      policy,
      '-I',
      waiver.approver,
      '-n',
      'prometheus-review-waiver',
      '-s',
      signature,
    ],
    { cwd: repository, input: `${raw}\n`, encoding: 'utf8' },
  );
  if (verified.status === 0) {
    console.log(JSON.stringify({ status: 'waived', approver: waiver.approver, ...expected }));
    process.exit(0);
  }
  process.stderr.write(verified.stderr || 'review waiver signature verification failed\n');
}

console.error(
  'local review certification failed: cumulative diff requires a completed review receipt or SSH-signed waiver',
);
process.exit(1);
