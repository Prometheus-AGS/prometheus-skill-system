#!/usr/bin/env node
/**
 * SP-016 — Skill description collision detection
 *
 * Reads all skills SKILL.md frontmatter, extracts name + description,
 * computes pairwise Jaccard similarity on lowercase word shingles, and
 * reports pairs above the threshold.
 *
 * Usage:
 *   node scripts/skill-matrix.js
 *   node scripts/skill-matrix.js --threshold 0.5
 *   node scripts/skill-matrix.js --ci        (exit 1 on new collisions not in allowlist)
 *
 * CI mode: fail if any pair above threshold is absent from
 *   scripts/skill-collision-allowlist.json
 */

import { readFileSync, existsSync } from 'fs';
import { resolve, relative, join, dirname } from 'path';
import { fileURLToPath } from 'url';
import globPkg from 'glob';
const { sync: globSync } = globPkg;

const __filename = fileURLToPath(import.meta.url);
const __dirname  = dirname(__filename);

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

const REPO_ROOT   = resolve(__dirname, '..');
const SKILLS_GLOB = 'skills/**/SKILL.md';
const ALLOWLIST   = join(__dirname, 'skill-collision-allowlist.json');

const args      = process.argv.slice(2);
const CI_MODE   = args.includes('--ci');
const THRESHOLD = (() => {
  const idx = args.indexOf('--threshold');
  return idx >= 0 ? parseFloat(args[idx + 1]) : 0.6;
})();

// ---------------------------------------------------------------------------
// YAML frontmatter parser (minimal — no external deps)
// ---------------------------------------------------------------------------

function parseFrontmatter(content) {
  const match = content.match(/^---\n([\s\S]*?)\n---/);
  if (!match) return {};
  const yaml = match[1];
  const result = {};
  let currentKey = null;
  let inBlock = false;

  for (const line of yaml.split('\n')) {
    const kv = line.match(/^([a-zA-Z_][a-zA-Z0-9_]*):\s*(.*)/);
    if (kv) {
      currentKey = kv[1];
      const val = kv[2].trim();
      if (val === '>' || val === '|') {
        result[currentKey] = '';
        inBlock = true;
      } else {
        result[currentKey] = val.replace(/^['"]|['"]$/g, '');
        inBlock = false;
      }
    } else if (inBlock && currentKey && line.match(/^\s+\S/)) {
      result[currentKey] = ((result[currentKey] || '') + ' ' + line.trim()).trim();
    } else {
      inBlock = false;
    }
  }
  return result;
}

// ---------------------------------------------------------------------------
// Jaccard similarity on word-level bigrams
// ---------------------------------------------------------------------------

function tokenize(str) {
  return str.toLowerCase().replace(/[^a-z0-9\s]/g, ' ').split(/\s+/).filter(Boolean);
}

function shingles(tokens, k = 2) {
  const set = new Set();
  for (let i = 0; i <= tokens.length - k; i++) {
    set.add(tokens.slice(i, i + k).join(' '));
  }
  for (const t of tokens) set.add(t);
  return set;
}

function jaccard(a, b) {
  if (a.size === 0 && b.size === 0) return 1;
  let inter = 0;
  for (const item of a) { if (b.has(item)) inter++; }
  return inter / (a.size + b.size - inter);
}

// ---------------------------------------------------------------------------
// Load skills
// ---------------------------------------------------------------------------

const skillFiles = globSync(SKILLS_GLOB, { cwd: REPO_ROOT, absolute: true });

const skills = [];
for (const f of skillFiles) {
  const content = readFileSync(f, 'utf8');
  const fm = parseFrontmatter(content);
  if (!fm.name || !fm.description) continue;
  const relPath = relative(REPO_ROOT, f);
  skills.push({ name: fm.name, description: fm.description, path: relPath });
}

console.log(`Loaded ${skills.length} skills with name + description`);
console.log(`Threshold: ${THRESHOLD}  CI mode: ${CI_MODE}`);
console.log('');

// ---------------------------------------------------------------------------
// Compute pairwise similarity
// ---------------------------------------------------------------------------

const pairs = [];

for (let i = 0; i < skills.length; i++) {
  for (let j = i + 1; j < skills.length; j++) {
    const a = skills[i];
    const b = skills[j];
    const sa = shingles(tokenize(a.description));
    const sb = shingles(tokenize(b.description));
    const score = jaccard(sa, sb);
    if (score >= THRESHOLD) {
      pairs.push({ a: a.name, b: b.name, score, descA: a.description, descB: b.description });
    }
  }
}

pairs.sort((x, y) => y.score - x.score);

// ---------------------------------------------------------------------------
// Load allowlist
// ---------------------------------------------------------------------------

let allowlist = [];
if (existsSync(ALLOWLIST)) {
  try {
    allowlist = JSON.parse(readFileSync(ALLOWLIST, 'utf8'));
  } catch (e) {
    console.error('Warning: could not parse allowlist:', e.message);
  }
}

function isAllowed(pairA, pairB) {
  return allowlist.some(entry => {
    const [x, y] = [...entry.skills].sort();
    const [pa, pb] = [pairA, pairB].sort();
    return x === pa && y === pb;
  });
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

if (pairs.length === 0) {
  console.log('✓ No skill description collisions detected above threshold.');
  process.exit(0);
}

const newPairs  = pairs.filter(p => !isAllowed(p.a, p.b));
const knownPairs = pairs.filter(p => isAllowed(p.a, p.b));

if (knownPairs.length > 0) {
  console.log(`Known collisions (in allowlist): ${knownPairs.length}`);
  for (const p of knownPairs) {
    console.log(`  [${p.score.toFixed(2)}] ${p.a} <-> ${p.b}`);
  }
  console.log('');
}

if (newPairs.length > 0) {
  console.log(`WARNING: New collisions above threshold ${THRESHOLD}: ${newPairs.length}`);
  console.log('');
  for (const p of newPairs) {
    console.log(`  [${p.score.toFixed(2)}] ${p.a}`);
    console.log(`         "${p.descA}"`);
    console.log(`       <-> ${p.b}`);
    console.log(`         "${p.descB}"`);
    console.log('');
  }
  console.log('Remediation options:');
  console.log('  1. Rename one skill description to reduce overlap');
  console.log('  2. Add to scripts/skill-collision-allowlist.json with reasoning');
  console.log('');

  if (CI_MODE) {
    console.error(`CI FAIL: ${newPairs.length} new collision(s) not in allowlist.`);
    process.exit(1);
  }
} else {
  console.log('All collisions are in the allowlist.');
}
