#!/usr/bin/env node
/**
 * generate-skills-index.js — regenerate the "## Skills Index" section of SKILLS.md
 * from the actual skills on disk.
 *
 * Usage:
 *   node scripts/generate-skills-index.js            # rewrite SKILLS.md in place
 *   node scripts/generate-skills-index.js --check    # exit 1 if out of date (CI)
 *
 * WHY THIS EXISTS
 * The index was hand-maintained and drifted badly: it advertised "Process (9 skills)"
 * when there were 15, "DevOps (4)" when there were 5, "Testing (2)" when there were 5,
 * and omitted the learn, flint, research, and documentation categories entirely. The
 * Docusaurus catalog never drifted because site/scripts/generate-skills-catalog.mjs
 * computes it at build time — this brings SKILLS.md under the same discipline.
 *
 * Only the region between "## Skills Index" and the next top-level "## " heading is
 * rewritten; everything else in SKILLS.md is preserved byte-for-byte.
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(__dirname, '..');
const SKILLS_DIR = path.join(REPO, 'skills');
const TARGET = path.join(REPO, 'SKILLS.md');
const START = '## Skills Index';

/** Read `name` and `description` from a SKILL.md YAML frontmatter block. */
function readFrontmatter(file) {
  let raw;
  try {
    raw = fs.readFileSync(file, 'utf8');
  } catch {
    return null;
  }
  // Tolerate a UTF-8 BOM and CRLF line endings.
  raw = raw.replace(/^﻿/, '').replace(/\r\n/g, '\n');
  const m = raw.match(/^---\n([\s\S]*?)\n---/);
  if (!m) return null;
  const block = m[1];

  const name =
    (block.match(/^name:\s*['"]?([^'"\n]+?)['"]?\s*$/m) || [])[1] ||
    path.basename(path.dirname(file));

  // description may be inline or a `>`/`|` folded block.
  let description = '';
  const inline = block.match(/^description:\s*(?![>|])['"]?([^\n]+?)['"]?\s*$/m);
  if (inline) {
    description = inline[1];
  } else {
    const folded = block.match(/^description:\s*[>|][-+]?\s*\n((?:[ \t]+\S[^\n]*\n?)+)/m);
    if (folded) {
      description = folded[1]
        .split('\n')
        .map((l) => l.trim())
        .filter(Boolean)
        .join(' ');
    }
  }
  // Collapse to a single table-safe line.
  description = description.replace(/\s+/g, ' ').replace(/\|/g, '\\|').trim();
  return { name: name.trim(), description };
}

/** Count SKILL.md files strictly BELOW a skill root, at any depth. */
function countNested(root) {
  let n = 0;
  const walk = (dir, depth) => {
    let entries;
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch {
      return;
    }
    for (const e of entries) {
      if (!e.isDirectory()) continue;
      if (e.name === 'node_modules' || e.name === '.git') continue;
      const child = path.join(dir, e.name);
      if (fs.existsSync(path.join(child, 'SKILL.md'))) n++;
      if (depth < 4) walk(child, depth + 1);
    }
  };
  walk(root, 0);
  return n;
}

/** Top-level categories, each with its skills. Excludes imported/ submodules. */
function collect() {
  const categories = [];
  for (const entry of fs.readdirSync(SKILLS_DIR, { withFileTypes: true }).sort((a, b) =>
    a.name.localeCompare(b.name)
  )) {
    if (!entry.isDirectory() || entry.name === 'imported') continue;
    const catDir = path.join(SKILLS_DIR, entry.name);
    const skills = [];
    for (const sub of fs.readdirSync(catDir, { withFileTypes: true }).sort((a, b) =>
      a.name.localeCompare(b.name)
    )) {
      // lstat, not stat: the platform install dirs are symlink farms and following
      // them can recurse or double-count.
      if (!sub.isDirectory()) continue;
      const skillMd = path.join(catDir, sub.name, 'SKILL.md');
      if (!fs.existsSync(skillMd)) continue;
      const fm = readFrontmatter(skillMd);
      if (fm) {
        // Bundles nest sub-skills at skills/<cat>/<skill>/skills/<sub>/SKILL.md.
        // Count them so the header can reconcile with `npm run validate`, which
        // reports top-level + nested together.
        // Bundles nest sub-skills in two shapes: under a `skills/` subdir
        // (kbd-process-orchestrator, deep-research) and as direct children
        // (prometheus-entity-skills). Walk recursively instead of assuming one.
        fm.nested = countNested(path.join(catDir, sub.name));
        skills.push(fm);
      }
    }
    if (skills.length) categories.push({ category: entry.name, skills });
  }
  return categories;
}

function titleCase(slug) {
  const special = { devops: 'DevOps', htmx: 'HTMX', typescript: 'TypeScript', ui_ux: 'UI/UX' };
  if (special[slug]) return special[slug];
  return slug
    .split('-')
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ');
}

function render(categories) {
  const total = categories.reduce((n, c) => n + c.skills.length, 0);
  const nested = categories.reduce(
    (n, c) => n + c.skills.reduce((m, s) => m + (s.nested || 0), 0),
    0
  );
  const lines = [
    START,
    '',
    '<!-- GENERATED by scripts/generate-skills-index.js — do not edit by hand.',
    '     Run `npm run generate:skills-index` after adding or removing a skill. -->',
    '',
    `**${total} top-level skills** across ${categories.length} categories` +
      (nested
        ? `, plus ${nested} nested sub-skills bundled inside them — ${total + nested} total, which is the figure \`npm run validate\` reports.`
        : '.'),
    '',
  ];
  for (const { category, skills } of categories) {
    const label = titleCase(category);
    lines.push(`### ${label} (${skills.length} skill${skills.length === 1 ? '' : 's'})`);
    lines.push('');
    lines.push('| Skill | Description |');
    lines.push('| --- | --- |');
    for (const s of skills) {
      lines.push(`| \`${s.name}\` | ${s.description || '—'} |`);
    }
    lines.push('');
  }
  return lines.join('\n');
}

function rebuild(current, generated) {
  const start = current.indexOf(START);
  if (start === -1) {
    throw new Error(`"${START}" heading not found in SKILLS.md`);
  }
  // End at the next top-level "## " heading after the section start.
  const after = current.indexOf('\n## ', start + START.length);
  const end = after === -1 ? current.length : after + 1;
  return current.slice(0, start) + generated + current.slice(end);
}

const categories = collect();
const generated = render(categories);
const current = fs.readFileSync(TARGET, 'utf8');
const next = rebuild(current, generated);

if (process.argv.includes('--check')) {
  if (next !== current) {
    console.error('SKILLS.md skills index is OUT OF DATE.');
    console.error('  Run: npm run generate:skills-index');
    process.exit(1);
  }
  console.log('SKILLS.md skills index is up to date.');
  process.exit(0);
}

if (next === current) {
  console.log('SKILLS.md already up to date.');
} else {
  fs.writeFileSync(TARGET, next);
  const total = categories.reduce((n, c) => n + c.skills.length, 0);
  console.log(`SKILLS.md regenerated: ${total} skills across ${categories.length} categories.`);
}
