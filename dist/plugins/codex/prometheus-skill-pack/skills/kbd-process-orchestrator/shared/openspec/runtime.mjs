import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { spawn } from 'node:child_process';
import { acceptedVersion, readJson, writeJson, REGISTRY } from './state.mjs';

export function nodeProcess(entry, args, { cwd, remaining, inherit = false }) {
  const budget = remaining();
  return new Promise((resolve, reject) => {
    let output = '';
    let timedOut = false;
    const child = spawn(process.execPath, [entry, ...args], {
      cwd, shell: false, windowsHide: true,
      env: { ...process.env, OPENSPEC_NO_UPDATE_CHECK: '1', OPEN_SPEC_INTERACTIVE: '0', DO_NOT_TRACK: '1' },
      stdio: ['ignore', inherit ? 'inherit' : 'pipe', inherit ? 'inherit' : 'pipe'],
    });
    const stop = () => child.kill('SIGKILL');
    const interrupt = () => { timedOut = true; stop(); };
    process.once('SIGTERM', interrupt);
    process.once('SIGINT', interrupt);
    const timer = setTimeout(interrupt, budget);
    const capture = bytes => { output = (output + bytes.toString()).slice(-65536); };
    child.stdout?.on('data', capture);
    child.stderr?.on('data', capture);
    const cleanup = () => {
      clearTimeout(timer);
      process.removeListener('SIGTERM', interrupt);
      process.removeListener('SIGINT', interrupt);
    };
    child.once('error', error => { cleanup(); reject(error); });
    child.once('close', (code, signal) => {
      cleanup();
      if (timedOut) reject(Object.assign(new Error('OpenSpec child stopped at its time budget or parent interruption'), { status: 'pending', output }));
      else resolve({ code: code ?? 1, signal, output });
    });
  });
}

// Resolve the installed npm JavaScript entry, never a project-provided .cmd shim.
function npmEntry() {
  const bin = path.dirname(process.execPath);
  const candidates = [path.join(bin, 'node_modules/npm/bin/npm-cli.js'), path.resolve(bin, '../lib/node_modules/npm/bin/npm-cli.js')];
  for (const directory of (process.env.PATH || '').split(path.delimiter).filter(Boolean)) {
    const npm = path.join(directory, 'npm');
    if (fs.existsSync(npm)) {
      const real = fs.realpathSync(npm);
      if (path.basename(real) === 'npm-cli.js') candidates.push(real);
    }
    candidates.push(path.join(directory, 'node_modules/npm/bin/npm-cli.js'));
  }
  try { candidates.push(createRequire(process.execPath).resolve('npm/bin/npm-cli.js')); } catch {}
  const found = candidates.find(file => fs.existsSync(file));
  if (!found) throw new Error('Cannot resolve npm-cli.js from the Node installation or PATH; install npm with Node');
  return found;
}

export function cachedRuntime(home, version) {
  acceptedVersion(version);
  const prefix = path.join(home, 'versions', version);
  const root = path.join(prefix, 'node_modules', '@fission-ai', 'openspec');
  const manifest = readJson(path.join(root, 'package.json'));
  const installed = readJson(path.join(prefix, 'installed.json'));
  if (!manifest || installed?.version !== version || manifest.name !== '@fission-ai/openspec' || manifest.version !== version) return null;
  const relative = typeof manifest.bin === 'string' ? manifest.bin : manifest.bin?.openspec;
  if (!relative) throw new Error('Managed OpenSpec package has no JavaScript CLI');
  const entry = path.resolve(root, relative);
  if (!entry.startsWith(root + path.sep) || !/\.[cm]?js$/.test(entry)) throw new Error('Managed OpenSpec CLI escapes its package or is not JavaScript');
  return fs.existsSync(entry) ? { version, prefix, root, entry } : null;
}

export function selectedRuntime(home) {
  const selection = readJson(path.join(home, 'selected.json'));
  return selection ? cachedRuntime(home, selection.version) : null;
}

export async function selectVersion(home, remaining) {
  const pin = process.env.PROMETHEUS_OPENSPEC_VERSION;
  if (pin) return { version: acceptedVersion(pin), latestVerified: false, policy: 'operator-pin' };
  try {
    const response = await fetch(`${REGISTRY}/@fission-ai%2fopenspec/latest`, {
      signal: AbortSignal.timeout(Math.max(1, Math.min(5000, remaining() - 500))), redirect: 'error',
    });
    if (!response.ok) throw new Error(`npm registry returned HTTP ${response.status}`);
    const body = await response.json();
    if (body.name !== '@fission-ai/openspec') throw new Error('npm registry returned another package');
    return { version: acceptedVersion(body.version), latestVerified: true, policy: 'registry-latest' };
  } catch (error) {
    const selected = selectedRuntime(home);
    if (!selected) throw error;
    return { version: selected.version, latestVerified: false, policy: 'cached-offline', registryError: error.message };
  }
}

export async function ensureRuntime(home, selection, remaining) {
  const cached = cachedRuntime(home, selection.version);
  if (cached) return cached;
  // A session hook must not launch an installation its parent can kill mid-write.
  if (remaining() < 60000) throw Object.assign(new Error('OpenSpec installation requires a phase refresh with at least 60 seconds remaining; no install started'), { status: 'pending' });
  const prefix = path.join(home, 'versions', selection.version);
  fs.mkdirSync(prefix, { recursive: true, mode: 0o700 });
  const installed = await nodeProcess(npmEntry(), [
    'install', '--prefix', prefix, '--global=false', '--ignore-scripts', '--no-audit', '--no-fund', '--bin-links=false', '--save-exact',
    `@fission-ai/openspec@${selection.version}`, `--registry=${REGISTRY}`,
  ], { cwd: home, remaining });
  if (installed.code !== 0) throw Object.assign(new Error(`Managed OpenSpec installation failed (exit ${installed.code})`), { output: installed.output });
  writeJson(path.join(prefix, 'installed.json'), { version: selection.version, installedAt: new Date().toISOString(), registry: REGISTRY });
  let runtime;
  try {
    runtime = cachedRuntime(home, selection.version);
    if (!runtime) throw new Error('Installed OpenSpec package did not provide the expected CLI/version');
    const probe = await nodeProcess(runtime.entry, ['--version'], { cwd: home, remaining });
    if (probe.code || probe.output.trim() !== selection.version) throw new Error('Installed OpenSpec CLI version probe failed');
  } catch (error) {
    fs.unlinkSync(path.join(prefix, 'installed.json'));
    throw error;
  }
  return runtime;
}
