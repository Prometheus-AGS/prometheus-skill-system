#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

const root = process.cwd();
const args = new Set(process.argv.slice(2));
const checkOnly = args.has('--check');
const changedBetween = process.argv.indexOf('--changed-between');

const sourceInputs = [
  'package.json',
  '.claude-plugin/plugin.json',
  '.claude-plugin/marketplace.json',
  '.codex-plugin/plugin.json',
  'scripts/install-plugin-generation.js',
  'substrate/sovereign-sync/src/',
  'substrate/learner-model/src/',
  'substrate/skill-index/',
  'skills/',
];

if (changedBetween >= 0) {
  const before = process.argv[changedBetween + 1];
  const after = process.argv[changedBetween + 2];
  if (!before || !after) throw new Error('--changed-between requires two commits');
  let files = [];
  if (!/^0+$/.test(before)) {
    files = execFileSync('git', ['diff', '--name-only', before, after], {
      cwd: root,
      encoding: 'utf8',
    })
      .trim()
      .split('\n')
      .filter(Boolean);
  } else {
    files = sourceInputs;
  }
  const relevant = files.some(file =>
    sourceInputs.some(input => (input.endsWith('/') ? file.startsWith(input) : file === input))
  );
  if (process.env.GITHUB_OUTPUT) {
    fs.appendFileSync(process.env.GITHUB_OUTPUT, `relevant=${relevant}\n`);
  }
  console.log(`Documentation source inputs changed: ${relevant}`);
  process.exit(0);
}

function read(relative) {
  return fs.readFileSync(path.join(root, relative), 'utf8');
}

function json(relative) {
  return JSON.parse(read(relative));
}

function walk(directory, predicate, result = []) {
  const absolute = path.join(root, directory);
  if (!fs.existsSync(absolute)) return result;
  for (const entry of fs.readdirSync(absolute, { withFileTypes: true })) {
    if (
      ['.git', 'node_modules', 'imported', 'tests', 'fixtures'].includes(entry.name)
    )
      continue;
    const relative = path.join(directory, entry.name);
    if (entry.isDirectory()) walk(relative, predicate, result);
    else if (predicate(relative)) result.push(relative.split(path.sep).join('/'));
  }
  return result;
}

function rustBlock(source, declaration) {
  const start = source.indexOf(declaration);
  if (start < 0) throw new Error(`Rust declaration not found: ${declaration}`);
  const brace = source.indexOf('{', start);
  let depth = 0;
  for (let index = brace; index < source.length; index += 1) {
    if (source[index] === '{') depth += 1;
    else if (source[index] === '}') depth -= 1;
    if (depth === 0) return source.slice(brace + 1, index);
  }
  throw new Error(`Unterminated Rust declaration: ${declaration}`);
}

function rustFields(source, name) {
  const block = rustBlock(source, `pub struct ${name} {`);
  return [...block.matchAll(/^\s*pub\s+([a-zA-Z0-9_]+)\s*:\s*(.+),\s*$/gm)].map(match => ({
    name: match[1].replace(/_([a-z])/g, (_, letter) => letter.toUpperCase()),
    type: match[2].replace(/\s+/g, ' ').trim().replaceAll('|', '\\|'),
  }));
}

function rustVariants(source, name) {
  return rustBlock(source, `pub enum ${name} {`)
    .split('\n')
    .map(line => line.trim().match(/^([A-Z][A-Za-z0-9_]*)[,]?$/)?.[1])
    .filter(Boolean)
    .map(value => value.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase());
}

function routeReference(source) {
  const constants = new Map(
    [...source.matchAll(/pub const\s+([A-Z0-9_]+):\s*&str\s*=\s*"([^"]+)";/g)].map(match => [
      match[1],
      match[2],
    ])
  );
  const routes = [];
  for (const match of source.matchAll(
    /\.route\(\s*("[^"]+"|[A-Z0-9_]+)\s*,\s*(get|post|put|patch|delete)\(/gs
  )) {
    const token = match[1];
    const route = token.startsWith('"') ? token.slice(1, -1) : constants.get(token);
    if (route) routes.push({ method: match[2].toUpperCase(), route });
  }
  return routes
    .filter(
      (route, index, all) =>
        all.findIndex(item => item.method === route.method && item.route === route.route) === index
    )
    .sort(
      (left, right) =>
        left.route.localeCompare(right.route) || left.method.localeCompare(right.method)
    );
}

function cliFlags(source) {
  const flags = [...source.matchAll(/#\[arg\(([\s\S]*?)\)\]\s*(?:pub\s+)?([a-z][a-z0-9_]*)\s*:/g)]
    .filter(match => /\blong\b/.test(match[1]))
    .map(match => `--${match[2].replaceAll('_', '-')}`);
  return [...new Set(flags)].sort();
}

function pluginTargets(source) {
  const match = source.match(/const TARGETS = \[([\s\S]*?)\];/);
  if (!match) throw new Error('Plugin target matrix not found');
  const copy = new Set(
    [...source.matchAll(/const COPY_TARGETS = new Set\(\[([^\]]+)\]\)/g)].flatMap(item =>
      [...item[1].matchAll(/'([^']+)'/g)].map(target => target[1])
    )
  );
  return [...match[1].matchAll(/'([^']+)'/g)].map(item => ({
    target: item[1],
    mode: copy.has(item[1]) ? 'verified copy' : 'symlink',
  }));
}

function table(headers, rows) {
  return [
    `| ${headers.join(' | ')} |`,
    `| ${headers.map(() => '---').join(' | ')} |`,
    ...rows.map(row => `| ${row.join(' | ')} |`),
  ].join('\n');
}

const pkg = json('package.json');
const restSource = read('substrate/sovereign-sync/src/rest_api.rs');
const mainSource = read('substrate/sovereign-sync/src/main.rs');
const pluginSource = read('scripts/install-plugin-generation.js');
const routes = routeReference(restSource);
const requestFields = rustFields(restSource, 'SignedSyncPushRequest');
const receiptFields = rustFields(restSource, 'PushReceipt');
const states = rustVariants(restSource, 'PushLocalState');
const flags = cliFlags(mainSource);
const targets = pluginTargets(pluginSource);
const skillFiles = walk('skills', file => file.endsWith('/SKILL.md')).sort();

const marker = 'PROMETHEUS DOCS SYNC: runtime-reference';
const body = `<!-- BEGIN ${marker} -->

Release: **${pkg.version}**

Generated by: \`npm run docs:sync\`

Skill definitions: **${skillFiles.length}**

Plugin targets: **${targets.length}**

## HTTP routes

${table(
  ['Method', 'Route'],
  routes.map(item => [`\`${item.method}\``, `\`${item.route}\``])
)}

## SignedSyncPushRequest schema

${table(
  ['Field', 'Rust type'],
  requestFields.map(item => [`\`${item.name}\``, `\`${item.type}\``])
)}

## PushReceipt schema

${table(
  ['Field', 'Rust type'],
  receiptFields.map(item => [`\`${item.name}\``, `\`${item.type}\``])
)}

## Push local states

${states.map(state => `- \`${state}\``).join('\n')}

## Sovereign Sync CLI/config reference

${flags.map(flag => `- \`${flag}\``).join('\n')}

## Plugin target matrix

${table(
  ['Target', 'Projection'],
  targets.map(item => [`\`${item.target}\``, item.mode])
)}

## Capability status

${table(
  ['Capability', 'Code-backed status'],
  [
    ['Local transport', 'Unix socket by default; loopback TCP is explicit and token-authenticated'],
    ['Sync push', 'Signed v2 request with durable exact-replay receipt'],
    ['Learner model', 'Loro immutable evidence with deterministic derived-state fold'],
    ['Plugin distribution', 'Ed25519-signed generation, shared index, and 14 signed receipts'],
    ['Validation', 'Local certification; hosted automation limited to docs sync and Pages'],
  ]
)}

<!-- END ${marker} -->
`;

const outputs = new Map([
  [
    'docs/generated/runtime-reference.md',
    `# Generated runtime reference\n\nThis file contains deterministic reference data. Edit source declarations, not the managed block.\n\n${body}`,
  ],
  [
    'site/docs/operations/generated-reference.md',
    `---\ntitle: Generated runtime reference\ndescription: Deterministic API, schema, CLI, capability, skill, target, and release metadata.\n---\n\n# Generated runtime reference\n\nThis page is synchronized from code declarations. Narrative design belongs in the authored guides.\n\n${body}`,
  ],
]);

const changed = [];
for (const [relative, content] of outputs) {
  const absolute = path.join(root, relative);
  const normalized = content.endsWith('\n') ? content : `${content}\n`;
  const existing = fs.existsSync(absolute) ? fs.readFileSync(absolute, 'utf8') : null;
  if (existing === normalized) continue;
  changed.push(relative);
  if (!checkOnly) {
    fs.mkdirSync(path.dirname(absolute), { recursive: true });
    fs.writeFileSync(absolute, normalized);
  }
}

if (changed.length > 0) {
  const verb = checkOnly ? 'out of date' : 'updated';
  console.log(`Documentation sync ${verb}:\n${changed.map(file => `- ${file}`).join('\n')}`);
  if (checkOnly) process.exit(1);
} else {
  console.log('Documentation sync is clean.');
}
