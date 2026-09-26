import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { copyPathExact } from './support.mjs';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const invoke = (file, args = [], options = {}) => spawnSync(process.execPath, [file, ...args], { encoding: 'utf8', shell: false, ...options });
function fixture(t) { const directory = mkdtempSync(join(tmpdir(), 'Builder ü space-')); t.after(() => rmSync(directory, { recursive: true, force: true })); return directory; }

test('installed hooks consume real stdin, emit protocol JSON and tolerate malformed payloads', t => {
  const directory = fixture(t); mkdirSync(join(directory, '.claude/hooks'), { recursive: true }); mkdirSync(join(directory, '.knowme-builder'));
  for (const name of ['skill-activation', 'a11y-reminder']) copyPathExact(join(root, 'scripts', `${name}.mjs`), join(directory, '.claude/hooks', `${name}.mjs`));
  writeFileSync(join(directory, '.knowme-builder/activation-manifest.json'), JSON.stringify({ skills: [{ name: 'flutter-golden-ui', terms: ['flutter'] }] }));
  let result = invoke(join(directory, '.claude/hooks/skill-activation.mjs'), [], { cwd: directory, input: JSON.stringify({ prompt: 'Build FLUTTER' }) });
  assert.equal(result.status, 0, result.stderr); assert.match(JSON.parse(result.stdout).hookSpecificOutput.additionalContext, /flutter-golden-ui/);
  result = invoke(join(directory, '.claude/hooks/a11y-reminder.mjs'), [], { cwd: directory, input: JSON.stringify({ tool_input: { file_path: 'C:\\project\\src\\feature.tsx' } }) });
  assert.equal(JSON.parse(result.stdout).hookSpecificOutput.hookEventName, 'PostToolUse');
  for (const input of ['{', 'null', '', JSON.stringify({ prompt: 'unrelated' })]) {
    result = invoke(join(directory, '.claude/hooks/skill-activation.mjs'), [], { cwd: directory, input });
    assert.equal(result.status, 0); assert.equal(result.stdout, '');
  }
});

test('copied package installs twice preserving user configuration and rejects modified skills', t => {
  const directory = fixture(t), payload = join(directory, 'payload'), project = join(directory, 'consumer');
  mkdirSync(payload); mkdirSync(join(project, '.claude'), { recursive: true });
  for (const name of ['scripts', 'skills', 'templates']) copyPathExact(join(root, name), join(payload, name));
  for (const name of ['builder.manifest.json', 'plugin.json', 'marketplace.json', '.claude-plugin']) copyPathExact(join(root, name), join(payload, name));
  const verify = () => invoke(join(payload, 'scripts/verify-skill-manifest.mjs'), [payload]);
  let contract = verify(); assert.equal(contract.status, 0, contract.stderr);
  mkdirSync(join(payload, 'skills/undeclared'));
  contract = verify(); assert.equal(contract.status, 1); assert.match(contract.stderr, /Undeclared skill/);
  rmSync(join(payload, 'skills/undeclared'), { recursive: true });
  const user = { permissions: { allow: ['Read'] }, hooks: { UserPromptSubmit: [{ matcher: '*', hooks: [{ type: 'command', command: 'user-owned-command' }] }] } };
  writeFileSync(join(project, '.claude/settings.json'), JSON.stringify(user));
  const script = join(payload, 'scripts/add-project-skills.mjs');
  let result = invoke(script, [project], { cwd: directory }); assert.equal(result.status, 0, result.stderr);
  const settings = readFileSync(join(project, '.claude/settings.json'), 'utf8');
  result = invoke(script, [project], { cwd: directory }); assert.equal(result.status, 0, result.stderr); assert.equal(readFileSync(join(project, '.claude/settings.json'), 'utf8'), settings);
  const actual = JSON.parse(settings); assert.deepEqual(actual.permissions, user.permissions); assert.equal(actual.hooks.UserPromptSubmit[0].hooks[0].command, 'user-owned-command');
  assert.ok(existsSync(join(project, '.knowme-builder/activation-manifest.json')));
  writeFileSync(join(project, '.claude/skills/a11y-gate/SKILL.md'), 'user changed content');
  result = invoke(script, [project], { cwd: directory }); assert.equal(result.status, 1); assert.match(result.stderr, /refusing overwrite/);
  assert.equal(readFileSync(join(project, '.claude/skills/a11y-gate/SKILL.md'), 'utf8'), 'user changed content');
});

test('process layer launches real Node and resolves npm batch shims without shell interpolation', async t => {
  const directory = fixture(t), binary = join(directory, 'tool.cmd');
  mkdirSync(join(directory, 'node_modules/tool'), { recursive: true }); writeFileSync(binary, '@echo off');
  writeFileSync(join(directory, 'node_modules/tool/package.json'), JSON.stringify({ name: 'tool', bin: { tool: 'cli.mjs' } }));
  writeFileSync(join(directory, 'node_modules/tool/cli.mjs'), 'process.stdout.write(JSON.stringify(process.argv.slice(2)));process.exitCode=7;');
  const { run } = await import('../../scripts/portable/platform.mjs');
  const argumentsToPreserve = ['space name', 'ü', '$HOME', '`literal`', '&not-a-command'];
  const result = run(binary, argumentsToPreserve, { capture: true, allowFailure: true });
  assert.equal(result.status, 7); assert.deepEqual(JSON.parse(result.stdout), argumentsToPreserve);
  assert.throws(() => run(binary, argumentsToPreserve, { capture: true }), error => error.exitCode === 7);
});

test('wrapper invalid options and installer dry-run leave the destination untouched', t => {
  const directory = fixture(t), destination = join(directory, 'must-not-exist');
  let result = invoke(join(root, 'scripts/scaffold-hybrid.mjs'), [destination, '--legacy-ignored-option']);
  assert.equal(result.status, 1); assert.match(result.stderr, /not ignored/); assert.equal(existsSync(destination), false);
  result = invoke(join(root, 'scripts/install-harness-package.mjs'), ['--harness', 'opencode', '--scope', 'project', '--check'], { cwd: directory });
  assert.equal(result.status, 0, result.stderr); assert.match(result.stdout, /no host state changed/); assert.equal(existsSync(join(directory, '.knowme-builder')), false);
});
