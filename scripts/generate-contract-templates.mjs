#!/usr/bin/env node
// generate-contract-templates.mjs — derive the contract field lists in the KBD
// authoring templates from the adopted-contract v2 schema (VLBV CR-20), so the
// templates and the schema cannot disagree about which fields exist.
//
//   node scripts/generate-contract-templates.mjs           rewrite the generated blocks
//   node scripts/generate-contract-templates.mjs --check   exit 1 if a block is stale
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const refs = path.join(root, 'skills/process/kbd-process-orchestrator/references');
const schema = JSON.parse(
  fs.readFileSync(path.join(refs, 'schemas/adopted-contract.v2.schema.json'), 'utf8')
);
const check = process.argv.includes('--check');

const scenario = schema.properties.scenarioMatrix.items;
const describe = (props, key) =>
  (props[key]?.description ?? '').replace(/\s+/g, ' ').replace(/\|/g, '/');
const shape = value =>
  value?.type === 'object'
    ? `object (${(value.required ?? []).join(', ')})`
    : value?.type === 'array'
      ? 'array'
      : value?.const !== undefined
        ? `\`${JSON.stringify(value.const)}\``
        : (value?.type ?? (value?.enum ? value.enum.join(' / ') : 'any'));

function block() {
  const lines = [
    '<!-- generated:contract-fields:start — scripts/generate-contract-templates.mjs; do not edit by hand -->',
    `Scenario ID pattern: \`${scenario.properties.id.pattern}\` (for example \`SCN-AUTH-001\`).`,
    '',
    `Contract (v${schema.properties.contractVersion.const}) required top-level fields: ${schema.required.map(k => `\`${k}\``).join(', ')}.`,
    '',
    '| Scenario field | Required | Shape | Meaning |',
    '|---|---|---|---|',
  ];
  for (const [key, value] of Object.entries(scenario.properties)) {
    lines.push(
      `| \`${key}\` | ${(scenario.required ?? []).includes(key) ? 'yes' : 'no'} | ${shape(value)} | ${describe(scenario.properties, key)} |`
    );
  }
  lines.push(
    '',
    `Status vocabulary: ${schema.properties.statusVocabulary.const.map(v => `\`${v}\``).join(' / ')}.`,
    `Boundary packet command kinds: ${schema.properties.boundaryPacket.properties.commands.items.properties.kind.enum.map(v => `\`${v}\``).join(', ')}.`,
    '<!-- generated:contract-fields:end -->'
  );
  return lines.join('\n');
}

const generated = block();
const pattern =
  /<!-- generated:contract-fields:start[\s\S]*?<!-- generated:contract-fields:end -->/;
let stale = 0;
for (const name of ['templates/acceptance.template.md', 'templates/adopted-contract.template.md']) {
  const file = path.join(refs, name);
  const text = fs.readFileSync(file, 'utf8');
  if (!pattern.test(text)) {
    process.stderr.write(`${name}: missing generated:contract-fields block\n`);
    stale++;
    continue;
  }
  const next = text.replace(pattern, () => generated);
  if (next === text) continue;
  if (check) {
    process.stderr.write(
      `${name}: contract field list is stale; run node scripts/generate-contract-templates.mjs\n`
    );
    stale++;
  } else fs.writeFileSync(file, next);
}
if (check && stale) process.exit(1);
process.stdout.write(
  check
    ? 'contract templates: in sync with adopted-contract.v2.schema.json\n'
    : 'contract templates: regenerated\n'
);
