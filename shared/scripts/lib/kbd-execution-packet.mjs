// Document-only preflight. No execution, signature, authority, or closure outcome.
const DIGEST = /^[0-9a-f]{64}$/;
const ID = /^[A-Za-z0-9][A-Za-z0-9._:-]*$/;
const COMMAND_KINDS = new Set(['build', 'run', 'scenario', 'negativeControl']);
const PARENT_KEYS = ['projectId', 'runId', 'phaseId', 'contractDigest', 'bindingRevision'];

function object(value, label) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  return value;
}

function fields(value, allowed, label) {
  object(value, label);
  for (const name of Object.keys(value)) {
    if (!allowed.includes(name)) throw new Error(`${label} has unknown field ${name}`);
  }
}

function id(value, label) {
  if (typeof value !== 'string' || !ID.test(value)) throw new Error(`invalid ${label}`);
  return value;
}

function digest(value, label) {
  if (typeof value !== 'string' || !DIGEST.test(value)) throw new Error(`invalid ${label}`);
}

function unique(values, label, { empty = false } = {}) {
  if (!Array.isArray(values) || (!empty && !values.length)) {
    throw new Error(`${label} must be ${empty ? 'an' : 'a nonempty'} array`);
  }
  const set = new Set(values.map(value => id(value, label)));
  if (set.size !== values.length) throw new Error(`duplicate ${label}`);
  return set;
}

function matrix(contract) {
  if (contract?.kind !== 'kbd.adopted-contract' || contract.contractVersion !== '2') {
    throw new Error('parent must be a version 2 adopted contract');
  }
  const scenarios = contract.scenarioMatrix;
  if (!Array.isArray(scenarios)) throw new Error('parent scenarioMatrix is missing');
  unique(scenarios.map(row => row?.id), 'parent scenario IDs');
  const fullCommands = contract.boundaryPacket?.commands;
  if (!Array.isArray(fullCommands)) throw new Error('parent command matrix is missing');
  unique(fullCommands.map(command => command?.id), 'parent command IDs');
  const fullById = new Map(fullCommands.map(command => [command.id, command]));
  for (const row of scenarios) {
    if (!fullCommands.some(c => c.kind === 'scenario' && Array.isArray(c.scenarioIds) && c.scenarioIds.includes(row.id))
        || fullById.get(row.negativeControl?.commandRef)?.kind !== 'negativeControl') {
      throw new Error(`parent lacks case or control mapping for ${row.id}`);
    }
  }
  return new Map(scenarios.map(row => [row.id, row]));
}

export function preflightExecutionPacket(contract, binding, packet) {
  const rows = matrix(contract);
  fields(packet, ['schemaVersion', 'kind', 'parent', 'attemptId', 'candidateDigest', 'targets',
    'scenarioIds', 'commands'], 'packet');
  if (packet.kind !== 'kbd.execution-packet' || packet.schemaVersion !== 1) {
    throw new Error('unsupported execution packet format');
  }
  object(binding, 'bound parent identity');
  fields(packet.parent, PARENT_KEYS, 'packet parent');
  for (const key of ['projectId', 'runId', 'phaseId', 'contractDigest']) {
    if (packet.parent[key] !== binding[key] || packet.parent[key] !== contract.identity?.[key]) {
      throw new Error(`parent ${key} mismatch`);
    }
  }
  if (!Number.isSafeInteger(binding.bindingRevision) || binding.bindingRevision < 1
      || packet.parent.bindingRevision !== binding.bindingRevision) throw new Error('parent bindingRevision mismatch');
  for (const key of ['projectId', 'runId', 'phaseId']) id(binding[key], key);
  digest(binding.contractDigest, 'parent contractDigest');
  digest(packet.candidateDigest, 'candidateDigest');
  id(packet.attemptId, 'attemptId');

  const targets = unique(packet.targets, 'targets');
  const selection = unique(packet.scenarioIds, 'selected scenario IDs');
  for (const scenarioId of selection) {
    if (!rows.has(scenarioId)) throw new Error(`scenario is not bound to parent: ${scenarioId}`);
  }
  if (!Array.isArray(packet.commands)) throw new Error('commands must be an array');
  unique(packet.commands.map(command => command?.id), 'packet command IDs');
  const commands = new Map();
  const required = new Set();
  const observations = new Map();
  const forKey = (scenarioId, target, kind) => JSON.stringify([scenarioId, target, kind]);
  for (const command of packet.commands) {
    fields(command, ['id', 'kind', 'argv', 'cwd', 'target', 'requires', 'scenarioIds'], 'command');
    if (!COMMAND_KINDS.has(command.kind)) throw new Error(`invalid command kind: ${command.id}`);
    if (!targets.has(command.target)) throw new Error(`unknown command target: ${command.id}`);
    if (!Array.isArray(command.argv) || !command.argv.length || !command.argv[0]
        || command.argv.some(arg => typeof arg !== 'string' || arg.includes('\0'))) {
      throw new Error(`invalid command argv: ${command.id}`);
    }
    if (command.cwd != null && (typeof command.cwd !== 'string' || !command.cwd
        || command.cwd.includes('\0'))) throw new Error(`invalid command cwd: ${command.id}`);
    const dependencies = unique(command.requires, `prerequisites of ${command.id}`, { empty: true });
    const ancestors = new Set();
    for (const prerequisite of dependencies) {
      const prior = commands.get(prerequisite);
      if (!prior) throw new Error(`missing or out-of-order prerequisite: ${command.id} -> ${prerequisite}`);
      ancestors.add(prerequisite);
      for (const inherited of prior.ancestors) ancestors.add(inherited);
    }
    commands.set(command.id, { ancestors });
    const isBehavior = command.kind === 'scenario' || command.kind === 'negativeControl';
    const observed = unique(command.scenarioIds, `scenario IDs of ${command.id}`, { empty: !isBehavior });
    if (!isBehavior && observed.size) throw new Error(`prerequisite cannot claim a scenario: ${command.id}`);
    for (const scenarioId of observed) {
      if (!selection.has(scenarioId)) throw new Error(`command expands selected scope: ${command.id} -> ${scenarioId}`);
      const lookup = forKey(scenarioId, command.target, command.kind);
      if (observations.has(lookup)) throw new Error(`duplicate ${command.kind} for ${scenarioId} on ${command.target}`);
      observations.set(lookup, command.id);
    }
    if (isBehavior) {
      required.add(command.id);
      for (const prerequisite of ancestors) required.add(prerequisite);
    }
  }
  for (const scenarioId of selection) {
    for (const target of targets) {
      const positive = observations.get(forKey(scenarioId, target, 'scenario'));
      const control = observations.get(forKey(scenarioId, target, 'negativeControl'));
      if (!positive || !control) throw new Error(`missing case or control for ${scenarioId} on ${target}`);
      if (!commands.get(control).ancestors.has(positive)) {
        throw new Error(`control must depend on its exact case: ${scenarioId} on ${target}`);
      }
    }
  }
  for (const commandId of commands.keys()) {
    if (!required.has(commandId)) throw new Error(`command is outside prerequisite closure: ${commandId}`);
  }
  return {
    status: 'PREFLIGHT_ONLY',
    executionAuthorized: false,
    parentClosureEligible: false,
    parentScenarioCount: rows.size,
    selectedScenarioIds: [...selection],
    outOfPacketScenarioIds: [...rows.keys()].filter(scenarioId => !selection.has(scenarioId)),
    targetIds: [...targets],
    orderedCommandIds: [...commands.keys()],
  };
}
