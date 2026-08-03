import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const siteRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(siteRoot, '..');
const require = createRequire(import.meta.url);
const sidebars = require(path.join(siteRoot, 'sidebars.js'));
const configSource = fs.readFileSync(path.join(siteRoot, 'docusaurus.config.js'), 'utf8');
const failures = [];
const release = '1.6.1';

const json = file => JSON.parse(fs.readFileSync(file, 'utf8'));
for (const [label, file] of [
  ['root package', path.join(repoRoot, 'package.json')],
  ['root lockfile', path.join(repoRoot, 'package-lock.json')],
  ['site package', path.join(siteRoot, 'package.json')],
  ['site lockfile', path.join(siteRoot, 'package-lock.json')],
  ['Claude plugin', path.join(repoRoot, '.claude-plugin/plugin.json')],
  ['Codex plugin', path.join(repoRoot, '.codex-plugin/plugin.json')],
]) {
  if (json(file).version !== release) failures.push(`${label} is not version ${release}`);
}

const requiredSidebars = {
  memorySidebar: ['memory/overview', 'memory/operation-api', 'memory/executor-and-recovery'],
  knowledgeLearningSidebar: [
    'knowledge-learning/snapshots-and-context',
    'knowledge-learning/hooks-worker-and-receipts',
    'knowledge-learning/migration-and-troubleshooting',
  ],
  pluginDistributionSidebar: [
    'plugin-distribution/immutable-generations',
    'plugin-distribution/targets-and-dispatchers',
    'plugin-distribution/activation-rollback-uninstall',
  ],
  operationsSidebar: [
    'operations/installation-and-upgrades',
    'operations/doctors-and-mac-certification',
    'operations/logs-recovery-and-failures',
  ],
};
for (const [sidebarId, ids] of Object.entries(requiredSidebars)) {
  const encoded = JSON.stringify(sidebars[sidebarId] ?? []);
  if (!sidebars[sidebarId]) failures.push(`missing sidebar ${sidebarId}`);
  for (const id of ids) {
    if (!encoded.includes(`"${id}"`)) failures.push(`${sidebarId} misses ${id}`);
    if (!fs.existsSync(path.join(siteRoot, 'docs', `${id}.md`))) failures.push(`missing doc ${id}.md`);
  }
  if (!configSource.includes(`sidebarId: '${sidebarId}'`)) failures.push(`navbar misses ${sidebarId}`);
}

const driftFiles = [
  ...fs.readdirSync(path.join(siteRoot, 'docs/memory')).map(name => path.join(siteRoot, 'docs/memory', name)),
  ...fs.readdirSync(path.join(siteRoot, 'docs/knowledge-learning')).map(name => path.join(siteRoot, 'docs/knowledge-learning', name)),
  ...fs.readdirSync(path.join(siteRoot, 'docs/plugin-distribution')).map(name => path.join(siteRoot, 'docs/plugin-distribution', name)),
  ...fs.readdirSync(path.join(siteRoot, 'docs/operations')).map(name => path.join(siteRoot, 'docs/operations', name)),
  ...['06-memory-and-learning.md', '13-tools-reference.md', '15-hooks-and-lifecycle.md', '16-cli-and-scripts.md', '17-platform-support.md', '18-plugins-and-marketplace.md', '19-installation.md', '20-updating.md'].map(name => path.join(repoRoot, 'docs/guide', name)),
];
const forbidden = [
  [/\bpk focus\b/i, 'removed pk focus command'],
  [/\/api\/v1\/memory\b/i, 'stale v1 memory write route'],
  [/synchronous writeback/i, 'obsolete inline writeback claim'],
  [/retry[- ]counts?\s+(prove|mean|determine|indicate)/i, 'retry-count correctness claim'],
  [/prometheus-skill-pack\/1\.6\.0/i, 'hardcoded stale plugin path'],
  [/\b1\.6\.0\b/i, 'stale release metadata'],
  [/Docker (is )?(required|recommended)/i, 'Docker-only installation claim'],
];
for (const file of driftFiles) {
  const content = fs.readFileSync(file, 'utf8');
  for (const [pattern, message] of forbidden) {
    if (pattern.test(content)) failures.push(`${path.relative(repoRoot, file)}: ${message}`);
  }
}

const rootSpec = path.join(siteRoot, 'static/openapi/surreal-memory-v2.openapi.json');
const submoduleSpec = path.join(repoRoot, 'tools/surreal-memory-server/openapi/surreal-memory-v2.openapi.json');
if (fs.existsSync(submoduleSpec) && fs.readFileSync(rootSpec, 'utf8') !== fs.readFileSync(submoduleSpec, 'utf8')) {
  failures.push('Docusaurus OpenAPI copy differs from the pinned server specification');
}

if (failures.length) {
  console.error(failures.map(failure => `Docs contract: ${failure}`).join('\n'));
  process.exit(1);
}
console.log('Documentation sidebars, metadata, semantics, and OpenAPI parity are valid.');
