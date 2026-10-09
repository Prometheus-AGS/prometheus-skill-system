import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { guide } from './guidance.mjs';
import { validateTeam, object, text, strings, asJson, target } from './validation.mjs';
import { exportTeam } from './adapters.mjs';
import { writeExport } from './export-files.mjs';
import { installProject } from './project-install.mjs';
import { bindUiRoles } from './ui-bindings.mjs';
import { readState, initState, mutateState, mutateStateAsync, taskAction, completeKbdTask } from './state.mjs';
import { createHandoff, acceptHandoff } from './handoff.mjs';
import { inspectHandoff } from './handoff-provenance.mjs';
import { discoverModels, selectModel } from './models.mjs';
import { queueMemory, publishMemory } from './memory.mjs';
import { dispatchUarAuthoring, isUarAuthoringCommand, uarAuthoringCommands } from './uar-package/commands.mjs';
import {
  refuseUarActivation, uarBindingInstall, uarBindingPreflight, uarBindingStatus,
  uarCapabilities, uarPackageInstall, uarPackagePreflight, uarPackageStatus,
} from './uar-client.mjs';
import { publishCard, discoverCards, sendRequest, listIntakeIssues, importIssues, ackIssues } from './registry.mjs';
import type { ObjectValue, ModelPolicy, Json } from './types.mjs';

function revision(input: ObjectValue): number {
  const r = input.expectedRevision;
  if (!Number.isSafeInteger(r) || Number(r) < 0) throw Error('expectedRevision must be a nonnegative integer from the current state');
  return r as number;
}
const stateFile = (input: ObjectValue): string => path.resolve(text(input.state, 'state'));

const RESOLVER = path.join('shared', 'scripts', 'lib', 'project-id.sh');

/** CLAUDE_PLUGIN_ROOT, then PLUGIN_ROOT, then repo-relative (flat plugin layout, source tree, then cwd ancestors). */
function locateProjectIdResolver(): string | undefined {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const candidates: string[] = [];
  for (const name of ['CLAUDE_PLUGIN_ROOT', 'PLUGIN_ROOT']) {
    const root = process.env[name];
    if (root && root.trim()) candidates.push(path.join(root, RESOLVER));
  }
  candidates.push(path.resolve(here, '..', '..', '..', RESOLVER), path.resolve(here, '..', '..', '..', '..', RESOLVER));
  for (let dir = process.cwd(); ; dir = path.dirname(dir)) {
    candidates.push(path.join(dir, RESOLVER));
    if (path.dirname(dir) === dir) break;
  }
  return candidates.find(file => fs.existsSync(file));
}

let resolvedProjectId: string | null | undefined;
/** Runs project-id.sh --json once per process; undefined when it cannot be located or answers nothing usable. */
function projectId(): string | undefined {
  if (resolvedProjectId !== undefined) return resolvedProjectId ?? undefined;
  resolvedProjectId = null;
  const resolver = locateProjectIdResolver();
  if (!resolver) return undefined;
  try {
    const out = execFileSync('bash', [resolver, '--json'], { cwd: process.cwd(), encoding: 'utf8', timeout: 10000, stdio: ['ignore', 'pipe', 'ignore'] });
    const id = object(JSON.parse(out), 'project-id.sh output').projectId;
    // project:unknown is the resolver's no-answer sentinel; treat it as unresolved so writes fail closed.
    if (typeof id === 'string' && id.trim() && id !== 'project:unknown') resolvedProjectId = id;
  } catch {
    // Leave unresolved: memory.mts fails closed with an explicit project-id error.
  }
  return resolvedProjectId ?? undefined;
}

function withQueueProject(entry: ObjectValue): ObjectValue {
  const kbd = entry.provenance && typeof entry.provenance === 'object' && !Array.isArray(entry.provenance) ? (entry.provenance as ObjectValue).kbd : undefined;
  if (entry.projectId !== undefined || kbd !== undefined) return entry;
  const id = projectId();
  return id === undefined ? entry : { ...entry, projectId: id };
}

function withPublishProject(publication: ObjectValue): ObjectValue {
  const mapping = publication.scopeMapping && typeof publication.scopeMapping === 'object' && !Array.isArray(publication.scopeMapping) ? publication.scopeMapping as ObjectValue : {};
  if (publication.projectId !== undefined || mapping.userId !== undefined || publication.provider !== 'surreal-memory') return publication;
  const id = projectId();
  return id === undefined ? publication : { ...publication, projectId: id };
}

async function dispatch(command: string, input: ObjectValue): Promise<unknown> {
  if (isUarAuthoringCommand(command)) return dispatchUarAuthoring(command, input);
  switch (command) {
    case 'guide': return guide(input);
    case 'validate': return { valid: true, team: validateTeam(input.team) };
    case 'init': return initState(stateFile(input), bindUiRoles(validateTeam(input.team)));
    case 'install-project': return installProject(input);
    case 'status': return readState(stateFile(input));
    case 'team-update': return mutateState(stateFile(input), revision(input), state => {
      const team = validateTeam(input.team);
      if (team.id !== state.team.id) throw Error('team-update cannot change team identity');
      if (state.tasks.some(t => !team.roles.some(r => r.id === t.owner))) throw Error('Cannot remove a role referenced by a task; preserve history and reassign active work explicitly');
      state.team = team;
    });
    case 'export': {
      const team = validateTeam(input.team ?? readState(stateFile(input)).team);
      const result = exportTeam(team, target(input.target ?? team.harness));
      for (const role of team.roles) if (!role.owns.length) result.diagnostics.push(`${role.id}: file ownership is not yet assigned; resolve it before parallel edits.`);
      return { ...writeExport(text(input.out, 'out'), result), verification: result.verification, capabilities: result.capabilities, diagnostics: result.diagnostics, instructions: result.instructions };
    }
    case 'task': return mutateState(stateFile(input), revision(input), state => taskAction(state, object(input.task, 'task action')));
    case 'complete-kbd': return mutateState(stateFile(input), revision(input), state => {
      completeKbdTask(state, object(input.task, 'task action'), text(input.cwd, 'cwd'));
    });
    case 'handoff-create': return mutateState(stateFile(input), revision(input), state => {
      createHandoff(state, object(input.handoff, 'handoff'), text(input.cwd, 'cwd'));
    });
    case 'handoff-inspect': return inspectHandoff(readState(stateFile(input)), text(input.id, 'handoff id'), text(input.cwd, 'cwd'), input);
    case 'handoff-accept': return mutateState(stateFile(input), revision(input), state => {
      const destination = object(input.destination, 'destination');
      const harness = target(destination.harness);
      if (harness === 'bossfang') throw Error('Destination harness must name an execution harness');
      acceptHandoff(state, text(input.id, 'handoff id'), { owner: text(destination.owner, 'destination owner'), harness });
    });
    case 'models-discover': return discoverModels(input);
    case 'models-select': return selectModel(validateTeam(input.team), text(input.roleId, 'roleId'), strings(input.skills ?? [], 'skills'), (input.taskPolicy ?? {}) as ModelPolicy, input.catalog);
    case 'memory-queue': return mutateState(stateFile(input), revision(input), state => { queueMemory(state, withQueueProject(object(input.entry, 'entry'))); });
    case 'memory-publish': {
      let receipt: Json = null;
      const state = await mutateStateAsync(stateFile(input), revision(input), async state => { receipt = await publishMemory(state, withPublishProject(object(input.publication, 'publication'))); });
      return { state, publication: receipt };
    }
    case 'team-publish': return publishCard({ ...input, team: (input.team ?? readState(stateFile(input)).team) as unknown as Json });
    case 'team-discover': return { matches: discoverCards(input) };
    case 'team-request': {
      let state: unknown = null;
      const result = sendRequest(input, apply => { state = mutateState(stateFile(input), revision(input), apply); });
      return { ...result, state };
    }
    case 'team-intake': {
      const before = readState(stateFile(input));
      const card = before.team.card;
      if (!card) throw Error(`Team ${before.team.id} has no card; nothing to import`);
      const issues = listIntakeIssues(card);
      let imported: number[] = [];
      const state = mutateState(stateFile(input), revision(input), s => { imported = importIssues(s, issues); });
      const acked = input.ack === true && imported.length ? ackIssues(card, imported) : [];
      return { imported, skipped: issues.length - imported.length, acked, state };
    }
    case 'uar-capabilities': return uarCapabilities(input);
    case 'uar-package-preflight': return uarPackagePreflight(input);
    case 'uar-package-install': return uarPackageInstall(input);
    case 'uar-package-status': return uarPackageStatus(input);
    case 'uar-binding-preflight': return uarBindingPreflight(input);
    case 'uar-binding-install': return uarBindingInstall(input);
    case 'uar-binding-status': return uarBindingStatus(input);
    case 'uar-activate': return refuseUarActivation();
    default: throw Error(`Unknown command: ${command}`);
  }
}

const commands = ['guide','validate','init','status','team-update','export','install-project','task','complete-kbd','handoff-create','handoff-inspect','handoff-accept','models-discover','models-select','memory-queue','memory-publish','team-publish','team-discover','team-request','team-intake',...uarAuthoringCommands,'uar-capabilities','uar-package-preflight','uar-package-install','uar-package-status','uar-binding-preflight','uar-binding-install','uar-binding-status','uar-activate'];
async function main(): Promise<void> {
  if (Number(process.versions.node.split('.')[0]) < 22) throw Error('Node.js 22 or newer is required');
  const [command, ...args] = process.argv.slice(2);
  if (!command || command === '--help') {
    process.stdout.write(JSON.stringify({ usage: 'node <skill>/scripts/cli.mjs <command> --input <request.json>', commands, note: 'JSON requests preserve spaces and native configuration. UAR package installation is catalog-only; team activation is refused until I2.' }, null, 2) + '\n');
    return;
  }
  let input: ObjectValue = {};
  if (args[0] === '--input' && args[1]) {
    input = object(JSON.parse(fs.readFileSync(args[1], 'utf8').replace(/^\uFEFF/, '')));
    args.splice(0, 2);
  } else if (command !== 'install-project') throw Error('Expected <command> --input <request.json>');
  if (command === 'install-project') while (args.length) {
    const flag = args.shift();
    if (flag === '--check') input.check = true;
    else if (flag === '--dry-run') input.dryRun = true;
    else if (['--project', '--team', '--target'].includes(flag ?? '')) {
      const value = args.shift();
      if (!value || value.startsWith('--')) throw Error(`Missing value for ${flag}`);
      input[flag === '--team' ? 'teamId' : flag!.slice(2)] = value;
    } else throw Error(`Unknown install-project option: ${flag}`);
  }
  if (args.length) throw Error('Unexpected arguments');
  const result = asJson(await dispatch(command, input));
  process.stdout.write(JSON.stringify(result, null, 2) + '\n');
  if (command === 'install-project' && input.check === true && object(result).clean === false) process.exitCode = 2;
}
main().catch(error => {
  process.stderr.write(JSON.stringify({ error: error instanceof Error ? error.message : 'Operation failed' }) + '\n');
  process.exitCode = 1;
});
