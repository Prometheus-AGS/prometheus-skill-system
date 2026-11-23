// Fetch the native bundle belonging to this checkout's pack version. Exit 3
// denotes an unpublished asset or unsupported host. Every integrity failure is fatal.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createReadStream, createWriteStream, promises as fs } from 'node:fs';
import { tmpdir, homedir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { pipeline } from 'node:stream/promises';
import { spawnSync } from 'node:child_process';

const root = fileURLToPath(new URL('..', import.meta.url));
const missing = 3;
const suffix = process.platform === 'win32' ? '.exe' : '';
const os = { darwin: 'darwin', win32: 'windows', linux: 'linux' }[process.platform];
const arch = { x64: 'amd64', arm64: 'arm64' }[process.arch];

async function sha256(file) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(file)) hash.update(chunk);
  return hash.digest('hex');
}
function untar(args) {
  const result = spawnSync('tar', args, { encoding: 'utf8' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(result.stderr || 'Could not read release archive');
  return result.stdout.trimEnd();
}
async function get(base, name) {
  const url = new URL(name, base);
  const response = await fetch(url, { signal: AbortSignal.timeout(900_000) });
  if (!response.ok && response.status !== 404) throw new Error(`${url}: HTTP ${response.status}`);
  if (new URL(response.url).protocol !== url.protocol) throw new Error('Unexpected change of release URL protocol');
  return response;
}
async function identity() {
  const { releaseVersion: version } = JSON.parse(await fs.readFile(path.join(root, 'skill-system.json'), 'utf8'));
  assert.match(version, /^\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?$/);
  if (!os || !arch) return null;
  const platform = `${os}-${arch}`;
  const tag = `v${version}`;
  const name = `prometheus-binaries-${tag}-${platform}.tar.gz`;
  const releases = process.env.PROMETHEUS_BINARY_RELEASE_BASE_URL || 'https://github.com/Prometheus-AGS/prometheus-skill-system/releases/download';
  const base = new URL(`${releases.replace(/\/$/, '')}/${tag}/`);
  if (!(base.protocol === 'https:' || (base.protocol === 'http:' && ['127.0.0.1', '[::1]', 'localhost'].includes(base.hostname)))) {
    throw new Error('Binary releases require HTTPS or a loopback test server');
  }
  return { version, tag, platform, name, base };
}
function checksum(sums, name) {
  const rows = sums.split(/\r?\n/).flatMap(line => {
    const entry = /^([a-f0-9]{64}) [ *](\S+)$/.exec(line);
    return entry?.[2] === name ? [entry[1]] : [];
  });
  assert.equal(rows.length, 1, `Expected one SHA256SUMS entry for ${name}`);
  return rows[0];
}
async function readSums(base) {
  const response = await get(base, 'SHA256SUMS');
  if (!response.ok) throw new Error('Published release lacks SHA256SUMS');
  return response.text();
}
function sidecar(name) { return name.replace(/\.tar\.gz$/, '.manifest.json'); }
async function prepare(directory) {
  const release = await identity();
  if (!release) return false;
  const { name, base } = release;
  const response = await get(base, name);
  if (response.status === 404) {
    const listing = await get(base, 'SHA256SUMS');
    if (listing.ok && (await listing.text()).split(/\r?\n/).some(line => line.endsWith(' ' + name) || line.endsWith('*' + name))) {
      throw new Error(`Published checksum lists missing archive ${name}`);
    }
    console.error(`No published ${name}; using source when available`);
    return false;
  }
  const sums = await readSums(base);
  const archiveSha256 = checksum(sums, name);
  const archive = path.join(directory, 'download.tar.gz');
  await pipeline(response.body, createWriteStream(archive, { flags: 'wx', mode: 0o600 }));
  assert.equal(await sha256(archive), archiveSha256, 'Release archive SHA-256 mismatch');

  // Check all names and types before tar can write a path. Never extract links,
  // devices, case aliases, absolute paths or parent traversal into the stage.
  const members = untar(['-tzf', archive]).split('\n').map(value => value.replace(/\/$/, ''));
  assert.equal(new Set(members.map(value => value.toLowerCase())).size, members.length, 'Duplicate archive member');
  for (const member of members) {
    assert(member.split('/').every(part => /^[A-Za-z0-9_+-][a-zA-Z0-9._+-]*$/.test(part)), `Unsafe archive path ${member}`);
    assert(member === 'manifest.json' || member === 'bin' || member.startsWith('bin/'), `Unexpected archive member ${member}`);
  }
  const listing = untar(['-tvzf', archive]).split('\n');
  assert.equal(listing.length, members.length, 'Invalid archive listing');
  assert(listing.every(line => /^[-d]/.test(line)), 'Release archive contains a special file');
  untar(['-xzf', archive, '-C', directory, '--no-same-owner', '--no-same-permissions']);
  await fs.unlink(archive);
  await verifyStage(directory, release, sums, members);
  return true;
}
async function verifyStage(directory, release, sums, members = []) {
  const { tag, platform, version, name } = release;
  const raw = await fs.readFile(path.join(directory, 'manifest.json'));
  assert.equal(createHash('sha256').update(raw).digest('hex'), checksum(sums, sidecar(name)), 'Release manifest SHA-256 mismatch');
  const manifest = JSON.parse(raw);
  assert.equal(manifest.schemaVersion, 1, 'Unsupported release manifest');
  assert.equal(manifest.version, version, 'Wrong pack version');
  assert.equal(manifest.tag, tag, 'Wrong release tag');
  assert.equal(manifest.platform, platform, 'Wrong release platform');
  assert.match(manifest.sourceCommit, /^[a-f0-9]{40}$/);
  const local = spawnSync('git', ['-C', root, 'rev-parse', '--verify', `refs/tags/${tag}^{commit}`], { encoding: 'utf8' });
  if (local.status === 0) assert.equal(manifest.sourceCommit, local.stdout.trim(), 'Release source disagrees with the local tag');
  const verified = new Set();
  for (const file of manifest.files) {
    const resource = file.name.startsWith('mlx-swift_Cmlx.bundle/');
    if (!resource) assert.match(file.name, /^[a-z0-9][a-z0-9-]*$/);
    assert.equal(file.path, `bin/${file.name}${resource ? '' : suffix}`);
    assert(!verified.has(file.path), `Duplicate release record for ${file.path}`);
    assert(file.path.split('/').every(part => /^[A-Za-z0-9_+-][A-Za-z0-9._+-]*$/.test(part)), 'Invalid manifest file path');
    const item = path.join(directory, file.path);
    const stat = await fs.lstat(item);
    assert(stat.isFile() && stat.size === file.size && stat.size > 0, `Wrong release size for ${file.path}`);
    assert.equal(await sha256(item), file.sha256, `Wrong release hash for ${file.path}`);
    if (members.length) await fs.chmod(item, resource ? 0o644 : 0o755);
    verified.add(file.path);
  }
  const target = { 'darwin-arm64': 'aarch64-apple-darwin', 'darwin-amd64': 'x86_64-apple-darwin',
    'linux-arm64': 'aarch64-unknown-linux-gnu', 'linux-amd64': 'x86_64-unknown-linux-gnu',
    'windows-arm64': 'aarch64-pc-windows-msvc', 'windows-amd64': 'x86_64-pc-windows-msvc' }[platform];
  const pins = JSON.parse(await fs.readFile(path.join(root, 'config/prometheus-exec-binary.json'), 'utf8'));
  const pin = pins.expectedBuildSha256ByTarget?.[target];
  const releasePin = pins.releaseBuildsByPackVersion?.[version]?.[target]?.sha256;
  const execHash = manifest.files.find(file => file.name === 'prometheus-exec')?.sha256;
  if (os === 'darwin' || pin || releasePin) {
    assert.match(pin || '', /^[0-9a-f]{64}$/, 'Missing independent Darwin exec pin');
    if (releasePin !== undefined) assert.match(releasePin, /^[0-9a-f]{64}$/, 'Invalid release-specific exec pin');
    assert(execHash === pin || (releasePin !== undefined && execHash === releasePin), 'Release violates the independent prometheus-exec pin');
  }
  for (const member of members) {
    if (member !== 'manifest.json' && (await fs.lstat(path.join(directory, member))).isFile()) {
      assert(verified.has(member), `Unlisted release file ${member}`);
    }
  }
  const catalog = JSON.parse(await fs.readFile(path.join(root, 'config/release-binaries.json'), 'utf8'));
  for (const component of catalog.components.filter(entry => !entry.platforms || entry.platforms.includes(platform))) {
    for (const binary of component.binaries) assert(verified.has(`bin/${binary}${suffix}`), `Release lacks ${binary}`);
    if (component.resourceTree) assert(verified.has(`bin/${component.resourceTree}/Contents/Resources/default.metallib`), 'Release lacks required MLX Metal resource');
  }
  // Retain the authenticated manifest for doctor and signed hook activation.
  const cache = path.join(process.env.HOME || homedir(), '.prometheus/install/releases', tag, platform);
  await fs.mkdir(cache, { recursive: true, mode: 0o700 });
  const scratch = await fs.mkdtemp(path.join(cache, '.staged-'));
  try {
    const evidence = { schemaVersion: 1, version, tag, platform, archive: name,
      archiveSha256: checksum(sums, name), manifestSha256: checksum(sums, sidecar(name)), baseUrl: release.base.href };
    for (const [file, content] of [['manifest.json', raw], ['SHA256SUMS', sums], ['release.json', JSON.stringify(evidence) + '\n']]) {
      await fs.writeFile(path.join(scratch, file), content, { mode: 0o600 });
      await fs.rename(path.join(scratch, file), path.join(cache, file));
    }
  } finally { await fs.rm(scratch, { recursive: true, force: true }); }
  console.error(`Verified ${name} (${verified.size} files)`);
  return { tag, platform, target, execHash, manifestHash: checksum(sums, sidecar(name)) };
}
async function markInstalled(destination, stage, buildHash) {
  const release = await identity();
  if (!release) return;
  const { tag, platform } = release;
  const name = path.basename(destination, suffix);
  const cache = path.join(process.env.HOME || homedir(), '.prometheus/install/releases', tag, platform);
  const parent = path.join(cache, 'installed', name);
  destination = path.resolve(destination);
  const receipt = path.join(parent, createHash('sha256').update(destination).digest('hex') + '.json');
  if (!stage) { await fs.rm(receipt, { force: true }); return; }
  const raw = await fs.readFile(path.join(stage, 'manifest.json'));
  const manifestSha256 = createHash('sha256').update(raw).digest('hex');
  const record = JSON.parse(raw).files.find(file => file.name === name);
  const installedSha256 = await sha256(destination);
  assert.equal(buildHash || installedSha256, record?.sha256, `Installed image mismatch for ${name}`);
  await fs.mkdir(parent, { recursive: true, mode: 0o700 });
  const scratch = await fs.mkdtemp(path.join(parent, '.staged-'));
  try {
    const data = {schemaVersion: 1, name, destination, buildSha256: record.sha256, installedSha256, tag, platform, manifestSha256};
    await fs.writeFile(path.join(scratch, 'entry'), JSON.stringify(data) + '\n', { mode: 0o600 });
    await fs.rename(path.join(scratch, 'entry'), receipt);
  } finally { await fs.rm(scratch, {recursive: true, force: true}); }
}
async function copyVerified(source, destination, expectedHash) {
  const parent = path.dirname(destination);
  await fs.mkdir(parent, {recursive: true});
  const tmp = await fs.mkdtemp(path.join(parent, '.prometheus-'));
  try {
    const staged = path.join(tmp, path.basename(source));
    await fs.copyFile(source, staged);
    await fs.chmod(staged, 0o755);
    assert.equal(await sha256(staged), expectedHash, `Release binary changed before installation: ${source}`);
    await fs.rename(staged, destination);
  } finally { await fs.rm(tmp, {recursive: true, force: true}); }
}
async function readAuthenticatedManifest(stage) {
  const release = await identity();
  assert(release, 'Unsupported release host');
  const bytes = await fs.readFile(path.join(stage, 'manifest.json'));
  const sums = await readSums(release.base);
  assert.equal(createHash('sha256').update(bytes).digest('hex'), checksum(sums, sidecar(release.name)), 'Invalid release manifest digest');
  const manifest = JSON.parse(bytes);
  assert.equal(manifest.platform, release.platform);
  assert.equal(manifest.tag, release.tag);
  assert.equal(manifest.version, release.version);
  return manifest;
}
async function windowsExec(stage, destination, source, record, manifest) {
  const config = JSON.parse(await fs.readFile(path.join(root, 'config/prometheus-exec-binary.json'), 'utf8'));
  const catalog = JSON.parse(await fs.readFile(path.join(root, 'config/release-binaries.json'), 'utf8'));
  const version = config.expectedVersion;
  const probe = binary => {
    const result = spawnSync(binary, ['--version'], {encoding: 'utf8', windowsHide: true});
    assert(result.status === 0 && result.stdout.trim() === version, `Wrong version for ${binary}`);
  };
  assert.equal(await sha256(source), record.sha256);
  probe(source);
  const install = path.join(process.env.HOME || homedir(), '.prometheus/install');
  let backup, previous;
  try { previous = await sha256(destination); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  if (previous) {
    backup = path.join(install, 'backups', `prometheus-exec.${previous}`);
    await copyVerified(destination, backup, previous);
  }
  await copyVerified(source, destination, record.sha256);
  try {
    probe(destination);
    assert.equal(await sha256(destination), record.sha256);
    const dir = path.join(install, 'manifests');
    await fs.mkdir(dir, {recursive:true});
    const scratch = await fs.mkdtemp(path.join(dir, '.staged-'));
    try {
      const receipt = {schemaVersion:1, name:'prometheus-exec', version, target:catalog.platforms[manifest.platform].target,
        expectedBuildSha256:record.sha256, source:record.path, destination, buildSha256:record.sha256, installedSha256:record.sha256,
        signature:'not-applicable', releaseTag:manifest.tag, releasePlatform:manifest.platform, releaseManifestSha256:await sha256(path.join(stage, 'manifest.json'))};
      await fs.writeFile(path.join(scratch,'receipt'), JSON.stringify(receipt, null, 2)+'\n');
      await fs.rename(path.join(scratch,'receipt'), path.join(dir, 'prometheus-exec.json'));
    } finally { await fs.rm(scratch,{recursive:true,force:true}); }
  } catch (error) {
    if (backup) await copyVerified(backup, destination, previous);
    else await fs.rm(destination, {force:true});
    throw error;
  }
}
async function main() {
  const [action, destination, ...names] = process.argv.slice(2);
  if (action === 'mark' && destination) {
    await markInstalled(destination, names[0], names[1]);
    return 0;
  }
  if (action === 'copy' && destination && names.length === 1) {
    const source = path.resolve(names[0]);
    const stage = path.dirname(path.dirname(source));
    const manifest = await readAuthenticatedManifest(stage);
    const name = path.basename(destination, suffix);
    assert.notEqual(name, 'prometheus-exec', 'Use the dedicated exec installer');
    const record = manifest.files.find(file => file.name === name);
    assert(record && path.resolve(stage, record.path) === source, 'Invalid release source');
    await copyVerified(source, destination, record.sha256);
    await markInstalled(destination, stage);
    return 0;
  }
  if (!destination || !['prepare','verify','verify-exec','install','windows'].includes(action)) throw new Error('Usage: binary-release.mjs prepare EMPTY_DIR | install BIN_DIR NAME ...');
  if (action === 'verify' || action === 'verify-exec') {
    const release = await identity();
    if (!release) return missing;
    const verified = await verifyStage(destination, release, await readSums(release.base));
    if (action === 'verify-exec') {
      console.log(verified.tag, verified.platform, verified.target, verified.execHash, verified.manifestHash);
    }
    return 0;
  }
  if (action === 'prepare') {
    assert.equal((await fs.readdir(destination)).length, 0, 'Release stage must be empty');
    return await prepare(destination) ? 0 : missing;
  }
  if (action === 'windows') assert.equal(os, 'windows', 'Use scripts/install-binaries.sh on macOS and Linux');
  else assert(names.length && names.every(name => /^[a-z0-9][a-z0-9-]*$/.test(name) && name !== 'prometheus-exec'), 'Use the dedicated installer for prometheus-exec');
  const stage = await fs.mkdtemp(path.join(tmpdir(), 'prometheus-release-'));
  try {
    if (!await prepare(stage)) return missing;
    const manifest = JSON.parse(await fs.readFile(path.join(stage, 'manifest.json'), 'utf8'));
    if (action === 'windows') names.push(...manifest.files.map(file => file.name));
    for (const name of names) {
      const record = manifest.files.find(file => file.name === name);
      assert(record, `Release omits ${name}`);
      const source = path.join(stage, record.path);
      const installed = path.join(destination, name + suffix);
      if (name === 'prometheus-exec') await windowsExec(stage, installed, source, record, manifest);
      else await copyVerified(source, installed, record.sha256);
      await markInstalled(installed, stage);
    }
    return 0;
  } finally { await fs.rm(stage, { recursive: true, force: true }); }
}
try { process.exitCode = await main(); }
catch (error) { console.error(`Binary release installation failed: ${error.message}`); process.exitCode = 1; }
