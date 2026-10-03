import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { writeJson } from './state.mjs';

const upstream = (runtime, relative) => import(pathToFileURL(path.join(runtime.root, 'dist', 'core', relative)).href);

export function legacyCommandPath(commandPath, currentRoot, legacyRoot) {
  if (path.isAbsolute(commandPath)) return undefined;
  const segments = commandPath.split(/[\\/]/);
  if (segments[0] !== currentRoot) return undefined;
  segments[0] = legacyRoot;
  return path.join(...segments);
}

// Read the selected official CLI's own path declarations. If its internal API
// changes, refuse before update rather than guess which user files it may delete.
export async function updateFootprint(runtime, project) {
  const [config, profiles, commands, legacy, migration, global, skills] = await Promise.all([
    upstream(runtime, 'config.js'), upstream(runtime, 'profiles.js'),
    upstream(runtime, 'command-generation/index.js'), upstream(runtime, 'legacy-cleanup.js'),
    upstream(runtime, 'migration.js'), upstream(runtime, 'global-config.js'),
    upstream(runtime, 'shared/skill-paths.js'),
  ]);
  const paths = new Set();
  const add = value => paths.add(path.isAbsolute(value) ? value : path.resolve(project, value));
  for (const tool of config.AI_TOOLS) {
    if (!tool.skillsDir && !tool.globalSkillsDir) continue;
    const skillRoot = skills.resolveToolSkillsDir(project, tool);
    for (const name of config.OPENSPEC_SKILL_NAMES) add(path.join(skillRoot, name));
    add(path.join(skillRoot, '.openspec-target'));
    const legacyRoots = new Set([
      ...(tool.legacySkillsDirs || []),
      ...(migration.LEGACY_TOOL_ROOTS[tool.value] || []).map(entry => entry.root),
    ]);
    for (const root of legacyRoots) {
      for (const name of config.OPENSPEC_SKILL_NAMES) add(path.join(root, 'skills', name));
      add(path.join(root, 'skills', '.openspec-target'));
    }
    const adapter = commands.CommandAdapterRegistry.get(tool.value);
    if (!adapter) continue;
    for (const workflow of profiles.ALL_WORKFLOWS) {
      const command = adapter.getFilePath(workflow);
      add(command);
      for (const root of legacyRoots) {
        const legacyCommand = legacyCommandPath(command, tool.skillsDir, root);
        if (legacyCommand) add(legacyCommand);
      }
    }
  }
  const detected = await legacy.detectLegacyArtifacts(project);
  for (const key of ['configFilesToUpdate', 'slashCommandDirs', 'slashCommandFiles', 'globalSlashCommandFiles']) {
    for (const file of detected[key]) add(file);
  }
  add('openspec/AGENTS.md');
  add(global.getGlobalConfigPath());
  for (const file of ['.github/workflows/copilot-setup-steps.yml', '.github/agents/openspec.agent.md', '.github/agents/openspec.md']) add(file);
  return { paths: [...paths].sort(), legacyProject: detected.hasProjectMd };
}

function fingerprint(file, remaining) {
  remaining();
  const stat = fs.lstatSync(file, { throwIfNoEntry: false });
  if (!stat) return null;
  if (stat.isSymbolicLink()) return { link: fs.readlinkSync(file) };
  if (stat.isDirectory()) return Object.fromEntries(fs.readdirSync(file).sort().map(name => [name, fingerprint(path.join(file, name), remaining)]));
  if (!stat.isFile()) throw new Error(`Cannot back up a non-file OpenSpec artifact: ${file}`);
  return { sha256: crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex'), mode: stat.mode & 0o777 };
}

export function backupArtifacts(home, project, version, footprint, remaining) {
  const backup = path.join(home, 'backups', `${Date.now()}-${crypto.randomUUID()}`);
  const protectedPaths = [
    'openspec/specs', 'openspec/changes', 'openspec/config.yaml', 'openspec/project.md',
    '.kbd-orchestrator/project.json', '.kbd-orchestrator/current-waypoint.json', '.prometheus/project.json',
  ].map(relative => path.join(project, relative));
  const authored = protectedPaths.map(file => ({ path: file, before: fingerprint(file, remaining) }));
  const entries = [];
  fs.mkdirSync(backup, { recursive: true, mode: 0o700 });
  for (const file of footprint.paths) {
    const before = fingerprint(file, remaining);
    const saved = path.join(backup, 'files', String(entries.length));
    const symlinks = [];
    if (before !== null && !before.link) {
      fs.mkdirSync(path.dirname(saved), { recursive: true });
      fs.cpSync(file, saved, {
        recursive: true, dereference: false, preserveTimestamps: true,
        filter(source) {
          remaining();
          if (!fs.lstatSync(source).isSymbolicLink()) return true;
          symlinks.push({ path: source, target: fs.readlinkSync(source) });
          return false;
        },
      });
    }
    if (before?.link) symlinks.push({ path: file, target: before.link });
    entries.push({ path: file, saved: before === null || before.link ? null : saved, before, symlinks });
  }
  writeJson(path.join(backup, 'manifest.json'), { schemaVersion: 1, project, version, createdAt: new Date().toISOString(), entries, authored });
  return { backup, entries, authored };
}

// Audit without attributing changes to upstream: another agent may have edited
// authored state concurrently. Never roll back that agent's legitimate work.
export function auditAuthored(snapshot, remaining) {
  return snapshot.authored.filter(entry => JSON.stringify(fingerprint(entry.path, remaining)) !== JSON.stringify(entry.before)).map(entry => entry.path);
}
