import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { readSkillSystem } from '../lib/skill-system.js';

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), '../..');
const canonical = JSON.parse(fs.readFileSync(path.join(root, 'skill-system.json'), 'utf8'));

function fixture(mutator = () => {}) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'skill-system-lifecycle-'));
  const contract = structuredClone(canonical);
  for (const target of contract.targets.filter(entry => entry.sourceTreeLifecycle === 'required')) {
    const tree = path.join(directory, target.path);
    fs.mkdirSync(tree, { recursive: true });
    fs.writeFileSync(path.join(tree, 'SKILL.md'), '---\nname: fixture\n---\n');
  }
  mutator({ directory, contract });
  fs.writeFileSync(path.join(directory, 'skill-system.json'), `${JSON.stringify(contract, null, 2)}\n`);
  return directory;
}

function snapshot(directory) {
  const visit = relative => fs.readdirSync(path.join(directory, relative)).sort().flatMap(name => {
    const child = path.join(relative, name);
    const stat = fs.lstatSync(path.join(directory, child));
    return stat.isDirectory()
      ? [`d:${child}`, ...visit(child)]
      : [`f:${child}:${fs.readFileSync(path.join(directory, child), 'utf8')}`];
  });
  return visit('');
}

const valid = fixture();
assert.doesNotThrow(() => readSkillSystem(valid), 'absent install-only trees must be accepted');
const before = snapshot(valid);
readSkillSystem(valid);
readSkillSystem(valid);
assert.deepEqual(snapshot(valid), before, 'validation must be idempotent');

const omitted = fixture(({ contract }) => delete contract.targets[0].sourceTreeLifecycle);
assert.throws(
  () => readSkillSystem(omitted),
  /target claude must declare sourceTreeLifecycle/,
  'omitted lifecycle policy must fail and name the target'
);

const missing = fixture(({ directory, contract }) => {
  const target = contract.targets.find(entry => entry.sourceTreeLifecycle === 'required');
  fs.rmSync(path.join(directory, target.path), { recursive: true });
});
assert.throws(
  () => readSkillSystem(missing),
  /required target source tree is missing: opencode/,
  'missing required tree must fail and name the target'
);

const empty = fixture(({ directory, contract }) => {
  const target = contract.targets.find(entry => entry.sourceTreeLifecycle === 'required');
  fs.rmSync(path.join(directory, target.path), { recursive: true });
  fs.mkdirSync(path.join(directory, target.path), { recursive: true });
});
assert.throws(
  () => readSkillSystem(empty),
  /required target source tree is empty: opencode/,
  'empty required tree must fail and name the target'
);

console.log('PASS: required/install-only source-tree lifecycle policy and idempotency');
