// Issue #256: the read-only conflict audit, exercised through its shipped
// entry point (bootstrap.sh --audit-conflicts) against real Git checkouts.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, realpathSync, rmSync, statSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const BOOTSTRAP = path.join(REPO, 'skills/process/prometheus-context-bootstrap/scripts/bootstrap.sh');

function sandbox(t) {
  const base = realpathSync(mkdtempSync(path.join(os.tmpdir(), 'conflict-audit-')));
  t.after(() => rmSync(base, { recursive: true, force: true }));
  const home = path.join(base, 'home');
  const project = path.join(base, 'project');
  mkdirSync(home); mkdirSync(project);
  const env = { ...process.env, HOME: home, PROMETHEUS_PLUGIN_ROOT: path.join(home, '.prometheus/plugins/prometheus-skill-pack') };
  delete env.PROMETHEUS_HARNESS;
  const git = (cwd, ...args) => execFileSync('git', ['-C', cwd, '-c', 'user.name=Audit', '-c', 'user.email=audit@example.invalid', ...args],
    { encoding: 'utf8', env, stdio: ['ignore', 'pipe', 'pipe'] });
  git(project, 'init', '-q', '-b', 'main');
  const write = (root, relative, text) => {
    const file = path.join(root, relative);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, text);
  };
  const commit = message => { git(project, 'add', '-A'); git(project, 'commit', '-qm', message); };
  const audit = (...args) => spawnSync('bash', [BOOTSTRAP, '--audit-conflicts', '--path', project, ...args],
    { encoding: 'utf8', env, timeout: 120_000 });
  return { base, home, project, env, git, write, commit, audit };
}

/** Digest of every file under a directory, to prove the audit wrote nothing. */
function treeDigest(root) {
  const hash = createHash('sha256');
  const visit = directory => {
    for (const name of readdirSync(directory).sort()) {
      const full = path.join(directory, name);
      const stat = statSync(full);
      if (stat.isDirectory()) { hash.update(`d:${path.relative(root, full)}\n`); visit(full); }
      else hash.update(`f:${path.relative(root, full)}:${stat.size}:${createHash('sha256').update(readFileSync(full)).digest('hex')}\n`);
    }
  };
  visit(root);
  return hash.digest('hex');
}

const ALIGNED = `# Project instructions

Implement the whole agreed increment first. Do not run tests, builds or
compiler checks after each edit. Never use test-first or red/green loops.
The adopted boundary packet runs once at the value-phase boundary.
`;

test('an aligned project reports no conflict and is not modified', t => {
  const box = sandbox(t);
  box.write(box.project, 'AGENTS.md', ALIGNED);
  box.write(box.project, 'CLAUDE.md', '@AGENTS.md\n');
  box.commit('aligned rules');
  const before = treeDigest(box.project);

  const result = box.audit();
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /^CLEAN no recognized conflict in \d+ rule file\(s\)$/m);
  assert.doesNotMatch(result.stdout, /^CONFLICT /m);
  // No generation is installed in the scratch home: reported, never a failure.
  assert.match(result.stdout, /^GENERATION not verified: /m);
  assert.match(result.stdout, /Nothing was changed\./);
  assert.equal(treeDigest(box.project), before, 'the audit must not write to the project');
  assert.equal(box.git(box.project, 'status', '--porcelain').trim(), '');
});

test('conflicts in nested, imported and rules-directory files are listed with file and line', t => {
  const box = sandbox(t);
  box.write(box.project, 'AGENTS.md', ALIGNED);
  box.write(box.project, 'CLAUDE.md', '# Claude\n\n@docs/agent/testing.md\n@~/.claude/private-rules.md\n');
  box.write(box.project, 'docs/agent/testing.md', '# Testing\n\nUse TDD: write the failing test first, then the code.\n');
  box.write(box.project, 'services/api/AGENTS.md', '# API rules\n\nAlways run cargo check after every edit to catch errors early.\n');
  box.write(box.project, '.claude/rules/rust.md', '# Rust\n\n- T0: `cargo check -p <touched-crate>` on each save\n');
  box.write(box.project, '.kbd-orchestrator/constraints.md',
    '# Constraints\n\n```yaml\nworkflow_triggers:\n  - event: on_change_complete\n    action:\n      type: command\n      target: \'pnpm test\'\n```\n');
  // Ignored directories are not rule sources.
  box.write(box.project, 'node_modules/pkg/AGENTS.md', 'Run the tests after every change.\n');
  box.commit('conflicting rules');
  const before = treeDigest(box.project);

  const result = box.audit('--json');
  assert.equal(result.status, 0, result.stderr);
  const report = JSON.parse(result.stdout);
  assert.equal(report.readOnly, true);
  const byFile = Object.fromEntries(report.findings.map(finding => [finding.file, finding]));

  assert.equal(byFile['docs/agent/testing.md']?.issue, 'test-first');
  assert.equal(byFile['docs/agent/testing.md'].line, 3);
  assert.equal(byFile['docs/agent/testing.md'].loadedBy, 'import');
  assert.equal(byFile['services/api/AGENTS.md']?.issue, 'per-edit-checks');
  assert.equal(byFile['services/api/AGENTS.md'].line, 3);
  assert.equal(byFile['services/api/AGENTS.md'].loadedBy, 'nested');
  assert.equal(byFile['.claude/rules/rust.md']?.issue, 'tier-ladder');
  assert.equal(byFile['.claude/rules/rust.md'].loadedBy, 'rules-directory');
  assert.equal(byFile['.kbd-orchestrator/constraints.md']?.issue, 'per-change-trigger');
  assert.equal(byFile['.kbd-orchestrator/constraints.md'].line, 5);

  // The aligned root file, which only forbids these practices, is not reported.
  assert.equal(byFile['AGENTS.md'], undefined);
  assert.ok(!report.findings.some(finding => finding.file.startsWith('node_modules/')));
  // An import outside the project is named and not read.
  assert.ok(report.skipped.some(entry => entry.import === '~/.claude/private-rules.md' && /outside the project/.test(entry.reason)));
  assert.equal(report.generation.status, 'not-verified');
  assert.equal(treeDigest(box.project), before, 'the audit must not write to the project');

  const text = box.audit();
  assert.equal(text.status, 0);
  assert.match(text.stdout, /^CONFLICT services\/api\/AGENTS\.md:3 per-edit-checks \(nested\) — /m);
  assert.match(text.stdout, /^FOUND 4 conflict\(s\) in \d+ rule file\(s\)$/m);
});

test('a conflict in another worktree of the repository is listed, and --no-worktrees skips it', t => {
  const box = sandbox(t);
  box.write(box.project, 'AGENTS.md', ALIGNED);
  box.commit('aligned rules');
  const other = path.join(box.base, 'other-worktree');
  box.git(box.project, 'worktree', 'add', '-q', '-b', 'feature', other);
  box.write(other, 'AGENTS.md', '# Feature branch rules\n\nRun the unit tests after each change before moving on.\n');

  const result = box.audit('--json');
  assert.equal(result.status, 0, result.stderr);
  const report = JSON.parse(result.stdout);
  const resolvedOther = realpathSync(other);
  const finding = report.findings.find(entry => entry.worktree === resolvedOther);
  assert.ok(finding, 'the other worktree conflict must be reported');
  assert.equal(finding.file, 'AGENTS.md');
  assert.equal(finding.issue, 'per-edit-checks');
  assert.ok(report.worktrees.some(entry => entry.path === resolvedOther && entry.branch === 'feature'));
  assert.ok(!report.findings.some(entry => entry.worktree === '.'), 'the current checkout is aligned');

  const local = JSON.parse(box.audit('--json', '--no-worktrees').stdout);
  assert.equal(local.findings.length, 0);
});

test('a verified installed generation is reported, and a broken one is reported without failing', t => {
  const box = sandbox(t);
  box.write(box.project, 'AGENTS.md', ALIGNED);
  box.commit('aligned rules');
  const store = box.env.PROMETHEUS_PLUGIN_ROOT;
  const generation = 'a'.repeat(64);
  const verifier = path.join(store, 'current/scripts/install-plugin-generation.js');
  mkdirSync(path.dirname(verifier), { recursive: true });

  // Stands in for the installer's verifier contract: 64-hex id on success.
  writeFileSync(verifier, `if (!process.argv.includes('--verify')) process.exit(2);\nconsole.log('${generation}');\n`);
  const verified = JSON.parse(box.audit('--json').stdout);
  assert.equal(verified.generation.status, 'verified');
  assert.equal(verified.generation.generation, generation);

  writeFileSync(verifier, `console.error('install-plugin-generation: target marker mismatch'); process.exit(1);\n`);
  const broken = box.audit();
  assert.equal(broken.status, 0, 'a failed generation check is a report, not a failure');
  assert.match(broken.stdout, /^GENERATION not verified: install-plugin-generation: target marker mismatch$/m);
});

test('wrong arguments are a usage error and nothing else is', t => {
  const box = sandbox(t);
  const unknown = box.audit('--fix');
  assert.equal(unknown.status, 1);
  assert.match(unknown.stderr, /USAGE unknown argument: --fix/);
  const missing = spawnSync('bash', [BOOTSTRAP, '--audit-conflicts', '--path', path.join(box.base, 'absent')],
    { encoding: 'utf8', env: box.env });
  assert.equal(missing.status, 1);
  assert.match(missing.stderr, /USAGE path does not exist/);
});
