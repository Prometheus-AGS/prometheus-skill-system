import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const siteRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(siteRoot, '..');
const require = createRequire(import.meta.url);
const sidebars = require(path.join(siteRoot, 'sidebars.js'));
const configSource = fs.readFileSync(path.join(siteRoot, 'docusaurus.config.js'), 'utf8');
const failures = [];
const release = '1.7.0';

const json = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const text = file => fs.readFileSync(file, 'utf8');
const sitePackage = json(path.join(siteRoot, 'package.json'));
const cargoTableVersion = (file, table) => {
  const source = text(file);
  const header = `[${table}]`;
  const start = source.indexOf(header);
  if (start < 0) return undefined;
  const bodyStart = start + header.length;
  const nextTable = source.indexOf('\n[', bodyStart);
  const section = source.slice(bodyStart, nextTable < 0 ? undefined : nextTable);
  return section.match(/^version\s*=\s*"([^"]+)"\s*$/m)?.[1];
};
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
if (sitePackage.scripts?.['build:deploy'] !== 'npm run generate:catalog && docusaurus build') {
  failures.push('Pages build:deploy does not generate the skills catalog before packaging');
}
if (!sitePackage.scripts?.['docs:check']?.includes('check:mermaid')) {
  failures.push('docs:check does not validate Mermaid source');
}
const marketplace = json(path.join(repoRoot, '.claude-plugin/marketplace.json'));
if (marketplace.version !== release || marketplace.plugins?.[0]?.version !== release) {
  failures.push(`Claude marketplace root plugin is not version ${release}`);
}
if (
  json(path.join(repoRoot, 'shared/harnesses/generated/release-manifest.json')).sourceVersion !==
  release
) {
  failures.push(`release manifest sourceVersion is not ${release}`);
}
if (
  !new RegExp(`^version: ${release.replaceAll('.', '\\.')}$`, 'm').test(
    fs.readFileSync(path.join(repoRoot, 'SKILLS.md'), 'utf8')
  )
) {
  failures.push(`SKILLS.md is not version ${release}`);
}

for (const [label, file, table] of [
  [
    'Prometheus CLI workspace',
    path.join(repoRoot, 'tools/prometheus-cli/Cargo.toml'),
    'workspace.package',
  ],
  [
    'Knowledge workspace',
    path.join(repoRoot, 'tools/prometheus-knowledge/Cargo.toml'),
    'workspace.package',
  ],
  [
    'Memory server package',
    path.join(repoRoot, 'tools/surreal-memory-server/Cargo.toml'),
    'package',
  ],
  ['Prometheus Exec binary', path.join(repoRoot, 'crates/prometheus-exec/Cargo.toml'), 'package'],
  [
    'Prometheus Exec contracts',
    path.join(repoRoot, 'substrate/exec-contracts/Cargo.toml'),
    'package',
  ],
  ['Prometheus Exec service', path.join(repoRoot, 'substrate/exec-service/Cargo.toml'), 'package'],
]) {
  if (cargoTableVersion(file, table) !== release) {
    failures.push(`${label} is not version ${release}`);
  }
}

const knowledgeManifest = text(path.join(repoRoot, 'tools/prometheus-knowledge/Cargo.toml'));
for (const crate of [
  'pk-core',
  'pk-store',
  'pk-watcher',
  'pk-librarian',
  'pk-mcp',
  'pk-event-store',
]) {
  const dependency = new RegExp(
    `^${crate}\\s*=\\s*\\{[^\\n]*version\\s*=\\s*"${release.replaceAll('.', '\\.')}"`,
    'm'
  );
  if (!dependency.test(knowledgeManifest)) {
    failures.push(`Knowledge dependency ${crate} is not pinned to ${release}`);
  }
}

const requiredSidebars = {
  memorySidebar: ['memory/overview', 'memory/operation-api', 'memory/executor-and-recovery'],
  knowledgeLearningSidebar: [
    'knowledge-learning/snapshots-and-context',
    'knowledge-learning/hooks-worker-and-receipts',
    'knowledge-learning/loro-evidence-and-migration',
    'knowledge-learning/migration-and-troubleshooting',
  ],
  pluginDistributionSidebar: [
    'plugin-distribution/immutable-generations',
    'plugin-distribution/signing-index-and-receipts',
    'plugin-distribution/targets-and-dispatchers',
    'plugin-distribution/activation-rollback-uninstall',
  ],
  operationsSidebar: [
    'operations/installation-and-upgrades',
    'operations/local-validation-and-docs-automation',
    'operations/generated-reference',
    'operations/doctors-and-mac-certification',
    'operations/logs-recovery-and-failures',
  ],
  kbdSidebar: [
    'kbd/operator-controls',
    'kbd/checkpoints-compaction-recovery',
    'kbd/migration-and-rollout',
  ],
  sovereignSidebar: [
    'sovereign-sync/pair-two-machines',
    'sovereign-sync/signed-pushes-and-receipts',
    'sovereign-sync/rest-api',
  ],
};
for (const [sidebarId, ids] of Object.entries(requiredSidebars)) {
  const encoded = JSON.stringify(sidebars[sidebarId] ?? []);
  if (!sidebars[sidebarId]) failures.push(`missing sidebar ${sidebarId}`);
  for (const id of ids) {
    if (!encoded.includes(`"${id}"`)) failures.push(`${sidebarId} misses ${id}`);
    if (!fs.existsSync(path.join(siteRoot, 'docs', `${id}.md`)))
      failures.push(`missing doc ${id}.md`);
  }
  if (!configSource.includes(`sidebarId: '${sidebarId}'`))
    failures.push(`navbar misses ${sidebarId}`);
}

for (const decision of [
  'unrestricted-agent-tools-certification-integrity.md',
  'loro-only-deterministic-learner-convergence.md',
  'unix-socket-durable-p2p-pairing.md',
  'signed-transactional-plugin-generations.md',
  'local-only-validation-documentation-automation.md',
]) {
  if (!fs.existsSync(path.join(repoRoot, 'docs/decisions', decision))) {
    failures.push(`missing release architecture decision ${decision}`);
  }
}

const markdownFiles = directory => {
  const files = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const candidate = path.join(directory, entry.name);
    if (entry.isDirectory()) files.push(...markdownFiles(candidate));
    else if (entry.isFile() && /\.mdx?$/.test(entry.name)) files.push(candidate);
  }
  return files;
};
const driftFiles = [
  ...[
    'memory',
    'knowledge-learning',
    'plugin-distribution',
    'operations',
    'kbd',
    'sovereign-sync',
    'mobile',
  ].flatMap(directory => markdownFiles(path.join(siteRoot, 'docs', directory))),
  ...['kbd-runtime.md', 'learner-model.md'].map(name =>
    path.join(siteRoot, 'docs/substrate', name)
  ),
  ...[
    '06-memory-and-learning.md',
    '13-tools-reference.md',
    '15-hooks-and-lifecycle.md',
    '16-cli-and-scripts.md',
    '17-platform-support.md',
    '18-plugins-and-marketplace.md',
    '19-installation.md',
    '20-updating.md',
  ].map(name => path.join(repoRoot, 'docs/guide', name)),
];
const forbidden = [
  [/\bpk focus\b/i, 'removed pk focus command'],
  [/\/api\/v1\/memory\b/i, 'stale v1 memory write route'],
  [/synchronous writeback/i, 'obsolete inline writeback claim'],
  [/retry[- ]counts?\s+(prove|mean|determine|indicate)/i, 'retry-count correctness claim'],
  [/prometheus-skill-pack\/1\.6\.0/i, 'hardcoded stale plugin path'],
  [/\b1\.6\.[012]\b/i, 'stale release metadata'],
  [/Docker (is )?(required|recommended)/i, 'Docker-only installation claim'],
  [/\b92%\s+production readiness\b/i, 'unsupported production-readiness percentage'],
  [/\bAutomerge(?:-backed)?\b/i, 'stale learner CRDT implementation'],
  [/operator_id\s+(?:topic|credential|identity)/i, 'obsolete operator_id identity/topic claim'],
  [
    /(?:remaining|installed|active)[^\n]{0,40}PreToolUse[^\n]{0,100}(?:BDD|test|mutation)|PreToolUse\s+(?:guard\s+)?(?:blocks?|protects?)/i,
    'obsolete mutation guard claim',
  ],
  [/GitHub Actions[^\n]{0,120}(?:test|lint|doctor|certif|validat)/i, 'hosted validation claim'],
];
for (const file of driftFiles) {
  const content = fs.readFileSync(file, 'utf8');
  for (const [pattern, message] of forbidden) {
    if (pattern.test(content)) failures.push(`${path.relative(repoRoot, file)}: ${message}`);
  }
}

const rootSpec = path.join(siteRoot, 'static/openapi/surreal-memory-v2.openapi.json');
if (json(rootSpec).info?.version !== release)
  failures.push(`Memory OpenAPI is not version ${release}`);
const sovereignSpec = path.join(siteRoot, 'static/openapi/sovereign-sync-v2.openapi.json');
if (!fs.existsSync(sovereignSpec) || json(sovereignSpec).info?.version !== release) {
  failures.push(`Sovereign Sync OpenAPI is not version ${release}`);
}
const execReferenceSpec = path.join(repoRoot, 'docs/reference/api/prometheus-exec.openapi.json');
const execSiteSpec = path.join(siteRoot, 'static/openapi/prometheus-exec.openapi.json');
if (
  !fs.existsSync(execSiteSpec) ||
  json(execSiteSpec).info?.version !== release ||
  JSON.stringify(json(execSiteSpec)) !== JSON.stringify(json(execReferenceSpec))
) {
  failures.push('Prometheus Exec OpenAPI is missing, stale, or not version 1.7.0');
}
const execBinary = json(path.join(repoRoot, 'config/prometheus-exec-binary.json'));
if (execBinary.expectedVersion !== `prometheus-exec ${release}`) {
  failures.push(`Prometheus Exec installation manifest is not version ${release}`);
}
const execComponent = json(path.join(repoRoot, 'config/prometheus-exec-component.json'));
const componentBytes = fs.readFileSync(path.join(repoRoot, execComponent.sourcePath));
const componentHash = crypto.createHash('sha256').update(componentBytes).digest('hex');
if (
  execComponent.release !== release ||
  execComponent.world !== 'prometheus:component@0.1.0' ||
  execComponent.sha256 !== componentHash ||
  execComponent.sizeBytes !== componentBytes.length ||
  execComponent.capabilities?.hostImports?.length !== 7 ||
  execComponent.capabilities?.wasiAdapterImports?.length !== 14
) {
  failures.push('Prometheus Exec reference component metadata is stale or invalid');
}
const pluginTargets = [
  ...text(path.join(repoRoot, 'scripts/install-plugin-generation.js')).matchAll(
    /^\s*'([^']+\/skills)',?$/gm
  ),
].map(match => match[1]);
if (pluginTargets.length !== 14) failures.push('Prometheus plugin target matrix is not 14');
const submoduleSpec = path.join(
  repoRoot,
  'tools/surreal-memory-server/openapi/surreal-memory-v2.openapi.json'
);
if (
  fs.existsSync(submoduleSpec) &&
  JSON.stringify(json(rootSpec)) !== JSON.stringify(json(submoduleSpec))
) {
  failures.push('Docusaurus OpenAPI copy differs from the pinned server specification');
}

if (failures.length) {
  console.error(failures.map(failure => `Docs contract: ${failure}`).join('\n'));
  process.exit(1);
}
console.log('Documentation sidebars, metadata, semantics, and OpenAPI parity are valid.');
