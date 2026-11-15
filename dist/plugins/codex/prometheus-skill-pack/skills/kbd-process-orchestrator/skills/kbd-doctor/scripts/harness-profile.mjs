#!/usr/bin/env node
// harness-profile.mjs — print a harness capability profile (VLBV CR-23, FR-H7).
// Usage: node harness-profile.mjs [harness-id] [--json]
// The harness defaults to $PROMETHEUS_HARNESS. Every capability whose status is
// not "supported" is listed as UNKNOWN/UNSUPPORTED so no skill relies on it.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const candidates = [process.env.PROMETHEUS_HARNESS_PROFILES,
  path.resolve(here, '../../../../../../shared/harnesses/capabilities.json'),
  path.resolve(here, '../../../../../shared/harnesses/capabilities.json'),
  path.resolve(here, '../../../../shared/harnesses/capabilities.json'),
  path.resolve(here, '../../../shared/harnesses/capabilities.json')].filter(Boolean);
const file = candidates.find(candidate => fs.existsSync(candidate));
if (!file) { process.stderr.write('harness-profile: capabilities.json not found (set PROMETHEUS_HARNESS_PROFILES)\n'); process.exit(2); }
const profiles = JSON.parse(fs.readFileSync(file, 'utf8')).profiles ?? {};
const args = process.argv.slice(2);
const json = args.includes('--json');
const id = args.find(a => !a.startsWith('--')) ?? process.env.PROMETHEUS_HARNESS;
if (!id || !profiles[id]) {
  process.stderr.write(`harness-profile: name a harness (${Object.keys(profiles).filter(k => !k.startsWith('_')).join(', ')})\n`);
  process.exit(2);
}
const profile = profiles[id];
const rows = Object.entries(profile).filter(([, v]) => v && typeof v === 'object' && 'status' in v)
  .map(([capability, v]) => ({ capability, status: v.status, source: v.source ?? v.documentedAs ?? null }));
if (json) { process.stdout.write(JSON.stringify({ harness: id, profile, unknown: rows.filter(r => r.status !== 'supported') }, null, 2) + '\n'); process.exit(0); }
process.stdout.write(`Harness profile: ${id}\n`);
process.stdout.write(`  rule files: ${profile.rulesFiles?.precedence?.join(' > ')}${profile.rulesFiles?.firstMatchOnly ? ' (first match only)' : ''}${profile.rulesFiles?.overrideFile ? `; override ${profile.rulesFiles.overrideFile}` : ''}\n`);
process.stdout.write(`  skill roots: project ${profile.skillRoots?.project?.join(', ')}; user ${profile.skillRoots?.user?.join(', ')}\n`);
process.stdout.write(`  invocation: ${profile.invocation?.explicit}\n`);
for (const r of rows) process.stdout.write(`  ${r.status === 'supported' ? 'SUPPORTED' : r.status.toUpperCase()}  ${r.capability}${r.source ? `  (${r.source})` : ''}\n`);
process.stdout.write(`  verified: ${profile.verified?.exercised ? `${profile.verified.date} ${profile.verified.version} — ${profile.verified.evidence}` : 'not exercised on this machine'}\n`);
process.stdout.write(`  suppress before B4: ${(profile.builtinsToSuppress ?? []).join('; ') || 'none'}\n`);
