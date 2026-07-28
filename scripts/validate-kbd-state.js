#!/usr/bin/env node

import { readFileSync, readdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import Ajv2020 from 'ajv/dist/2020.js';

const repoRoot = resolve(new URL('..', import.meta.url).pathname);
const schemaRoot = join(repoRoot, 'skills/process/kbd-process-orchestrator/references/schemas');

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

function walk(dir, name, out = []) {
  let entries = [];
  try {
    entries = readdirSync(dir, { withFileTypes: true });
  } catch {
    return out;
  }
  for (const entry of entries) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) walk(path, name, out);
    else if (entry.name === name) out.push(path);
  }
  return out;
}

const ajv = new Ajv2020({
  allErrors: true,
  strict: false,
  formats: { 'date-time': true },
});
const progressSchema = readJson(join(schemaRoot, 'progress.schema.json'));
const waypointSchema = readJson(join(schemaRoot, 'current-waypoint.schema.json'));
const validateProgress = ajv.compile(progressSchema);
const validateWaypoint = ajv.compile(waypointSchema);

let failures = 0;
let canonical = 0;
let legacy = 0;
for (const path of walk(join(repoRoot, '.kbd-orchestrator'), 'progress.json')) {
  let data;
  try {
    data = readJson(path);
  } catch (error) {
    console.error(`INVALID JSON ${path}: ${error.message}`);
    failures += 1;
    continue;
  }
  if (data.schemaVersion !== '2') {
    legacy += 1;
    continue;
  }
  canonical += 1;
  if (!validateProgress(data)) {
    console.error(`INVALID schema v2 progress ${path}`);
    for (const error of validateProgress.errors ?? []) {
      console.error(`  ${error.instancePath || '/'} ${error.message}`);
    }
    failures += 1;
  }
  if (
    data.generatedBy === 'kbd-runtime' &&
    (!Number.isInteger(data.sourceRevision) || data.sourceRevision < 1)
  ) {
    console.error(`INVALID runtime projection revision ${path}`);
    failures += 1;
  }
}

const waypointPath = join(repoRoot, '.kbd-orchestrator/current-waypoint.json');
try {
  const waypoint = readJson(waypointPath);
  if (!validateWaypoint(waypoint)) {
    console.error(`INVALID waypoint schema ${waypointPath}`);
    for (const error of validateWaypoint.errors ?? []) {
      console.error(`  ${error.instancePath || '/'} ${error.message}`);
    }
    failures += 1;
  }
  if (waypoint.generatedBy === 'kbd-runtime' && waypoint.sourceRevision !== waypoint.revision) {
    console.error(
      `INVALID runtime waypoint revision: sourceRevision=${waypoint.sourceRevision} ` +
        `revision=${waypoint.revision}`
    );
    failures += 1;
  }
  const aliasPairs = [
    ['changesCompleted', 'changes_completed'],
    ['changesTotal', 'changes_total'],
    ['exactNextCommand', 'exact_next_command'],
    ['currentTask', 'current_task'],
  ];
  for (const [canonicalKey, legacyKey] of aliasPairs) {
    if (
      Object.hasOwn(waypoint, canonicalKey) &&
      Object.hasOwn(waypoint, legacyKey) &&
      JSON.stringify(waypoint[canonicalKey]) !== JSON.stringify(waypoint[legacyKey])
    ) {
      console.error(
        `CONFLICT waypoint aliases ${canonicalKey}/${legacyKey}: ` +
          `${JSON.stringify(waypoint[canonicalKey])} != ${JSON.stringify(waypoint[legacyKey])}`
      );
      failures += 1;
    }
  }
} catch (error) {
  console.error(`INVALID waypoint ${waypointPath}: ${error.message}`);
  failures += 1;
}

console.log(
  `KBD state: ${canonical} canonical progress ledgers, ${legacy} legacy ledgers, ` +
    `${failures} failures`
);
process.exit(failures === 0 ? 0 : 1);
