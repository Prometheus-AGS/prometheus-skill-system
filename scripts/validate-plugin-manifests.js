#!/usr/bin/env node
/**
 * Validate every plugin.json in the tree against schemas/plugin.schema.json.
 *
 * The schema is descriptive, not aspirational: it was written by validating it
 * against all 27 manifests already in this repo and widening it until it
 * accepted every form that genuinely ships (skills/mcpServers as either a
 * string path or a structured value; compatibility as either free text or an
 * object). A schema that rejects working manifests is worse than no schema, so
 * if this fails on a manifest that demonstrably loads, widen the schema rather
 * than "fixing" the manifest.
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import Ajv from 'ajv/dist/2020.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SKIP = new Set(['node_modules', '.git', 'dist', 'target']);

function findManifests(dir, out = []) {
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return out;
  }
  for (const e of entries) {
    if (e.isDirectory()) {
      if (SKIP.has(e.name)) continue;
      findManifests(path.join(dir, e.name), out);
    } else if (e.name === 'plugin.json') {
      out.push(path.join(dir, e.name));
    }
  }
  return out;
}

const schemaPath = path.join(root, 'schemas', 'plugin.schema.json');
const ajv = new Ajv({ allErrors: true, strict: false });
const validate = ajv.compile(JSON.parse(fs.readFileSync(schemaPath, 'utf8')));

const manifests = findManifests(root);
let failed = 0;

for (const file of manifests) {
  const rel = path.relative(root, file);
  let data;
  try {
    data = JSON.parse(fs.readFileSync(file, 'utf8'));
  } catch (err) {
    console.error(`❌ ${rel}\n   not valid JSON: ${err.message}`);
    failed++;
    continue;
  }
  if (!validate(data)) {
    failed++;
    console.error(`❌ ${rel}`);
    for (const err of validate.errors) {
      console.error(`   ${err.instancePath || '/'} ${err.message}`);
    }
  }
}

if (failed === 0) {
  console.log(`✅ ${manifests.length} plugin manifests valid`);
  process.exit(0);
}
console.error(`\n${failed} of ${manifests.length} plugin manifests invalid`);
process.exit(1);
