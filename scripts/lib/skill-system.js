import fs from 'node:fs';
import path from 'node:path';

export function readSkillSystem(sourceRoot) {
  const file = path.join(sourceRoot, 'skill-system.json');
  if (!fs.existsSync(file)) throw new Error(`distribution contract is missing: ${file}`);
  const contract = JSON.parse(fs.readFileSync(file, 'utf8'));
  if (contract.schemaVersion !== 'prometheus-skill-system-v1') {
    throw new Error(`unsupported distribution contract: ${contract.schemaVersion ?? 'missing'}`);
  }
  if (!Array.isArray(contract.targets) || contract.targets.length !== 14) {
    throw new Error('distribution contract must declare the canonical 14 targets');
  }
  if (compareVersions(contract.releaseVersion, contract.minimumActiveVersion) < 0) {
    throw new Error('releaseVersion cannot be below minimumActiveVersion');
  }
  return contract;
}

export function compareVersions(left, right) {
  const parse = value => {
    const match = String(value).match(/^(\d+)\.(\d+)\.(\d+)(?:[-+].*)?$/);
    if (!match) throw new Error(`invalid semantic version: ${value}`);
    return match.slice(1).map(Number);
  };
  const a = parse(left);
  const b = parse(right);
  for (let index = 0; index < 3; index += 1) {
    if (a[index] !== b[index]) return a[index] < b[index] ? -1 : 1;
  }
  return 0;
}

export function assertMinimumActiveVersion(version, contract, context) {
  if (compareVersions(version, contract.minimumActiveVersion) < 0) {
    throw new Error(
      `${context} ${version} is below minimum active version ${contract.minimumActiveVersion}`
    );
  }
}

export function targetsById(contract, selection = 'all') {
  const wanted = selection === 'all'
    ? new Set(contract.targets.map(target => target.id))
    : new Set(String(selection).split(',').map(value => value.trim()).filter(Boolean));
  const unknown = [...wanted].filter(id => !contract.targets.some(target => target.id === id));
  if (unknown.length) throw new Error(`unknown target id(s): ${unknown.join(', ')}`);
  return contract.targets.filter(target => wanted.has(target.id));
}

export function verifyImportPins(sourceRoot, contract, profile) {
  const required = contract.imports.filter(entry => entry.requiredFor?.includes(profile));
  for (const entry of required) {
    const absolute = path.join(sourceRoot, entry.path);
    if (!fs.existsSync(absolute)) throw new Error(`required import is unavailable: ${entry.path}`);
    const gitFile = path.join(absolute, '.git');
    if (!fs.existsSync(gitFile)) throw new Error(`required import is not initialized: ${entry.path}`);
  }
  return required;
}

function skillName(skillFile) {
  const text = fs.readFileSync(skillFile, 'utf8');
  const frontmatter = text.match(/^---\r?\n([\s\S]*?)\r?\n---/)?.[1] ?? '';
  const match = frontmatter.match(/^name:\s*['"]?([^'"\r\n]+)['"]?/m);
  const name = match?.[1].trim() || path.basename(path.dirname(skillFile));
  if (!/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(name)) throw new Error(`unsafe skill name in ${skillFile}: ${name}`);
  return name;
}

export function collectDistributionSkills(sourceRoot, contract) {
  const result = [];
  const add = directory => {
    const skillFile = path.join(directory, 'SKILL.md');
    if (fs.existsSync(skillFile)) result.push({ name: skillName(skillFile), source: directory });
  };
  const visit = (directory, exclusions) => {
    for (const name of fs.readdirSync(directory).sort()) {
      const child = path.join(directory, name);
      const stat = fs.lstatSync(child);
      if (!stat.isDirectory() || stat.isSymbolicLink()) continue;
      if (exclusions.has(name)) continue;
      add(child);
      visit(child, exclusions);
    }
  };
  for (const root of contract.inventory.roots) {
    const absolute = path.join(sourceRoot, root.path);
    if (!fs.existsSync(absolute)) throw new Error(`skill inventory root is unavailable: ${root.path}`);
    if (root.scan === 'root') add(absolute);
    else if (root.scan === 'children') {
      for (const name of fs.readdirSync(absolute).sort()) {
        if (root.excludeNames?.includes(name)) continue;
        const child = path.join(absolute, name);
        if (fs.lstatSync(child).isDirectory()) add(child);
      }
    } else if (root.scan === 'recursive') {
      visit(absolute, new Set(root.excludeSegments ?? []));
    } else throw new Error(`unsupported inventory scan mode: ${root.scan}`);
  }
  result.sort((left, right) => left.name.localeCompare(right.name));
  for (let index = 1; index < result.length; index += 1) {
    if (result[index - 1].name === result[index].name) {
      throw new Error(`duplicate distributed skill name: ${result[index].name}`);
    }
  }
  return result;
}
