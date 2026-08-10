#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { collectDistributionSkills, readSkillSystem } from './lib/skill-system.js';

const sourceRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');
const check = process.argv.includes('--check');
const contract = readSkillSystem(sourceRoot);
const skills = collectDistributionSkills(sourceRoot, contract);

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === 'object') return Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])]));
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
    fs.copyFileSync(source, destination);
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
    description: 'Complete Prometheus skill system: process orchestration, React, GitOps, testing, research, learning, and portable agent tooling.',
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
  if (serialized.includes(sourceRoot) || serialized.includes(os.homedir())) throw new Error('MCP template contains a machine-specific path');
  if (/tvly-[A-Za-z0-9_-]{12,}/.test(serialized)) throw new Error('MCP template contains a literal Tavily credential');
  return mcp;
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
    copy(path.join(sourceRoot, 'shared'), path.join(root, 'shared'));
  } else {
    write(root, '.codex-plugin/plugin.json', {
      ...baseManifest(),
      interface: {
        displayName: 'Prometheus Skill Pack',
        shortDescription: 'Portable Prometheus skills for agentic software work.',
        longDescription: 'A self-contained distribution of Prometheus process, engineering, research, learning, and quality skills.',
        developerName: 'Travis James',
        category: 'productivity',
        capabilities: ['skills', 'mcp'],
        defaultPrompt: 'Use the Prometheus skill system to select and apply the most relevant installed skill for this task.',
        websiteURL: 'https://github.com/Prometheus-AGS/prometheus-skill-system'
      }
    });
  }
}

function marketplaceEntries(platform) {
  const localSource = entry => platform === 'claude'
    ? `./${entry.path}`
    : { source: 'local', path: `./${entry.path}` };
  const policy = { installation: 'AVAILABLE', authentication: 'ON_INSTALL' };
  const umbrella = {
    name: contract.name,
    source: platform === 'claude'
      ? `./${contract.outputs.claudePackage}`
      : { source: 'local', path: `./${contract.outputs.codexPackage}` },
    version: contract.releaseVersion,
    description: baseManifest().description,
    category: 'productivity',
    ...(platform === 'codex' ? { policy: { installation: 'INSTALLED_BY_DEFAULT', authentication: 'ON_INSTALL' } } : {}),
  };
  const adjacent = contract.marketplace.plugins.map(entry => ({
    name: entry.name,
    source: localSource(entry),
    version: entry.version,
    category: entry.category,
    ...(platform === 'codex' ? { policy } : {}),
  }));
  const imports = contract.imports.filter(entry => entry.marketplace).map(entry => {
    const name = entry.id;
    if (platform === 'claude') return {
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
      const bytes = stat.isSymbolicLink() ? Buffer.from(fs.readlinkSync(absolute)) : fs.readFileSync(absolute);
      entries.push({ path: child.split(path.sep).join('/'), mode: (stat.mode & 0o7777).toString(8), sha256: crypto.createHash('sha256').update(bytes).digest('hex') });
    }
  }
  return entries;
}

const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-distribution.'));
try {
  materialize(temporary);
  const outputPaths = [contract.outputs.claudePackage, contract.outputs.codexPackage, contract.outputs.claudeMarketplace, contract.outputs.codexMarketplace];
  if (check) {
    for (const output of outputPaths) {
      const expected = fs.lstatSync(path.join(temporary, output)).isDirectory()
        ? collect(path.join(temporary, output))
        : fs.readFileSync(path.join(temporary, output));
      const actualPath = path.join(sourceRoot, output);
      const actual = fs.existsSync(actualPath)
        ? (fs.lstatSync(actualPath).isDirectory() ? collect(actualPath) : fs.readFileSync(actualPath))
        : null;
      if (JSON.stringify(expected) !== JSON.stringify(actual)) throw new Error(`generated output is stale: ${output}`);
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
