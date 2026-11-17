#!/usr/bin/env node
// Build a static call graph of the skills in a plugin: which skills name which
// other skills in their text. Used to tell skills that other skills call from
// skills only an operator or a plain request reaches.
//
// Usage: node skill-call-graph.mjs <plugin-dir> [--json]
//
// A reference is a skill's name written as `name`, /name, "the name skill" or
// "name skill". A bare word is not counted, because many skill names are
// ordinary words (plan, docs, performance). The graph is therefore a lower
// bound on prose references and an upper bound on real hand-offs: naming a
// skill is not the same as invoking it.
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';

const args = process.argv.slice(2);
const pluginDir = args.find(a => !a.startsWith('--'));
if (!pluginDir) {
  console.error('usage: skill-call-graph.mjs <plugin-dir> [--json]');
  process.exit(64);
}
const yaml = createRequire(import.meta.url)('js-yaml');
const TEXT = /\.(md|mdx|sh|bash|mjs|js|ts|py|json|ya?ml|toml|txt)$/;
const SKIP = new Set(['node_modules', '.git', 'target', 'evals', 'tests', 'fixtures']);
const MAX_BYTES = 400_000;

function frontmatter(file) {
  const match = fs.readFileSync(file, 'utf8').match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!match) return null;
  try {
    return yaml.load(match[1]);
  } catch {
    return null;
  }
}

// Every skill directory, top-level or nested, keyed by its real path.
const skillsDir = path.join(pluginDir, 'skills');
const skills = [];
const findSkills = (directory, depth) => {
  for (const name of fs.readdirSync(directory).sort()) {
    if (SKIP.has(name)) continue;
    const child = path.join(directory, name);
    if (!fs.lstatSync(child).isDirectory()) continue;
    const file = path.join(child, 'SKILL.md');
    if (fs.existsSync(file)) {
      const fm = frontmatter(file);
      if (fm?.name) {
        const description = `${fm.description ?? ''} ${fm.when_to_use ?? ''}`.trim();
        skills.push({ name: String(fm.name), dir: child, nested: depth > 0, descriptionChars: description.length, modelInvocable: fm['disable-model-invocation'] !== true });
      }
    }
    findSkills(child, depth + 1);
  }
};
findSkills(skillsDir, 0);

const names = [...new Set(skills.map(s => s.name))].sort((a, b) => b.length - a.length);
const escape = text => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const pattern = new RegExp(
  '(?:`(?:[a-z0-9-]+:)?(' + names.map(escape).join('|') + ')`)' +
    '|(?:(?<![A-Za-z0-9_.\\-/])/(' + names.map(escape).join('|') + ')(?![A-Za-z0-9_\\-/.]))' +
    '|(?:\\b(' + names.map(escape).join('|') + ') skill\\b)',
  'g'
);

// Text that belongs to a skill, excluding any nested skill's own directory.
const skillDirs = new Set(skills.map(s => s.dir));
function ownText(skill) {
  let text = '';
  const visit = directory => {
    for (const name of fs.readdirSync(directory)) {
      if (SKIP.has(name)) continue;
      const child = path.join(directory, name);
      const stat = fs.lstatSync(child);
      if (stat.isDirectory()) {
        if (child !== skill.dir && skillDirs.has(child)) continue;
        visit(child);
      } else if (stat.isFile() && TEXT.test(name) && stat.size <= MAX_BYTES) {
        text += `\n${fs.readFileSync(child, 'utf8')}`;
      }
    }
  };
  visit(skill.dir);
  return text;
}

// One node per skill name; a name that exists top-level and nested is one node.
const nodes = new Map();
for (const skill of skills) {
  const node = nodes.get(skill.name) ?? { name: skill.name, topLevel: false, nested: false, descriptionChars: 0, modelInvocable: true, calls: new Set(), calledBy: new Set() };
  if (skill.nested) node.nested = true;
  else node.topLevel = true;
  node.descriptionChars = Math.max(node.descriptionChars, skill.descriptionChars);
  node.modelInvocable = node.modelInvocable && skill.modelInvocable;
  nodes.set(skill.name, node);
}
for (const skill of skills) {
  const text = ownText(skill);
  for (const match of text.matchAll(pattern)) {
    const target = match[1] ?? match[2] ?? match[3];
    if (target === skill.name) continue;
    nodes.get(skill.name).calls.add(target);
    nodes.get(target).calledBy.add(skill.name);
  }
}

// References from outside skills: agents, hooks and shared scripts in the plugin.
const external = new Map();
for (const area of ['agents', 'hooks', 'shared', 'scripts']) {
  const base = path.join(pluginDir, area);
  if (!fs.existsSync(base)) continue;
  const visit = directory => {
    for (const name of fs.readdirSync(directory)) {
      if (SKIP.has(name)) continue;
      const child = path.join(directory, name);
      const stat = fs.lstatSync(child);
      if (stat.isDirectory()) visit(child);
      else if (stat.isFile() && TEXT.test(name) && stat.size <= MAX_BYTES) {
        for (const match of fs.readFileSync(child, 'utf8').matchAll(pattern)) {
          const target = match[1] ?? match[2] ?? match[3];
          external.set(target, (external.get(target) ?? new Set()).add(area));
        }
      }
    }
  };
  visit(base);
}

const list = [...nodes.values()].map(node => ({
  name: node.name,
  topLevel: node.topLevel,
  nested: node.nested,
  modelInvocable: node.modelInvocable,
  descriptionChars: node.descriptionChars,
  calls: [...node.calls].sort(),
  calledBy: [...node.calledBy].sort(),
  referencedFrom: [...(external.get(node.name) ?? [])].sort(),
}));

if (args.includes('--json')) {
  console.log(JSON.stringify(list, null, 1));
} else {
  const called = list.filter(n => n.calledBy.length > 0);
  const edges = list.reduce((total, n) => total + n.calls.length, 0);
  console.log(`skills (by name): ${list.length}; edges: ${edges}`);
  console.log(`named by at least one other skill: ${called.length}`);
  console.log(`named by no other skill: ${list.length - called.length}`);
  console.log(`duplicated top-level and nested: ${list.filter(n => n.topLevel && n.nested).length}`);
  console.log('most-called:');
  for (const n of [...called].sort((a, b) => b.calledBy.length - a.calledBy.length).slice(0, 12)) {
    console.log(`  ${String(n.calledBy.length).padStart(3)}  ${n.name}`);
  }
}
