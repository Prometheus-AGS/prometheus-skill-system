#!/usr/bin/env node
/**
 * The gating check: every host leg must report the same identity.
 *
 * This is the assertion no single host can make. Each leg is produced locally
 * by `verify-host-leg.mjs`; this reads the collected receipts and fails when
 * they disagree, when a required leg is missing, or when a leg's own local
 * checks did not pass.
 *
 * `--require-all` is the release gate. Without it, the comparison still runs
 * over whatever legs are present, which is what makes it useful DURING the work
 * rather than only at the end: two legs that already disagree are worth knowing
 * about before the other two are collected.
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const legsRoot = path.join(
  repoRoot,
  'openspec/changes/change-win-001-host-portable-activation/evidence/legs'
);
const requireAll = process.argv.includes('--require-all');
const config = JSON.parse(fs.readFileSync(path.join(repoRoot, 'config/host-legs.json'), 'utf8'));

const failures = [];
const legs = [];

if (fs.existsSync(legsRoot)) {
  for (const name of fs.readdirSync(legsRoot).sort()) {
    if (!name.endsWith('.json')) continue;
    const receipt = JSON.parse(fs.readFileSync(path.join(legsRoot, name), 'utf8'));
    if (receipt.schemaVersion !== 1) {
      failures.push(`${name}: unsupported leg receipt schema ${receipt.schemaVersion}`);
      continue;
    }
    if (`${receipt.legId}.json` !== name) {
      failures.push(`${name}: receipt claims to be leg ${receipt.legId}`);
      continue;
    }
    legs.push(receipt);
  }
}

// A leg must be what it says it is. The id is derived from the probe on the
// producing host, and re-checked here against the declared shape, so a Windows
// host with Developer Mode ON cannot be filed as the Developer-Mode-OFF leg.
const declared = new Map(config.required.map(entry => [entry.id, entry]));
for (const leg of legs) {
  const expected = declared.get(leg.legId);
  if (!expected) continue;
  if (leg.host.platform !== expected.platform) {
    failures.push(`${leg.legId}: recorded on ${leg.host.platform}, expected ${expected.platform}`);
  }
  if (leg.capabilities.directoryLinkStrategy !== expected.directoryLinkStrategy) {
    failures.push(
      `${leg.legId}: directory link strategy is ${leg.capabilities.directoryLinkStrategy}, ` +
        `expected ${expected.directoryLinkStrategy}`
    );
  }
  const failed = (leg.checks ?? []).filter(check => check.exitCode !== 0);
  for (const check of failed) failures.push(`${leg.legId}: local check failed: ${check.name}`);
}

// The gate itself.
for (const field of ['bundleId', 'goldenPayloadDigest']) {
  const values = new Map();
  for (const leg of legs) {
    const value = leg.identity?.[field];
    if (!values.has(value)) values.set(value, []);
    values.get(value).push(leg.legId);
  }
  if (values.size > 1) {
    const rendered = [...values.entries()]
      .map(([value, ids]) => `    ${value}  <- ${ids.join(', ')}`)
      .join('\n');
    failures.push(`hosts disagree on ${field}:\n${rendered}`);
  }
}

const present = new Set(legs.map(leg => leg.legId));
const missing = config.required.filter(entry => !present.has(entry.id));
if (requireAll && missing.length) {
  for (const entry of missing) {
    failures.push(`required leg not collected: ${entry.id} (${entry.why})`);
  }
}

process.stdout.write(
  `host legs collected: ${legs.length ? legs.map(l => l.legId).join(', ') : 'none'}\n`
);
if (legs.length) {
  process.stdout.write(`  bundleId: ${legs[0].identity.bundleId}\n`);
  process.stdout.write(`  golden payload digest: ${legs[0].identity.goldenPayloadDigest}\n`);
  for (const leg of legs) {
    const degraded = leg.degradations?.length
      ? `${leg.degradations.length} degradation(s): ${leg.degradations.map(d => `${d.path}->${d.realized}`).join(', ')}`
      : 'no degradations';
    process.stdout.write(
      `  ${leg.legId}: ${leg.capabilities.directoryLinkStrategy} links, ${degraded}\n`
    );
  }
}
if (missing.length) {
  process.stdout.write(`  not yet collected: ${missing.map(entry => entry.id).join(', ')}\n`);
}

if (failures.length) {
  process.stderr.write(`\n${failures.join('\n')}\n`);
  process.exitCode = 1;
} else if (requireAll) {
  process.stdout.write('All required host legs agree on bundle identity.\n');
} else {
  process.stdout.write(
    `${legs.length} leg(s) agree so far; run with --require-all to gate on the full matrix.\n`
  );
}
