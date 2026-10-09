import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';

const readJson = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const digest = file => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');

function dataRoot() {
  if (process.env.PROMETHEUS_DATA_DIR) return process.env.PROMETHEUS_DATA_DIR;
  if (process.platform === 'darwin') return path.join(os.homedir(), 'Library', 'Application Support');
  if (process.platform === 'win32') {
    if (!process.env.LOCALAPPDATA) throw new Error('LOCALAPPDATA is unavailable');
    return process.env.LOCALAPPDATA;
  }
  return process.env.XDG_DATA_HOME || path.join(os.homedir(), '.local', 'share');
}

function authorityFiles(root) {
  const files = ['project.loro'];
  if (!fs.existsSync(path.join(root, 'project.loro'))) throw new Error('Canonical project.loro is unavailable');
  if (fs.existsSync(path.join(root, 'events.jsonl'))) files.push('events.jsonl');
  const replicas = path.join(root, 'replicas');
  if (fs.existsSync(replicas)) {
    for (const dir of fs.readdirSync(replicas, { withFileTypes: true })) {
      if (dir.isDirectory() && fs.existsSync(path.join(replicas, dir.name, 'events.jsonl'))) {
        files.push(path.join('replicas', dir.name, 'events.jsonl'));
      }
    }
  }
  return files.sort();
}

// Existing CLI startup may recover journals and refresh projections. Run it on
// an isolated copy of authority, never against the inspected live project.
export function readReconcileState(cwd, options = {}) {
  cwd = fs.realpathSync(cwd);
  const waypointPath = path.join(cwd, '.kbd-orchestrator/current-waypoint.json');
  const waypoint = readJson(waypointPath);
  if (waypoint.generatedBy !== 'kbd-runtime') return { authoritative: false, state: null, waypoint };
  const manifestPath = path.join(cwd, '.prometheus/project.json');
  const manifest = readJson(manifestPath);
  if (!/^[A-Za-z0-9-]+$/.test(manifest.projectId || '')) throw new Error('Invalid canonical project identity');
  const store = path.join(dataRoot(), 'prometheus/kbd');
  const registryPath = path.join(store, 'registry.json');
  const registry = readJson(registryPath);
  const registration = registry.replicas?.[cwd];
  if (!registration || registration.projectId !== manifest.projectId) throw new Error('Canonical replica registration is unavailable');
  const source = path.join(store, 'projects', manifest.projectId);
  const files = authorityFiles(source);
  const originals = [manifestPath, registryPath, waypointPath, ...files.map(file => path.join(source, file))];
  const hashes = originals.map(digest);
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), 'kbd-reconcile-'));
  try {
    const project = path.join(temp, 'project');
    const scratchStore = path.join(temp, 'data/prometheus/kbd');
    const destination = path.join(scratchStore, 'projects', manifest.projectId);
    fs.mkdirSync(path.join(project, '.prometheus'), { recursive: true });
    fs.mkdirSync(path.join(project, '.kbd-orchestrator'), { recursive: true });
    fs.copyFileSync(manifestPath, path.join(project, '.prometheus/project.json'));
    fs.copyFileSync(waypointPath, path.join(project, '.kbd-orchestrator/current-waypoint.json'));
    for (const file of files) {
      fs.mkdirSync(path.dirname(path.join(destination, file)), { recursive: true });
      fs.copyFileSync(path.join(source, file), path.join(destination, file));
    }
    fs.writeFileSync(path.join(scratchStore, 'registry.json'), JSON.stringify({
      ...registry, replicas: { [fs.realpathSync(project)]: registration }, redirects: {},
    }));
    const result = spawnSync(options.program || 'prometheus', ['kbd', '--path', project, 'status', '--json'], {
      cwd: project, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024,
      env: { ...process.env, PROMETHEUS_DATA_DIR: path.join(temp, 'data'), PROMETHEUS_KBD_CONTROL_PLANE: '0' },
    });
    if (result.error || result.status !== 0) throw new Error(`Canonical snapshot read failed: ${result.error?.message || result.stderr?.trim() || result.status}`);
    const state = JSON.parse(result.stdout);
    if (!state.phases || !Number.isInteger(state.revision) || state.projectId !== manifest.projectId) throw new Error('Canonical snapshot did not return initialized state');
    if (JSON.stringify(files) !== JSON.stringify(authorityFiles(source)) || originals.some((file, i) => digest(file) !== hashes[i])) {
      throw new Error('Canonical source changed while reading; retry reconciliation');
    }
    return { authoritative: true, state, waypoint };
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
}
