#!/usr/bin/env node
// kbd-contract-lint.mjs — adoption-time lint for KBD adopted contracts (VLBV CR-19, CR-30).
//
// Document checks only: it reads the contract (and, when supplied, the canonical
// status JSON and the phase's review findings) and never executes generated code,
// so it is legal before the value-phase boundary. kbd-contract-adopt.sh runs it
// after structural validation and before any canonical write.
//
// Usage: node kbd-contract-lint.mjs --contract <file> [--status <status.json>]
//                                    [--review <findings.json>] [--owner-decision <id>]
// Exit 0: no defect. Exit 2: one line per defect on stderr, "<code>: <message>".
import fs from 'node:fs';

const args = process.argv.slice(2);
const opt = name => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};
const contractPath = opt('--contract');
if (!contractPath) {
  process.stderr.write('kbd-contract-lint: --contract is required\n');
  process.exit(2);
}
const contract = JSON.parse(fs.readFileSync(contractPath, 'utf8'));
const readJson = file => {
  try {
    return file && fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, 'utf8')) : null;
  } catch {
    return null;
  }
};
const status = readJson(opt('--status'));
const review = readJson(opt('--review'));
const ownerDecision = opt('--owner-decision');

const defects = [];
const add = (code, message) => defects.push(`${code}: ${message}`);
const scenarios = Array.isArray(contract.scenarioMatrix) ? contract.scenarioMatrix : [];
const v2 = contract.contractVersion === '2';
const ID = /^[A-Z][A-Z0-9]*(-[A-Z0-9]+)*-[0-9]+$/;
const VERDICTS = ['PASS', 'FAIL', 'BLOCKED', 'NOT_EXERCISED'];

// --- every version -------------------------------------------------------
for (const s of scenarios)
  if (!ID.test(s.id ?? '')) add('scenario-id', `${s.id ?? '<missing>'} does not match ${ID}`);

// The remaining checks are new in version 2; a bound v1 contract keeps the rules
// it was adopted under until its phase closes.
if (v2) {
  // Placeholder text left from a template.
  const placeholder = /<[a-z][a-z0-9 _./-]*>|\bTODO\b|\bTBD\b|\bFIXME\b/;
  (function walk(value, where) {
    if (typeof value === 'string') {
      if (placeholder.test(value))
        add('placeholder', `${where} still contains template text: ${value.slice(0, 80)}`);
      return;
    }
    if (Array.isArray(value)) value.forEach((item, i) => walk(item, `${where}[${i}]`));
    else if (value && typeof value === 'object')
      for (const [k, v] of Object.entries(value)) {
        if (where === 'contract' && k === 'identity') continue; // stamped by the writer
        walk(v, `${where}.${k}`);
      }
  })(contract, 'contract');

  // ownerTask must exist in the canonical task surface once tasks are registered.
  if (status) {
    const phase = status.phases?.[contract.identity?.phaseId];
    const tasks = new Set();
    for (const [changeId, change] of Object.entries(phase?.changes ?? {})) {
      const list = Array.isArray(change.tasks) ? change.tasks : Object.values(change.tasks ?? {});
      for (const task of list)
        if (task?.id) {
          tasks.add(task.id);
          tasks.add(`${changeId}/${task.id}`);
        }
    }
    if (tasks.size)
      for (const s of scenarios)
        for (const task of String(s.ownerTask ?? '')
          .split('+')
          .map(t => t.trim())
          .filter(Boolean)) {
          if (!tasks.has(task))
            add(
              'owner-task',
              `${s.id}: ownerTask ${task} is not a registered task of phase ${contract.identity?.phaseId}`
            );
        }
  }

  // Unresolved CRITICAL review findings on acceptance content block adoption (CR-30, FR-C6).
  if (review) {
    const critical = (review.findings ?? []).filter(
      f =>
        f.severity === 'CRITICAL' &&
        /scenarioMatrix|boundaryPacket|oracle|negativeControl|scenario/i.test(
          `${f.file ?? ''} ${f.claim ?? ''} ${f.evidence ?? ''}`
        )
    );
    if (critical.length && !ownerDecision)
      add(
        'review-critical',
        `${critical.length} unresolved CRITICAL review finding(s) on acceptance content; adoption waits for an owner decision (--owner-decision <decision-id>)`
      );
  }

  if (JSON.stringify(contract.statusVocabulary) !== JSON.stringify(VERDICTS))
    add('status-vocabulary', `must be exactly ${JSON.stringify(VERDICTS)}`);
  const commands = Array.isArray(contract.boundaryPacket?.commands)
    ? contract.boundaryPacket.commands
    : [];
  const byId = new Map(commands.map(c => [c?.id, c]));
  for (const c of commands) {
    if (!c || typeof c !== 'object' || !Array.isArray(c.argv) || !c.argv.length)
      add(
        'packet-command',
        `boundaryPacket.commands entries must be typed objects with argv (got ${JSON.stringify(c)?.slice(0, 60)})`
      );
    else if (!['build', 'run', 'scenario', 'negativeControl'].includes(c.kind))
      add('packet-command', `${c.id}: kind must be build, run, scenario or negativeControl`);
  }
  const ids = new Set(scenarios.map(s => s.id));
  // Every functionalScope item maps to a scenario or is declared non-observable.
  const covered = new Set(scenarios.flatMap(s => (Array.isArray(s.scopeRefs) ? s.scopeRefs : [])));
  for (const item of contract.functionalScope ?? []) {
    const text = typeof item === 'string' ? item : item?.item;
    if (typeof item === 'object' && item?.nonObservable === true) continue;
    if (!covered.has(text))
      add(
        'scope-unmapped',
        `functionalScope item maps to no scenario (scopeRefs) and is not marked nonObservable: ${String(text).slice(0, 80)}`
      );
  }
  for (const s of scenarios) {
    // Each scenario has a scenario command and a negative-control command in the packet.
    if (!commands.some(c => c?.kind === 'scenario' && (c.scenarioIds ?? []).includes(s.id)))
      add('scenario-command', `${s.id}: no boundaryPacket command of kind scenario lists it`);
    const ref = s.negativeControl?.commandRef;
    const neg = ref ? byId.get(ref) : null;
    if (!ref) add('negative-control-command', `${s.id}: negativeControl.commandRef is missing`);
    else if (!neg || neg.kind !== 'negativeControl')
      add(
        'negative-control-command',
        `${s.id}: negativeControl.commandRef ${ref} is not a negativeControl command of the packet`
      );
    // Weak oracle: a bare exit status, or truth taken from the system under test.
    const comparator = String(s.oracle?.comparator ?? '');
    if (
      /^\s*(process\s+)?exit(\s*(code|status))?\s*(==|=|is)?\s*0?\s*$/i.test(comparator) ||
      /^exit[-_ ]code$/i.test(comparator)
    )
      add('weak-oracle', `${s.id}: comparator is only a process exit status`);
    const truth = String(s.oracle?.expectedTruthSource ?? '')
      .trim()
      .toLowerCase();
    if (
      !truth ||
      /system under test|the sut\b|same (binary|service|process)/.test(truth) ||
      (s.productionEntry && truth === String(s.productionEntry).trim().toLowerCase())
    )
      add(
        'weak-oracle',
        `${s.id}: expectedTruthSource must be independent of the system under test`
      );
    // Nondeterministic claims declare their sampling before results.
    if (
      s.nondeterministic === true &&
      !(
        s.stochastic &&
        Number.isInteger(s.stochastic.samples) &&
        s.stochastic.grader &&
        s.stochastic.stoppingRule
      )
    )
      add(
        'stochastic-missing',
        `${s.id}: marked nondeterministic without a stochastic block (samples, grader, stoppingRule)`
      );
    if (!/^(openspec:|speckit:|native:|assumption: ).+/.test(String(s.sourceRequirement ?? '')))
      add(
        'source-requirement',
        `${s.id}: sourceRequirement must be an openspec:/speckit:/native: reference or 'assumption: <text>'`
      );
  }
  // Each mandatory safety check traces to existing scenarios.
  for (const check of contract.requiredSafetyChecks ?? []) {
    if (typeof check !== 'object' || !Array.isArray(check.scenarioIds) || !check.scenarioIds.length)
      add(
        'safety-unmapped',
        `requiredSafetyChecks entry maps to no scenario: ${JSON.stringify(check).slice(0, 80)}`
      );
    else
      for (const id of check.scenarioIds)
        if (!ids.has(id)) add('safety-unmapped', `${check.id}: scenario ${id} does not exist`);
  }
  for (const key of [
    'implementationComplete',
    'behaviorAccepted',
    'releaseEligible',
    'releaseAuthorized',
  ]) {
    if (typeof contract.closure?.[key] !== 'boolean')
      add('closure', `closure.${key} must be a boolean`);
  }
  if (contract.method?.name !== 'VLBV')
    add(
      'method',
      'method.name must be "VLBV" (deviations are signed KBD-METHOD decisions, not contract text)'
    );
  if (!/^[0-9a-f]{64}$/.test(contract.calibration?.sha256 ?? ''))
    add('calibration', 'calibration.sha256 must be the SHA-256 of the owner-reviewed examples');
}

if (defects.length) {
  process.stderr.write(defects.join('\n') + '\n');
  process.exit(2);
}
process.stdout.write(
  `kbd-contract-lint: ${scenarios.length} scenario(s) clean (contract v${contract.contractVersion})\n`
);
