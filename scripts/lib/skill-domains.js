// Skill domains: which skills an install projects into a harness's skill root.
//
// The domain index is GENERATED (scripts/generate-skill-domains.mjs) into
// shared/skill-domains.json and ships in every payload, because the installer
// usually runs from a packaged payload whose skills are flat and whose domain
// plugin manifests are gone. This module only reads and resolves it, and has no
// dependency outside node, so the packaged installer can import it.
import fs from 'node:fs';
import path from 'node:path';

export const DOMAIN_INDEX_FILE = 'shared/skill-domains.json';
export const DOMAIN_INDEX_SCHEMA = 'prometheus-skill-domains-v1';
export const DOMAIN_ID_PATTERN = /^[a-z0-9][a-z0-9-]*$/;

/** The domain index under `root`, or null when that payload ships none. */
export function readDomainIndex(root) {
  const file = path.join(root, ...DOMAIN_INDEX_FILE.split('/'));
  if (!fs.lstatSync(file, { throwIfNoEntry: false })?.isFile()) return null;
  const index = JSON.parse(fs.readFileSync(file, 'utf8'));
  if (index.schemaVersion !== DOMAIN_INDEX_SCHEMA || !Array.isArray(index.domains)) {
    throw new Error(`unsupported skill domain index: ${file}`);
  }
  const ids = new Set();
  const owners = new Map();
  for (const domain of index.domains) {
    if (!DOMAIN_ID_PATTERN.test(domain?.id ?? '') || ids.has(domain.id)) {
      throw new Error(`skill domain index has an invalid or repeated id: ${domain?.id}`);
    }
    ids.add(domain.id);
    if (!Array.isArray(domain.skills) || !Array.isArray(domain.requires)) {
      throw new Error(`skill domain ${domain.id} is malformed`);
    }
    for (const skill of domain.skills) {
      if (owners.has(skill)) {
        throw new Error(`skill ${skill} is in two domains: ${owners.get(skill)} and ${domain.id}`);
      }
      owners.set(skill, domain.id);
    }
  }
  for (const domain of index.domains) {
    const unknown = domain.requires.filter(id => !ids.has(id));
    if (unknown.length) {
      throw new Error(
        `skill domain ${domain.id} requires unknown domain(s): ${unknown.join(', ')}`
      );
    }
  }
  return index;
}

/** Parse a `--domains` style value into a sorted, de-duplicated id list. */
export function parseDomainList(value, flag) {
  const ids = String(value ?? '')
    .split(',')
    .map(entry => entry.trim())
    .filter(Boolean);
  if (ids.length === 0) throw new Error(`missing value for ${flag}`);
  const invalid = ids.filter(id => !DOMAIN_ID_PATTERN.test(id));
  if (invalid.length) throw new Error(`invalid domain id(s) for ${flag}: ${invalid.join(', ')}`);
  return [...new Set(ids)].sort();
}

/**
 * Expand requested domain ids through `requires` and return the skills they
 * name. Ids the index does not know are returned in `unknown`, never guessed.
 */
export function resolveDomains(index, requested) {
  const byId = new Map(index.domains.map(domain => [domain.id, domain]));
  const unknown = requested.filter(id => !byId.has(id));
  const resolved = new Set();
  const visit = id => {
    if (resolved.has(id) || !byId.has(id)) return;
    resolved.add(id);
    for (const required of byId.get(id).requires) visit(required);
  };
  for (const id of requested) visit(id);
  const domains = [...resolved].sort();
  const skills = new Set(domains.flatMap(id => byId.get(id).skills));
  return { domains, skills, unknown };
}

/** Every skill name the index assigns to a domain. */
export function indexedSkills(index) {
  return new Set(index.domains.flatMap(domain => domain.skills));
}
