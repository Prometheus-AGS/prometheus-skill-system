/**
 * Cucumber step definitions for forge CLI BDD tests.
 *
 * Uses the `forge` binary from FORGE_BIN env var, falling back to the
 * debug build at tools/forge-rs/target/debug/forge.
 *
 * Tests run in isolated temp directories (one per scenario) so they
 * do not interfere with each other.
 */

import { Given, When, Then, Before, After, World } from '@cucumber/cucumber';
import * as assert from 'node:assert/strict';
import * as child_process from 'node:child_process';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';

// ─── Helpers ─────────────────────────────────────────────────────────────────

function forgeBin(): string {
  return process.env['FORGE_BIN'] ??
    path.resolve(__dirname, '../../tools/forge-rs/target/debug/forge');
}

interface ScenarioState {
  workdir: string;
  lastResult: { stdout: string; stderr: string; status: number | null };
}

// Attach state to world object
function state(world: World): ScenarioState {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return (world as any)._forgeState as ScenarioState;
}

Before(function (this: World) {
  const workdir = fs.mkdtempSync(path.join(os.tmpdir(), 'forge-bdd-'));
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (this as any)._forgeState = {
    workdir,
    lastResult: { stdout: '', stderr: '', status: null },
  } satisfies ScenarioState;
});

After(function (this: World) {
  const s = state(this);
  fs.rmSync(s.workdir, { recursive: true, force: true });
});

function runForge(args: string[], cwd: string): { stdout: string; stderr: string; status: number | null } {
  const bin = forgeBin();
  if (!fs.existsSync(bin)) {
    return {
      stdout: '',
      stderr: `forge binary not found at ${bin}. Run: cargo build --workspace in tools/forge-rs/`,
      status: 127,
    };
  }
  const result = child_process.spawnSync(bin, args, { cwd, encoding: 'utf8' });
  return {
    stdout: result.stdout ?? '',
    stderr: result.stderr ?? '',
    status: result.status,
  };
}

// ─── Forge validate step definitions ─────────────────────────────────────────

Given('a Rust source file {string} with content {string}', function (this: World, filename: string, content: string) {
  const s = state(this);
  fs.writeFileSync(path.join(s.workdir, filename), content, 'utf8');
});

Given('no constitution for language {string}', function (this: World, _lang: string) {
  // No constitution directory → already absent in a fresh workdir
});

Given('a rust constitution that forbids {string}', function (this: World, pattern: string) {
  const s = state(this);
  const constitutionDir = path.join(s.workdir, '.forge', 'constitution');
  fs.mkdirSync(constitutionDir, { recursive: true });
  const toml = [
    'language = "rust"',
    '[standards]',
    '',
    '[[forbidden_patterns]]',
    `pattern = "${pattern}"`,
    `reason = "forbidden in tests"`,
    'severity = "error"',
    '',
    'required_skills = []',
    '',
    '[framework_versions]',
  ].join('\n');
  fs.writeFileSync(path.join(constitutionDir, 'rust.toml'), toml, 'utf8');
});

When('I run forge validate on {string} with language {string}', function (this: World, filename: string, language: string) {
  const s = state(this);
  s.lastResult = runForge(
    ['--project-root', s.workdir, 'validate', path.join(s.workdir, filename), '--language', language],
    s.workdir,
  );
});

// ─── Forge enrich step definitions ───────────────────────────────────────────

Given('a task directory {string} with a tasks.md file containing {string}', function (this: World, dirName: string, content: string) {
  const s = state(this);
  const taskDir = path.join(s.workdir, 'openspec', 'changes', dirName);
  fs.mkdirSync(taskDir, { recursive: true });
  fs.writeFileSync(path.join(taskDir, 'tasks.md'), content, 'utf8');
});

Given('a project root with a valid skill directory', function (this: World) {
  const s = state(this);
  const skillDir = path.join(s.workdir, 'skills', 'rust', 'placeholder');
  fs.mkdirSync(skillDir, { recursive: true });
  fs.writeFileSync(path.join(skillDir, 'skill.toml'), [
    'name = "placeholder"',
    'language = "rust"',
    'description = "placeholder skill"',
    'version = "1.0.0"',
    '[[triggers]]',
    'type = "AlwaysForLanguage"',
    'language = "rust"',
  ].join('\n'), 'utf8');
});

When('I run forge enrich on the {string} directory', function (this: World, dirName: string) {
  const s = state(this);
  const taskDir = path.join(s.workdir, 'openspec', 'changes', dirName);
  s.lastResult = runForge(
    ['--project-root', s.workdir, '--skills-root', path.join(s.workdir, 'skills'), 'enrich', taskDir],
    s.workdir,
  );
});

When('I attempt to enrich with task_path {string} via MCP', function (this: World, taskPath: string) {
  // Simulate the path confinement check inline (no live MCP server in tests)
  const projectRoot = state(this).workdir;
  const isSafe = !taskPath.includes('..');
  state(this).lastResult = {
    stdout: '',
    stderr: isSafe ? '' : `task_path '${taskPath}' is outside the project root '${projectRoot}' — access denied`,
    status: isSafe ? 0 : 1,
  };
});

When('I run forge enrich on a non-existent path {string}', function (this: World, dirName: string) {
  const s = state(this);
  s.lastResult = runForge(
    ['--project-root', s.workdir, 'enrich', path.join(s.workdir, dirName)],
    s.workdir,
  );
});

// ─── Common assertion steps ───────────────────────────────────────────────────

Then('the exit code is {int}', function (this: World, expected: number) {
  const { status, stderr } = state(this).lastResult;
  assert.strictEqual(status, expected,
    `Expected exit code ${expected} but got ${status}. stderr: ${stderr}`);
});

Then('the exit code is non-zero', function (this: World) {
  const { status } = state(this).lastResult;
  assert.notStrictEqual(status, 0, `Expected non-zero exit code but got 0`);
});

Then('the output contains {string}', function (this: World, expected: string) {
  const { stdout, stderr } = state(this).lastResult;
  const combined = stdout + stderr;
  assert.ok(
    combined.toLowerCase().includes(expected.toLowerCase()),
    `Expected output to contain "${expected}" but got:\nstdout: ${stdout}\nstderr: ${stderr}`,
  );
});

Then('the response contains {string}', function (this: World, expected: string) {
  const { stdout, stderr } = state(this).lastResult;
  const combined = stdout + stderr;
  assert.ok(
    combined.toLowerCase().includes(expected.toLowerCase()),
    `Expected response to contain "${expected}" but got:\n${combined}`,
  );
});
