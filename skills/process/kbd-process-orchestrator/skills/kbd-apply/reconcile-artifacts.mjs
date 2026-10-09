// Read backend task artifacts without invoking a CLI, migrating tasks, or writing projections.
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';

export const readJson = (file) => JSON.parse(readFileSync(file, 'utf8'));
export const complete = (status) => ['complete', 'completed', 'done'].includes(String(status).toLowerCase());
export const cancelled = (status) => ['cancelled', 'canceled'].includes(String(status).toLowerCase());

export function safeId(id, label = 'identifier') {
  if (typeof id !== 'string' || !id || id === '.' || id === '..' || /[\\/\0]/.test(id)) {
    throw new Error(`invalid ${label}`);
  }
  return id;
}

export function phaseDirectory(root, id) {
  const segments = id.split('::').map((part) => safeId(part, 'phase'));
  return path.join(root, '.kbd-orchestrator', 'phases', ...segments.flatMap((part, i) => i ? ['children', part] : [part]));
}

export function activePhase(waypoint) {
  return waypoint.activePhaseId || (Array.isArray(waypoint.phaseIds) && waypoint.phaseIds.at(-1)) ||
    (Array.isArray(waypoint.path) && waypoint.path.join('::')) ||
    [waypoint.phase, waypoint.childPointer].filter(Boolean).join('::');
}

export function resolvePhase(root, requested, waypoint, state) {
  const active = state?.activePath?.phaseId || activePhase(waypoint);
  let phase = requested || active;
  if (!phase) throw new Error('no phase given and no active phase found');
  if (state && !state.phases?.[phase]) {
    const matches = Object.keys(state.phases ?? {}).filter((id) => id.split('::').at(-1) === phase);
    if (matches.length !== 1) throw new Error(`phase ${phase} is unknown or ambiguous`);
    [phase] = matches;
  } else if (!state && !existsSync(phaseDirectory(root, phase))) {
    const matches = [];
    const walk = (dir, prefix = []) => {
      if (!existsSync(dir)) return;
      for (const entry of readdirSync(dir, { withFileTypes: true })) {
        if (!entry.isDirectory()) continue;
        const parts = [...prefix, entry.name];
        if (entry.name === phase) matches.push(parts.join('::'));
        walk(path.join(dir, entry.name, 'children'), parts);
      }
    };
    walk(path.join(root, '.kbd-orchestrator', 'phases'));
    if (matches.length !== 1) throw new Error(`phase ${phase} is unknown or ambiguous`);
    [phase] = matches;
  }
  return { phase, active, directory: phaseDirectory(root, phase) };
}

const layouts = {
  openspec: ['openspec', 'changes'],
  speckit: ['specs'],
  'native-kbd': ['.kbd-orchestrator', 'changes'],
};

function locate(root, change, backend) {
  const base = path.join(root, ...layouts[backend]);
  const direct = path.join(base, change);
  if (existsSync(direct)) return { directory: direct, archived: false, backend };
  const archive = path.join(base, 'archive');
  const entries = existsSync(archive) ? readdirSync(archive, { withFileTypes: true }) : [];
  const matches = entries.filter((entry) => entry.isDirectory() &&
    (entry.name === change || (/^\d{4}-\d{2}-\d{2}-/.test(entry.name) && entry.name.slice(11) === change)));
  if (matches.length > 1) throw new Error(`ambiguous ${backend} archives for ${change}`);
  return matches.length ? { directory: path.join(archive, matches[0].name), archived: true, backend } : null;
}

function markdownTasks(text, backend) {
  const tasks = [];
  for (const line of text.split(/\r?\n/)) {
    const match = line.match(/^\s*[-*]\s*\[([^\]]*)\]\s*(.*)$/);
    if (!match) continue;
    const sequence = tasks.length + 1;
    let id = String(sequence);
    let title = match[2].trim();
    const token = backend === 'speckit' ? title.match(/^(T\d+)\s*/) :
      backend === 'native-kbd' ? title.match(/^(\d+)\.\s*/) : null;
    if (token) { id = token[1]; title = title.slice(token[0].length); }
    if (!title) throw new Error(`empty ${backend} task title at sequence ${sequence}`);
    tasks.push({ id, title, done: match[1].trim().toLowerCase() === 'x', sequence });
  }
  if (!tasks.length) throw new Error(`no task checkboxes found in ${backend} task artifact`);
  return tasks;
}

export function readBackendTasks(root, change, pinned) {
  safeId(change, 'change');
  if (pinned && pinned !== 'auto' && !layouts[pinned]) throw new Error(`unsupported backend ${pinned}`);
  const candidates = (layouts[pinned] ? [pinned] : ['native-kbd', 'openspec', 'speckit'])
    .map((backend) => locate(root, change, backend)).filter(Boolean);
  if (candidates.length !== 1) throw new Error(`no unique task artifact for ${change}`);
  const artifact = candidates[0];
  let tasks;
  const nativeFile = path.join(artifact.directory, 'tasks.json');
  if (artifact.backend === 'native-kbd' && existsSync(nativeFile)) {
    const parsed = readJson(nativeFile);
    if (!Array.isArray(parsed.tasks) || !parsed.tasks.length) throw new Error(`invalid tasks.json for ${change}`);
    tasks = parsed.tasks.map((task, index) => {
      if (!task || typeof task.id !== 'string' || !task.id || typeof task.title !== 'string' || !task.title || typeof task.done !== 'boolean') {
        throw new Error(`invalid task row ${index + 1} for ${change}`);
      }
      return { id: task.id, title: task.title, done: task.done, sequence: index + 1 };
    });
  } else {
    const filename = artifact.backend === 'native-kbd' ? 'change.md' : 'tasks.md';
    tasks = markdownTasks(readFileSync(path.join(artifact.directory, filename), 'utf8'), artifact.backend);
  }
  if (new Set(tasks.map((task) => task.id)).size !== tasks.length) throw new Error(`duplicate task IDs for ${change}`);
  return { ...artifact, tasks };
}
