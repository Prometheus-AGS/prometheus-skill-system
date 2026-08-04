#!/usr/bin/env node
/**
 * build-codex-plugin.js — generate + validate Codex plugin + marketplace artifacts
 * from the canonical Claude-Code sources. GENERATED OUTPUT — do not hand-edit targets.
 *
 * Source of truth:
 *   .claude-plugin/plugin.json       → .codex-plugin/plugin.json
 *   .claude-plugin/marketplace.json  → .agents/plugins/marketplace.json
 *   .mcp.json / hooks/codex-hooks.json (generated runtime pointers)
 *
 * Usage:
 *   node scripts/build-codex-plugin.js            # generate + validate
 *   node scripts/build-codex-plugin.js --check    # CI: fail if artifacts are stale/invalid (no write)
 *
 * Idempotent: re-running produces byte-identical output. Verified against
 * codex-cli 0.144.1 (marketplace add → 11 plugins resolve → 7 MCP servers register).
 * Spec: .kbd-orchestrator/phases/phase-codex-plugin-implementation/references/codex-plugin-spec-digest.md
 */
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const CHECK = process.argv.includes('--check');

execFileSync(
  process.execPath,
  [path.join(ROOT, 'scripts/generate-harness-adapters.js'), ...(CHECK ? ['--check'] : [])],
  { cwd: ROOT, stdio: 'inherit' }
);

const read = p => JSON.parse(fs.readFileSync(path.join(ROOT, p), 'utf8'));
const rel = s => (s === '.' ? './' : s.startsWith('./') ? s : './' + s.replace(/^\/+/, ''));

// Marketplace source type — `local` (default, in-repo dogfood) keeps output
// byte-stable. `git-subdir`/`git` emit publishable sources (needs a pushed commit).
const MP_SOURCE = process.env.CODEX_MARKETPLACE_SOURCE || 'local'; // local | git-subdir | git
const MP_REF = process.env.CODEX_MARKETPLACE_REF || 'main';
const marketplaceSource = (pluginSource, repoUrl) => {
  if (MP_SOURCE === 'git-subdir')
    return { source: 'git-subdir', url: repoUrl, ref: MP_REF, path: rel(pluginSource) };
  if (MP_SOURCE === 'git') return { source: 'git', url: repoUrl, ref: MP_REF };
  return { source: 'local', path: rel(pluginSource) };
};
const insideRoot = p => typeof p === 'string' && p.startsWith('./') && !p.split('/').includes('..');

const drift = [];
const errors = [];
const fail = m => errors.push(m);

function emit(p, obj) {
  const abs = path.join(ROOT, p);
  const next = JSON.stringify(obj, null, 2) + '\n';
  if (CHECK) {
    const cur = fs.existsSync(abs) ? fs.readFileSync(abs, 'utf8') : null;
    if (cur !== next) drift.push(p);
  } else {
    fs.mkdirSync(path.dirname(abs), { recursive: true });
    fs.writeFileSync(abs, next);
  }
  return { path: p, obj };
}

// ---- .codex-plugin/plugin.json (G-02) ---------------------------------------
function buildPluginManifest() {
  const c = read('.claude-plugin/plugin.json');
  return emit('.codex-plugin/plugin.json', {
    name: c.name,
    version: c.version,
    description: c.description,
    author: c.author,
    homepage: c.homepage,
    repository: c.repository,
    license: c.license,
    keywords: c.keywords,
    skills: c.skills, // curated array — parity with Claude; budget curated via config/codex-catalog.txt
    mcpServers: './.mcp.json', // Codex accepts the mcpServers-wrapper .mcp.json (verified 0.144.1)
    hooks: './hooks/codex-hooks.json',
    interface: {
      displayName: 'Prometheus Skill Pack',
      shortDescription:
        'React entity mgmt, GitOps, KBD orchestration, strategic evolution, BDD, deep-research — with surreal-memory.',
      longDescription: c.description,
      developerName: (c.author && c.author.name) || 'Prometheus AGS',
      category: 'productivity',
      capabilities: ['skills', 'mcp'],
      website: c.homepage,
    },
  });
}

// ---- .agents/plugins/marketplace.json (G-03) --------------------------------
function buildMarketplace() {
  const m = read('.claude-plugin/marketplace.json');
  const repoUrl = read('.claude-plugin/plugin.json').repository;
  return emit('.agents/plugins/marketplace.json', {
    name: m.name,
    version: m.version,
    description: m.description,
    owner: m.owner,
    interface: { displayName: 'Prometheus Skill Pack' },
    plugins: m.plugins.map(p => ({
      name: p.name,
      description: p.description,
      source: marketplaceSource(p.source, repoUrl),
      version: p.version,
      tags: p.tags,
      category: p.category,
      policy: {
        installation: p.source === '.' ? 'INSTALLED_BY_DEFAULT' : 'AVAILABLE',
        authentication: 'ON_INSTALL',
      },
    })),
  });
}

// ---- Validation --------------------------------------------------------------
function validate(pluginObj, marketObj) {
  for (const f of ['name', 'version', 'description'])
    if (!pluginObj[f]) fail(`plugin.json missing required field: ${f}`);
  for (const ptr of ['mcpServers', 'hooks'])
    if (pluginObj[ptr] && !insideRoot(pluginObj[ptr]))
      fail(`plugin.json ${ptr} not ./-inside-root: ${pluginObj[ptr]}`);
  const hookPath = path.join(ROOT, pluginObj.hooks ?? '');
  if (!fs.existsSync(hookPath)) fail(`plugin.json hooks target is missing: ${pluginObj.hooks}`);
  else {
    const hookPayload = JSON.parse(fs.readFileSync(hookPath, 'utf8'));
    const commands = Object.values(hookPayload.hooks ?? {}).flatMap(groups =>
      groups.flatMap(group => (group.hooks ?? []).map(hook => hook.command))
    );
    if (commands.length === 0) fail('Codex hooks target contains no commands');
    for (const command of commands) {
      if (
        typeof command !== 'string' ||
        !command.includes('/runtime/v1/run-hook') ||
        !command.includes('--bundle') ||
        !command.includes('bootstrap-hook-runtime.sh')
      ) {
        fail('Codex hook bypasses the generated bundle-pinned runtime template');
      }
    }
  }
  (pluginObj.skills || []).forEach(s => {
    if (!insideRoot(s)) fail(`plugin.json skill path not ./-inside-root: ${s}`);
  });
  if (!marketObj.name) fail('marketplace.json missing name');
  if (!marketObj.interface || !marketObj.interface.displayName)
    fail('marketplace.json missing interface.displayName');
  (marketObj.plugins || []).forEach(p => {
    if (!p.name) fail('marketplace plugin missing name');
    if (!p.source || !insideRoot(p.source.path))
      fail(`marketplace plugin ${p.name}: source.path not ./-inside-root`);
    if (!p.policy || !p.policy.installation)
      fail(`marketplace plugin ${p.name}: missing policy.installation`);
    if (p.source?.source === 'local' && insideRoot(p.source.path)) {
      const sourceRoot = path.join(ROOT, p.source.path);
      const candidates =
        p.source.path === './'
          ? [path.join(ROOT, '.codex-plugin/plugin.json')]
          : [
              path.join(sourceRoot, '.codex-plugin/plugin.json'),
              path.join(sourceRoot, '.claude-plugin/plugin.json'),
            ];
      const manifestPath = candidates.find(candidate => fs.existsSync(candidate));
      if (!manifestPath) {
        fail(`marketplace plugin ${p.name}: source has no plugin.json`);
      } else {
        const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
        if (manifest.name !== p.name) {
          fail(
            `marketplace plugin ${p.name}: source manifest name is ${manifest.name || '(missing)'}`
          );
        }
        if (manifest.version && p.version && manifest.version !== p.version) {
          fail(
            `marketplace plugin ${p.name}: source version ${manifest.version} does not match ${p.version}`
          );
        }
      }
    }
  });
}

const { obj: pluginObj } = buildPluginManifest();
const { obj: marketObj } = buildMarketplace();
validate(pluginObj, marketObj);

if (errors.length) {
  console.error('Codex artifact validation FAILED:');
  errors.forEach(e => console.error('  ✗ ' + e));
  process.exit(1);
}
if (CHECK) {
  if (drift.length) {
    console.error('Codex artifacts are STALE (run `npm run build:codex`):');
    drift.forEach(d => console.error('  ✗ ' + d));
    process.exit(1);
  }
  console.log('✓ Codex artifacts up to date and valid.');
} else {
  console.log('Generated + validated Codex artifacts:');
  console.log('  ✓ .codex-plugin/plugin.json');
  console.log('  ✓ .agents/plugins/marketplace.json');
}
