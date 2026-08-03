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
import { execFileSync } from 'node:child_process';
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
        .map(l => l.trim())
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
  for (const entry of fs
    .readdirSync(SKILLS_DIR, { withFileTypes: true })
    .sort((a, b) => a.name.localeCompare(b.name))) {
    if (!entry.isDirectory() || entry.name === 'imported') continue;
    const catDir = path.join(SKILLS_DIR, entry.name);
    const skills = [];
    for (const sub of fs
      .readdirSync(catDir, { withFileTypes: true })
      .sort((a, b) => a.name.localeCompare(b.name))) {
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
    .map(w => w.charAt(0).toUpperCase() + w.slice(1))
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

/**
 * Rewrite the provenance keys in SKILLS.md's YAML frontmatter.
 *
 * WHY THIS EXISTS
 * UAR pinned this pack at a commit that was 359 commits and two months stale,
 * seeing 161 skills where the pack had 220, and NOTHING DETECTED IT — because
 * nothing recorded which version had been loaded. A consumer cannot compare
 * against a version the producer never states.
 *
 * The commit is written HERE, at generation time, by the one process that can
 * read git. A consumer (UAR, and especially a phone) must never shell out to
 * git: on mobile there is no git and no .git directory to read.
 */
function stampProvenance(text, commit, skillCount) {
  const fields = {
    commit,
    skill_count: String(skillCount),
    generated_at: new Date().toISOString().replace(/\.\d{3}Z$/, 'Z'),
  };
  let out = text;
  for (const [key, value] of Object.entries(fields)) {
    const re = new RegExp(`^${key}: .*$`, 'm');
    if (re.test(out)) {
      out = out.replace(re, `${key}: ${value}`);
    } else {
      // Insert after `version:` so provenance stays grouped with identity.
      out = out.replace(/^(version: .*)$/m, `$1\n${key}: ${value}`);
    }
  }
  return out;
}

/**
 * Count every SKILL.md a consumer would load — NOT the 69 shown in the index.
 *
 * The index deliberately lists only top-level owned skills for readability. A
 * consumer (UAR) walks the whole tree minus `imported/`, and sees ~146. Emitting
 * 69 would guarantee the counts never match, so the field could never detect the
 * drift it exists to detect: a consumer comparing 161-vs-69 learns nothing.
 *
 * Mirrors builtin_loader.rs, which skips paths containing `imported/` unless
 * explicitly opted in, and never descends into node_modules.
 */
function loadableSkillCount(dir = SKILLS_DIR) {
  let n = 0;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === 'node_modules' || entry.name === 'imported') continue;
      if (entry.name === 'tests' || entry.name === 'fixtures') continue;
      n += loadableSkillCount(full);
    } else if (entry.name === 'SKILL.md') {
      n += 1;
    }
  }
  return n;
}

function packCommit() {
  // Failure is reported, never papered over: an unknown commit must read as
  // "unknown", not as a plausible-looking wrong value.
  try {
    // execFileSync, not execSync: no shell, so a path with spaces or a hostile
    // env cannot turn this into command injection. And `require` is unavailable
    // here — this file is an ES module ("type": "module"), which is exactly how
    // the first version of this function silently returned 'unknown' forever.
    return execFileSync('git', ['rev-parse', 'HEAD'], {
      cwd: REPO,
      stdio: ['ignore', 'pipe', 'ignore'],
    })
      .toString()
      .trim();
  } catch {
    return 'unknown';
  }
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
const skillTotal = categories.reduce((n, c) => n + c.skills.length, 0);
let next = rebuild(current, generated);
next = stampProvenance(next, packCommit(), loadableSkillCount());

if (process.argv.includes('--check')) {
  // `generated_at` is a timestamp: it differs on every run by design, so a
  // byte-for-byte comparison could NEVER pass. The recorded commit may be
  // HEAD while changes are staged or HEAD^ after the generated file is
  // committed; accepting anything older would hide real provenance drift.
  const recordedCommit = current.match(/^commit: (.*)$/m)?.[1];
  let parentCommit;
  try {
    parentCommit = execFileSync('git', ['rev-parse', 'HEAD^'], {
      cwd: REPO,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim();
  } catch {
    parentCommit = undefined;
  }
  const validProvenance = recordedCommit === packCommit() || recordedCommit === parentCommit;
  const normaliseVolatileProvenance = t =>
    t
      .replace(/^generated_at: .*$/m, 'generated_at: <ignored>')
      .replace(/^commit: .*$/m, 'commit: <validated>');
  if (
    !validProvenance ||
    normaliseVolatileProvenance(next) !== normaliseVolatileProvenance(current)
  ) {
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
  console.log(
    `SKILLS.md regenerated: ${skillTotal} skills across ${categories.length} categories.`
  );
}
