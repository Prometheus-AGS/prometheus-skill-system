#!/usr/bin/env node
// generate-skills-catalog.mjs — build-time skills catalog generator (cand-006).
//
// Walks ../skills/**/SKILL.md (excluding imported/ submodules), parses YAML
// frontmatter (same fields scripts/validate-skills.js validates), and emits
// MDX pages into site/docs-catalog/ — one page per top-level category plus an
// index. The output dir is generated + gitignored; wired via `prebuild`.
//
// Idempotent: output depends only on the SKILL.md inputs.

import { readdirSync, readFileSync, lstatSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { join, dirname, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import YAML from 'yaml';

const siteDir = dirname(dirname(fileURLToPath(import.meta.url)));
const repoRoot = dirname(siteDir);
const skillsRoot = join(repoRoot, 'skills');
const outDir = join(siteDir, 'docs-catalog');

/** Recursively find SKILL.md files, skipping imported/ and node_modules. */
function findSkillFiles(dir) {
  const out = [];
  for (const entry of readdirSync(dir)) {
    if (entry === 'imported' || entry === 'node_modules' || entry.startsWith('.')) continue;
    const p = join(dir, entry);
    let st;
    try {
      st = lstatSync(p); // lstat: never follow symlinks (repo tooling manages
      // symlink farms; following them risks cycles and dangling-link crashes)
    } catch {
      console.warn(`[skills-catalog] WARN: cannot stat ${p} — skipped`);
      continue;
    }
    if (st.isSymbolicLink()) continue;
    if (st.isDirectory()) {
      out.push(...findSkillFiles(p));
    } else if (entry === 'SKILL.md') {
      out.push(p);
    }
  }
  return out;
}

/** Parse frontmatter; returns null when absent/invalid (validator's job, not ours). */
function parseFrontmatter(file) {
  const text = readFileSync(file, 'utf8');
  // BOM- and CRLF-tolerant: a CRLF SKILL.md must not silently lose its metadata
  const m = text.match(/^﻿?---\r?\n([\s\S]*?)\r?\n---/);
  if (!m) {
    console.warn(`[skills-catalog] WARN: no frontmatter in ${file} — degraded entry`);
    return null;
  }
  try {
    return YAML.parse(m[1]);
  } catch {
    console.warn(`[skills-catalog] WARN: unparseable frontmatter in ${file} — degraded entry`);
    return null;
  }
}

const files = findSkillFiles(skillsRoot).sort();
const byCategory = new Map();
let count = 0;

for (const file of files) {
  const fm = parseFrontmatter(file);
  const rel = relative(skillsRoot, file); // e.g. process/adversarial-review/SKILL.md
  const category = rel.split('/')[0];
  const name = (fm && fm.name) || rel.split('/').slice(-2, -1)[0];
  const description = ((fm && fm.description) || '').toString().trim().replace(/\s+/g, ' ');
  const tags = (fm && fm.metadata && Array.isArray(fm.metadata.tags)) ? fm.metadata.tags : [];
  const version = (fm && (fm.version || (fm.metadata && fm.metadata.version))) || '';
  const isSub = rel.split('/').length > 3; // nested under a parent skill's skills/
  if (!byCategory.has(category)) byCategory.set(category, []);
  byCategory.get(category).push({ name, description, tags, version, rel, isSub });
  count += 1;
}

rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

const esc = (s) => s.replace(/[<>{}]/g, (c) => ({ '<': '&lt;', '>': '&gt;', '{': '&#123;', '}': '&#125;' }[c]));

const categories = [...byCategory.keys()].sort();
let indexRows = '';
for (const cat of categories) {
  const skills = byCategory.get(cat);
  indexRows += `| [${cat}](./${cat}) | ${skills.length} |\n`;

  let body = `---\ntitle: ${cat} skills\nsidebar_label: ${cat}\n---\n\n# ${cat} skills\n\n`;
  body += `${skills.length} skills. Source of truth: [\`skills/${cat}/\`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/skills/${cat}).\n\n`;
  // explicit locale: output must not depend on the build machine's ICU
  for (const s of skills.sort((a, b) => a.name.localeCompare(b.name, 'en'))) {
    body += `## ${esc(s.name)}${s.isSub ? ' *(sub-skill)*' : ''}\n\n`;
    if (s.description) body += `${esc(s.description)}\n\n`;
    const meta = [];
    if (s.version) meta.push(`v${s.version}`);
    if (s.tags.length) meta.push(s.tags.map((t) => `\`${t}\``).join(' '));
    meta.push(`[source](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/skills/${s.rel})`);
    body += `${meta.join(' · ')}\n\n`;
  }
  writeFileSync(join(outDir, `${cat}.md`), body);
}

writeFileSync(
  join(outDir, 'index.md'),
  `---\ntitle: Skills Catalog\nsidebar_label: Overview\n---\n\n# Skills Catalog\n\n` +
  `Generated at build time from SKILL.md frontmatter — **${count} skills** across ${categories.length} categories ` +
  `(excludes \`skills/imported/\` submodules).\n\n| Category | Skills |\n|---|---|\n${indexRows}`,
);

console.log(`[skills-catalog] generated ${count} skills across ${categories.length} categories -> ${relative(siteDir, outDir)}`);
