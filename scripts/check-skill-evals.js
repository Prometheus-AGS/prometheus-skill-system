#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const corpus = JSON.parse(
  fs.readFileSync(path.join(root, 'evals/skill-activation/critical-30.json'), 'utf8')
);
const budgets = JSON.parse(
  fs.readFileSync(path.join(root, 'evals/skill-activation/harness-budgets.json'), 'utf8')
);
const failures = [];

if (corpus.schemaVersion !== '1' || corpus.cases.length !== 30 || corpus.trialsPerHarness !== 3) {
  failures.push('critical corpus must be schema 1 with 30 cases and three trials');
}
if (new Set(corpus.cases.map(entry => entry.id)).size !== 30) {
  failures.push('critical corpus case ids are not unique');
}
if (new Set(corpus.cases.map(entry => entry.skill)).size !== 5) {
  failures.push('critical corpus must cover exactly five skills');
}
const kinds = Object.fromEntries(
  ['explicit', 'implicit', 'near_miss'].map(kind => [
    kind,
    corpus.cases.filter(entry => entry.kind === kind).length,
  ])
);
if (kinds.explicit !== 5 || kinds.implicit !== 15 || kinds.near_miss !== 10) {
  failures.push(`unexpected corpus distribution: ${JSON.stringify(kinds)}`);
}
for (const entry of corpus.cases) {
  if (!Array.isArray(entry.expectedCommands) || typeof entry.expectedInvocation !== 'boolean') {
    failures.push(`${entry.id}: missing deterministic trace contract`);
  }
  if (entry.forbidDirectWrites !== true) {
    failures.push(`${entry.id}: direct compatibility writes are not forbidden`);
  }
}

if (budgets.schemaVersion !== '1' || budgets.inventorySkills !== 145) {
  failures.push('harness budget baseline does not match the 145-skill inventory');
}
for (const harness of ['claude-code', 'codex', 'opencode', 'kimi']) {
  if (!budgets.harnesses?.[harness]) {
    failures.push(`${harness}: missing discovery budget record`);
  }
}

if (failures.length) {
  console.error(failures.join('\n'));
  process.exit(1);
}
console.log(
  `Skill evaluation contract: ${corpus.cases.length} prompts, ${corpus.cases.length * corpus.trialsPerHarness} scheduled trials, 4 budget records`
);
