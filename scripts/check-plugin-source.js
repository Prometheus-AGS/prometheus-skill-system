#!/usr/bin/env node
/**
 * Report where this pack's plugin marketplace sources point and whether they are safe.
 *
 *   check-plugin-source.js [--json] [--home <dir>]
 *       exit 0 ok/warn/skip, 1 when a registered source is missing
 *   check-plugin-source.js --enforce <checkout> [--allow-topic-branch]
 *       exit 3 when <checkout> IS a registered source and is on a topic branch: the
 *       installer must not make a branch-lifetime checkout a durable plugin source.
 *       A checkout that is not a registered source is never blocked.
 *
 * The logic lives in scripts/lib/plugin-source-topology.js; the Rust doctor and the
 * installer scripts both call this entry point so there is one implementation.
 */
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';

import {
  canonicalPath,
  evaluateSources,
  gitFacts,
  isReleaseLineBranch,
  nativeCacheSkew,
  readRegisteredSources,
} from './lib/plugin-source-topology.js';

const argv = process.argv.slice(2);
const flag = name => argv.includes(name);
const value = name => (argv.includes(name) ? argv[argv.indexOf(name) + 1] : undefined);
const home = value('--home') ?? os.homedir();

// The report must always be valid JSON: a crash here would be read by the doctor as "checker
// unavailable" and could hide a missing source. Anything unexpected becomes a finding.
let sources;
try {
  sources = readRegisteredSources({ home });
} catch (error) {
  sources = [];
  process.stderr.write(`could not read plugin registrations: ${error?.message ?? error}\n`);
}

if (value('--enforce')) {
  // Canonical paths: a source registered through a symlink is still the registered source.
  const canonical = canonicalPath;
  const target = canonical(value('--enforce'));
  // An unreadable registration has no path to compare; it stays an advisory finding.
  const registered = sources.some(source => typeof source.path === 'string' && canonical(source.path) === target);
  let facts;
  try {
    facts = gitFacts(target);
  } catch (error) {
    // The branch could not be established: stay advisory rather than block an install on a failed probe.
    process.stderr.write(`could not inspect ${target}: ${error.message}\n`);
    process.exit(0);
  }
  if (registered && facts.repo && !isReleaseLineBranch(facts.branch) && !flag('--allow-topic-branch')) {
    process.stderr.write(
      `refusing: ${target} is a registered plugin source but is on topic branch '${facts.branch}'. ` +
        'If that branch or worktree is removed, every hook of this plugin fails. ' +
        'Switch the checkout to a release-line branch, or pass --allow-topic-branch to accept the risk.\n'
    );
    process.exit(3);
  }
  process.exit(0);
}

let topology;
try {
  topology = evaluateSources(sources);
} catch (error) {
  topology = {
    status: 'warn',
    findings: [{ code: 'INSPECTION_FAILED', severity: 'warn', message: `plugin sources could not be inspected: ${error?.message ?? error}` }],
    sources: [],
  };
}
// The native cache is advisory: an unreadable cache entry must not hide the topology report.
let skew;
try {
  skew = nativeCacheSkew({ home });
} catch (error) {
  skew = {
    installedVersion: null,
    activeGeneration: null,
    findings: [{ code: 'SKEW_UNREADABLE', severity: 'info', message: `the native plugin cache could not be inspected: ${error.message}` }],
  };
}
if (flag('--json')) {
  process.stdout.write(`${JSON.stringify({ topology, skew }, null, 2)}\n`);
} else {
  for (const f of [...topology.findings, ...skew.findings]) process.stdout.write(`[${f.severity}] ${f.code}: ${f.message}\n`);
  if (topology.status === 'ok' && skew.findings.length === 0) process.stdout.write('plugin sources: ok\n');
}
process.exit(topology.status === 'fail' ? 1 : 0);
