#!/usr/bin/env node

import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
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
  'hook-runtime-v1.sh',
  'bootstrap-hook-runtime.sh',
  'generated/hook-dispatch-v1.sh',
  'karpathy-hook-dispatch.sh',
  'detect-project-context.sh',
  'memory-outbox-flush.sh',
  'pk-health.sh',
  'enqueue-learning-job.py',
  'enqueue-memory-operation.py',
];
const STABLE_SCRIPTS = [
  'karpathy-hook-dispatch.sh',
  'detect-project-context.sh',
  'memory-outbox-flush.sh',
  'pk-health.sh',
];
const STABLE_HELPERS = ['enqueue-learning-job.py', 'enqueue-memory-operation.py'];
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
const MANIFEST_SIGNATURE = 'manifest.sig.json';
const SKILL_INDEX_SCHEMA = 'prometheus-skill-index-v1';
const SIGNATURE_NAMESPACE = 'prometheus-plugin-generation-v1';

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
    expectedBundle: null,
    signingKey: null,
    trustStore: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--verify') args.verify = true;
    else if (value === '--rollback') args.rollback = true;
    else if (value === '--uninstall') args.uninstall = true;
    else if (value === '--expected-bundle') args.expectedBundle = argv[++index];
    else if (value === '--source-root') args.sourceRoot = argv[++index];
    else if (value === '--plugin-root') args.pluginRoot = argv[++index];
    else if (value === '--home') args.home = argv[++index];
    else if (value === '--signing-key') args.signingKey = argv[++index];
    else if (value === '--trust-store') args.trustStore = argv[++index];
    else fail(`unknown argument: ${value}`);
  }
  for (const key of ['sourceRoot', 'pluginRoot', 'home']) {
    if (!args[key])
      fail(`missing value for --${key.replace(/[A-Z]/g, c => `-${c.toLowerCase()}`)}`);
    args[key] = path.resolve(args[key]);
  }
  args.signingKey = path.resolve(
    args.signingKey ?? path.join(args.home, '.prometheus/plugin-signing/ed25519-private.pem')
  );
  args.trustStore = path.resolve(
    args.trustStore ?? path.join(args.pluginRoot, 'trust/allowed-signers.json')
  );
  if (args.expectedBundle && !/^[a-f0-9]{64}$/.test(args.expectedBundle)) {
    fail('invalid value for --expected-bundle');
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

function keyId(publicKey) {
  return sha256(publicKey.export({ type: 'spki', format: 'der' }));
}

function ensureSigningIdentity(privateKeyPath, trustStorePath) {
  ensureDirectory(path.dirname(privateKeyPath), 0o700);
  const trustExists = fs.existsSync(trustStorePath);
  let privateKey;
  if (fs.existsSync(privateKeyPath)) {
    const mode = fs.statSync(privateKeyPath).mode & 0o777;
    if (mode !== 0o600) fail(`plugin signing key must have mode 0600: ${privateKeyPath}`);
    privateKey = crypto.createPrivateKey(fs.readFileSync(privateKeyPath));
  } else {
    if (trustExists) {
      fail(`plugin signing key is missing but a trust store already exists: ${privateKeyPath}`);
    }
    const pair = crypto.generateKeyPairSync('ed25519');
    atomicWrite(privateKeyPath, pair.privateKey.export({ type: 'pkcs8', format: 'pem' }), 0o600);
    privateKey = pair.privateKey;
  }
  const publicKey = crypto.createPublicKey(privateKey);
  const signer = keyId(publicKey);
  const encoded = publicKey.export({ type: 'spki', format: 'pem' }).toString();
  ensureDirectory(path.dirname(trustStorePath), 0o700);
  let trust = { schemaVersion: 1, signers: [] };
  if (trustExists) trust = JSON.parse(fs.readFileSync(trustStorePath, 'utf8'));
  if (!Array.isArray(trust.signers)) fail('plugin trust store has no signers array');
  const existing = trust.signers.find(entry => entry.keyId === signer);
  if (existing && existing.publicKey !== encoded) fail(`plugin signer collision: ${signer}`);
  if (!existing && trustExists) fail(`plugin signer is not enrolled: ${signer}`);
  if (!existing) {
    trust.signers.push({ keyId: signer, algorithm: 'Ed25519', publicKey: encoded });
    trust.signers.sort((left, right) => left.keyId.localeCompare(right.keyId));
    atomicWrite(trustStorePath, canonicalJson(trust), 0o600);
  }
  return { privateKey, publicKey, keyId: signer };
}

function readTrustedKey(trustStorePath, signer) {
  if (!fs.existsSync(trustStorePath)) fail(`plugin trust store is missing: ${trustStorePath}`);
  const trust = JSON.parse(fs.readFileSync(trustStorePath, 'utf8'));
  const entry = trust.signers?.find(candidate => candidate.keyId === signer);
  if (!entry || entry.algorithm !== 'Ed25519') fail(`untrusted plugin signer: ${signer}`);
  const publicKey = crypto.createPublicKey(entry.publicKey);
  if (keyId(publicKey) !== signer) fail(`plugin trust-store key fingerprint mismatch: ${signer}`);
  return publicKey;
}

function signaturePayload(value) {
  return Buffer.from(`${SIGNATURE_NAMESPACE}\n${canonicalJson(value)}`);
}

function signValue(value, identity) {
  return {
    schemaVersion: 1,
    namespace: SIGNATURE_NAMESPACE,
    algorithm: 'Ed25519',
    signerKeyId: identity.keyId,
    signature: crypto.sign(null, signaturePayload(value), identity.privateKey).toString('base64'),
  };
}

function verifySignedValue(value, signature, trustStorePath) {
  if (
    signature?.schemaVersion !== 1 ||
    signature?.namespace !== SIGNATURE_NAMESPACE ||
    signature?.algorithm !== 'Ed25519'
  ) {
    fail('invalid plugin signature envelope');
  }
  const publicKey = readTrustedKey(trustStorePath, signature.signerKeyId);
  if (
    !crypto.verify(
      null,
      signaturePayload(value),
      publicKey,
      Buffer.from(signature.signature ?? '', 'base64')
    )
  ) {
    fail('plugin signature verification failed');
  }
  return signature.signerKeyId;
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
    if (itemRelative === 'manifest.json' || itemRelative === MANIFEST_SIGNATURE) continue;
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

function readSkillDescription(skillFile) {
  const text = fs.readFileSync(skillFile, 'utf8').replace(/\r\n/g, '\n');
  const frontmatter = text.match(/^---\n([\s\S]*?)\n---/)?.[1] ?? '';
  const inline = frontmatter.match(/^description:\s*(?![>|])['"]?([^'"\n]+)['"]?\s*$/m);
  if (inline) return inline[1].replace(/\s+/g, ' ').trim();
  const folded = frontmatter.match(/^description:\s*[>|][-+]?\s*\n((?:[ \t]+\S[^\n]*\n?)+)/m);
  return (folded?.[1] ?? '')
    .split('\n')
    .map(line => line.trim())
    .filter(Boolean)
    .join(' ');
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
        skills.push({
          name: readSkillName(skillFile),
          description: readSkillDescription(skillFile),
          relative: path.relative(skillsRoot, entry).split(path.sep).join('/'),
        });
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

function buildSkillIndex(skills) {
  const entries = skills.map(skill => ({
    id: skill.name,
    description: skill.description,
    relativePath: skill.relative,
    searchText: `${skill.name} ${skill.description}`.toLocaleLowerCase('en-US'),
  }));
  const body = { schemaVersion: SKILL_INDEX_SCHEMA, entries };
  return { ...body, sha256: sha256(canonicalJson(body)) };
}

function sourceProvenance(sourceRoot) {
  const git = (...args) => {
    const result = spawnSync('git', args, { cwd: sourceRoot, encoding: 'utf8' });
    return result.status === 0 ? result.stdout.trim() : null;
  };
  const sourceCommit = git('rev-parse', 'HEAD');
  const sourceTreeState = git('status', '--porcelain') ? 'modified' : 'clean';
  const configuredPaths = git('config', '-f', '.gitmodules', '--get-regexp', 'path');
  const externalSources = [];
  if (configuredPaths) {
    for (const line of configuredPaths.split('\n').filter(Boolean)) {
      const sourcePath = line.trim().split(/\s+/).at(-1);
      const treeEntry = git('ls-tree', 'HEAD', '--', sourcePath);
      const match = treeEntry?.match(/^160000 commit ([a-f0-9]{40,64})\t/);
      if (!match) fail(`external source has no commit gitlink: ${sourcePath}`);
      const checkedOutCommit = fs.existsSync(path.join(sourceRoot, sourcePath, '.git'))
        ? git('-C', sourcePath, 'rev-parse', 'HEAD')
        : null;
      if (checkedOutCommit && checkedOutCommit !== match[1]) {
        fail(`external source checkout differs from its commit pin: ${sourcePath}`);
      }
      externalSources.push({ path: sourcePath, commit: match[1] });
    }
  }
  externalSources.sort((left, right) => left.path.localeCompare(right.path));
  return { sourceCommit, sourceTreeState, externalSources };
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

function verifyReleaseManifest(payloadRoot, expectedBundle = null) {
  const manifestPath = path.join(payloadRoot, 'shared/harnesses/generated/release-manifest.json');
  if (!fs.existsSync(manifestPath)) fail(`release manifest is missing: ${manifestPath}`);
  const release = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const identity = {
    schemaVersion: release.schemaVersion,
    sourceVersion: release.sourceVersion,
    contractSchemaVersion: release.contractSchemaVersion,
    dispatcherAbi: release.dispatcherAbi,
    contractSha256: release.contractSha256,
    runtimeFiles: release.runtimeFiles,
  };
  const bundleId = sha256(canonicalJson(identity));
  if (release.bundleId !== bundleId) fail('release manifest bundle identity mismatch');
  if (expectedBundle && bundleId !== expectedBundle) {
    fail(`release bundle ${bundleId} does not match expected ${expectedBundle}`);
  }
  if (
    release.dispatcherAbi !== 'hook-runtime-v1' ||
    !Array.isArray(release.runtimeFiles) ||
    release.runtimeFiles.length === 0
  ) {
    fail('release manifest hook runtime contract is invalid');
  }
  const seen = new Set();
  for (const entry of release.runtimeFiles) {
    const normalized = path.posix.normalize(entry.path ?? '');
    if (
      !normalized ||
      normalized !== entry.path ||
      normalized.startsWith('../') ||
      path.isAbsolute(normalized) ||
      seen.has(normalized)
    ) {
      fail(`unsafe or duplicate release payload path: ${entry.path}`);
    }
    seen.add(normalized);
    const absolute = path.join(payloadRoot, ...normalized.split('/'));
    const stat = fs.lstatSync(absolute, { throwIfNoEntry: false });
    if (
      !stat?.isFile() ||
      sha256(fs.readFileSync(absolute)) !== entry.sha256 ||
      modeString(stat.mode) !== entry.mode
    ) {
      fail(`release payload verification failed for ${normalized}`);
    }
  }
  return release;
}

function verifyGeneration(
  generationPath,
  expectedName = path.basename(generationPath),
  trustStorePath = path.join(
    path.dirname(path.dirname(generationPath)),
    'trust/allowed-signers.json'
  )
) {
  const manifestPath = path.join(generationPath, 'manifest.json');
  if (!fs.existsSync(manifestPath)) fail(`generation has no manifest: ${generationPath}`);
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const identity = {
    schemaVersion: manifest.schemaVersion,
    sourceVersion: manifest.sourceVersion,
    signerKeyId: manifest.signerKeyId,
    bundleId: manifest.bundleId,
    hookRuntime: manifest.hookRuntime,
    sourceProvenance: manifest.sourceProvenance,
    skillIndex: manifest.skillIndex,
    files: manifest.files,
    targetPayloads: manifest.targetPayloads,
  };
  const digest = sha256(canonicalJson(identity));
  if (manifest.generation !== digest || expectedName !== digest)
    fail(`generation identity mismatch: ${generationPath}`);
  const signaturePath = path.join(generationPath, MANIFEST_SIGNATURE);
  if (!fs.existsSync(signaturePath)) fail(`generation signature is missing: ${generationPath}`);
  const signerKeyId = verifySignedValue(
    manifest,
    JSON.parse(fs.readFileSync(signaturePath, 'utf8')),
    trustStorePath
  );
  if (manifest.signerKeyId !== signerKeyId) fail('manifest signer does not match its signature');
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
  const indexPath = path.join(generationPath, 'indexes/skills.json');
  const mobilePath = path.join(generationPath, 'mobile/skill-index.json');
  const agentPath = path.join(generationPath, 'agents/skill-index.json');
  const parityPath = path.join(generationPath, 'mobile/parity.json');
  if (
    !fs.existsSync(indexPath) ||
    !fs.existsSync(mobilePath) ||
    !fs.existsSync(agentPath) ||
    !fs.existsSync(parityPath)
  ) {
    fail('generation omits a host, generated-agent, or mobile skill-index projection');
  }
  const indexBytes = fs.readFileSync(indexPath);
  if (
    !indexBytes.equals(fs.readFileSync(mobilePath)) ||
    !indexBytes.equals(fs.readFileSync(agentPath))
  ) {
    fail('host/generated-agent/mobile skill indexes diverge');
  }
  const index = JSON.parse(indexBytes);
  const indexBody = { schemaVersion: index.schemaVersion, entries: index.entries };
  if (
    index.schemaVersion !== SKILL_INDEX_SCHEMA ||
    index.sha256 !== sha256(canonicalJson(indexBody)) ||
    manifest.skillIndex?.sha256 !== index.sha256 ||
    manifest.skillIndex?.entryCount !== index.entries.length
  ) {
    fail('generation skill index receipt is invalid');
  }
  const parity = JSON.parse(fs.readFileSync(parityPath, 'utf8'));
  if (
    parity.schemaVersion !== 1 ||
    parity.skillIndexSchema !== SKILL_INDEX_SCHEMA ||
    parity.skillIndexSha256 !== index.sha256 ||
    parity.entryCount !== index.entries.length
  ) {
    fail('mobile skill-index parity receipt is invalid');
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
  const release = verifyReleaseManifest(generationPath, manifest.bundleId);
  const dispatcher = release.runtimeFiles.find(
    entry => entry.path === 'shared/scripts/generated/hook-dispatch-v1.sh'
  );
  const runner = release.runtimeFiles.find(
    entry => entry.path === 'shared/scripts/hook-runtime-v1.sh'
  );
  if (
    manifest.hookRuntime?.abi !== 'hook-runtime-v1' ||
    manifest.hookRuntime?.bundleId !== manifest.bundleId ||
    manifest.hookRuntime?.dispatcherPath !== 'shared/scripts/generated/hook-dispatch-v1.sh' ||
    manifest.hookRuntime?.dispatcherSha256 !== dispatcher?.sha256 ||
    manifest.hookRuntime?.runnerSha256 !== runner?.sha256
  ) {
    fail('generation hook runtime receipt is invalid');
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
    fail(`skill target collision: ${destination}`);
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
  if (fs.existsSync(destination) && !isManagedCopy(destination, target)) {
    fail(`skill target collision: ${destination}`);
  }
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

function receiptFile(pluginRoot, generation, target) {
  return path.join(
    pluginRoot,
    'receipts',
    generation,
    `${target.replaceAll('/', '__').replace(/^\./, '')}.json`
  );
}

function targetReceipt(manifest, targetPayload) {
  return {
    schemaVersion: 1,
    generation: manifest.generation,
    signerKeyId: manifest.signerKeyId,
    target: targetPayload.target,
    mode: targetPayload.mode,
    skillCount: targetPayload.skillCount,
    skillsSha256: targetPayload.skillsSha256,
    skillIndexSha256: manifest.skillIndex.sha256,
    status: 'verified',
  };
}

function writeTargetReceipts(pluginRoot, manifest, identity) {
  for (const targetPayload of manifest.targetPayloads) {
    const body = targetReceipt(manifest, targetPayload);
    atomicWrite(
      receiptFile(pluginRoot, manifest.generation, targetPayload.target),
      canonicalJson({ body, signature: signValue(body, identity) }),
      0o600
    );
  }
}

function verifyTargetReceipts(pluginRoot, manifest, trustStorePath) {
  if (manifest.targetPayloads.length !== TARGETS.length)
    fail('target receipt matrix is incomplete');
  for (const targetPayload of manifest.targetPayloads) {
    const file = receiptFile(pluginRoot, manifest.generation, targetPayload.target);
    if (!fs.existsSync(file)) fail(`target receipt is missing: ${targetPayload.target}`);
    const receipt = JSON.parse(fs.readFileSync(file, 'utf8'));
    const expected = targetReceipt(manifest, targetPayload);
    if (canonicalJson(receipt.body) !== canonicalJson(expected)) {
      fail(`target receipt body mismatch: ${targetPayload.target}`);
    }
    verifySignedValue(receipt.body, receipt.signature, trustStorePath);
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
  atomicSymlink(stable, 'skill-index.json', '../current/indexes/skills.json');
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
  const skillIndex = path.join(stable, 'skill-index.json');
  const indexStat = fs.lstatSync(skillIndex, { throwIfNoEntry: false });
  const expectedIndex = path.join(pluginRoot, 'current/indexes/skills.json');
  if (
    !indexStat?.isSymbolicLink() ||
    path.resolve(stable, fs.readlinkSync(skillIndex)) !== expectedIndex ||
    !fs.statSync(skillIndex).isFile()
  ) {
    fail('stable skill index projection is invalid');
  }
}

function verifyHookRuntime(
  pluginRoot,
  manifest,
  trustStorePath = path.join(pluginRoot, 'trust/allowed-signers.json')
) {
  const runner = path.join(pluginRoot, 'runtime/v1/run-hook');
  const runnerStat = fs.lstatSync(runner, { throwIfNoEntry: false });
  if (
    !runnerStat?.isFile() ||
    (runnerStat.mode & 0o111) === 0 ||
    sha256(fs.readFileSync(runner)) !== manifest.hookRuntime.runnerSha256
  ) {
    fail('stable hook runtime v1 is missing or invalid');
  }
  const bundleLink = path.join(pluginRoot, 'bundles', manifest.bundleId);
  const stat = fs.lstatSync(bundleLink, { throwIfNoEntry: false });
  if (!stat?.isSymbolicLink()) fail(`bundle index is missing: ${manifest.bundleId}`);
  const resolved = path.resolve(path.dirname(bundleLink), fs.readlinkSync(bundleLink));
  if (!isWithin(path.join(pluginRoot, 'generations'), resolved)) {
    fail(`bundle index escapes generations: ${manifest.bundleId}`);
  }
  const indexed = verifyGeneration(resolved, path.basename(resolved), trustStorePath);
  if (
    indexed.bundleId !== manifest.bundleId ||
    indexed.hookRuntime.dispatcherSha256 !== manifest.hookRuntime.dispatcherSha256
  ) {
    fail(`bundle index collision: ${manifest.bundleId}`);
  }
}

function validateBundleIndex(pluginRoot, generationPath, manifest, trustStorePath) {
  if (!isWithin(path.join(pluginRoot, 'generations'), generationPath)) {
    fail(`bundle generation escapes immutable storage: ${manifest.bundleId}`);
  }
  const bundles = path.join(pluginRoot, 'bundles');
  ensureDirectory(bundles);
  const bundleLink = path.join(bundles, manifest.bundleId);
  const existing = fs.lstatSync(bundleLink, { throwIfNoEntry: false });
  if (!existing) return;
  if (!existing.isSymbolicLink()) fail(`bundle index is not a symlink: ${manifest.bundleId}`);
  const resolved = path.resolve(bundles, fs.readlinkSync(bundleLink));
  if (!isWithin(path.join(pluginRoot, 'generations'), resolved)) {
    fail(`bundle index escapes generations: ${manifest.bundleId}`);
  }
  const indexed = verifyGeneration(resolved, path.basename(resolved), trustStorePath);
  if (
    indexed.bundleId !== manifest.bundleId ||
    indexed.hookRuntime.dispatcherSha256 !== manifest.hookRuntime.dispatcherSha256
  ) {
    fail(`bundle identity collision: ${manifest.bundleId}`);
  }
}

function installHookRuntime(pluginRoot, generationPath, manifest, trustStorePath) {
  validateBundleIndex(pluginRoot, generationPath, manifest, trustStorePath);
  const runnerSource = path.join(generationPath, 'shared/scripts/hook-runtime-v1.sh');
  atomicWrite(path.join(pluginRoot, 'runtime/v1/run-hook'), fs.readFileSync(runnerSource), 0o755);
  const bundles = path.join(pluginRoot, 'bundles');
  const expectedTarget = `../generations/${manifest.generation}`;
  const bundleLink = path.join(bundles, manifest.bundleId);
  const existingTarget = fs.existsSync(bundleLink)
    ? fs.readlinkSync(bundleLink, { encoding: 'utf8' })
    : null;
  if (existingTarget !== expectedTarget) {
    atomicSymlink(bundles, manifest.bundleId, `../generations/${manifest.generation}`);
  }
  verifyHookRuntime(pluginRoot, manifest, trustStorePath);
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

function verifyActive(
  pluginRoot,
  trustStorePath = path.join(pluginRoot, 'trust/allowed-signers.json')
) {
  const target = currentTarget(pluginRoot, 'current');
  if (!target) fail('no active plugin generation');
  const resolved = path.resolve(pluginRoot, target);
  if (!isWithin(path.join(pluginRoot, 'generations'), resolved))
    fail('active pointer escapes generations directory');
  const manifest = verifyGeneration(resolved, path.basename(resolved), trustStorePath);
  verifyStableDispatchers(pluginRoot);
  verifyHookRuntime(pluginRoot, manifest, trustStorePath);
  verifyTargetReceipts(pluginRoot, manifest, trustStorePath);
  return manifest;
}

function rollback(pluginRoot, home, trustStorePath) {
  const active = currentTarget(pluginRoot, 'current');
  const previous = currentTarget(pluginRoot, 'previous');
  if (!active || !previous) fail('rollback requires current and previous generations');
  const generationPath = path.resolve(pluginRoot, previous);
  if (!isWithin(path.join(pluginRoot, 'generations'), generationPath))
    fail('previous pointer escapes generations directory');
  const manifest = verifyGeneration(generationPath, path.basename(previous), trustStorePath);
  const skills = collectSkills(path.join(generationPath, 'skills'));
  validateBundleIndex(pluginRoot, generationPath, manifest, trustStorePath);
  installTargets(home, pluginRoot, generationPath, manifest.generation, skills);
  verifyTargets(home, pluginRoot, generationPath, manifest.generation, skills);
  installHookRuntime(pluginRoot, generationPath, manifest, trustStorePath);
  atomicSymlink(pluginRoot, 'current', previous);
  atomicSymlink(pluginRoot, 'previous', active);
  createStableDispatchers(pluginRoot);
  verifyTargetReceipts(pluginRoot, manifest, trustStorePath);
  return path.basename(previous);
}

function install(args) {
  const source = args.sourceRoot;
  for (const required of [
    'skills',
    'shared/scripts',
    'shared/harnesses/generated/release-manifest.json',
    'hooks',
    '.claude-plugin',
    '.codex-plugin',
  ]) {
    if (!fs.existsSync(path.join(source, required))) fail(`source payload is missing ${required}`);
  }
  for (const script of REQUIRED_SCRIPTS) {
    const absolute = path.join(source, 'shared/scripts', script);
    if (!fs.existsSync(absolute) || (fs.statSync(absolute).mode & 0o111) === 0)
      fail(`required script is missing or not executable: ${script}`);
  }

  const generations = path.join(args.pluginRoot, 'generations');
  ensureDirectory(generations);
  const signingIdentity = ensureSigningIdentity(args.signingKey, args.trustStore);
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
    const skillIndex = buildSkillIndex(skills);
    atomicWrite(path.join(staging, 'indexes/skills.json'), canonicalJson(skillIndex));
    atomicWrite(path.join(staging, 'mobile/skill-index.json'), canonicalJson(skillIndex));
    atomicWrite(path.join(staging, 'agents/skill-index.json'), canonicalJson(skillIndex));
    atomicWrite(
      path.join(staging, 'mobile/parity.json'),
      canonicalJson({
        schemaVersion: 1,
        skillIndexSchema: SKILL_INDEX_SCHEMA,
        skillIndexSha256: skillIndex.sha256,
        entryCount: skillIndex.entries.length,
      })
    );
    const pluginMetadata = JSON.parse(
      fs.readFileSync(path.join(staging, '.claude-plugin/plugin.json'), 'utf8')
    );
    const release = verifyReleaseManifest(staging, args.expectedBundle);
    const dispatcher = release.runtimeFiles.find(
      entry => entry.path === 'shared/scripts/generated/hook-dispatch-v1.sh'
    );
    const runner = release.runtimeFiles.find(
      entry => entry.path === 'shared/scripts/hook-runtime-v1.sh'
    );
    if (!dispatcher || !runner) fail('release manifest omits hook runtime payloads');
    const hookRuntime = {
      abi: release.dispatcherAbi,
      bundleId: release.bundleId,
      dispatcherPath: dispatcher.path,
      dispatcherSha256: dispatcher.sha256,
      runnerSha256: runner.sha256,
    };
    const identity = {
      schemaVersion: 1,
      sourceVersion: String(pluginMetadata.version ?? 'unknown'),
      signerKeyId: signingIdentity.keyId,
      bundleId: release.bundleId,
      hookRuntime,
      sourceProvenance: sourceProvenance(source),
      skillIndex: { sha256: skillIndex.sha256, entryCount: skillIndex.entries.length },
      files: collectManifestFiles(staging),
      targetPayloads: targetPayloads(skills),
    };
    const generation = sha256(canonicalJson(identity));
    const manifest = { ...identity, generation };
    atomicWrite(path.join(staging, 'manifest.json'), canonicalJson(manifest));
    atomicWrite(
      path.join(staging, MANIFEST_SIGNATURE),
      canonicalJson(signValue(manifest, signingIdentity)),
      0o600
    );
    verifyGeneration(staging, generation, args.trustStore);

    const generationPath = path.join(generations, generation);
    if (fs.existsSync(generationPath)) {
      verifyGeneration(generationPath, generation, args.trustStore);
      fs.rmSync(staging, { recursive: true, force: true });
    } else {
      fs.renameSync(staging, generationPath);
      syncDirectory(generations);
    }

    validateBundleIndex(args.pluginRoot, generationPath, manifest, args.trustStore);
    installTargets(args.home, args.pluginRoot, generationPath, generation, skills);
    verifyTargets(args.home, args.pluginRoot, generationPath, generation, skills);
    writeTargetReceipts(args.pluginRoot, manifest, signingIdentity);
    installHookRuntime(args.pluginRoot, generationPath, manifest, args.trustStore);
    const active = currentTarget(args.pluginRoot, 'current');
    if (active !== `generations/${generation}`) {
      if (active) atomicSymlink(args.pluginRoot, 'previous', active);
      atomicSymlink(args.pluginRoot, 'current', `generations/${generation}`);
    }
    createStableDispatchers(args.pluginRoot);
    verifyActive(args.pluginRoot, args.trustStore);
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
  if (args.verify) generation = verifyActive(args.pluginRoot, args.trustStore).generation;
  else if (args.rollback) generation = rollback(args.pluginRoot, args.home, args.trustStore);
  else if (args.uninstall) generation = uninstall(args.home, args.pluginRoot);
  else generation = install(args);
  process.stdout.write(`${generation}\n`);
} catch (error) {
  process.stderr.write(`install-plugin-generation: ${error.message}\n`);
  process.exitCode = 1;
}
