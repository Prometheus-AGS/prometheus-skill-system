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

const sources = readRegisteredSources({ home });

if (value('--enforce')) {
  // Canonical paths: a source registered through a symlink is still the registered source.
  const canonical = p => {
    try {
      return fs.realpathSync(p);
    } catch {
      return path.resolve(p);
    }
  };
  const target = canonical(value('--enforce'));
  // An unreadable registration has no path to compare; it stays an advisory finding.
  const registered = sources.some(source => typeof source.path === 'string' && canonical(source.path) === target);
  const facts = gitFacts(target);
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

const topology = evaluateSources(sources);
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
