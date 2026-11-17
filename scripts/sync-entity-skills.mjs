#!/usr/bin/env node
// Keep skills/react/prometheus-entity-skills identical to the entity skills in
// the pinned prometheus-entity-management submodule.
//
// The pack ships a checked-in copy of those skills. Before this script existed
// nothing compared the two, and the copy drifted to a state older than upstream
// v2.1.0 while the submodule was pinned at v4.2.0. Upstream is the source of
// truth; this copy is generated.
//
//   node scripts/sync-entity-skills.mjs           rewrite the copy from the submodule
//   node scripts/sync-entity-skills.mjs --check   exit 1 and list differences on drift
//
// Pack-only entries are never touched or compared. Frontmatter is normalised
// to the Agent Skills specification (metadata.version, string-valued tags) on
// the way in, and a skill with no version at all is given DEFAULT_VERSION,
// because the pack's strict validation requires one. Once upstream carries
// both, the normalisation is a no-op.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SOURCE = path.join(root, 'skills/imported/prometheus-entity-management/prometheus-entity-skills');
const TARGET = path.join(root, 'skills/react/prometheus-entity-skills');

// Paths, relative to TARGET, that exist only in the pack.
const PACK_ONLY = ['.claude-plugin', 'entity-graph-optimize/component', 'entity-graph-optimize/skill.wasm'];
const SKIP_NAMES = new Set(['.git', 'node_modules', '.DS_Store']);
const DEFAULT_VERSION = '1.0.0';

const check = process.argv.includes('--check');
const isPackOnly = relative => PACK_ONLY.some(entry => relative === entry || relative.startsWith(`${entry}/`));
const unquote = value => value.trim().replace(/^(['"])(.*)\1$/, '$2');

// Text-level so nothing else in the frontmatter is reformatted.
function normaliseFrontmatter(text) {
  const match = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!match) return text;
  let lines = match[1].split('\n');
  const metadataIndex = lines.findIndex(line => /^metadata:\s*$/.test(line));
  if (metadataIndex < 0) return text;
  lines = lines.map(line => {
    const tags = line.match(/^(\s+tags:\s*)\[(.*)\]\s*$/);
    return tags ? `${tags[1]}"${tags[2].split(',').map(unquote).filter(Boolean).join(', ')}"` : line;
  });
  const topVersion = lines.findIndex(line => /^version:\s*\S/.test(line));
  if (topVersion >= 0) {
    const value = unquote(lines[topVersion].replace(/^version:\s*/, '').replace(/\s+#.*$/, ''));
    lines.splice(topVersion, 1);
    const metadata = lines.findIndex(line => /^metadata:\s*$/.test(line));
    let end = metadata + 1;
    while (end < lines.length && /^\s+\S/.test(lines[end])) end += 1;
    const existing = lines.slice(metadata + 1, end).findIndex(line => /^ {2}version:\s*\S/.test(line));
    if (existing >= 0) lines[metadata + 1 + existing] = `  version: '${value}'`;
    else lines.splice(metadata + 1, 0, `  version: '${value}'`);
  }
  if (!lines.some(line => /^ {2}version:\s*\S/.test(line))) {
    lines.splice(lines.findIndex(line => /^metadata:\s*$/.test(line)) + 1, 0, `  version: '${DEFAULT_VERSION}'`);
  }
  return text.replace(match[1], () => lines.join('\n'));
}

function listFiles(base) {
  const files = [];
  const visit = directory => {
    for (const name of fs.readdirSync(directory).sort()) {
      if (SKIP_NAMES.has(name)) continue;
      const absolute = path.join(directory, name);
      const stat = fs.lstatSync(absolute);
      if (stat.isDirectory()) visit(absolute);
      else if (stat.isFile() || stat.isSymbolicLink()) files.push(path.relative(base, absolute));
    }
  };
  visit(base);
  return files;
}

function expectedBytes(relative) {
  const bytes = fs.readFileSync(path.join(SOURCE, relative));
  return path.basename(relative) === 'SKILL.md' ? Buffer.from(normaliseFrontmatter(bytes.toString('utf8'))) : bytes;
}

if (!fs.existsSync(SOURCE)) {
  // The submodule is optional in a partial checkout. Absence is not drift.
  console.log('entity skills: submodule not initialised; nothing to compare.');
  process.exit(0);
}

const sourceFiles = listFiles(SOURCE);
const targetFiles = fs.existsSync(TARGET) ? listFiles(TARGET).filter(relative => !isPackOnly(relative)) : [];
const sourceSet = new Set(sourceFiles);
const differences = [];

for (const relative of sourceFiles) {
  const destination = path.join(TARGET, relative);
  const wanted = expectedBytes(relative);
  const current = fs.existsSync(destination) ? fs.readFileSync(destination) : null;
  if (current && current.equals(wanted)) continue;
  differences.push(`${current ? 'differs' : 'missing'}: ${relative}`);
  if (!check) {
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, wanted);
    fs.chmodSync(destination, fs.statSync(path.join(SOURCE, relative)).mode & 0o777);
  }
}
for (const relative of targetFiles) {
  if (sourceSet.has(relative)) continue;
  differences.push(`not upstream: ${relative}`);
  if (!check) fs.rmSync(path.join(TARGET, relative));
}

if (differences.length === 0) {
  console.log(`entity skills: in sync with the pinned submodule (${sourceFiles.length} files).`);
  process.exit(0);
}
if (check) {
  console.error(`entity skills: ${differences.length} difference(s) from the pinned submodule:`);
  for (const line of differences.slice(0, 40)) console.error(`  ${line}`);
  if (differences.length > 40) console.error(`  ... and ${differences.length - 40} more`);
  console.error('Run: npm run sync:entity-skills');
  process.exit(1);
}
console.log(`entity skills: updated ${differences.length} file(s) from the pinned submodule.`);
