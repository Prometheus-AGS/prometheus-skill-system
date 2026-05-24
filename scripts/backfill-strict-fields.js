#!/usr/bin/env node
// Backfills missing version, license, and metadata.tags into SKILL.md files.
// Skips skills/imported/. Derives tags from category path when not present.
// Usage: node scripts/backfill-strict-fields.js [--dry-run]

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const SKILLS_DIR = path.join(ROOT, 'skills');

const CATEGORY_TAGS = {
  react: ['react', 'typescript', 'entity-management'],
  process: ['process', 'orchestration', 'automation'],
  rust: ['rust', 'patterns'],
  devops: ['devops', 'gitops', 'kubernetes'],
  testing: ['testing', 'bdd', 'automation'],
  go: ['go', 'patterns'],
  tauri: ['tauri', 'desktop', 'rust'],
  python: ['python', 'bridge'],
  htmx: ['htmx', 'alpine', 'frontend'],
  flutter: ['flutter', 'dart', 'ffi'],
  architecture: ['architecture', 'clean-architecture', 'patterns'],
  typescript: ['typescript', 'patterns'],
};

const dryRun = process.argv.includes('--dry-run');
let modified = 0;
let skipped = 0;

function findSkillMds(dir, results = []) {
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return results;
  }
  for (const entry of entries) {
    if (entry.name === 'imported' || entry.name === 'node_modules' || entry.name.startsWith('.'))
      continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) findSkillMds(full, results);
    else if (entry.name === 'SKILL.md') results.push(full);
  }
  return results;
}

function inferCategory(skillPath) {
  const rel = path.relative(SKILLS_DIR, skillPath);
  return rel.split(path.sep)[0];
}

function parseFrontmatter(content) {
  const match = content.match(/^---\n([\s\S]*?)\n---\n?([\s\S]*)$/s);
  if (!match) return null;
  return { fmText: match[1], body: match[2] };
}

function backfill(skillPath) {
  const content = fs.readFileSync(skillPath, 'utf-8');
  const parsed = parseFrontmatter(content);
  if (!parsed) {
    skipped++;
    return;
  }

  let { fmText, body } = parsed;
  const category = inferCategory(skillPath);
  let changed = false;

  const hasLicense = /^license:/m.test(fmText);
  const hasVersion = /^version:/m.test(fmText);
  const hasTags = /^\s*tags:/m.test(fmText);
  const hasMetadata = /^metadata:/m.test(fmText);

  if (hasLicense && hasVersion && hasTags) {
    skipped++;
    return;
  }

  // Add license before name: line if missing
  if (!hasLicense) {
    fmText = fmText.replace(/^(name:)/m, 'license: MIT\n$1');
    changed = true;
  }

  // Add version after name: line if missing
  if (!hasVersion) {
    fmText = fmText.replace(/^(name: .+)$/m, `$1\nversion: '1.0.0'`);
    changed = true;
  }

  // Add metadata.tags if missing
  if (!hasTags) {
    const tags = CATEGORY_TAGS[category] || [category];
    const tagsYaml = `  tags: [${tags.join(', ')}]`;
    if (hasMetadata) {
      // Insert tags as first item under existing metadata: block
      fmText = fmText.replace(/^(metadata:\s*)$/m, `$1\n${tagsYaml}`);
    } else {
      fmText = `${fmText}\nmetadata:\n${tagsYaml}`;
    }
    changed = true;
  }

  if (!changed) {
    skipped++;
    return;
  }

  const newContent = `---\n${fmText}\n---\n${body}`;
  if (!dryRun) fs.writeFileSync(skillPath, newContent, 'utf-8');
  console.log(`${dryRun ? '[DRY]' : '[MOD]'} ${path.relative(ROOT, skillPath)}`);
  modified++;
}

for (const p of findSkillMds(SKILLS_DIR)) backfill(p);
console.log(
  `\nDone: ${modified} modified, ${skipped} unchanged.${dryRun ? ' (dry run — no files written)' : ''}`
);
