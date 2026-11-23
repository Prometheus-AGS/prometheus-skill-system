// A release version may outlive a pinned import change on the pack branch.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { promises as fs } from 'node:fs';
import path from 'node:path';

function run(cwd, tool, args, capture = false) {
  const result = spawnSync(tool, args, { cwd, encoding: 'utf8', stdio: capture ? 'pipe' : 'inherit' });
  if (result.error?.code === 'ENOENT') throw new Error(`${tool} is required for the pinned source build in ${cwd}`);
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${tool} ${args[0]} failed in ${cwd}: ${result.stderr?.trim() || result.status}`);
  return result.stdout?.trim();
}

export async function changedImports(root, manifest, names, getReleaseContract) {
  assert.match(manifest.sourceCommit, /^[a-f0-9]{40}$/);
  const contract = JSON.parse(await fs.readFile(path.join(root, 'skill-system.json')));
  const catalog = JSON.parse(await fs.readFile(path.join(root, 'config/release-binaries.json')));
  const products = catalog.components.filter(c => (!c.platforms || c.platforms.includes(manifest.platform))
    && c.binaries.some(name => names.has(name)));
  const pins = contract.imports.filter(pin => products.some(c => c.directory === pin.path || c.directory.startsWith(pin.path + '/')));
  if (!pins.length) return new Map();
  // The release identifies the exact pack tree that selected its submodules.
  // A gitless distribution can read the contract at that immutable commit.
  const tree = spawnSync('git', ['-C', root, 'ls-tree', manifest.sourceCommit, '--', ...pins.map(p => p.path)], { encoding: 'utf8' });
  const released = new Map();
  if (tree.status === 0) {
    for (const line of tree.stdout.trim().split('\n')) {
      const entry = /^160000 commit ([a-f0-9]{40})\t(.+)$/.exec(line);
      if (entry) released.set(entry[2], entry[1]);
    }
  } else {
    const snapshot = await getReleaseContract(manifest.sourceCommit);
    for (const pin of snapshot.imports) released.set(pin.path, pin.commit);
  }
  const changed = new Map();
  for (const pin of pins) {
    assert.match(pin.commit, /^[a-f0-9]{40}$/);
    if (released.get(pin.path) === pin.commit) continue;
    for (const component of products.filter(c => c.directory === pin.path || c.directory.startsWith(pin.path + '/'))) {
      for (const name of component.binaries) changed.set(name, {pin, component});
    }
  }
  return changed;
}

// All three native PK products share one locked workspace and one source pin.
// Keep compatible products installable even on a machine missing Rust or Git.
export async function buildPinned(root, {pin, component}, names, platform, scratch) {
  let source = path.join(root, pin.path);
  const git = (cwd, ...args) => run(cwd, 'git', args, true);
  const hasCheckout = Boolean(await fs.stat(path.join(source, '.git')).catch(() => null));
  if (hasCheckout) {
    assert.equal(git(source, 'status', '--porcelain', '--untracked-files=normal'), '', `Cannot certify a modified source checkout at ${source}`);
  }
  if (!hasCheckout || git(source, 'rev-parse', 'HEAD') !== pin.commit) {
    if (await fs.stat(path.join(root, '.git')).catch(() => null)) {
      run(root, 'git', ['submodule', 'update', '--init', '--recursive', '--', pin.path]);
    } else {
      // A source archive has no gitlinks to initialize. Build the exact import
      // in private scratch space and leave any unpacked user files untouched.
      source = path.join(scratch, 'import');
      await fs.mkdir(source);
      run(source, 'git', ['init', '--quiet']);
      run(source, 'git', ['fetch', '--depth=1', '--', pin.repository, pin.commit]);
      run(source, 'git', ['checkout', '--quiet', '--detach', 'FETCH_HEAD']);
      run(source, 'git', ['submodule', 'update', '--init', '--recursive']);
    }
  }
  assert.equal(git(source, 'rev-parse', 'HEAD'), pin.commit, `Source checkout must match ${pin.path}@${pin.commit}`);
  const workspace = path.join(source, path.relative(pin.path, component.directory));
  if (component.resourceTree || (platform.startsWith('windows') && component.directory === 'tools/cowork-skills/cli')) {
    throw new Error(`The release predates ${pin.path}@${pin.commit}. Build ${names.join(', ')} with the checkout's platform-specific source installer.`);
  }
  console.error(`Building ${names.join(', ')} from ${pin.path}@${pin.commit}`);
  const args = ['build', '--release', '--locked'];
  for (const pkg of component.packages || []) args.push('-p', pkg);
  for (const name of names) args.push('--bin', name);
  if (component.noDefaultFeatures) args.push('--no-default-features');
  const features = [...(component.features || []), ...(component.featuresByPlatform?.[platform] || [])];
  if (features.length) args.push('--features', features.join(','));
  // Use Cargo's artifact paths, including redirected, target-specific builds.
  const output = spawnSync('cargo', [...args, '--message-format=json-render-diagnostics'], {
    cwd: workspace, encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'], maxBuffer: 64 * 1024 * 1024,
  });
  if (output.error?.code === 'ENOENT') throw new Error(`Cargo is required to build ${pin.path}@${pin.commit}; install the native Rust toolchain`);
  if (output.error) throw output.error;
  assert.equal(output.status, 0, `Cargo failed to build ${pin.path}@${pin.commit}`);
  const files = new Map();
  for (const line of output.stdout.trimEnd().split('\n')) {
    if (!line) continue;
    const artifact = JSON.parse(line);
    if (artifact.reason === 'compiler-artifact' && artifact.executable && artifact.target.kind.includes('bin')) {
      files.set(artifact.target.name, artifact.executable);
    }
  }
  for (const name of names) assert(files.has(name), `Cargo did not emit ${name}`);
  return files;
}
