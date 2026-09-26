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

assert.equal(contract.releaseVersion, '1.10.0');
assert.equal(contract.minimumActiveVersion, '1.10.0');
assert.equal(contract.targets.length, 14);
assert.equal(new Set(skills.map(skill => skill.name)).size, skills.length);
assert(skills.some(skill => skill.name === 'artifact-refiner'));
assert(skills.some(skill => skill.name === 'sycophancy-correction'));
assert(!skills.some(skill => skill.source.includes('prometheus-entity-management')));
assert(!skills.some(skill => skill.source.includes('artifact-refiner/shared/sycophancy-correction')));
assert(!skills.some(skill => skill.source.includes('hybrid-mobile-architecture')));

const hybrid = contract.imports.find(entry => entry.id === 'hybrid-mobile-architecture');
assert(hybrid, 'hybrid-mobile-architecture adjacent import is missing');
assert.equal(hybrid.distribution?.mode, 'adjacent-plugin');
assert.equal(hybrid.distribution?.version, '2.0.0-alpha.4');
assert.equal(hybrid.distribution?.variant, 'full');
assert(
  contract.inventory.excludedImports.some(entry => entry.path === hybrid.path),
  'hybrid-mobile-architecture must be explicitly excluded from the umbrella inventory'
);

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

const codexManifest = JSON.parse(fs.readFileSync(path.join(root, contract.outputs.codexPackage, '.codex-plugin/plugin.json')));
assert.equal(codexManifest.skills, './skills');
assert.equal(codexManifest.hooks, undefined);
assert(codexManifest.interface.defaultPrompt);
assert(codexManifest.interface.websiteURL.startsWith('https://'));

for (const entry of contract.imports) {
  const tree = spawnSync('git', ['ls-files', '--stage', '--', entry.path], { cwd: root, encoding: 'utf8' }).stdout.trim();
  const gitlink = tree.match(/^160000 ([a-f0-9]{40}) \d+\t/)?.[1];
  assert.equal(gitlink, entry.commit, entry.path);
}

for (const marketplacePath of [contract.outputs.claudeMarketplace, contract.outputs.codexMarketplace]) {
  const marketplace = JSON.parse(fs.readFileSync(path.join(root, marketplacePath)));
  assert.equal(marketplace.version, contract.releaseVersion);
  assert.equal(new Set(marketplace.plugins.map(plugin => plugin.name)).size, marketplace.plugins.length);
}

const claudeMarketplace = JSON.parse(
  fs.readFileSync(path.join(root, contract.outputs.claudeMarketplace), 'utf8')
);
const claudeHybrid = claudeMarketplace.plugins.find(plugin => plugin.name === hybrid.id);
assert(claudeHybrid, 'Claude marketplace is missing hybrid-mobile-architecture');
assert.equal(
  claudeHybrid.source,
  `./${hybrid.distribution.outputs.claude}/package`
);
assert.equal(claudeHybrid.version, hybrid.distribution.version);
assert.equal(claudeHybrid.strict, undefined, 'a native plugin must not use bare-skill strict:false mode');
assert.equal(claudeHybrid.skills, undefined, 'a native plugin must load its own manifest skill inventory');

const codexMarketplace = JSON.parse(
  fs.readFileSync(path.join(root, contract.outputs.codexMarketplace), 'utf8')
);
const codexHybrid = codexMarketplace.plugins.find(plugin => plugin.name === hybrid.id);
assert(codexHybrid, 'Codex marketplace is missing hybrid-mobile-architecture');
assert.deepEqual(codexHybrid.source, {
  path: `./${hybrid.distribution.outputs.codex}/package`,
  source: 'local',
});
assert.equal(codexHybrid.version, hybrid.distribution.version);
assert.equal(codexHybrid.metadata.sha, hybrid.commit);
assert.equal(codexHybrid.policy.installation, 'AVAILABLE');

let adjacentPayloadSha;
for (const platform of ['claude', 'codex']) {
  const output = path.join(root, hybrid.distribution.outputs[platform]);
  const packageRoot = path.join(output, 'package');
  const receipt = JSON.parse(fs.readFileSync(path.join(output, 'receipt.json'), 'utf8'));
  assert.equal(receipt.variant, 'full');
  assert.equal(receipt.schemaVersion, 1);
  assert(receipt.files.length > 0, `${platform} adjacent payload receipt is empty`);
  assert.equal(adjacentPayloadSha ??= receipt.payloadSha256, receipt.payloadSha256);
  const actual = digestTree(packageRoot)
    .map(entry => ({ path: entry.path, sha256: entry.sha256 }))
    .sort((left, right) => left.path.localeCompare(right.path, 'en'));
  assert.deepEqual(actual, receipt.files, `${platform} adjacent payload differs from its receipt`);
  const nativeManifest = JSON.parse(
    fs.readFileSync(
      path.join(packageRoot, platform === 'claude' ? hybrid.distribution.claudeManifest : hybrid.distribution.codexManifest),
      'utf8'
    )
  );
  assert.equal(nativeManifest.name, hybrid.id);
  assert.equal(nativeManifest.version, hybrid.distribution.version);
}

console.log(`PASS: ${skills.length} canonical skills, payload parity, modes, pins, manifests, and marketplaces`);
