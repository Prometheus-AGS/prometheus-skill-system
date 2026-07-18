#!/usr/bin/env node

import { existsSync, readFileSync, readdirSync, statSync } from 'fs';
import { homedir } from 'os';
import { basename, dirname, join, relative, resolve } from 'path';
import { fileURLToPath } from 'url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const skillsRoot = join(repoRoot, 'skills');
const args = process.argv.slice(2);
const jsonOutput = args.includes('--json');
const requestedPlatform = (() => {
  const index = args.indexOf('--platform');
  return index >= 0 ? args[index + 1] : null;
})();

const home = homedir();
const platforms = [
  ['claude-code', join(home, '.claude', 'skills')],
  ['opencode', join(home, '.opencode', 'skills')],
  ['kimi-code', join(home, '.kimi-code', 'skills')],
  ['minimax', join(home, '.minimax', 'skills')],
  ['cursor', join(home, '.cursor', 'skills')],
  ['codex', join(home, '.codex', 'skills')],
  ['gemini', join(home, '.gemini', 'skills')],
  ['roo', join(home, '.roo', 'skills')],
  ['windsurf', join(home, '.windsurf', 'skills')],
  ['windsurf-legacy', join(home, '.codeium', 'windsurf', 'skills')],
  ['amp', join(home, '.agents', 'skills')],
  ['zed', join(home, '.config', 'zed', 'skills')],
  ['antigravity', join(home, '.zed', 'skills')],
  ['cline', join(home, '.cline', 'skills')],
];

function parseName(skillMd) {
  const source = readFileSync(skillMd, 'utf8');
  const frontmatter = source.match(/^---\n([\s\S]*?)\n---/)?.[1] ?? '';
  return frontmatter.match(/^name:\s*['"]?([^'"\n]+)['"]?/m)?.[1]?.trim();
}

function collectSkills() {
  const found = [];
  function walk(dir) {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const path = join(dir, entry.name);
      const rel = relative(skillsRoot, path).split('\\').join('/');
      if (rel === 'imported' || rel.startsWith('imported/')) continue;
      if (entry.isDirectory()) walk(path);
      else if (entry.name === 'SKILL.md') {
        found.push({ name: parseName(path) || basename(dir), dir });
      }
    }
  }
  walk(skillsRoot);
  found.sort((a, b) => a.dir.length - b.dir.length || a.name.localeCompare(b.name));
  for (const skill of found) {
    skill.top =
      found.find(candidate => candidate !== skill && skill.dir.startsWith(`${candidate.dir}/`)) ||
      skill;
  }
  return found;
}

function walkFiles(dir, root = dir, files = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (['.git', 'node_modules', 'target'].includes(entry.name)) continue;
    const path = join(dir, entry.name);
    if (entry.isDirectory()) walkFiles(path, root, files);
    else if (entry.isFile()) files.push(relative(root, path));
  }
  return files;
}

function locateInstalled(root, skill) {
  return [
    join(root, skill.name),
    join(root, `prometheus-${skill.name}`),
    join(root, 'prometheus-skill-pack', relative(repoRoot, skill.dir)),
    join(root, basename(skill.top.dir), relative(skill.top.dir, skill.dir)),
  ].filter(path => existsSync(path) && statSync(path).isDirectory());
}

function comparePayload(source, installed) {
  const issues = [];
  for (const rel of walkFiles(source)) {
    const sourcePath = join(source, rel);
    const installedPath = join(installed, rel);
    if (!existsSync(installedPath)) {
      issues.push(`missing:${rel}`);
      continue;
    }
    if (!readFileSync(sourcePath).equals(readFileSync(installedPath))) {
      issues.push(`content:${rel}`);
    }
    const sourceExec = (statSync(sourcePath).mode & 0o111) !== 0;
    const installedExec = (statSync(installedPath).mode & 0o111) !== 0;
    if (sourceExec !== installedExec) issues.push(`mode:${rel}`);
  }
  return issues;
}

function codexPromptAvailable(skill) {
  const prompt = join(home, '.codex', 'prompts', `${skill.name}.md`);
  if (!existsSync(prompt)) return false;
  const content = readFileSync(prompt, 'utf8');
  return content.includes(relative(repoRoot, join(skill.dir, 'SKILL.md')));
}

const skills = collectSkills();
const results = [];
for (const [name, root] of platforms) {
  if (requestedPlatform && requestedPlatform !== name) continue;
  if (!existsSync(dirname(root))) continue;
  const failures = [];
  let payloads = 0;
  let commandFallbacks = 0;
  let shadowedCollisions = 0;

  for (const skill of skills) {
    const installedCandidates = locateInstalled(root, skill);
    if (!installedCandidates.length) {
      if (name === 'codex' && codexPromptAvailable(skill)) {
        commandFallbacks += 1;
        continue;
      }
      failures.push({ skill: skill.name, issues: ['missing:skill-directory'] });
      continue;
    }
    const candidateResults = installedCandidates.map(installed => {
      const issues = comparePayload(skill.dir, installed);
      if (name === 'minimax') {
        const meta = join(installed, '_meta.json');
        if (!existsSync(meta)) issues.push('missing:_meta.json');
        else {
          try {
            if (JSON.parse(readFileSync(meta, 'utf8')).platform !== 'minimax') {
              issues.push('invalid:_meta.json');
            }
          } catch {
            issues.push('invalid:_meta.json');
          }
        }
      }
      return { installed, issues };
    });
    const matching = candidateResults.find(candidate => candidate.issues.length === 0);
    if (matching) {
      payloads += 1;
      if (candidateResults[0] !== matching) shadowedCollisions += 1;
    } else {
      failures.push({
        skill: skill.name,
        issues: candidateResults[0].issues,
        candidates: candidateResults.map(candidate => candidate.installed),
      });
    }
  }

  results.push({
    platform: name,
    root,
    skills_expected: skills.length,
    payloads_verified: payloads,
    command_fallbacks: commandFallbacks,
    shadowed_collisions: shadowedCollisions,
    failures,
    ok: failures.length === 0 && payloads + commandFallbacks === skills.length,
  });
}

const summary = {
  repo_root: repoRoot,
  skills_discovered: skills.length,
  platforms_checked: results.length,
  platforms_ok: results.filter(result => result.ok).length,
  ok: results.length > 0 && results.every(result => result.ok),
  results,
};

if (jsonOutput) console.log(JSON.stringify(summary, null, 2));
else {
  console.log(`Skills discovered: ${skills.length}`);
  for (const result of results) {
    console.log(
      `${result.ok ? 'PASS' : 'FAIL'} ${result.platform}: ` +
        `${result.payloads_verified} payloads, ${result.command_fallbacks} command fallbacks, ` +
        `${result.failures.length} failures`
    );
    for (const failure of result.failures.slice(0, 10)) {
      console.log(`  - ${failure.skill}: ${failure.issues.slice(0, 5).join(', ')}`);
    }
    if (result.failures.length > 10) {
      console.log(`  ... ${result.failures.length - 10} more`);
    }
  }
}

process.exit(summary.ok ? 0 : 1);
