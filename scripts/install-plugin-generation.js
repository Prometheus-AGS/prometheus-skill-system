#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const TARGETS = [
  '.claude/skills',
  '.opencode/skills',
  '.kimi-code/skills',
  '.minimax/skills',
  '.cursor/skills',
  '.codex/skills',
  '.gemini/skills',
  '.roo/skills',
  '.windsurf/skills',
  '.codeium/windsurf/skills',
  '.agents/skills',
  '.config/zed/skills',
  '.zed/skills',
  '.cline/skills',
];
const COPY_TARGETS = new Set(['.minimax/skills', '.codex/skills']);
const REQUIRED_SCRIPTS = [
  'karpathy-hook-dispatch.sh',
  'detect-project-context.sh',
  'memory-outbox-flush.sh',
  'pk-health.sh',
  'enqueue-learning-job.py',
  'enqueue-memory-operation.py',
];
const STABLE_SCRIPTS = REQUIRED_SCRIPTS.slice(0, 4);
const STABLE_HELPERS = REQUIRED_SCRIPTS.slice(4);
const STABLE_DIRECTORIES = ['lib'];
const PAYLOAD_ROOTS = [
  'skills',
  'agents',
  'hooks',
  'shared',
  'scripts',
  '.claude-plugin',
  '.codex-plugin',
  '.agents/plugins',
  '.mcp.json',
];

function fail(message) {
  throw new Error(message);
}

function parseArgs(argv) {
  const args = {
    sourceRoot: process.cwd(),
    pluginRoot: path.join(os.homedir(), '.prometheus/plugins/prometheus-skill-pack'),
    home: os.homedir(),
    verify: false,
    rollback: false,
    uninstall: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--verify') args.verify = true;
    else if (value === '--rollback') args.rollback = true;
    else if (value === '--uninstall') args.uninstall = true;
    else if (value === '--source-root') args.sourceRoot = argv[++index];
    else if (value === '--plugin-root') args.pluginRoot = argv[++index];
    else if (value === '--home') args.home = argv[++index];
    else fail(`unknown argument: ${value}`);
  }
  for (const key of ['sourceRoot', 'pluginRoot', 'home']) {
    if (!args[key])
      fail(`missing value for --${key.replace(/[A-Z]/g, c => `-${c.toLowerCase()}`)}`);
    args[key] = path.resolve(args[key]);
  }
  return args;
}

function assertSafeRoot(root, home) {
  const parsed = path.parse(root);
  if (home === path.parse(home).root) fail(`refusing unsafe home target: ${home}`);
  if (root === parsed.root || root === home || root.length < parsed.root.length + 8) {
    fail(`refusing unsafe plugin root: ${root}`);
  }
}

function sha256(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map(key => [key, canonical(value[key])])
    );
  }
  return value;
}

function canonicalJson(value) {
  return `${JSON.stringify(canonical(value), null, 2)}\n`;
}

function modeString(mode) {
  return (mode & 0o7777).toString(8).padStart(4, '0');
}

function isWithin(parent, candidate) {
  const relative = path.relative(parent, candidate);
  return (
    relative === '' ||
    (!relative.startsWith(`..${path.sep}`) && relative !== '..' && !path.isAbsolute(relative))
  );
}

function ensureDirectory(directory, mode = 0o755) {
  fs.mkdirSync(directory, { recursive: true, mode });
}

function syncPath(target) {
  const descriptor = fs.openSync(target, 'r');
  try {
    fs.fsyncSync(descriptor);
  } finally {
    fs.closeSync(descriptor);
  }
}

function syncDirectory(directory) {
  try {
    syncPath(directory);
  } catch (error) {
    if (!['EINVAL', 'ENOTSUP', 'EISDIR'].includes(error.code)) throw error;
  }
}

function atomicWrite(file, content, mode = 0o644) {
  ensureDirectory(path.dirname(file));
  const temporary = path.join(path.dirname(file), `.${path.basename(file)}.${process.pid}.tmp`);
  const descriptor = fs.openSync(temporary, 'wx', mode);
  try {
    fs.writeFileSync(descriptor, content);
    fs.fsyncSync(descriptor);
  } finally {
    fs.closeSync(descriptor);
  }
  fs.renameSync(temporary, file);
  syncDirectory(path.dirname(file));
}

function atomicSymlink(directory, name, target) {
  ensureDirectory(directory);
  const temporaryName = `.${name}.${process.pid}.tmp`;
  const temporary = path.join(directory, temporaryName);
  try {
    fs.unlinkSync(temporary);
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  }
  fs.symlinkSync(target, temporary);
  fs.renameSync(temporary, path.join(directory, name));
  syncDirectory(directory);
}

function copyEntry(source, destination, durable = true) {
  const stat = fs.lstatSync(source);
  if (stat.isSymbolicLink()) {
    ensureDirectory(path.dirname(destination));
    fs.symlinkSync(fs.readlinkSync(source), destination);
    return;
  }
  if (stat.isDirectory()) {
    ensureDirectory(destination, stat.mode & 0o7777);
    for (const name of fs.readdirSync(source).sort()) {
      if (name === 'node_modules' || name === 'target' || name === '.git') continue;
      copyEntry(path.join(source, name), path.join(destination, name), durable);
    }
    fs.chmodSync(destination, stat.mode & 0o7777);
    if (durable) syncDirectory(destination);
    return;
  }
  if (!stat.isFile()) fail(`unsupported payload entry: ${source}`);
  ensureDirectory(path.dirname(destination));
  fs.copyFileSync(source, destination, fs.constants.COPYFILE_EXCL);
  fs.chmodSync(destination, stat.mode & 0o7777);
  if (durable) syncPath(destination);
}

function collectManifestFiles(root, relative = '') {
  const result = [];
  const absolute = path.join(root, relative);
  for (const name of fs.readdirSync(absolute).sort()) {
    const itemRelative = path.posix.join(relative.split(path.sep).join('/'), name);
    if (itemRelative === 'manifest.json') continue;
    const itemAbsolute = path.join(root, ...itemRelative.split('/'));
    const stat = fs.lstatSync(itemAbsolute);
    if (stat.isDirectory()) {
      result.push(...collectManifestFiles(root, itemRelative));
    } else if (stat.isSymbolicLink()) {
      const target = fs.readlinkSync(itemAbsolute);
      result.push({
        path: itemRelative,
        type: 'symlink',
        sha256: sha256(target),
        size: Buffer.byteLength(target),
        mode: modeString(stat.mode),
      });
    } else if (stat.isFile()) {
      const bytes = fs.readFileSync(itemAbsolute);
      result.push({
        path: itemRelative,
        type: 'file',
        sha256: sha256(bytes),
        size: bytes.length,
        mode: modeString(stat.mode),
      });
    }
  }
  return result.sort((left, right) => left.path.localeCompare(right.path));
}

function readSkillName(skillFile) {
  const text = fs.readFileSync(skillFile, 'utf8');
  const frontmatter = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  const match = frontmatter?.[1].match(/^name:\s*['\"]?([^'\"\r\n]+)['\"]?/m);
  const name = match?.[1].trim() || path.basename(path.dirname(skillFile));
  if (!name || name === '.' || name === '..' || name.includes('/') || name.includes('\\')) {
    fail(`unsafe skill name in ${skillFile}: ${name}`);
  }
  return name;
}

function collectSkills(skillsRoot) {
  const skills = [];
  function visit(directory) {
    for (const name of fs.readdirSync(directory).sort()) {
      const entry = path.join(directory, name);
      const stat = fs.lstatSync(entry);
      if (!stat.isDirectory() || stat.isSymbolicLink()) continue;
      const relative = path.relative(skillsRoot, entry).split(path.sep);
      if (relative.some(part => ['imported', 'tests', 'fixtures'].includes(part))) continue;
      const skillFile = path.join(entry, 'SKILL.md');
      if (fs.existsSync(skillFile))
        skills.push({ name: readSkillName(skillFile), relative: path.relative(skillsRoot, entry) });
      visit(entry);
    }
  }
  visit(skillsRoot);
  skills.sort((left, right) => left.name.localeCompare(right.name));
  for (let index = 1; index < skills.length; index += 1) {
    if (skills[index - 1].name === skills[index].name) {
      fail(`duplicate installed skill name: ${skills[index].name}`);
    }
  }
  return skills;
}

function targetPayloads(skills) {
  const digest = sha256(canonicalJson(skills));
  return TARGETS.map(target => ({
    target,
    mode: COPY_TARGETS.has(target) ? 'copy' : 'symlink',
    skillCount: skills.length,
    skillsSha256: digest,
  }));
}

function verifyGeneration(generationPath, expectedName = path.basename(generationPath)) {
  const manifestPath = path.join(generationPath, 'manifest.json');
  if (!fs.existsSync(manifestPath)) fail(`generation has no manifest: ${generationPath}`);
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const identity = {
    schemaVersion: manifest.schemaVersion,
    sourceVersion: manifest.sourceVersion,
    files: manifest.files,
    targetPayloads: manifest.targetPayloads,
  };
  const digest = sha256(canonicalJson(identity));
  if (manifest.generation !== digest || expectedName !== digest)
    fail(`generation identity mismatch: ${generationPath}`);
  if (!Array.isArray(manifest.targetPayloads) || manifest.targetPayloads.length !== TARGETS.length)
    fail('generation does not certify all 14 targets');
  for (let index = 0; index < TARGETS.length; index += 1) {
    const payload = manifest.targetPayloads[index];
    const wantedMode = COPY_TARGETS.has(TARGETS[index]) ? 'copy' : 'symlink';
    if (payload?.target !== TARGETS[index] || payload?.mode !== wantedMode) {
      fail('generation target payload matrix does not match the canonical 14 targets');
    }
  }
  if (canonicalJson(collectManifestFiles(generationPath)) !== canonicalJson(manifest.files)) {
    fail(`generation contains unmanifested or missing payload files: ${generationPath}`);
  }
  for (const entry of manifest.files) {
    const absolute = path.join(generationPath, ...entry.path.split('/'));
    const stat = fs.lstatSync(absolute);
    const bytes = stat.isSymbolicLink()
      ? Buffer.from(fs.readlinkSync(absolute))
      : fs.readFileSync(absolute);
    const type = stat.isSymbolicLink() ? 'symlink' : 'file';
    if (
      type !== entry.type ||
      sha256(bytes) !== entry.sha256 ||
      bytes.length !== entry.size ||
      modeString(stat.mode) !== entry.mode
    ) {
      fail(`generation verification failed for ${entry.path}`);
    }
    if (
      stat.isSymbolicLink() &&
      !isWithin(generationPath, path.resolve(path.dirname(absolute), fs.readlinkSync(absolute)))
    ) {
      fail(`generation symlink escapes the immutable payload: ${entry.path}`);
    }
  }
  for (const script of REQUIRED_SCRIPTS) {
    const absolute = path.join(generationPath, 'shared/scripts', script);
    if (!fs.existsSync(absolute) || (fs.statSync(absolute).mode & 0o111) === 0)
      fail(`required script is missing or not executable: ${script}`);
  }
  if (
    !manifest.files.some(
      entry => /(^|\/)tests?\//.test(entry.path) || /(^|\/)fixtures?\//.test(entry.path)
    )
  ) {
    fail('generation contains no regression fixture or test');
  }
  return manifest;
}

function currentTarget(pluginRoot, name) {
  const pointer = path.join(pluginRoot, name);
  return fs.existsSync(pointer) || fs.lstatSync(pointer, { throwIfNoEntry: false })
    ? fs.readlinkSync(pointer)
    : null;
}

function isManagedSkillLink(destination, pluginRoot, skill) {
  const resolved = path.resolve(path.dirname(destination), fs.readlinkSync(destination));
  if (isWithin(pluginRoot, resolved)) return true;
  const legacySuffix = path.join('skills', skill.relative);
  if (!resolved.endsWith(`${path.sep}${legacySuffix}`)) return false;
  const skillFile = path.join(resolved, 'SKILL.md');
  return fs.existsSync(skillFile) && readSkillName(skillFile) === skill.name;
}

function installLinkTarget(targetRoot, skill, pluginRoot) {
  ensureDirectory(targetRoot);
  let destination = path.join(targetRoot, skill.name);
  const managedTarget = path.join(pluginRoot, 'current/skills', skill.relative);
  const existing = fs.lstatSync(destination, { throwIfNoEntry: false });
  if (
    existing &&
    !(existing.isSymbolicLink() && isManagedSkillLink(destination, pluginRoot, skill))
  ) {
    destination = path.join(targetRoot, `prometheus-${skill.name}`);
  }
  const fallback = fs.lstatSync(destination, { throwIfNoEntry: false });
  if (
    fallback &&
    !(fallback.isSymbolicLink() && isManagedSkillLink(destination, pluginRoot, skill))
  )
    return;
  const relativeTarget = path.relative(path.dirname(destination), managedTarget);
  const temporary = `${destination}.${process.pid}.tmp`;
  try {
    fs.unlinkSync(temporary);
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  }
  fs.symlinkSync(relativeTarget, temporary);
  fs.renameSync(temporary, destination);
}

function isManagedCopy(destination, target) {
  if (fs.existsSync(path.join(destination, '.prometheus-generation'))) return true;
  if (target === '.codex/skills' && fs.existsSync(path.join(destination, '.prometheus-pack')))
    return true;
  if (target === '.minimax/skills') {
    try {
      return (
        JSON.parse(fs.readFileSync(path.join(destination, '_meta.json'), 'utf8')).platform ===
        'minimax'
      );
    } catch {
      return false;
    }
  }
  return false;
}

function copySkill(source, targetRoot, target, skill, generation) {
  ensureDirectory(targetRoot);
  let destination = path.join(targetRoot, skill.name);
  if (fs.existsSync(destination) && !isManagedCopy(destination, target))
    destination = path.join(targetRoot, `prometheus-${skill.name}`);
  if (fs.existsSync(destination) && !isManagedCopy(destination, target)) return;
  const temporary = path.join(targetRoot, `.${path.basename(destination)}.${process.pid}.tmp`);
  fs.rmSync(temporary, { recursive: true, force: true });
  // The immutable generation was fsynced before activation. Copy-based
  // platform projections are replaceable caches, so fsync only their receipt
  // and parent-directory rename instead of flushing thousands of duplicate files.
  copyEntry(source, temporary, false);
  if (target === '.minimax/skills') {
    atomicWrite(
      path.join(temporary, '_meta.json'),
      canonicalJson({ platform: 'minimax', name: skill.name, generation })
    );
  }
  atomicWrite(path.join(temporary, '.prometheus-generation'), `${generation}\n`);
  const prior = `${destination}.${process.pid}.old`;
  fs.rmSync(prior, { recursive: true, force: true });
  if (fs.existsSync(destination)) fs.renameSync(destination, prior);
  fs.renameSync(temporary, destination);
  fs.rmSync(prior, { recursive: true, force: true });
  syncDirectory(targetRoot);
}

function installTargets(home, pluginRoot, generationPath, generation, skills) {
  for (const target of TARGETS) {
    const targetRoot = path.join(home, ...target.split('/'));
    for (const skill of skills) {
      if (COPY_TARGETS.has(target))
        copySkill(
          path.join(generationPath, 'skills', skill.relative),
          targetRoot,
          target,
          skill,
          generation
        );
      else installLinkTarget(targetRoot, skill, pluginRoot);
    }
  }
}

function targetDestination(targetRoot, target, skill, pluginRoot) {
  const primary = path.join(targetRoot, skill.name);
  const existing = fs.lstatSync(primary, { throwIfNoEntry: false });
  if (!existing) return primary;
  if (existing.isSymbolicLink()) {
    const resolved = path.resolve(path.dirname(primary), fs.readlinkSync(primary));
    if (isWithin(pluginRoot, resolved)) return primary;
  }
  if (isManagedCopy(primary, target)) return primary;
  return path.join(targetRoot, `prometheus-${skill.name}`);
}

function verifyTargets(home, pluginRoot, generationPath, generation, skills) {
  for (const target of TARGETS) {
    const targetRoot = path.join(home, ...target.split('/'));
    for (const skill of skills) {
      const destination = targetDestination(targetRoot, target, skill, pluginRoot);
      if (COPY_TARGETS.has(target)) {
        const receipt = path.join(destination, '.prometheus-generation');
        const sourceSkill = path.join(generationPath, 'skills', skill.relative, 'SKILL.md');
        const targetSkill = path.join(destination, 'SKILL.md');
        const minimaxMetadataValid =
          target !== '.minimax/skills' ||
          JSON.parse(fs.readFileSync(path.join(destination, '_meta.json'), 'utf8')).platform ===
            'minimax';
        if (
          !fs.existsSync(receipt) ||
          fs.readFileSync(receipt, 'utf8').trim() !== generation ||
          !fs.existsSync(targetSkill) ||
          !minimaxMetadataValid ||
          sha256(fs.readFileSync(sourceSkill)) !== sha256(fs.readFileSync(targetSkill))
        ) {
          fail(`copy target validation failed: ${target}/${skill.name}`);
        }
      } else {
        const stat = fs.lstatSync(destination, { throwIfNoEntry: false });
        const wanted = path.join(pluginRoot, 'current/skills', skill.relative);
        if (
          !stat?.isSymbolicLink() ||
          path.resolve(path.dirname(destination), fs.readlinkSync(destination)) !== wanted
        ) {
          fail(`symlink target validation failed: ${target}/${skill.name}`);
        }
      }
    }
  }
}

function createStableDispatchers(pluginRoot) {
  const stable = path.join(pluginRoot, 'stable');
  ensureDirectory(stable);
  for (const script of [...STABLE_SCRIPTS, ...STABLE_HELPERS]) {
    atomicSymlink(stable, script, `../current/shared/scripts/${script}`);
  }
  for (const directory of STABLE_DIRECTORIES) {
    atomicSymlink(stable, directory, `../current/shared/scripts/${directory}`);
  }
}

function verifyStableDispatchers(pluginRoot) {
  const stable = path.join(pluginRoot, 'stable');
  for (const script of [...STABLE_SCRIPTS, ...STABLE_HELPERS]) {
    const projected = path.join(stable, script);
    const stat = fs.lstatSync(projected, { throwIfNoEntry: false });
    const expected = path.join(pluginRoot, 'current/shared/scripts', script);
    if (
      !stat?.isSymbolicLink() ||
      path.resolve(stable, fs.readlinkSync(projected)) !== expected ||
      !fs.statSync(projected).isFile() ||
      (fs.statSync(projected).mode & 0o111) === 0
    ) {
      fail(`stable script projection is invalid: ${script}`);
    }
  }
  for (const directory of STABLE_DIRECTORIES) {
    const projected = path.join(stable, directory);
    const stat = fs.lstatSync(projected, { throwIfNoEntry: false });
    const expected = path.join(pluginRoot, 'current/shared/scripts', directory);
    if (
      !stat?.isSymbolicLink() ||
      path.resolve(stable, fs.readlinkSync(projected)) !== expected ||
      !fs.statSync(projected).isDirectory()
    ) {
      fail(`stable support directory projection is invalid: ${directory}`);
    }
  }
}

function uninstall(home, pluginRoot) {
  for (const target of TARGETS) {
    const targetRoot = path.join(home, ...target.split('/'));
    if (!fs.existsSync(targetRoot)) continue;
    for (const name of fs.readdirSync(targetRoot)) {
      const destination = path.join(targetRoot, name);
      const stat = fs.lstatSync(destination, { throwIfNoEntry: false });
      if (
        stat?.isSymbolicLink() &&
        isWithin(pluginRoot, path.resolve(targetRoot, fs.readlinkSync(destination)))
      ) {
        fs.unlinkSync(destination);
      } else if (stat?.isDirectory() && isManagedCopy(destination, target)) {
        fs.rmSync(destination, { recursive: true, force: true });
      }
    }
    syncDirectory(targetRoot);
  }
  if (fs.existsSync(pluginRoot)) {
    fs.rmSync(pluginRoot, { recursive: true, force: true });
    syncDirectory(path.dirname(pluginRoot));
  }
  return 'uninstalled';
}

function verifyActive(pluginRoot) {
  const target = currentTarget(pluginRoot, 'current');
  if (!target) fail('no active plugin generation');
  const resolved = path.resolve(pluginRoot, target);
  if (!isWithin(path.join(pluginRoot, 'generations'), resolved))
    fail('active pointer escapes generations directory');
  const manifest = verifyGeneration(resolved, path.basename(resolved));
  verifyStableDispatchers(pluginRoot);
  return manifest;
}

function rollback(pluginRoot, home) {
  const active = currentTarget(pluginRoot, 'current');
  const previous = currentTarget(pluginRoot, 'previous');
  if (!active || !previous) fail('rollback requires current and previous generations');
  const generationPath = path.resolve(pluginRoot, previous);
  if (!isWithin(path.join(pluginRoot, 'generations'), generationPath))
    fail('previous pointer escapes generations directory');
  const manifest = verifyGeneration(generationPath, path.basename(previous));
  const skills = collectSkills(path.join(generationPath, 'skills'));
  installTargets(home, pluginRoot, generationPath, manifest.generation, skills);
  verifyTargets(home, pluginRoot, generationPath, manifest.generation, skills);
  atomicSymlink(pluginRoot, 'current', previous);
  atomicSymlink(pluginRoot, 'previous', active);
  createStableDispatchers(pluginRoot);
  return path.basename(previous);
}

function install(args) {
  const source = args.sourceRoot;
  for (const required of ['skills', 'shared/scripts', 'hooks', '.claude-plugin', '.codex-plugin']) {
    if (!fs.existsSync(path.join(source, required))) fail(`source payload is missing ${required}`);
  }
  for (const script of REQUIRED_SCRIPTS) {
    const absolute = path.join(source, 'shared/scripts', script);
    if (!fs.existsSync(absolute) || (fs.statSync(absolute).mode & 0o111) === 0)
      fail(`required script is missing or not executable: ${script}`);
  }

  const generations = path.join(args.pluginRoot, 'generations');
  ensureDirectory(generations);
  const staging = path.join(
    generations,
    `.staging-${process.pid}-${crypto.randomBytes(6).toString('hex')}`
  );
  ensureDirectory(staging);
  try {
    for (const root of PAYLOAD_ROOTS) {
      const absolute = path.join(source, root);
      if (fs.existsSync(absolute)) copyEntry(absolute, path.join(staging, root));
    }
    const skills = collectSkills(path.join(staging, 'skills'));
    if (skills.length === 0) fail('generation contains no installable skills');
    const pluginMetadata = JSON.parse(
      fs.readFileSync(path.join(staging, '.claude-plugin/plugin.json'), 'utf8')
    );
    const identity = {
      schemaVersion: 1,
      sourceVersion: String(pluginMetadata.version ?? 'unknown'),
      files: collectManifestFiles(staging),
      targetPayloads: targetPayloads(skills),
    };
    const generation = sha256(canonicalJson(identity));
    const manifest = { ...identity, generation };
    atomicWrite(path.join(staging, 'manifest.json'), canonicalJson(manifest));
    verifyGeneration(staging, generation);

    const generationPath = path.join(generations, generation);
    if (fs.existsSync(generationPath)) {
      verifyGeneration(generationPath, generation);
      fs.rmSync(staging, { recursive: true, force: true });
    } else {
      fs.renameSync(staging, generationPath);
      syncDirectory(generations);
    }

    installTargets(args.home, args.pluginRoot, generationPath, generation, skills);
    verifyTargets(args.home, args.pluginRoot, generationPath, generation, skills);
    const active = currentTarget(args.pluginRoot, 'current');
    if (active !== `generations/${generation}`) {
      if (active) atomicSymlink(args.pluginRoot, 'previous', active);
      atomicSymlink(args.pluginRoot, 'current', `generations/${generation}`);
    }
    createStableDispatchers(args.pluginRoot);
    verifyActive(args.pluginRoot);
    return generation;
  } catch (error) {
    fs.rmSync(staging, { recursive: true, force: true });
    throw error;
  }
}

const args = parseArgs(process.argv.slice(2));
assertSafeRoot(args.pluginRoot, args.home);
try {
  let generation;
  if (args.verify) generation = verifyActive(args.pluginRoot).generation;
  else if (args.rollback) generation = rollback(args.pluginRoot, args.home);
  else if (args.uninstall) generation = uninstall(args.home, args.pluginRoot);
  else generation = install(args);
  process.stdout.write(`${generation}\n`);
} catch (error) {
  process.stderr.write(`install-plugin-generation: ${error.message}\n`);
  process.exitCode = 1;
}
