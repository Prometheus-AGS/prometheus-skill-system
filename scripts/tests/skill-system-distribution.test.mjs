import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

import { collectDistributionSkills, readSkillSystem } from '../lib/skill-system.js';

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), '../..');
const contract = readSkillSystem(root);
const skills = collectDistributionSkills(root, contract);

assert.equal(contract.releaseVersion, '1.8.0');
assert.equal(contract.minimumActiveVersion, '1.8.0');
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
      const bytes = stat.isSymbolicLink() ? Buffer.from(fs.readlinkSync(absolute)) : fs.readFileSync(absolute);
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
