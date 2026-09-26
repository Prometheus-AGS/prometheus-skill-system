#!/usr/bin/env node

import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { canonicalBytes } from './lib/canonical-bytes.js';
import { collectDistributionSkills, readSkillSystem } from './lib/skill-system.js';

const sourceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const check = process.argv.includes('--check');
const contract = readSkillSystem(sourceRoot);
const skills = collectDistributionSkills(sourceRoot, contract);

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === 'object')
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map(key => [key, canonical(value[key])])
    );
  return value;
}

function json(value) {
  return `${JSON.stringify(canonical(value), null, 2)}\n`;
}

function copy(source, destination) {
  const stat = fs.lstatSync(source);
  if (stat.isDirectory()) {
    fs.mkdirSync(destination, { recursive: true, mode: stat.mode & 0o7777 });
    for (const name of fs.readdirSync(source).sort()) {
      if (['.git', 'node_modules', 'target', '.kbd-orchestrator'].includes(name)) continue;
      copy(path.join(source, name), path.join(destination, name));
    }
    fs.chmodSync(destination, stat.mode & 0o7777);
  } else if (stat.isSymbolicLink()) {
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.symlinkSync(fs.readlinkSync(source), destination);
  } else if (stat.isFile()) {
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, canonicalBytes(source));
    fs.chmodSync(destination, stat.mode & 0o7777);
  }
}

function write(root, relative, value, mode = 0o644) {
  const file = path.join(root, relative);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, typeof value === 'string' ? value : json(value), { mode });
  fs.chmodSync(file, mode);
}

function baseManifest() {
  return {
    name: contract.name,
    version: contract.releaseVersion,
    description:
      'Complete Prometheus skill system: process orchestration, React, GitOps, testing, research, learning, and portable agent tooling.',
    author: { name: 'Travis James', url: 'https://travisjames.ai' },
    homepage: 'https://github.com/Prometheus-AGS/prometheus-skill-system',
    repository: 'https://github.com/Prometheus-AGS/prometheus-skill-system',
    license: 'MIT',
    keywords: ['agent-skills', 'process-orchestration', 'react', 'gitops', 'testing', 'research'],
    skills: './skills',
    mcpServers: './.mcp.json',
  };
}

function sanitizedMcp() {
  const mcp = JSON.parse(fs.readFileSync(path.join(sourceRoot, '.mcp.json'), 'utf8'));
  const serialized = JSON.stringify(mcp);
  if (serialized.includes(sourceRoot) || serialized.includes(os.homedir()))
    throw new Error('MCP template contains a machine-specific path');
  if (/tvly-[A-Za-z0-9_-]{12,}/.test(serialized))
    throw new Error('MCP template contains a literal Tavily credential');
  return mcp;
}

function packagedContract() {
  return {
    ...contract,
    inventory: {
      roots: [{ id: 'packaged', path: 'skills', scan: 'children' }],
      excludedImports: [],
    },
    imports: [],
    targets: contract.targets.map(target => ({
      ...target,
      sourceTreeLifecycle: 'install-only',
    })),
  };
}

function packagedExecutionDescriptor() {
  const descriptor = JSON.parse(
    fs.readFileSync(path.join(sourceRoot, 'config/prometheus-exec-component.json'), 'utf8')
  );
  return {
    ...descriptor,
    release: contract.releaseVersion,
    sourcePath: 'skills/entity-graph-optimize/skill.wasm',
  };
}

function copySignedSkillRuntimeFiles(root) {
  const release = JSON.parse(
    fs.readFileSync(
      path.join(sourceRoot, 'shared/harnesses/generated/release-manifest.json'),
      'utf8'
    )
  );
  for (const entry of release.runtimeFiles ?? []) {
    const relative = path.posix.normalize(entry.path ?? '');
    if (!relative.startsWith('skills/')) continue;
    copy(path.join(sourceRoot, ...relative.split('/')), path.join(root, ...relative.split('/')));
  }
}

// Every file hooks.json tells the harness to run must ship with it. When one is missing, every hook
// event dies with MODULE_NOT_FOUND before any pack code runs, so it can only be prevented here.
// Derived from hooks.json rather than listed by hand: the list that was kept by hand is how
// scripts/hook-entry.mjs came to be referenced but never packaged.
function copyHookTargets(root) {
  const hooks = fs.readFileSync(path.join(sourceRoot, 'hooks/hooks.json'), 'utf8');
  const targets = new Set(
    [...hooks.matchAll(/\$\{CLAUDE_PLUGIN_ROOT\}\/([^"]+)/g)].map(match =>
      path.posix.normalize(match[1])
    )
  );
  for (const target of [...targets].sort()) {
    if (target.startsWith('../') || path.posix.isAbsolute(target))
      throw new Error(`hooks.json references a path outside the plugin root: ${target}`);
    const source = path.join(sourceRoot, ...target.split('/'));
    if (!fs.existsSync(source)) throw new Error(`hooks.json references a missing file: ${target}`);
    copy(source, path.join(root, ...target.split('/')));
  }
}

function materializePackage(root, platform) {
  for (const skill of skills) copy(skill.source, path.join(root, 'skills', skill.name));
  write(root, '.mcp.json', sanitizedMcp());
  write(root, 'skill-index.json', {
    schemaVersion: 'prometheus-distribution-skill-index-v1',
    releaseVersion: contract.releaseVersion,
    skills: skills.map(skill => ({ name: skill.name, path: `skills/${skill.name}` })),
  });
  if (platform === 'claude') {
    write(root, '.claude-plugin/plugin.json', baseManifest());
    copy(path.join(sourceRoot, 'hooks/hooks.json'), path.join(root, 'hooks/hooks.json'));
    copyHookTargets(root);
    copy(path.join(sourceRoot, 'shared'), path.join(root, 'shared'));
    copySignedSkillRuntimeFiles(root);
    copy(
      path.join(sourceRoot, 'scripts/install-plugin-generation.js'),
      path.join(root, 'scripts/install-plugin-generation.js')
    );
    copy(
      path.join(sourceRoot, 'scripts/lib/skill-system.js'),
      path.join(root, 'scripts/lib/skill-system.js')
    );
    write(root, 'package.json', {
      name: '@prometheus-ags/prometheus-skill-pack-payload',
      version: contract.releaseVersion,
      private: true,
      type: 'module',
    });
    write(root, 'skill-system.json', packagedContract());
    write(root, 'config/prometheus-exec-component.json', packagedExecutionDescriptor());
  } else {
    write(root, '.codex-plugin/plugin.json', {
      ...baseManifest(),
      interface: {
        displayName: 'Prometheus Skill Pack',
        shortDescription: 'Portable Prometheus skills for agentic software work.',
        longDescription:
          'A self-contained distribution of Prometheus process, engineering, research, learning, and quality skills.',
        developerName: 'Travis James',
        category: 'productivity',
        capabilities: ['skills', 'mcp'],
        defaultPrompt:
          'Use the Prometheus skill system to select and apply the most relevant installed skill for this task.',
        websiteURL: 'https://github.com/Prometheus-AGS/prometheus-skill-system',
      },
    });
  }
}

function adjacentPlugin(entry) {
  const distribution = entry.distribution;
  if (distribution?.mode !== 'adjacent-plugin') return null;

  const manifests = {
    claude: distribution.claudeManifest,
    codex: distribution.codexManifest,
  };
  for (const [platform, relative] of Object.entries(manifests)) {
    if (
      !relative ||
      path.posix.isAbsolute(relative) ||
      path.posix.normalize(relative).startsWith('../')
    ) {
      throw new Error(`adjacent import ${entry.id} has an unsafe ${platform} manifest path`);
    }
    const manifestPath = path.join(sourceRoot, entry.path, ...relative.split('/'));
    if (!fs.existsSync(manifestPath)) {
      throw new Error(
        `adjacent import ${entry.id} is missing its ${platform} manifest: ${relative}`
      );
    }
    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
    if (manifest.name !== entry.id) {
      throw new Error(
        `adjacent import ${entry.id} ${platform} manifest declares ${manifest.name ?? '<missing>'}`
      );
    }
    if (manifest.version !== distribution.version) {
      throw new Error(
        `adjacent import ${entry.id} ${platform} manifest version ${manifest.version ?? '<missing>'} does not match ${distribution.version}`
      );
    }
  }
  if (!['full', 'mini'].includes(distribution.variant)) {
    throw new Error(`adjacent import ${entry.id} has an unsupported distribution variant`);
  }
  if (
    !distribution.stager ||
    path.posix.isAbsolute(distribution.stager) ||
    path.posix.normalize(distribution.stager).startsWith('../')
  ) {
    throw new Error(`adjacent import ${entry.id} has an unsafe stager path`);
  }
  for (const platform of Object.keys(manifests)) {
    const output = distribution.outputs?.[platform];
    if (
      !output ||
      path.posix.isAbsolute(output) ||
      path.posix.normalize(output).startsWith('../')
    ) {
      throw new Error(`adjacent import ${entry.id} has an unsafe ${platform} output path`);
    }
  }
  return distribution;
}

function materializeAdjacentPackages(root) {
  for (const entry of contract.imports) {
    const adjacent = adjacentPlugin(entry);
    if (!adjacent) continue;
    const stager = path.join(sourceRoot, entry.path, ...adjacent.stager.split('/'));
    if (!fs.existsSync(stager))
      throw new Error(`adjacent import ${entry.id} is missing its stager`);
    for (const output of Object.values(adjacent.outputs)) {
      const destination = path.join(root, ...output.split('/'));
      const result = spawnSync(
        process.execPath,
        [stager, '--output', destination, '--variant', adjacent.variant],
        { cwd: sourceRoot, encoding: 'utf8' }
      );
      if (result.status !== 0) {
        throw new Error(
          `failed to stage adjacent import ${entry.id}: ${result.stderr || result.stdout || `exit ${result.status}`}`
        );
      }
    }
  }
}

function marketplaceEntries(platform) {
  const localSource = entry =>
    platform === 'claude' ? `./${entry.path}` : { source: 'local', path: `./${entry.path}` };
  const policy = { installation: 'AVAILABLE', authentication: 'ON_INSTALL' };
  const umbrella = {
    name: contract.name,
    source:
      platform === 'claude'
        ? `./${contract.outputs.claudePackage}`
        : { source: 'local', path: `./${contract.outputs.codexPackage}` },
    version: contract.releaseVersion,
    description: baseManifest().description,
    category: 'productivity',
    ...(platform === 'codex'
      ? { policy: { installation: 'INSTALLED_BY_DEFAULT', authentication: 'ON_INSTALL' } }
      : {}),
  };
  const adjacent = contract.marketplace.plugins.map(entry => ({
    name: entry.name,
    source: localSource(entry),
    version: entry.version,
    category: entry.category,
    ...(platform === 'codex' ? { policy } : {}),
  }));
  const imports = contract.imports
    .filter(entry => entry.marketplace)
    .map(entry => {
      const name = entry.id;
      const adjacent = adjacentPlugin(entry);
      if (adjacent) {
        const packagePath = `${adjacent.outputs[platform]}/package`;
        if (platform === 'claude') {
          return {
            name,
            source: `./${packagePath}`,
            version: adjacent.version,
            category: adjacent.category,
          };
        }
        return {
          name,
          source: { source: 'local', path: `./${packagePath}` },
          version: adjacent.version,
          category: adjacent.category,
          policy,
          metadata: { repository: entry.repository, sha: entry.commit },
        };
      }
      if (platform === 'claude')
        return {
          name,
          source: {
            source: 'github',
            repo: entry.repository.replace(/^https:\/\/github\.com\//, '').replace(/\.git$/, ''),
            sha: entry.commit,
          },
          strict: false,
          skills: name === 'artifact-refiner' ? './skills' : './',
          category: 'productivity',
        };
      return {
        name,
        source: { source: 'local', path: `./${entry.path}` },
        category: 'productivity',
        policy,
        metadata: { repository: entry.repository, sha: entry.commit },
      };
    });
  return [umbrella, ...adjacent, ...imports];
}

function materialize(root) {
  materializePackage(path.join(root, contract.outputs.claudePackage), 'claude');
  materializePackage(path.join(root, contract.outputs.codexPackage), 'codex');
  materializeAdjacentPackages(root);
  write(root, contract.outputs.claudeMarketplace, {
    name: contract.name,
    version: contract.releaseVersion,
    description: 'Prometheus skill system marketplace',
    owner: { name: 'Travis James', url: 'https://travisjames.ai' },
    plugins: marketplaceEntries('claude'),
  });
  write(root, contract.outputs.codexMarketplace, {
    name: contract.name,
    version: contract.releaseVersion,
    description: 'Prometheus skill system marketplace',
    owner: { name: 'Travis James', url: 'https://travisjames.ai' },
    interface: { displayName: 'Prometheus Skill System' },
    plugins: marketplaceEntries('codex'),
  });
}

function collect(root, relative = '') {
  const entries = [];
  if (!fs.existsSync(root)) return entries;
  for (const name of fs.readdirSync(path.join(root, relative)).sort()) {
    const child = path.join(relative, name);
    const absolute = path.join(root, child);
    const stat = fs.lstatSync(absolute);
    if (stat.isDirectory()) entries.push(...collect(root, child));
    else {
      const bytes = stat.isSymbolicLink()
        ? Buffer.from(fs.readlinkSync(absolute))
        : canonicalBytes(absolute);
      entries.push({
        path: child.split(path.sep).join('/'),
        mode: (stat.mode & 0o7777).toString(8),
        sha256: crypto.createHash('sha256').update(bytes).digest('hex'),
      });
    }
  }
  return entries;
}

const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-distribution.'));
try {
  materialize(temporary);
  const outputPaths = [
    contract.outputs.claudePackage,
    contract.outputs.codexPackage,
    contract.outputs.claudeMarketplace,
    contract.outputs.codexMarketplace,
    ...contract.imports.flatMap(entry =>
      entry.distribution?.mode === 'adjacent-plugin'
        ? Object.values(entry.distribution.outputs ?? {})
        : []
    ),
  ];
  if (check) {
    for (const output of outputPaths) {
      const expected = fs.lstatSync(path.join(temporary, output)).isDirectory()
        ? collect(path.join(temporary, output))
        : fs.readFileSync(path.join(temporary, output));
      const actualPath = path.join(sourceRoot, output);
      const actual = fs.existsSync(actualPath)
        ? fs.lstatSync(actualPath).isDirectory()
          ? collect(actualPath)
          : fs.readFileSync(actualPath)
        : null;
      if (JSON.stringify(expected) !== JSON.stringify(actual))
        throw new Error(`generated output is stale: ${output}`);
    }
  } else {
    for (const output of outputPaths) {
      const destination = path.join(sourceRoot, output);
      fs.rmSync(destination, { recursive: true, force: true });
      copy(path.join(temporary, output), destination);
    }
    process.stdout.write(`Generated ${skills.length} skills for Claude and Codex.\n`);
  }
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}
