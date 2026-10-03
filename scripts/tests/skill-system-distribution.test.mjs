import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { canonicalBytes } from '../lib/canonical-bytes.js';
import { collectDistributionSkills, readSkillSystem } from '../lib/skill-system.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const contract = readSkillSystem(root);
const skills = collectDistributionSkills(root, contract);

const packageVersion = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8')).version;
assert.equal(contract.releaseVersion, packageVersion);
assert.equal(contract.minimumActiveVersion, packageVersion);
assert.equal(contract.targets.length, 14);
assert.equal(new Set(skills.map(skill => skill.name)).size, skills.length);
assert(skills.some(skill => skill.name === 'artifact-refiner'));
assert(skills.some(skill => skill.name === 'sycophancy-correction'));
assert(!skills.some(skill => skill.source.includes('prometheus-entity-management')));
assert(!skills.some(skill => skill.source.includes('artifact-refiner/shared/sycophancy-correction')));

function digestTree(directory, relative = '') {
  const result = [];
  for (const name of fs.readdirSync(path.join(directory, relative)).sort()) {
    if (['.git', 'node_modules', 'target', '.kbd-orchestrator'].includes(name)) continue;
    const child = path.join(relative, name);
    const absolute = path.join(directory, child);
    const stat = fs.lstatSync(absolute);
    if (stat.isDirectory()) result.push(...digestTree(directory, child));
    else {
      const bytes = stat.isSymbolicLink() ? Buffer.from(fs.readlinkSync(absolute)) : canonicalBytes(absolute);
      result.push({
        path: child.split(path.sep).join('/'),
        mode: stat.mode & 0o7777,
        sha256: crypto.createHash('sha256').update(bytes).digest('hex'),
      });
    }
  }
  return result;
}

for (const platform of ['claude', 'codex']) {
  const packageRoot = path.join(root, 'dist/plugins', platform, contract.name);
  const packagedSkills = path.join(packageRoot, 'skills');
  const installedNames = fs.readdirSync(packagedSkills)
    .filter(name => fs.existsSync(path.join(packagedSkills, name, 'SKILL.md')))
    .sort();
  assert.deepEqual(installedNames, skills.map(skill => skill.name).sort());
  for (const skill of skills) {
    assert.deepEqual(digestTree(path.join(packageRoot, 'skills', skill.name)), digestTree(skill.source), `${platform}/${skill.name}`);
  }
  const serialized = JSON.stringify(JSON.parse(fs.readFileSync(path.join(packageRoot, '.mcp.json'))));
  assert(!serialized.includes(root));
  assert(!serialized.includes(process.env.HOME));
  assert(!/tvly-[A-Za-z0-9_-]{12,}/.test(serialized));
}

// Every file the packaged hooks.json tells the harness to run must ship in the payload. When one is
// missing, EVERY hook dies with MODULE_NOT_FOUND before any of our code runs, so nothing inside the
// hook can report or recover from it — it can only be caught here, at packaging time.
const claudePackageRoot = path.join(root, 'dist/plugins/claude', contract.name);
const packagedHooks = fs.readFileSync(path.join(claudePackageRoot, 'hooks/hooks.json'), 'utf8');
const hookTargets = [...new Set([...packagedHooks.matchAll(/\$\{CLAUDE_PLUGIN_ROOT\}\/([^"]+)/g)].map(match => match[1]))];
assert(hookTargets.length > 0, 'packaged hooks.json references no ${CLAUDE_PLUGIN_ROOT} path');
for (const target of hookTargets) {
  const packaged = path.join(claudePackageRoot, ...target.split('/'));
  assert(fs.existsSync(packaged), `hooks.json references ${target}, which is missing from the claude payload`);
  assert.deepEqual(fs.readFileSync(packaged), canonicalBytes(path.join(root, ...target.split('/'))), `${target} differs from source`);
}

// Import closure: the payload ships scripts by an explicit list, so a module a shipped script imports can
// be left behind and the failure only appears at run time on a user's machine
// (ERR_MODULE_NOT_FOUND from install-plugin-generation.js -> ./lib/capabilities.js). Resolve every static
// relative import of every packaged script against the packaged tree itself.
function packagedScripts(directory) {
  if (!fs.existsSync(directory)) return [];
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const absolute = path.join(directory, entry.name);
    if (entry.isDirectory()) return packagedScripts(absolute);
    return /\.(m?js)$/.test(entry.name) ? [absolute] : [];
  });
}
// `from` clauses may span lines; dynamic import() and side-effect imports are covered too.
const importSpecifier = /\bfrom\s*['"](\.{1,2}\/[^'"]+)['"]|\bimport\s*\(\s*['"](\.{1,2}\/[^'"]+)['"]|^\s*import\s+['"](\.{1,2}\/[^'"]+)['"]/gm;
const relativeImports = text => [...text.matchAll(importSpecifier)].map(match => match[1] ?? match[2] ?? match[3]);
assert.deepEqual(
  relativeImports("import {\n  a,\n  b,\n} from './multi.js';\nexport * from './star.js';\nawait import('./dyn.js');\nimport './side.js';\nimport fs from 'node:fs';"),
  ['./multi.js', './star.js', './dyn.js', './side.js'],
  'relative-import scanner self-check'
);
for (const platform of ['claude', 'codex']) {
  const scriptsRoot = path.join(root, 'dist/plugins', platform, contract.name, 'scripts');
  if (platform === 'claude') assert(fs.existsSync(path.join(scriptsRoot, 'lib')), 'claude payload has no scripts/lib');
  for (const script of packagedScripts(scriptsRoot)) {
    for (const specifier of relativeImports(fs.readFileSync(script, 'utf8'))) {
      const resolved = path.resolve(path.dirname(script), specifier);
      assert(fs.existsSync(resolved), `${platform}: ${path.relative(root, script)} imports ${specifier}, which is missing from the payload`);
    }
  }
}
// scripts/lib is copied wholesale, so keep tests, fixtures and dotfiles out of it.
for (const name of fs.readdirSync(path.join(root, 'dist/plugins/claude', contract.name, 'scripts/lib'))) {
  assert(!/\.test\.|^\./.test(name), `scripts/lib/${name} must not ship in the payload`);
}

const codexManifest = JSON.parse(fs.readFileSync(path.join(root, contract.outputs.codexPackage, '.codex-plugin/plugin.json')));
assert.equal(codexManifest.skills, './skills');
assert.equal(codexManifest.hooks, undefined);
assert(codexManifest.interface.defaultPrompt);
assert(codexManifest.interface.websiteURL.startsWith('https://'));

for (const entry of contract.imports) {
  const tree = spawnSync('git', ['ls-tree', 'HEAD', '--', entry.path], { cwd: root, encoding: 'utf8' }).stdout.trim();
  const gitlink = tree.match(/^160000 commit ([a-f0-9]{40})\t/)?.[1];
  assert.equal(gitlink, entry.commit, entry.path);
}

for (const marketplacePath of [contract.outputs.claudeMarketplace, contract.outputs.codexMarketplace]) {
  const marketplace = JSON.parse(fs.readFileSync(path.join(root, marketplacePath)));
  assert.equal(marketplace.version, contract.releaseVersion);
  assert.equal(new Set(marketplace.plugins.map(plugin => plugin.name)).size, marketplace.plugins.length);
}

console.log(`PASS: ${skills.length} canonical skills, payload parity, modes, pins, manifests, and marketplaces`);
