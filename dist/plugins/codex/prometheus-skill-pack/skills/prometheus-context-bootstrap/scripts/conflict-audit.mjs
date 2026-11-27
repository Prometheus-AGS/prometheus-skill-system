#!/usr/bin/env node
// Read-only audit of agent rule files for guidance that contradicts the VLBV
// boundary rule (generated code runs once, at the value-phase boundary, and
// again only for an owner-authorized repair). It also reports whether the
// installed pack generation verifies.
//
// It never writes, never changes a rule file and never blocks: exit 0 with or
// without findings, 1 only for a usage error. Only recognized patterns are
// checked; operator prose still needs a human read.
import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, readFileSync, readdirSync, realpathSync, statSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const USAGE = `usage: bootstrap.sh --audit-conflicts [--path <dir>] [--json] [--harness <id>] [--no-worktrees]

Lists rule-file guidance that contradicts the VLBV boundary rule across the
root and nested AGENTS.md / CLAUDE.md, their @imports, .claude/rules and the
other Git worktrees of the repository, and reports the installed generation.
Read-only. Exit 0 with or without findings; 1 on a usage error.`;

const ROOT_RULE_FILES = ['AGENTS.md', 'CLAUDE.md', 'CLAUDE.local.md', 'AGENTS.override.md', '.kbd-orchestrator/constraints.md'];
const NESTED_NAMES = new Set(['AGENTS.md', 'CLAUDE.md']);
const SKIP_DIRS = new Set(['.git', 'node_modules', 'target', 'dist', 'build', 'out', 'vendor', '.venv', 'venv',
  '__pycache__', '.next', '.cache', 'coverage', '.dart_tool', 'Pods', 'DerivedData']);
const MAX_DEPTH = 6;
const MAX_FILES = 400;
const MAX_BYTES = 512 * 1024;
const MAX_IMPORT_DEPTH = 5;
const VERIFY_TIMEOUT_MS = 60_000;

// A line that forbids or retires a practice is not a conflict.
const NEGATED = /\b(do not|don't|never|no longer|must not|should not|not allowed|forbidden|forbid|retired|superseded|instead of|rather than|without running|is not|are not|not run|nothing runs)\b/i;
const EXECUTES = /\b(test|tests|build|compile|compiler|lint|type-?check|tsc|cargo|pytest|vitest|jest|check)\b/i;

// Each rule returns true when a line states the practice as something to do.
const LINE_RULES = [
  { issue: 'tier-ladder',
    why: 'checks on every edit or when a unit completes run generated code before the boundary',
    test: line => /\bTier\s*[01]\b.{0,80}(every edit|unit complete|seconds)/i.test(line)
      || /\bT[01]\s*:\s*`?[a-z]/.test(line) && EXECUTES.test(line) },
  { issue: 'per-edit-checks',
    why: 'a test, build or check after each edit, task or change runs generated code before the boundary',
    test: line => /\bafter (each|every) (edit|change|file|task|commit|iteration|save)\b/i.test(line) && EXECUTES.test(line)
      || /\b(cheap checks?|feedback) .{0,40}run continuously\b/i.test(line) },
  { issue: 'multiple-checkpoints',
    why: 'several test/build checkpoints in one phase contradict the single boundary run',
    test: line => /\b(maximum|max) of 3 times\b|\b3 times (per|during an entire) epoch\b|\bearly, mid,? (and )?end\b/i.test(line) },
  { issue: 'test-first',
    why: 'test-first cycles execute code against an intentionally incomplete implementation',
    test: line => /\b(TDD|test[- ]first|red[- /]green|write (the |a )?(failing )?tests? first|tests? before (the )?(code|implementation))\b/i.test(line) },
  { issue: 'compiler-exception',
    why: 'an intermediate compiler check during generation is not allowed under VLBV',
    test: line => /compiler.{0,100}(exception|unblock)|(?:narrow|targeted|intermediate|package-scoped).{0,60}compiler (?:check|run)/i.test(line) },
  { issue: 'cadence-exception',
    why: 'time-based verification creates a boundary the method does not have',
    test: line => /(?:hourly|this hour).{0,160}(?:integration gate|delivery boundary)|(?:hourly|every hour).{0,100}(?:test|verif)/i.test(line) },
];

function parseArgs(argv) {
  const options = { path: process.cwd(), json: false, harness: process.env.PROMETHEUS_HARNESS ?? null, worktrees: true };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--audit-conflicts') continue;
    if (arg === '--json') options.json = true;
    else if (arg === '--no-worktrees') options.worktrees = false;
    else if (arg === '--path' || arg === '--harness') {
      const value = argv[index + 1];
      if (!value || value.startsWith('--')) return { error: `${arg} needs a value` };
      options[arg.slice(2)] = value; index += 1;
    } else if (arg.startsWith('--path=')) options.path = arg.slice(7);
    else if (arg.startsWith('--harness=')) options.harness = arg.slice(10);
    else if (arg === '-h' || arg === '--help') return { help: true };
    else return { error: `unknown argument: ${arg}` };
  }
  return { options };
}

const git = (cwd, args) => {
  try {
    return execFileSync('git', ['-C', cwd, ...args], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'], timeout: 15_000 }).trim();
  } catch { return null; }
};

const within = (root, target) => target === root || target.startsWith(root + path.sep);

function readText(file) {
  try {
    const stat = statSync(file);
    if (!stat.isFile() || stat.size > MAX_BYTES) return null;
    return readFileSync(file, 'utf8');
  } catch { return null; }
}

function walk(root, visit) {
  let seen = 0;
  const descend = (directory, depth) => {
    if (depth > MAX_DEPTH || seen >= MAX_FILES) return;
    let entries;
    try { entries = readdirSync(directory, { withFileTypes: true }); } catch { return; }
    for (const entry of entries) {
      if (seen >= MAX_FILES) return;
      const full = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        if (SKIP_DIRS.has(entry.name) || entry.isSymbolicLink()) continue;
        // A nested checkout (submodule or another worktree) has its own .git and is audited on its own.
        if (depth > 0 && existsSync(path.join(full, '.git'))) continue;
        descend(full, depth + 1);
      } else if (entry.isFile()) {
        seen += 1;
        visit(full, depth);
      }
    }
  };
  descend(root, 0);
}

/** `@path` import lines outside code fences, as Claude Code and Codex resolve them. */
function importsOf(text) {
  const imports = [];
  let fenced = false;
  for (const raw of text.split(/\r?\n/)) {
    if (/^\s*(```|~~~)/.test(raw)) { fenced = !fenced; continue; }
    if (fenced) continue;
    const match = raw.match(/^\s*@([^\s@`]+)\s*$/) ?? raw.match(/(?:^|\s)@((?:\.{1,2}\/|~\/|\/)[^\s`]+\.md)\b/);
    if (match) imports.push(match[1]);
  }
  return imports;
}

function scanText(text) {
  const findings = [];
  const lines = text.split(/\r?\n/);
  let inComment = false;
  lines.forEach((raw, index) => {
    let line = raw;
    if (inComment) {
      const end = line.indexOf('-->');
      if (end === -1) return;
      line = line.slice(end + 3); inComment = false;
    }
    line = line.replace(/<!--[\s\S]*?-->/g, '');
    const open = line.indexOf('<!--');
    if (open !== -1) { line = line.slice(0, open); inComment = true; }
    if (!line.trim()) return;

    // Workflow trigger that runs a build or test when an iteration or change completes.
    if (/event:\s*on_(iteration|change)_complete\b/.test(line)) {
      const window = lines.slice(index + 1, index + 6).join('\n');
      if (/(target|command|run)\s*:.{0,160}(test_command|build_health_command|lint_command|cargo |npm |pnpm |yarn |pytest|go test|tsc\b|make )/i.test(window)) {
        findings.push({ line: index + 1, issue: 'per-change-trigger',
          why: 'a trigger that builds or tests when a change completes runs generated code before the boundary',
          excerpt: line.trim().slice(0, 160) });
      }
      return;
    }
    if (NEGATED.test(line)) return;
    for (const rule of LINE_RULES) {
      if (rule.test(line)) {
        findings.push({ line: index + 1, issue: rule.issue, why: rule.why, excerpt: line.trim().slice(0, 160) });
        break;
      }
    }
  });
  return findings;
}

/** Every rule file a harness could load from one checkout, with how it is reached. */
function collectRuleFiles(root) {
  const files = new Map(); // absolute path -> loadedBy
  const skipped = [];
  const add = (file, loadedBy) => { if (!files.has(file)) files.set(file, loadedBy); };

  for (const name of ROOT_RULE_FILES) {
    const file = path.join(root, name);
    if (existsSync(file)) add(file, 'root');
  }
  walk(root, (file, depth) => {
    const relative = path.relative(root, file);
    const name = path.basename(file);
    if (relative.startsWith(path.join('.claude', 'rules') + path.sep) && name.endsWith('.md')) add(file, 'rules-directory');
    else if (NESTED_NAMES.has(name) && depth > 0) add(file, 'nested');
  });

  const followImports = (file, depth) => {
    if (depth > MAX_IMPORT_DEPTH) return;
    const text = readText(file);
    if (text === null) return;
    for (const reference of importsOf(text)) {
      if (reference.startsWith('~') || path.isAbsolute(reference)) {
        skipped.push({ from: path.relative(root, file), import: reference, reason: 'outside the project; not read' });
        continue;
      }
      const target = path.resolve(path.dirname(file), reference);
      if (!within(root, target)) {
        skipped.push({ from: path.relative(root, file), import: reference, reason: 'outside the project; not read' });
        continue;
      }
      if (!existsSync(target)) {
        skipped.push({ from: path.relative(root, file), import: reference, reason: 'imported file is missing' });
        continue;
      }
      if (!files.has(target)) { add(target, 'import'); followImports(target, depth + 1); }
    }
  };
  for (const file of [...files.keys()]) followImports(file, 1);
  return { files, skipped };
}

async function effectiveSet(root, harness) {
  // Optional: the orchestrator's engine knows which files a given harness really loads.
  const here = path.dirname(fileURLToPath(import.meta.url));
  const candidates = [
    path.resolve(here, '../../kbd-process-orchestrator/shared/method-context.mjs'),
    path.resolve(here, '../../../kbd-process-orchestrator/shared/method-context.mjs'),
  ];
  for (const candidate of candidates) {
    if (!existsSync(candidate)) continue;
    try {
      const engine = await import(pathToFileURL(candidate).href);
      if (typeof engine.effectiveRuleFiles !== 'function') continue;
      const result = engine.effectiveRuleFiles(root, harness ?? undefined);
      return { harness: result.harness, profile: result.profile, files: result.files, shadowing: result.shadowing };
    } catch { /* fall through: the audit works without it */ }
  }
  return { harness: harness ?? null, profile: 'unavailable', files: null, shadowing: null };
}

function auditCheckout(root, label) {
  const { files, skipped } = collectRuleFiles(root);
  const findings = [];
  for (const [file, loadedBy] of files) {
    const text = readText(file);
    if (text === null) { skipped.push({ from: path.relative(root, file), import: null, reason: 'unreadable or larger than 512 KiB' }); continue; }
    for (const finding of scanText(text)) {
      findings.push({ worktree: label, file: path.relative(root, file), loadedBy, ...finding });
    }
  }
  return { root, label, filesScanned: files.size, findings, skipped: skipped.map(entry => ({ worktree: label, ...entry })) };
}

function otherWorktrees(root) {
  const listing = git(root, ['worktree', 'list', '--porcelain']);
  if (!listing) return [];
  const worktrees = [];
  for (const block of listing.split(/\n\n+/)) {
    const location = (block.match(/^worktree (.+)$/m) ?? [])[1];
    if (!location) continue;
    const branch = (block.match(/^branch refs\/heads\/(.+)$/m) ?? [])[1] ?? (/^detached$/m.test(block) ? 'detached' : 'unknown');
    let resolved;
    try { resolved = realpathSync(location); } catch { worktrees.push({ path: location, branch, missing: true }); continue; }
    if (resolved !== root) worktrees.push({ path: resolved, branch, missing: false });
  }
  return worktrees;
}

function verifyGeneration() {
  const store = process.env.PROMETHEUS_PLUGIN_ROOT
    || path.join(os.homedir(), '.prometheus', 'plugins', 'prometheus-skill-pack');
  const current = path.join(store, 'current');
  if (!existsSync(current)) {
    return { status: 'not-verified', store, generation: null, reason: 'no active generation at this store (the pack installer has not run here)' };
  }
  const installer = path.join(current, 'scripts', 'install-plugin-generation.js');
  if (!existsSync(installer)) {
    return { status: 'not-verified', store, generation: null, reason: 'the active generation has no verifier script' };
  }
  const result = spawnSync(process.execPath,
    [installer, '--verify', '--source-root', current, '--plugin-root', store, '--home', os.homedir()],
    { encoding: 'utf8', timeout: VERIFY_TIMEOUT_MS, stdio: ['ignore', 'pipe', 'pipe'] });
  const generation = (result.stdout ?? '').trim().split(/\s+/).find(token => /^[0-9a-f]{64}$/.test(token)) ?? null;
  if (result.status === 0 && generation) return { status: 'verified', store, generation, reason: null };
  const reason = result.error?.code === 'ETIMEDOUT'
    ? 'verification did not finish in 60 seconds'
    : ((result.stderr ?? '').trim().split('\n')[0] || `verifier exited ${result.status}`);
  return { status: 'not-verified', store, generation, reason: reason.slice(0, 300) };
}

async function main() {
  const parsed = parseArgs(process.argv.slice(2));
  if (parsed.help) { console.log(USAGE); return 0; }
  if (parsed.error) { console.error(`USAGE ${parsed.error}\n${USAGE}`); return 1; }
  const { options } = parsed;

  let start;
  try { start = realpathSync(options.path); } catch { console.error(`USAGE path does not exist: ${options.path}`); return 1; }
  if (!statSync(start).isDirectory()) { console.error(`USAGE path is not a directory: ${options.path}`); return 1; }
  const top = git(start, ['rev-parse', '--show-toplevel']);
  const root = top ? realpathSync(top) : start;

  const checkouts = [auditCheckout(root, '.')];
  const worktrees = options.worktrees ? otherWorktrees(root) : [];
  for (const worktree of worktrees) {
    if (!worktree.missing) checkouts.push(auditCheckout(worktree.path, worktree.path));
  }

  const effective = await effectiveSet(root, options.harness);
  const findings = checkouts.flatMap(checkout => checkout.findings).map(finding => ({
    ...finding,
    effectiveForHarness: finding.worktree === '.' && effective.files ? effective.files.includes(finding.file) : null,
  }));
  const report = {
    schemaVersion: 1,
    root,
    readOnly: true,
    filesScanned: checkouts.reduce((total, checkout) => total + checkout.filesScanned, 0),
    worktrees: [{ path: root, branch: git(root, ['rev-parse', '--abbrev-ref', 'HEAD']) ?? null, current: true },
      ...worktrees.map(worktree => ({ ...worktree, current: false }))],
    effectiveRules: effective,
    findings,
    skipped: checkouts.flatMap(checkout => checkout.skipped),
    generation: verifyGeneration(),
    advisory: 'Only recognized patterns are checked. Review operator prose for other conflicts. Nothing was changed.',
  };

  if (options.json) { console.log(JSON.stringify(report, null, 2)); return 0; }
  for (const finding of report.findings) {
    const where = finding.worktree === '.' ? finding.file : `${finding.worktree}/${finding.file}`;
    console.log(`CONFLICT ${where}:${finding.line} ${finding.issue} (${finding.loadedBy}) — ${finding.excerpt}`);
    console.log(`  why: ${finding.why}`);
  }
  if (!report.findings.length) console.log(`CLEAN no recognized conflict in ${report.filesScanned} rule file(s)`);
  else console.log(`FOUND ${report.findings.length} conflict(s) in ${report.filesScanned} rule file(s)`);
  for (const entry of report.skipped) {
    console.log(`SKIPPED ${entry.from}${entry.import ? ` → ${entry.import}` : ''}: ${entry.reason}`);
  }
  for (const worktree of report.worktrees.filter(entry => entry.missing)) console.log(`SKIPPED worktree ${worktree.path}: directory is missing`);
  if (effective.shadowing) console.log(`NOTE ${effective.shadowing}`);
  const generation = report.generation;
  console.log(generation.status === 'verified'
    ? `GENERATION verified ${generation.generation}`
    : `GENERATION not verified: ${generation.reason}`);
  console.log(report.advisory);
  return 0;
}

main().then(code => { process.exitCode = code; }, error => {
  // An unexpected failure must not look like a clean audit.
  console.error(`ERROR conflict audit could not complete: ${error?.message ?? error}`);
  process.exitCode = 1;
});
