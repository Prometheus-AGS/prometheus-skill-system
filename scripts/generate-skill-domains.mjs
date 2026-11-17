#!/usr/bin/env node
// Generate shared/skill-domains.json: which distributed skill belongs to which
// install domain. The installer reads it to project only the selected domains
// into a harness's skill root (install-plugin-generation.js --domains).
//
//   node scripts/generate-skill-domains.mjs           write the file
//   node scripts/generate-skill-domains.mjs --check   fail when it is out of date
//
// Inputs: `domains` in skill-system.json, the domain plugin manifests they name,
// and the distribution inventory. Every distributed skill must land in exactly
// one domain; anything else fails here, not at install time.
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

import { collectDistributionSkills, readSkillSystem } from './lib/skill-system.js';
import { DOMAIN_ID_PATTERN, DOMAIN_INDEX_FILE, DOMAIN_INDEX_SCHEMA } from './lib/skill-domains.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const yaml = createRequire(import.meta.url)('js-yaml');
const MAX_DESCRIPTION = 1536; // Claude Code's per-entry listing cap
const ENTRY_OVERHEAD = 6;

function fail(message) {
  console.error(`generate-skill-domains: ${message}`);
  process.exit(1);
}

function frontmatter(skillDirectory) {
  const text = fs.readFileSync(path.join(skillDirectory, 'SKILL.md'), 'utf8');
  const match = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  try {
    return match ? (yaml.load(match[1]) ?? {}) : {};
  } catch {
    return {};
  }
}

function listingChars(skill) {
  const fm = frontmatter(skill.source);
  if (fm['disable-model-invocation'] === true) return 0;
  const description = `${fm.description ?? ''} ${fm.when_to_use ?? ''}`.trim();
  return Math.min(description.length, MAX_DESCRIPTION) + skill.name.length + ENTRY_OVERHEAD;
}

/** The skill directories a domain plugin registers, as absolute paths. */
function pluginSkillDirectories(entry) {
  const base = path.join(root, entry.path);
  const manifestFile = path.join(base, '.claude-plugin/plugin.json');
  if (!fs.existsSync(manifestFile)) fail(`plugin ${entry.name} has no manifest at ${entry.path}`);
  const manifest = JSON.parse(fs.readFileSync(manifestFile, 'utf8'));
  if (Array.isArray(manifest.skills)) {
    return manifest.skills.map(relative => path.resolve(base, relative));
  }
  if (fs.existsSync(path.join(base, 'SKILL.md'))) return [base];
  const nested = path.join(base, 'skills');
  if (!fs.existsSync(nested)) return [];
  return fs.readdirSync(nested).map(name => path.join(nested, name));
}

function build() {
  const contract = readSkillSystem(root, { requireTargetSourceTrees: false });
  if (!Array.isArray(contract.domains) || contract.domains.length === 0) {
    fail('skill-system.json declares no domains');
  }
  const skills = collectDistributionSkills(root, contract);
  const bySource = new Map(skills.map(skill => [path.resolve(skill.source), skill]));
  const plugins = new Map(contract.marketplace.plugins.map(entry => [entry.name, entry]));
  const roots = new Map(contract.inventory.roots.map(entry => [entry.id, entry]));
  const owner = new Map();
  const domains = [];
  const ids = new Set(contract.domains.map(domain => domain.id));
  if (ids.size !== contract.domains.length) fail('skill-system.json repeats a domain id');

  const assign = (skill, domain) => {
    if (owner.has(skill.name)) {
      fail(`skill ${skill.name} is in two domains: ${owner.get(skill.name)} and ${domain.id}`);
    }
    owner.set(skill.name, domain.id);
  };

  // Plugins first, so an inventory root can then take whatever is left under it.
  for (const domain of contract.domains) {
    if (!DOMAIN_ID_PATTERN.test(domain.id ?? '')) fail(`invalid domain id: ${domain.id}`);
    for (const required of domain.requires ?? []) {
      if (!ids.has(required)) fail(`domain ${domain.id} requires unknown domain ${required}`);
    }
    for (const name of domain.plugins ?? []) {
      const entry = plugins.get(name);
      if (!entry) fail(`domain ${domain.id} names unknown plugin ${name}`);
      for (const directory of pluginSkillDirectories(entry)) {
        const skill = bySource.get(directory);
        if (skill) assign(skill, domain);
      }
    }
  }
  for (const domain of contract.domains) {
    for (const id of domain.inventoryRoots ?? []) {
      const entry = roots.get(id);
      if (!entry) fail(`domain ${domain.id} names unknown inventory root ${id}`);
      const prefix = path.resolve(root, entry.path);
      for (const skill of skills) {
        const source = path.resolve(skill.source);
        if (owner.has(skill.name)) continue;
        if (source === prefix || source.startsWith(`${prefix}${path.sep}`)) assign(skill, domain);
      }
    }
  }
  const unassigned = skills.filter(skill => !owner.has(skill.name)).map(skill => skill.name);
  if (unassigned.length) fail(`skill(s) in no domain: ${unassigned.join(', ')}`);

  for (const domain of contract.domains) {
    const members = skills.filter(skill => owner.get(skill.name) === domain.id);
    if (members.length === 0) fail(`domain ${domain.id} has no skills`);
    domains.push({
      id: domain.id,
      requires: [...(domain.requires ?? [])].sort(),
      skills: members.map(skill => skill.name).sort(),
      listingChars: members.reduce((total, skill) => total + listingChars(skill), 0),
    });
  }
  domains.sort((left, right) => left.id.localeCompare(right.id));
  return `${JSON.stringify({ schemaVersion: DOMAIN_INDEX_SCHEMA, domains }, null, 2)}\n`;
}

const output = path.join(root, ...DOMAIN_INDEX_FILE.split('/'));
const content = build();
if (process.argv.includes('--check')) {
  const current = fs.existsSync(output) ? fs.readFileSync(output, 'utf8') : '';
  if (current !== content)
    fail(`${DOMAIN_INDEX_FILE} is out of date; run npm run generate:skill-domains`);
  console.log(`✓ ${DOMAIN_INDEX_FILE} is current`);
} else {
  fs.writeFileSync(output, content);
  const index = JSON.parse(content);
  console.log(
    `wrote ${DOMAIN_INDEX_FILE}: ${index.domains.length} domains, ${index.domains.reduce((n, d) => n + d.skills.length, 0)} skills`
  );
}
