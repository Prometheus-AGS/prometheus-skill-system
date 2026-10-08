import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { jcs } from './jcs.js';

export const REVIEWED_SKILL_CLOSURES_FILE = 'reviewed-skill-closures.json';
export const REVIEWED_SKILL_CLOSURES_SCHEMA = 'prometheus-reviewed-skill-closures-v1';

function digest(bytes) {
  return `sha256:${crypto.createHash('sha256').update(bytes).digest('hex')}`;
}

function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function relative(root, absolute) {
  const entry = path.relative(root, absolute).split(path.sep).join('/');
  if (!entry || entry === '..' || entry.startsWith('../') || path.posix.isAbsolute(entry)) {
    throw new Error(`reviewed closure path escapes payload: ${absolute}`);
  }
  return entry;
}

function collectFiles(root, location, result) {
  const stat = fs.lstatSync(location);
  if (stat.isSymbolicLink()) throw new Error(`reviewed closure refuses symlink: ${relative(root, location)}`);
  if (stat.isDirectory()) {
    for (const name of fs.readdirSync(location).sort(compare)) {
      const child = path.join(location, name);
      if (relative(root, child) === REVIEWED_SKILL_CLOSURES_FILE) continue;
      collectFiles(root, child, result);
    }
    return;
  }
  if (!stat.isFile()) throw new Error(`reviewed closure refuses non-file: ${relative(root, location)}`);
  const entry = relative(root, location);
  result.set(entry, { path: entry, sha256: digest(fs.readFileSync(location)) });
}

function readIdentity(skillFile) {
  const bytes = fs.readFileSync(skillFile);
  const frontmatter = bytes.toString('utf8').match(/^---\r?\n([\s\S]*?)\r?\n---/)?.[1] ?? '';
  const scalar = key => frontmatter.match(new RegExp(`^${key}:\\s*['\"]?([^'\"\\r\\n]+)['\"]?\\s*$`, 'm'))?.[1].trim() ?? null;
  const metadataVersion = frontmatter.match(/^metadata:\s*\r?\n(?:[ \t]+[^\r\n]*\r?\n)*?[ \t]+version:\s*['\"]?([^'\"\r\n]+)['\"]?\s*$/m)?.[1].trim() ?? null;
  const name = scalar('name');
  if (!name) throw new Error(`reviewed closure requires skill frontmatter name: ${skillFile}`);
  return { id: scalar('id'), name, version: scalar('version') ?? metadataVersion, artifactDigest: digest(bytes) };
}

function findSkills(root) {
  const skillsRoot = path.join(root, 'skills');
  if (!fs.existsSync(skillsRoot)) throw new Error(`reviewed closure requires skills root: ${skillsRoot}`);
  const skills = [];
  const visit = directory => {
    const stat = fs.lstatSync(directory);
    if (stat.isSymbolicLink()) throw new Error(`reviewed closure refuses skill symlink: ${relative(root, directory)}`);
    if (!stat.isDirectory()) return;
    const skillFile = path.join(directory, 'SKILL.md');
    if (fs.existsSync(skillFile)) skills.push({ directory, skillFile });
    for (const name of fs.readdirSync(directory).sort(compare)) visit(path.join(directory, name));
  };
  visit(skillsRoot);
  return skills;
}

function writeJson(file, value) {
  const temporary = `${file}.${process.pid}.tmp`;
  fs.writeFileSync(temporary, `${JSON.stringify(value, null, 2)}\n`, { mode: 0o644, flag: 'wx' });
  fs.renameSync(temporary, file);
}

/**
 * Build a conservative complete closure from the payload already staged for distribution.
 * A skill covers every file in its own directory and every shared runtime root shipped with it.
 */
export function writeReviewedSkillClosures(payloadRoot, { sharedRoots = [] } = {}) {
  const root = path.resolve(payloadRoot);
  const runtimeRoots = sharedRoots
    .map(entry => {
      const absolute = path.resolve(root, entry);
      if (!fs.existsSync(absolute)) throw new Error(`reviewed closure shared root is missing: ${entry}`);
      return relative(root, absolute);
    })
    .filter((entry, index, values) => values.indexOf(entry) === index)
    .sort(compare);
  const sharedFiles = new Map();
  for (const entry of runtimeRoots) collectFiles(root, path.join(root, ...entry.split('/')), sharedFiles);
  const files = new Map(sharedFiles);
  const skills = findSkills(root).map(({ directory, skillFile }) => {
    const ownFiles = new Map();
    collectFiles(root, directory, ownFiles);
    for (const [entry, file] of ownFiles) files.set(entry, file);
    const closurePaths = [...new Set([...sharedFiles.keys(), ...ownFiles.keys()])].sort(compare);
    const closureFiles = closurePaths.map(entry => files.get(entry));
    const skillRoot = relative(root, directory);
    return {
      identity: readIdentity(skillFile),
      entrypoint: `${skillRoot}/SKILL.md`,
      roots: [skillRoot, ...runtimeRoots],
      closure: { paths: closurePaths, digest: digest(jcs({ files: closureFiles })) },
    };
  });
  skills.sort((left, right) => compare(left.entrypoint, right.entrypoint));
  const body = {
    schemaVersion: REVIEWED_SKILL_CLOSURES_SCHEMA,
    hashAlgorithm: 'sha256',
    files: [...files.values()].sort((left, right) => compare(left.path, right.path)),
    skills,
  };
  const inventory = { ...body, inventoryDigest: digest(jcs(body)) };
  writeJson(path.join(root, REVIEWED_SKILL_CLOSURES_FILE), inventory);
  return inventory;
}
