import fs from 'node:fs';
import path from 'node:path';
import { randomUUID } from 'node:crypto';
import type {
  Json, ObjectValue, UarAuthoringDefinition, UarMigrationReceipt, UarWorkspace,
  UarWorkspaceIndex, UarWorkspaceStatus,
} from '../types.mjs';
import { UAR_PROFILE_V2 } from '../types.mjs';
import { object, relativeFile, text } from '../validation.mjs';
import { commitChanges, projectFile, stage } from '../project-files.mjs';
import { SEMVER } from './canonical.mjs';
import { normalizeAuthoring } from './migration.mjs';
import { compiledAsAuthoring, loadUarPackage } from './package-files.mjs';

const INDEX_FILE = 'workspace.json';
const DEFAULT_PAGE_SIZE = 20;
const MAX_PAGE_SIZE = 50;

function root(input: ObjectValue): string {
  const requested = path.resolve(text(input.project, 'project'));
  return fs.realpathSync(requested);
}

function workspacePath(input: ObjectValue): string {
  return relativeFile(text(input.workspace, 'workspace'));
}

function json(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function readJson(file: string, label: string): ObjectValue {
  return object(JSON.parse(fs.readFileSync(file, 'utf8').replace(/^\uFEFF/, '')), label);
}

function validateIndex(value: unknown): UarWorkspaceIndex {
  const index = object(value, 'workspace index');
  const allowed = new Set(['schemaVersion','profile','packageId','packageVersion','manifest','definitions','migrationReceipt','bindingIntent']);
  for (const field of Object.keys(index)) if (!allowed.has(field)) throw new Error(`Unknown workspace index field: ${field}`);
  if (index.schemaVersion !== 1) throw new Error('workspace.schemaVersion must be 1');
  if (index.profile !== UAR_PROFILE_V2) throw new Error(`workspace.profile must be ${UAR_PROFILE_V2}`);
  text(index.packageId, 'workspace.packageId');
  if (!SEMVER.test(text(index.packageVersion, 'workspace.packageVersion'))) throw new Error('workspace.packageVersion must be semantic version x.y.z');
  relativeFile(text(index.manifest, 'workspace.manifest'));
  if (!Array.isArray(index.definitions) || index.definitions.length === 0) throw new Error('workspace.definitions must be a nonempty array');
  const seen = new Set<string>();
  for (const [position, item] of index.definitions.entries()) {
    const file = relativeFile(text(item, `workspace.definitions[${position}]`));
    const folded = file.normalize('NFC').toLocaleLowerCase('en-US');
    if (seen.has(folded)) throw new Error(`Case-insensitive workspace path collision: ${file}`);
    seen.add(folded);
  }
  const manifestFolded = String(index.manifest).normalize('NFC').toLocaleLowerCase('en-US');
  if (seen.has(manifestFolded) || manifestFolded === INDEX_FILE) throw new Error('Workspace manifest path collides with another source document');
  if (index.migrationReceipt !== undefined) relativeFile(text(index.migrationReceipt, 'workspace.migrationReceipt'));
  if (index.bindingIntent !== undefined && !['package-only','package-and-binding'].includes(String(index.bindingIntent))) throw new Error('Invalid workspace.bindingIntent');
  return structuredClone(index) as unknown as UarWorkspaceIndex;
}

function lock(project: string, workspace: string): () => void {
  const lockFile = projectFile(project, `${workspace}.lock`);
  fs.mkdirSync(path.dirname(lockFile), { recursive: true });
  const token = randomUUID();
  let descriptor: number;
  try { descriptor = fs.openSync(lockFile, 'wx', 0o600); }
  catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'EEXIST') throw new Error(`Workspace lock held: ${workspace}.lock; inspect it before manual recovery`);
    throw error;
  }
  try { fs.writeFileSync(descriptor, JSON.stringify({ token, pid: process.pid, at: new Date().toISOString() })); fs.fsyncSync(descriptor); }
  catch (error) { fs.closeSync(descriptor); fs.rmSync(lockFile, { force: true }); throw error; }
  fs.closeSync(descriptor);
  return () => {
    try { if (JSON.parse(fs.readFileSync(lockFile, 'utf8')).token === token) fs.rmSync(lockFile); }
    catch { /* A replaced lock belongs to another writer and is left intact. */ }
  };
}

export function loadWorkspace(input: ObjectValue): UarWorkspace {
  const project = root(input), workspace = workspacePath(input);
  const base = `${workspace}/${INDEX_FILE}`;
  const index = validateIndex(readJson(projectFile(project, base), base));
  const manifestPath = `${workspace}/${index.manifest}`;
  const manifest = readJson(projectFile(project, manifestPath), manifestPath);
  if (manifest.id !== index.packageId || manifest.version !== index.packageVersion) throw new Error('Workspace index package identity/version does not match its manifest source');
  const definitions: UarAuthoringDefinition[] = index.definitions.map(file => ({
    path: file,
    document: readJson(projectFile(project, `${workspace}/${file}`), `${workspace}/${file}`),
  }));
  let migrationReceipt: UarMigrationReceipt | undefined;
  if (index.migrationReceipt) migrationReceipt = readJson(projectFile(project, `${workspace}/${index.migrationReceipt}`), 'migration receipt') as unknown as UarMigrationReceipt;
  return { root: workspace, index, package: { manifest, definitions }, ...(migrationReceipt ? { migrationReceipt } : {}) };
}

export function initializeWorkspace(input: ObjectValue): ObjectValue {
  const project = root(input), workspace = workspacePath(input);
  const release = lock(project, workspace);
  try {
    const indexFile = projectFile(project, `${workspace}/${INDEX_FILE}`);
    if (fs.existsSync(indexFile)) throw new Error(`Workspace already exists: ${workspace}`);
    const migrationSource = input.sourceDirectory === undefined ? input.source : compiledAsAuthoring(loadUarPackage(projectFile(project, relativeFile(text(input.sourceDirectory, 'sourceDirectory')))));
    const normalized = migrationSource === undefined ? null : normalizeAuthoring(migrationSource);
    const packageId = normalized ? text(normalized.package.manifest.id, 'manifest.id') : text(input.packageId, 'packageId');
    const packageVersion = normalized ? text(normalized.package.manifest.version, 'manifest.version') : text(input.packageVersion, 'packageVersion');
    if (!SEMVER.test(packageVersion)) throw new Error('packageVersion must be semantic version x.y.z');
    const definitionPaths = normalized ? normalized.package.definitions.map(item => item.path) : (() => {
      if (!Array.isArray(input.definitionPaths) || input.definitionPaths.length === 0) throw new Error('definitionPaths must declare at least one source document path');
      return input.definitionPaths.map((item, position) => relativeFile(text(item, `definitionPaths[${position}]`)));
    })();
    const manifest: ObjectValue = normalized ? normalized.package.manifest : { profile: UAR_PROFILE_V2, kind: 'PackageManifest', id: packageId, version: packageVersion };
    const definitions = normalized ? normalized.package.definitions : definitionPaths.map(file => ({ path: file, document: {} as ObjectValue }));
    const migrationReceipt: UarMigrationReceipt = normalized?.receipt ?? {
      schemaVersion: 1, sourceProfile: 'new-authoring', targetProfile: UAR_PROFILE_V2, diagnostics: [], activationBlocked: false,
    };
    const index = validateIndex({
      schemaVersion: 1, profile: UAR_PROFILE_V2,
      packageId, packageVersion,
      manifest: input.manifestPath === undefined ? 'manifest.source.json' : relativeFile(text(input.manifestPath, 'manifestPath')),
      definitions: definitionPaths, migrationReceipt: 'migration-receipt.json',
      bindingIntent: input.bindingIntent ?? 'package-only',
    });
    const changes = new Map();
    stage(changes, project, `${workspace}/${INDEX_FILE}`, json(index));
    stage(changes, project, `${workspace}/${index.manifest}`, json(manifest));
    for (const definition of definitions) stage(changes, project, `${workspace}/${definition.path}`, json(definition.document));
    stage(changes, project, `${workspace}/${index.migrationReceipt}`, json(migrationReceipt));
    const recovery = commitChanges(project, [...changes.values()]);
    return { workspace, packageId: index.packageId, packageVersion: index.packageVersion, documents: definitions.length + 2, recovery };
  } finally { release(); }
}

export function updateWorkspaceDocument(input: ObjectValue): ObjectValue {
  const project = root(input), workspace = workspacePath(input), file = relativeFile(text(input.path, 'path'));
  const release = lock(project, workspace);
  try {
    const current = loadWorkspace(input);
    const allowed = new Set([current.index.manifest, ...current.index.definitions]);
    if (!allowed.has(file)) throw new Error(`Workspace update path is not declared by workspace.json: ${file}`);
    const document = object(input.document, 'document');
    if (file === current.index.manifest) {
      if (document.kind !== 'PackageManifest' || document.id !== current.index.packageId || document.version !== current.index.packageVersion) throw new Error('Manifest update cannot change package kind, identity, or version');
    } else if (!['AgentDefinition','TeamDefinition','WorkflowDefinition'].includes(String(document.kind))) throw new Error('Definition update must contain one portable collaboration definition');
    const changes = new Map();
    stage(changes, project, `${workspace}/${file}`, json(document));
    const recovery = commitChanges(project, [...changes.values()]);
    return { workspace, path: file, kind: document.kind, id: document.id, version: document.version, changed: recovery !== null, recovery };
  } finally { release(); }
}

function nextQuestion(workspace: UarWorkspace): UarWorkspaceStatus['nextQuestion'] {
  const manifestDefinition: UarAuthoringDefinition = { path: workspace.index.manifest, document: workspace.package.manifest };
  const teams = workspace.package.definitions.filter(item => item.document.kind === 'TeamDefinition');
  const team = teams.find(item => Array.isArray(workspace.package.manifest.entrypoints) && workspace.package.manifest.entrypoints.some(entry => object(entry).id === item.document.id)) ?? teams[0];
  const checks: [UarAuthoringDefinition | undefined, string, string, (document: ObjectValue) => boolean][] = [
    [manifestDefinition, '/provenance', 'What source and authors establish package provenance?', document => Boolean(document.provenance)],
    [manifestDefinition, '/entrypoints', 'Which single TeamDefinition is the top-level package entrypoint?', document => Array.isArray(document.entrypoints) && document.entrypoints.length === 1],
    [team, '/members', 'Which agent or subteam definition should be the next root-team member?', document => Array.isArray(document.members) && document.members.length > 0],
    [team, '/coordinatorRole', 'Which agent member role coordinates this team?', document => typeof document.coordinatorRole === 'string' && document.coordinatorRole.length > 0],
    [team, '/communication', 'Which explicit root-team role communication edges are allowed?', document => Array.isArray(document.communication)],
    [team, '/taskAcceptance/allowedWorkflows', 'Which exact workflows may the root team accept?', document => Boolean(document.taskAcceptance) && Array.isArray(object(document.taskAcceptance).allowedWorkflows)],
  ];
  for (const definition of workspace.package.definitions.filter(item => item.document.kind === 'AgentDefinition')) checks.push(
    [definition, '/permittedChildren', `Which exact child-agent definitions may ${definition.document.id} invoke?`, document => Array.isArray(document.permittedChildren)],
    [definition, '/skills', `Which exact skill locks does ${definition.document.id} require?`, document => Array.isArray(document.skills)],
    [definition, '/models', `Which model capabilities and aliases does ${definition.document.id} request?`, document => Array.isArray(document.models) && document.models.length > 0],
    [definition, '/context', `Which bounded context may ${definition.document.id} receive?`, document => Boolean(document.context)],
    [definition, '/requestedLimits', `Which limits constrain ${definition.document.id}?`, document => Boolean(document.requestedLimits)],
  );
  for (const definition of teams.filter(item => item !== team)) checks.push(
    [definition, '/members', `Which agents or subteams belong to nested team ${definition.document.id}?`, document => Array.isArray(document.members) && document.members.length > 0],
    [definition, '/coordinatorRole', `Which agent role coordinates nested team ${definition.document.id}?`, document => typeof document.coordinatorRole === 'string' && document.coordinatorRole.length > 0],
  );
  for (const definition of workspace.package.definitions.filter(item => item.document.kind === 'WorkflowDefinition')) checks.push(
    [definition, '/steps', `Which finite role step begins workflow ${definition.document.id}?`, document => Array.isArray(document.steps) && document.steps.length > 0],
  );
  checks.push([team, '/limits', 'Which aggregate limits constrain the root team?', document => Boolean(document.limits)]);
  checks.push([team, '/budget', 'Which aggregate budget constrains the root team?', document => Boolean(document.budget)]);
  for (const [definition, pointer, question, complete] of checks) if (!definition || !complete(definition.document)) return {
    document: definition?.path ?? 'teams/root.json', pointer, question,
  };
  if (!workspace.index.bindingIntent) return { document: INDEX_FILE, pointer: '/bindingIntent', question: 'Should deployment stop after package installation or prepare a private binding?' };
  return null;
}

export function workspaceStatus(input: ObjectValue): UarWorkspaceStatus {
  const workspace = loadWorkspace(input);
  const diagnostics = workspace.migrationReceipt?.diagnostics ?? [];
  const cursor = input.cursor === undefined ? 0 : Number(input.cursor);
  const requested = input.pageSize === undefined ? DEFAULT_PAGE_SIZE : Number(input.pageSize);
  if (!Number.isSafeInteger(cursor) || cursor < 0) throw new Error('cursor must be a nonnegative integer');
  if (!Number.isSafeInteger(requested) || requested < 1 || requested > MAX_PAGE_SIZE) throw new Error(`pageSize must be between 1 and ${MAX_PAGE_SIZE}`);
  const items = diagnostics.slice(cursor, cursor + requested);
  const next = cursor + items.length;
  const question = nextQuestion(workspace);
  return {
    packageId: workspace.index.packageId, packageVersion: workspace.index.packageVersion, profile: UAR_PROFILE_V2,
    counts: {
      agents: workspace.package.definitions.filter(item => item.document.kind === 'AgentDefinition').length,
      teams: workspace.package.definitions.filter(item => item.document.kind === 'TeamDefinition').length,
      workflows: workspace.package.definitions.filter(item => item.document.kind === 'WorkflowDefinition').length,
      diagnostics: diagnostics.length,
    },
    complete: question === null && !workspace.migrationReceipt?.activationBlocked,
    nextQuestion: question,
    diagnostics: { items, cursor: next < diagnostics.length ? String(next) : null, remaining: Math.max(0, diagnostics.length - next) },
  };
}

export function reviseWorkspace(input: ObjectValue): ObjectValue {
  const current = loadWorkspace({ project: input.project as Json, workspace: input.workspace as Json });
  const nextVersion = text(input.nextVersion, 'nextVersion');
  if (!SEMVER.test(nextVersion)) throw new Error('nextVersion must be semantic version x.y.z');
  if (nextVersion === current.index.packageVersion) throw new Error('Maintenance requires a distinct semantic package version');
  const updateReference = (reference: Json): Json => {
    if (!reference || typeof reference !== 'object' || Array.isArray(reference)) return reference;
    const copy = structuredClone(reference);
    if (copy.version === current.index.packageVersion) { copy.version = nextVersion; delete copy.digest; }
    return copy;
  };
  const definitions = current.package.definitions.map(source => {
    const document = structuredClone(source.document);
    document.version = nextVersion; delete document.contentDigest;
    if (document.kind === 'AgentDefinition' && Array.isArray(document.permittedChildren)) document.permittedChildren = document.permittedChildren.map(updateReference);
    if (document.kind === 'TeamDefinition') {
      if (Array.isArray(document.members)) document.members = document.members.map(raw => {
        const member = object(structuredClone(raw), 'member'); member.definition = updateReference(member.definition as Json); return member;
      });
      const acceptance = object(structuredClone(document.taskAcceptance), 'taskAcceptance');
      if (Array.isArray(acceptance.allowedWorkflows)) acceptance.allowedWorkflows = acceptance.allowedWorkflows.map(updateReference);
      document.taskAcceptance = acceptance;
    }
    return { path: source.path, document };
  });
  const manifest = structuredClone(current.package.manifest);
  manifest.version = nextVersion; delete manifest.contentDigest; delete manifest.files; delete manifest.lock;
  if (Array.isArray(manifest.entrypoints)) manifest.entrypoints = manifest.entrypoints.map(updateReference);
  const receipt: UarMigrationReceipt = {
    schemaVersion: 1, sourceProfile: UAR_PROFILE_V2, targetProfile: UAR_PROFILE_V2,
    diagnostics: [], activationBlocked: false,
  };
  return initializeWorkspace({ project: input.project as Json, workspace: input.out as Json, source: { manifest, definitions }, bindingIntent: current.index.bindingIntent ?? 'package-only', receipt } as ObjectValue);
}
