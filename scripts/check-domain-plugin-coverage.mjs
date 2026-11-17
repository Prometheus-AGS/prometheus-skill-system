#!/usr/bin/env node
// Every distributed skill must be reachable through a domain plugin, and every
// domain plugin's skill listing must fit a harness's listing budget.
//
// Why: the full pack lists more skills than Claude Code presents in full, so
// skills stop triggering (docs/reports/2026-11-17-skill-catalogue-budget.md).
// Domain plugins are the install that fits. Before this check existed, 101 of
// 196 native skills were in no domain plugin, so the full pack was the only way
// to get them, and nothing noticed.
//
//   node scripts/check-domain-plugin-coverage.mjs           report and fail on gaps
//   node scripts/check-domain-plugin-coverage.mjs --json    machine-readable
//
// A skill nested inside another skill's directory must be listed by its own
// path: Claude Code registers only the skills a manifest names (or the
// top-level directories of a `skills/` folder), never nested ones.
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const yaml = createRequire(import.meta.url)('js-yaml');
const asJson = process.argv.includes('--json');

// Estimated listing budget on a 1M-token model (1% of the window at about four
// characters per token). A domain plugin should leave room for the user's other
// skills, so the ceiling is half of that.
const LISTING_CEILING_CHARS = 20_000;
const MAX_DESCRIPTION = 1536;
const ENTRY_OVERHEAD = 6;
const SKIP = new Set(['imported', 'tests', 'fixtures', 'evals', 'node_modules', 'target']);

const read = file => JSON.parse(fs.readFileSync(path.join(root, file), 'utf8'));
function frontmatter(file) {
  const match = fs.readFileSync(file, 'utf8').match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!match) return null;
  try {
    return yaml.load(match[1]);
  } catch {
    return null;
  }
}

// Inventory: every native skill the pack distributes.
const inventory = [];
const visit = directory => {
  for (const name of fs.readdirSync(directory).sort()) {
    if (SKIP.has(name)) continue;
    const child = path.join(directory, name);
    if (!fs.lstatSync(child).isDirectory()) continue;
    const file = path.join(child, 'SKILL.md');
    if (fs.existsSync(file)) {
      const fm = frontmatter(file);
      if (fm?.name) inventory.push({ name: String(fm.name), dir: path.relative(root, child), fm });
    }
    visit(child);
  }
};
visit(path.join(root, 'skills'));

// Domain plugins: the marketplace entries of skill-system.json that live under skills/.
const contract = read('skill-system.json');
const plugins = [];
const owner = new Map();
const problems = [];
for (const entry of contract.marketplace.plugins) {
  if (!entry.path.startsWith('skills/')) continue;
  const manifestFile = path.join(entry.path, '.claude-plugin/plugin.json');
  if (!fs.existsSync(path.join(root, manifestFile))) {
    problems.push(`${entry.name}: no manifest at ${manifestFile}`);
    continue;
  }
  const manifest = read(manifestFile);
  if (manifest.name !== entry.name) problems.push(`${entry.name}: manifest is named ${manifest.name}`);
  if (manifest.version !== entry.version) {
    problems.push(`${entry.name}: manifest version ${manifest.version} differs from skill-system.json ${entry.version}`);
  }
  let dirs = [];
  if (Array.isArray(manifest.skills)) dirs = manifest.skills.map(p => path.normalize(path.join(entry.path, p)).replace(/\/+$/, ''));
  else if (fs.existsSync(path.join(root, entry.path, 'SKILL.md'))) dirs = [entry.path];
  else if (fs.existsSync(path.join(root, entry.path, 'skills'))) {
    dirs = fs.readdirSync(path.join(root, entry.path, 'skills')).map(n => path.join(entry.path, 'skills', n));
  }
  let chars = 0;
  let count = 0;
  for (const dir of dirs) {
    const file = path.join(root, dir, 'SKILL.md');
    if (!fs.existsSync(file)) {
      problems.push(`${entry.name}: lists ${dir}, which has no SKILL.md`);
      continue;
    }
    if (owner.has(dir)) problems.push(`${dir} is listed by both ${owner.get(dir)} and ${entry.name}`);
    owner.set(dir, entry.name);
    const fm = frontmatter(file);
    if (!fm?.name || fm['disable-model-invocation'] === true) continue;
    const description = `${fm.description ?? ''} ${fm.when_to_use ?? ''}`.trim();
    chars += Math.min(description.length, MAX_DESCRIPTION) + String(fm.name).length + entry.name.length + 1 + ENTRY_OVERHEAD;
    count += 1;
  }
  if (chars > LISTING_CEILING_CHARS) {
    problems.push(`${entry.name}: listing is about ${chars} chars, over the ${LISTING_CEILING_CHARS} ceiling`);
  }
  plugins.push({ name: entry.name, path: entry.path, skills: dirs.length, listed: count, listingChars: chars });
}

const uncovered = inventory.filter(skill => !owner.has(skill.dir));
for (const skill of uncovered) problems.push(`${skill.name} (${skill.dir}) is in no domain plugin`);

const total = plugins.reduce((sum, plugin) => sum + plugin.listingChars, 0);
if (asJson) {
  console.log(JSON.stringify({ inventory: inventory.length, uncovered: uncovered.map(s => s.dir), plugins, totalListingChars: total, problems }, null, 2));
} else {
  for (const plugin of plugins.sort((a, b) => b.listingChars - a.listingChars)) {
    console.log(`${plugin.name.padEnd(34)} ${String(plugin.skills).padStart(3)} skills  about ${String(plugin.listingChars).padStart(6)} listing chars`);
  }
  console.log(`${inventory.length} native skills; ${inventory.length - uncovered.length} in a domain plugin; all domain plugins together about ${total} listing chars`);
  for (const problem of problems) console.error(`✗ ${problem}`);
  if (problems.length === 0) console.log('✓ every native skill is in exactly one domain plugin and every plugin fits the ceiling');
}
process.exit(problems.length === 0 ? 0 : 1);
