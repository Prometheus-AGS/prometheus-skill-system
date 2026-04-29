#!/usr/bin/env node
// Generates Claude Code native slash command files from skills SKILL.md frontmatter.
// Each output file: description frontmatter + file-reference body + $ARGUMENTS.
// Usage: node scripts/generate-commands.js --output ~/.claude/commands

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { homedir } from 'os';
import yaml from 'js-yaml';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const SKILLS_DIR = path.join(ROOT, 'skills');
const SKIP_DIRS = new Set(['imported', 'node_modules', 'target', 'dist', '.git']);

const args = process.argv.slice(2);
const outputIdx = args.indexOf('--output');
const outputDir = outputIdx !== -1
  ? path.resolve(args[outputIdx + 1].replace(/^~/, homedir()))
  : path.join(homedir(), '.claude', 'commands');

fs.mkdirSync(outputDir, { recursive: true });

function findSkillMds(dir, results = []) {
  let entries;
  try { entries = fs.readdirSync(dir, { withFileTypes: true }); } catch { return results; }
  for (const entry of entries) {
    if (SKIP_DIRS.has(entry.name) || entry.name.startsWith('.')) continue;
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) findSkillMds(fullPath, results);
    else if (entry.name === 'SKILL.md') results.push(fullPath);
  }
  return results;
}

function parseFrontmatter(content) {
  const match = content.match(/^---\n([\s\S]*?)\n---/);
  if (!match) return null;
  try { return yaml.load(match[1]); } catch { return null; }
}

const skillMds = findSkillMds(SKILLS_DIR);
let generated = 0;
let skipped = 0;

for (const skillMdPath of skillMds) {
  const content = fs.readFileSync(skillMdPath, 'utf-8');
  const fm = parseFrontmatter(content);
  if (!fm?.name || !fm?.description) { skipped++; continue; }

  const relPath = path.relative(ROOT, skillMdPath).replace(/\\/g, '/');
  const desc = fm.description.slice(0, 200).replace(/"/g, '\\"');
  const cmdContent = `---\ndescription: "${desc}"\n---\n\n{{file:${relPath}}}\n\n$ARGUMENTS\n`;

  fs.writeFileSync(path.join(outputDir, `${fm.name}.md`), cmdContent, 'utf-8');
  generated++;
}

console.log(`✅ Generated ${generated} command file(s) → ${outputDir} (skipped ${skipped})`);
