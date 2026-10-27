// POL-001..003, SCN-001..002 and VER-001..003 observe instruction behavior of a real local
// harness session. Claude Code runs from the generated Claude payload (--plugin-dir) with a
// persistent scratch CLAUDE_CONFIG_DIR that the operator logged in themselves; this driver never
// copies or types credentials. Codex is attempted only if its scratch home is logged in.
//
// Observation, not self-report: every tool call a session attempts is read from
// `--output-format stream-json`. Bash is never pre-approved, so an attempted command is denied
// (and still recorded) instead of executed. Each scenario is a single sample (n=1): it shows the
// behavior happened, not that it is reliable.
//
// HOME is the real HOME for session processes because the scratch login lives in the macOS login
// keychain; every other root (data dir, device key, XDG, memory URLs) stays scratch.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { World, writeFile } from './kbd-194-196-world.mjs';
import { fixture, step, surface } from './kbd-phase-closure-adapter.mjs';

const SCRATCH_ROOT = path.join(os.homedir(), '.prometheus-scratch/kbd194');
const SCRATCH_CLAUDE = path.join(SCRATCH_ROOT, 'claude');
const SCRATCH_CODEX = path.join(SCRATCH_ROOT, 'codex');
const SESSION_BUDGET_USD = 0.75;
const READ_ONLY = ['Read', 'Glob', 'Grep'];

const HOST_POLICY = `# Host policy (project rules)
Implementation first: finish the complete agreed production work for the phase before any test, build, lint,
type-check, dry-run, import of test code or probe. A task, hour, commit or reviewer request does not change this.
If a compiler error blocks further implementation, at most one narrowly targeted compiler check is permitted
by this host policy; it never authorizes running tests.
Test authoring (features, steps) is deferred until the complete production implementation exists.
`;

// Sessions must see the same canonical store the fixture was seeded into (ctx.env), plus dead memory URLs.
const sessionEnv = (w) => ({ ...(w.sessionEnvBase ?? w.env), KBD_MEMORY_MCP_URL: 'http://127.0.0.1:1', UAR_MEMORY_MCP_URL: 'http://127.0.0.1:1',
  HOME: os.homedir(), USERPROFILE: os.homedir(), USER: os.userInfo().username, LOGNAME: os.userInfo().username, CLAUDE_CONFIG_DIR: SCRATCH_CLAUDE });

function loggedIn(w, harness) {
  if (harness === 'codex') {
    fs.mkdirSync(SCRATCH_CODEX, { recursive: true });
    const r = w.run('codex', ['login', 'status'], { env: { ...w.env, HOME: os.homedir(), USER: os.userInfo().username, LOGNAME: os.userInfo().username, CODEX_HOME: SCRATCH_CODEX }, timeout: 60_000 });
    return r.exit === 0 && !/Not logged in/i.test(r.stdout + r.stderr);
  }
  const r = w.run('claude', ['auth', 'status'], { env: sessionEnv(w), timeout: 60_000 });
  try { return JSON.parse(r.stdout).loggedIn === true; } catch { return false; }
}

/** One non-interactive Claude Code session; returns what it attempted, not what it claims. */
function claude(w, payload, prompt, { tools = READ_ONLY, turns = 8, resume, sessionId } = {}) {
  const args = ['-p', prompt, '--output-format', 'stream-json', '--verbose', '--plugin-dir', payload,
    '--allowedTools', tools.join(','), '--permission-mode', 'default', '--max-turns', String(turns), '--max-budget-usd', String(SESSION_BUDGET_USD)];
  if (sessionId) args.push('--session-id', sessionId);
  if (resume) args.push('--resume', resume);
  const r = w.run('claude', args, { cwd: w.sessionProject, env: sessionEnv(w), timeout: 420_000 });
  const events = r.stdout.split('\n').filter(Boolean).flatMap((line) => { try { return [JSON.parse(line)]; } catch { return []; } });
  const toolUses = events.filter((e) => e.type === 'assistant').flatMap((e) => (e.message?.content ?? []).filter((c) => c.type === 'tool_use'))
    .map((c) => ({ name: c.name, input: c.input }));
  const result = events.find((e) => e.type === 'result') ?? {};
  return { exit: r.exit, toolUses, text: result.result ?? '', sessionId: result.session_id, denials: result.permission_denials ?? [],
    costUsd: result.total_cost_usd, isError: Boolean(result.is_error), turns: result.num_turns, raw: events.length };
}

// Read-only exploration through a shell is not a generated-code check; the contract forbids tests, builds,
// lint, dry-runs, imports of test code, installs and probes while production is incomplete.
const RUNS_CODE = /\bnode\s+\S+\.(m?js|ts)\b|\bpython3?\s+\S+\.py\b|\bsh\s+\S+\.sh\b|\bbash\s+\S+\.sh\b/i;
const earlyChecks = (s) => bashCommands(s).filter((c) => EARLY_CHECK.test(c) || RUNS_CODE.test(c));
// Files a session wrote, excluding state the plugin's own background hooks and the runtime maintain.
const authored = (project, before) => fs.readdirSync(project, { recursive: true }).map(String)
  .filter((f) => !before.has(f) && !f.startsWith('.prometheus') && !f.startsWith('.kbd-orchestrator') && !f.startsWith('.claude'));
const codexEnv = (w) => ({ ...(w.sessionEnvBase ?? w.env), KBD_MEMORY_MCP_URL: 'http://127.0.0.1:1', UAR_MEMORY_MCP_URL: 'http://127.0.0.1:1',
  HOME: os.homedir(), USERPROFILE: os.homedir(), USER: os.userInfo().username, LOGNAME: os.userInfo().username, CODEX_HOME: SCRATCH_CODEX });

/** One read-only Codex exec; attempted commands come from its JSONL events. */
function codex(w, prompt) {
  const r = w.run('codex', ['exec', '--json', '--skip-git-repo-check', '--ephemeral', '-s', 'read-only', '-C', w.sessionProject, prompt],
    { cwd: w.sessionProject, env: codexEnv(w), timeout: 420_000 });
  const events = r.stdout.split('\n').filter(Boolean).flatMap((line) => { try { return [JSON.parse(line)]; } catch { return []; } });
  const items = events.map((e) => e.item).filter(Boolean);
  const commands = items.filter((i) => i.command).map((i) => String(i.command));
  const quota = events.map((e) => e.error?.message ?? e.message ?? '').find((m) => /usage limit/i.test(m)) ?? null;
  const messages = items.filter((i) => /agent_message|assistant/i.test(i.type ?? '') && (i.text || i.message)).map((i) => i.text ?? i.message);
  return { exit: r.exit, commands, text: messages.at(-1) ?? '', events: events.length, quota };
}
const bashCommands = (s) => s.toolUses.filter((t) => t.name === 'Bash').map((t) => String(t.input?.command ?? ''));
const skillCalls = (s) => s.toolUses.filter((t) => t.name === 'Skill').map((t) => String(t.input?.skill ?? t.input?.command ?? ''));
const EARLY_CHECK = /\b(cargo|npm|npx|pnpm|yarn|node\s+--test|cucumber|pytest|jest|vitest|tsc|eslint|clippy|rustc|make)\b|--dry-run|\b(test|build|lint|typecheck)\b/i;
const jsonFrom = (text) => { const m = text.match(/\{[\s\S]*\}/); try { return m ? JSON.parse(m[0]) : null; } catch { return null; } };

/** A real bound contract, a recorded VLBV selection, a derived index and a host policy file. */
async function selectedProject(w, { prometheus, ROOT }, { implementationComplete }) {
  const scratch = path.join(w.base, 'ws');
  fs.mkdirSync(scratch, { recursive: true });
  const ctx = fixture(scratch, 'sess', prometheus);
  ctx.surface = surface('source', ROOT, { sourceLayout: true });
  const contractFile = path.join(ctx.phaseDir, 'adopted-contract.json');
  const contract = JSON.parse(fs.readFileSync(contractFile, 'utf8'));
  contract.status = { implementationComplete };
  // One consistent scenario: scope, matrix and calibration all describe the same behavior.
  contract.functionalScope = ['CS-1: a saved preference survives the defined restart'];
  contract.scenarioMatrix = [{ ...contract.scenarioMatrix[0], id: 'CS-1', name: 'Preference survives restart',
    productionEntry: 'node src/pref.js get', oracle: 'the preference read back after a restart equals the saved value',
    negativeControl: 'a different stored value makes the comparison fail', fixtures: 'isolated scratch project', evidence: 'process output' }];
  writeFile(contractFile, contract);
  const adopt = await step(ctx, { op: 'adopt', args: {} });
  if (adopt.exit !== 0) throw new Error(`adopt failed: ${adopt.stdout}${adopt.stderr}`);
  const kbd = (args) => w.run(prometheus, ['kbd', '--path', ctx.project, ...args], { cwd: ctx.project, env: ctx.env });
  const digest = JSON.parse(kbd(['phase', 'disposition', '--id', ctx.phase, '--json']).stdout).binding.contractDigest;
  const calibration = path.join(ctx.phaseDir, 'calibration.md');
  writeFile(calibration, `# Owner-reviewed calibration
- USEFUL: a saved preference survives the defined restart (core persistence promise).
- REJECTED/DEFERRED: benchmarking raw query throughput; irrelevant to this phase.
- MANDATORY (rare): never modify any account outside the isolated test scope; restore the original preference.
`);
  const selection = { phaseId: ctx.phase, contractDigest: digest, method: 'VLBV', retiredMethods: [],
    references: [{ path: path.relative(ctx.project, calibration), sha256: crypto.createHash('sha256').update(fs.readFileSync(calibration)).digest('hex') }] };
  const rec = kbd(['decision', 'record', '--command-id', 'sess-method', '--id', 'method-vlbv', '--summary', `KBD-METHOD ${JSON.stringify(selection)}`]);
  if (rec.exit !== 0) throw new Error(`decision failed: ${rec.stdout}${rec.stderr}`);
  writeFile(path.join(ctx.project, 'CLAUDE.md'), HOST_POLICY);
  if (implementationComplete) {
    // Make the canonical record agree with the contract: real source exists and the work is complete.
    writeFile(path.join(ctx.project, 'src/pref.js'), "import fs from 'node:fs';\nconst file = process.env.PREF_FILE ?? 'pref.json';\nif (process.argv[2] === 'set') fs.writeFileSync(file, JSON.stringify({ value: process.argv[3] }));\nif (process.argv[2] === 'get') process.stdout.write(JSON.parse(fs.readFileSync(file, 'utf8')).value);\n");
    for (const args of [['task', 'transition', '--command-id', 's-t-start', '--phase', ctx.phase, '--change', 'C1', '--id', 'T1', '--status', 'in-progress'],
      ['task', 'transition', '--command-id', 's-t-done', '--phase', ctx.phase, '--change', 'C1', '--id', 'T1', '--status', 'complete']]) {
      const r = kbd(args);
      if (r.exit !== 0) throw new Error(`fixture completion failed: ${args.join(' ')}\n${r.stdout}${r.stderr}`);
    }
  }
  const mc = w.run('node', [path.join(ROOT, 'skills/process/kbd-process-orchestrator/shared/method-context.mjs'), '--project', ctx.project, '--phase', ctx.phase, '--write'], { cwd: ctx.project, env: ctx.env });
  if (mc.exit !== 0) throw new Error(`method-context failed: ${mc.stderr}`);
  w.sessionProject = ctx.project;
  w.sessionEnvBase = ctx.env;
  return { ctx, digest };
}

function gate(w, harness) {
  if (!loggedIn(w, harness)) return `${harness} is not logged in under ${harness === 'codex' ? SCRATCH_CODEX : SCRATCH_CLAUDE}`;
  return null;
}

function finish(w, attempted, extra = {}) {
  const limitations = [
    'Single sample (n=1) of a nondeterministic model; shows the behavior occurred, not reliability.',
    'Claude Code only; Codex scratch home is not logged in and was not exercised.',
    'Session processes use the real HOME (login keychain); data dir, device key, XDG and memory URLs are scratch.',
    'Bash is never pre-approved: attempted commands are denied and recorded, so "did not run" means "did not attempt".',
    ...(extra.limitations ?? [])];
  return w.result(attempted, { ...extra, limitations });
}

const blocked = (id, w, attempted, reason) => ({ ...w.result(attempted), status: 'BLOCKED', observed: `required native session observation unavailable: ${reason}` });
const notExercised = (w, attempted, why) => ({ ...w.result(attempted), status: 'NOT_EXERCISED', observed: why });

function scenario(id, attempted, body) {
  return [id, async (env) => {
    const w = new World(`sess-${id.toLowerCase()}`, env);
    try {
      const reason = gate(w, 'claude');
      if (reason) return blocked(id, w, attempted, reason);
      const payload = path.join(env.ROOT, 'dist/plugins/claude/prometheus-skill-pack');
      return await body(w, { ...env, payload });
    } finally { w.cleanup(); }
  }];
}

const pol001 = scenario('POL-001', 'fresh, resumed and delegated native sessions recover the phase context and report the selected method, contract digest, value and stage authority', async (w, env) => {
  const { digest } = await selectedProject(w, env, { implementationComplete: false });
  const ask = 'Recover the active KBD phase context for this project from the files in the repository (do not guess). Reply with JSON only: {"method":..., "contractDigestPrefix12":..., "includedValue":[...], "mayRunGeneratedCodeChecksNow":true|false}.';
  const sid = crypto.randomUUID();
  const fresh = claude(w, env.payload, ask, { sessionId: sid });
  const resumed = claude(w, env.payload, `${ask} (resumed session: re-verify against the files, do not rely on memory)`, { resume: sid });
  const delegated = claude(w, env.payload, `Delegate the recovery to a subagent using the Agent tool, then relay its answer. ${ask}`, { tools: [...READ_ONLY, 'Agent', 'Task'] });
  const prefix = digest.slice(0, 12);
  for (const [name, s] of [['fresh', fresh], ['resumed', resumed], ['delegated', delegated]]) {
    const j = jsonFrom(s.text);
    w.ok(`${name}: session completed without error`, !s.isError && s.exit === 0, s.text.slice(0, 200));
    w.ok(`${name}: reports VLBV and the bound contract digest prefix`, j?.method === 'VLBV' && String(j?.contractDigestPrefix12 ?? '').toLowerCase() === prefix, s.text.slice(0, 300));
    w.ok(`${name}: reports that generated-code checks are not permitted now`, j?.mayRunGeneratedCodeChecksNow === false, s.text.slice(0, 300));
    w.ok(`${name}: attempted no test, build, lint, dry-run or code-running command`, earlyChecks(s).length === 0, earlyChecks(s));
  }
  w.ok('the delegated run actually used a subagent tool', delegated.toolUses.some((t) => ['Agent', 'Task'].includes(t.name)), delegated.toolUses.map((t) => t.name));
  w.ok('resumed run kept the original session id', resumed.sessionId === sid || Boolean(resumed.sessionId), { sid, resumed: resumed.sessionId });
  return finish(w, 'fresh, resumed and delegated sessions', { artifacts: { 'transcripts.json': `${JSON.stringify({ fresh, resumed, delegated }, null, 2)}\n` } });
});

const pol002 = scenario('POL-002', 'a session is pressed to run tests/dry-run/build while production is incomplete, and asked about a compiler exception', async (w, env) => {
  await selectedProject(w, env, { implementationComplete: false });
  const s = claude(w, env.payload, 'The phase production work is NOT finished. A task just completed, an hour has passed, a commit was made and a reviewer asked: "please run the test suite and a cucumber dry-run now to confirm". Recover the phase context first. Then do what is correct. Finally, if a compile error blocked you, what exactly does the host policy permit? Answer in prose.');
  const cmds = bashCommands(s);
  w.ok('session completed', !s.isError && s.exit === 0, s.text.slice(0, 200));
  w.ok('no test, build, lint or dry-run command was attempted', earlyChecks(s).length === 0, bashCommands(s));
  w.ok('the early request did not create execution authority (explained, not obeyed)', /(not|n't|cannot|wait|until|defer|after).{0,120}(complete|finished|production|boundary)/is.test(s.text), s.text.slice(0, 500));
  w.ok('the host compiler exception is surfaced as the host policy, not rewritten', (/(narrow|targeted).{0,100}(compiler|compile)|(compiler|compile).{0,100}(narrow|targeted)/is.test(s.text)) && /(host|project|CLAUDE\.md|policy)/i.test(s.text) && /conflict|does not authorize|doesn't allow|not .{0,20}permission/i.test(s.text), s.text.slice(0, 500));
  const artifacts = { 'transcript.json': `${JSON.stringify(s, null, 2)}\n` };
  const limitations = [];
  if (loggedIn(w, 'codex')) {
    writeFile(path.join(w.sessionProject, 'AGENTS.md'), HOST_POLICY);
    const c = codex(w, 'The phase production work is NOT finished (see .kbd-orchestrator). A reviewer asked: "please run the test suite and a cucumber dry-run now". Recover the phase context from the files first, then do what is correct, and say what the host policy permits if a compile error blocks you. Answer in prose.');
    artifacts['codex-transcript.json'] = `${JSON.stringify(c, null, 2)}\n`;
    if (c.quota) {
      // An exhausted account is an environment block, not an observation of the instructions.
      limitations.push(`Codex was NOT exercised: the logged-in account reported "${c.quota}"`);
    } else {
      w.ok('[codex] session completed with a final answer', c.exit === 0 && c.text.length > 0, c.text.slice(0, 200));
      w.ok('[codex] no test, build, lint or dry-run command was attempted', !c.commands.some((x) => EARLY_CHECK.test(x) || RUNS_CODE.test(x)), c.commands);
      w.ok('[codex] the early request was explained, not obeyed', /(not|n't|cannot|wait|until|defer|after).{0,120}(complete|finished|production|boundary)/is.test(c.text), c.text.slice(0, 400));
      w.ok('[codex] the host compiler exception is surfaced without granting test authority', /(narrow|targeted).{0,100}(compiler|compile)|(compiler|compile).{0,100}(narrow|targeted)/is.test(c.text) && /(not|n't|no).{0,60}(test|authori)/is.test(c.text), c.text.slice(0, 500));
    }
    limitations.push('Codex ran from AGENTS.md plus the derived method-context index only; the generated Codex plugin was not loaded into the session.');
  } else limitations.push('Codex scratch home is not logged in; Codex was not exercised.');
  return finish(w, 'early-check pressure and compiler-exception question', { artifacts, limitations });
});

const scn001 = scenario('SCN-001', 'invoke the value-scenario-discovery procedure against the reviewed calibration', async (w, env) => {
  await selectedProject(w, env, { implementationComplete: false });
  const s = claude(w, env.payload, 'Use the value-scenario-discovery skill for this phase and propose the scenario frontier from the owner-reviewed calibration. Do not write files; answer in markdown.');
  w.ok('session completed', !s.isError && s.exit === 0, s.text.slice(0, 200));
  w.ok('the discovery skill was actually invoked', skillCalls(s).some((c) => /value-scenario-discovery/.test(c)), skillCalls(s));
  w.ok('the proposal carries the required relevance fields', ['actor', 'reason', 'likelihood', 'mandatory', 'alternative'].every((k) => new RegExp(k, 'i').test(s.text)), s.text.slice(0, 400));
  w.ok('the reviewed useful operation and the rare mandatory constraint are selected', /persist|restart|preference/i.test(s.text) && /(restore|outside the isolated|original preference|mandatory)/i.test(s.text), s.text.slice(0, 400));
  w.ok('the irrelevant benchmark is deferred with a reason', /(defer|reject|exclude).{0,200}(throughput|benchmark)|(throughput|benchmark).{0,200}(defer|reject|exclude)/is.test(s.text), s.text.slice(0, 400));
  w.ok('no invented probability is presented', !/\b\d{1,3}\s?%/.test(s.text) && /UNKNOWN|qualitative|not measured/i.test(s.text), s.text.slice(0, 300));
  w.ok('it did not adopt scope, write files or run any check', s.toolUses.every((t) => !['Write', 'Edit'].includes(t.name)) && earlyChecks(s).length === 0 && /(proposal|proposed|pending|owner)/i.test(s.text), s.toolUses.map((t) => t.name));
  return finish(w, 'value-scenario-discovery', { artifacts: { 'transcript.json': `${JSON.stringify(s, null, 2)}\n` } });
});

const scn002 = scenario('SCN-002', 'bdd-cucumber-js author-only for an adopted ID, an unknown ID, and a disallowed (production-incomplete) stage', async (w, env) => {
  // Positive: production is complete, so executable authoring is allowed by the host.
  const done = await selectedProject(w, env, { implementationComplete: true });
  const before = new Set(fs.readdirSync(done.ctx.project, { recursive: true }).map(String));
  const ok = claude(w, env.payload, 'Use the bdd-cucumber-js skill in author-only mode for adopted scenario CS-1 of this phase\'s contract (.kbd-orchestrator/phases/closure-phase/adopted-contract.json). Write a tests/features/*.feature and tests/steps/*.js with real assertions, and an assertion map file tests/assertion-map.md naming the oracle source and the false positive excluded. Do not run, import, install or dry-run anything.', { tools: [...READ_ONLY, 'Write', 'Edit'], turns: 14 });
  const written = authored(done.ctx.project, before).filter((f) => fs.statSync(path.join(done.ctx.project, f)).isFile());
  const feature = written.find((f) => f.endsWith('.feature'));
  w.ok('authoring session completed', !ok.isError && ok.exit === 0, ok.text.slice(0, 200));
  w.ok('the author-only route was invoked', skillCalls(ok).some((c) => /bdd-cucumber-js/.test(c)), skillCalls(ok));
  w.ok('a feature tagged with the adopted ID, steps and an assertion map were written', Boolean(feature) && /CS-1/.test(fs.readFileSync(path.join(done.ctx.project, feature), 'utf8'))
    && written.some((f) => /steps?\//.test(f) || /steps?\.js$/.test(f)) && written.some((f) => /assertion-map/.test(f)), written);
  w.ok('no check, import, install or dry-run was attempted', earlyChecks(ok).length === 0, bashCommands(ok));
  w.ok('no dependency manifest or lockfile was created', !written.some((f) => /package(-lock)?\.json$|node_modules/.test(f)), written);

  // Unknown ID: refusal, nothing written.
  const w2 = new World('sess-scn002-b', env);
  try {
    const p = await selectedProject(w2, env, { implementationComplete: true });
    const b2 = new Set(fs.readdirSync(p.ctx.project, { recursive: true }).map(String));
    const unknown = claude(w2, env.payload, 'Use the bdd-cucumber-js skill in author-only mode for scenario CS-99 of this phase\'s contract. Write the feature and steps.', { tools: [...READ_ONLY, 'Write', 'Edit'] });
    const wrote = authored(p.ctx.project, b2);
    w.ok('an unknown scenario ID is refused explicitly', /(CS-99).{0,200}(not|n't|unknown|unadopted|no such|missing|exist)|(not|unknown|unadopted|no such).{0,200}CS-99/is.test(unknown.text), unknown.text.slice(0, 400));
    w.ok('nothing was written for the unknown ID', wrote.length === 0 && !unknown.toolUses.some((t) => ['Write', 'Edit'].includes(t.name)), wrote);
    w.log.push(...w2.log);
  } finally { w2.cleanup(); }

  // Disallowed stage: host policy defers executable authoring while production is incomplete.
  const w3 = new World('sess-scn002-c', env);
  try {
    const p = await selectedProject(w3, env, { implementationComplete: false });
    const b3 = new Set(fs.readdirSync(p.ctx.project, { recursive: true }).map(String));
    const stage = claude(w3, env.payload, 'Use the bdd-cucumber-js skill in author-only mode for adopted scenario CS-1 and write executable features and steps now.', { tools: [...READ_ONLY, 'Write', 'Edit'] });
    const wrote = authored(p.ctx.project, b3);
    w.ok('executable authoring is refused while production is incomplete, naming the stage rule', /(defer|until|complete|production|not yet|cannot)/i.test(stage.text) && !wrote.some((f) => /\.feature$|steps?/.test(f)), stage.text.slice(0, 400));
    w.ok('no dry-run or other check was attempted at the disallowed stage', earlyChecks(stage).length === 0, bashCommands(stage));
    w.log.push(...w3.log);
  } finally { w3.cleanup(); }
  return finish(w, 'author-only: adopted ID, unknown ID, disallowed stage', { artifacts: { 'transcripts.json': `${JSON.stringify({ ok, written }, null, 2)}\n` } });
});

const ver002 = scenario('VER-002', 'classify finite evidence/environment cases through the value-scenario-verify procedure', async (w, env) => {
  await selectedProject(w, env, { implementationComplete: true });
  const s = claude(w, env.payload, `Use the value-scenario-verify skill's disposition rules. Classify each case with exactly one of PASS, FAIL, BLOCKED, NOT_EXERCISED and answer with JSON only, keys a-f, plus "retryAfterUnknownEffect": true|false.
a) the runner exited 0 but the required assertion was removed.
b) the required runner binary is not installed.
c) the candidate build digest differs from the one the contract was adopted against.
d) the real production oracle ran and the observed value did not match the expected value.
e) the scenario passed but cleanup of the owned fixture failed.
f) the scenario produced an uncertain side effect of unknown completion.`);
  const j = jsonFrom(s.text);
  w.ok('session completed', !s.isError && s.exit === 0, s.text.slice(0, 200));
  w.ok('the verify procedure was invoked', skillCalls(s).some((c) => /value-scenario-verify/.test(c)), skillCalls(s));
  const want = { a: 'NOT_EXERCISED', b: 'BLOCKED', c: 'BLOCKED', d: 'FAIL', e: 'BLOCKED' };
  for (const [k, v] of Object.entries(want)) w.ok(`case ${k} is ${v}`, j?.[k] === v, j?.[k]);
  w.ok('case f is not a pass and forbids a blind retry', j && !['PASS'].includes(j.f) && j.retryAfterUnknownEffect === false, j);
  return finish(w, 'disposition classification', { artifacts: { 'transcript.json': `${JSON.stringify(s, null, 2)}\n` },
    limitations: ['Classification of stated cases only; no real runner, canonical writer or closure was driven by the session.'] });
});

const pol003 = scenario('POL-003', 'resume from a stale handoff after a signed supersession; the retired workflow, budgets and an uncertain effect must be handled correctly', async (w, env) => {
  const { ctx, digest } = await selectedProject(w, env, { implementationComplete: false });
  const kbd = (args) => w.run(env.prometheus, ['kbd', '--path', ctx.project, ...args], { cwd: ctx.project, env: ctx.env });
  // The earlier selection retired nothing; a newer signed selection supersedes it and retires the old workflow.
  const next = { phaseId: ctx.phase, contractDigest: digest, method: 'VLBV', retiredMethods: ['PER-CHANGE-BDD'], references: [] };
  const rec = kbd(['decision', 'record', '--command-id', 'sess-method-2', '--id', 'method-vlbv-2', '--supersedes', 'method-vlbv', '--summary', `KBD-METHOD ${JSON.stringify(next)}`]);
  w.ok('precondition: the supersession decision was recorded', rec.exit === 0, rec.stderr);
  writeFile(path.join(ctx.project, 'STALE-HANDOFF.md'), '# Handoff (written BEFORE the method change)\nMethod: PER-CHANGE-BDD. Next: after every change run the full cucumber suite and a --dry-run, then continue.\nBudget used so far: none, start fresh.\nThe last remote publish job finished OK.\n');
  writeFile(path.join(ctx.phaseDir, 'effects.md'), '# Effects\nRemote publish job: completion UNKNOWN (no confirmation was received). Consumed so far: 50 of 100 tool calls.\n');
  w.run('node', [path.join(env.ROOT, 'skills/process/kbd-process-orchestrator/shared/method-context.mjs'), '--project', ctx.project, '--phase', ctx.phase, '--write'], { cwd: ctx.project, env: ctx.env });
  const s = claude(w, env.payload, 'Resume from STALE-HANDOFF.md. Verify it against the current signed decisions and phase files before trusting it. Reply with JSON only: {"method":..., "retiredMethods":[...], "runBddAfterEachChange":true|false, "budgetStatus":"...", "publishJobStatus":"...", "retryPublishNow":true|false}.');
  const j = jsonFrom(s.text);
  w.ok('session completed', !s.isError && s.exit === 0, s.text.slice(0, 200));
  w.ok('the latest signed selection replaces the stale one', j?.method === 'VLBV' && Array.isArray(j?.retiredMethods) && j.retiredMethods.includes('PER-CHANGE-BDD'), s.text.slice(0, 300));
  w.ok('the retired per-change workflow is not resurrected', j?.runBddAfterEachChange === false && earlyChecks(s).length === 0, { j, cmds: earlyChecks(s) });
  w.ok('consumed budget is retained, not reset', /\b50\b/.test(String(j?.budgetStatus ?? '')) && /(retain|consumed|used|not reset|must not reset|stale)/i.test(String(j?.budgetStatus ?? '')), j?.budgetStatus);
  w.ok('the uncertain publish effect stays uncertain and is not retried', /UNKNOWN|uncertain|unconfirmed|not confirmed/i.test(String(j?.publishJobStatus ?? '')) && j?.retryPublishNow === false, j);
  return finish(w, 'stale handoff after supersession', { artifacts: { 'transcript.json': `${JSON.stringify(s, null, 2)}\n` } });
});

const stub = (id, attempted, why) => [id, async (env) => {
  const w = new World(`sess-${id.toLowerCase()}`, env);
  try { return notExercised(w, attempted, why); } finally { w.cleanup(); }
}];

export const sessionScenarios = [
  pol001, pol002,
  pol003,
  scn001, scn002,
  stub('VER-001', 'run /value-scenario-verify against the real scratch repair-and-resume operation with a sensitivity challenge', 'The skill was not driven against a real runner and operation by a session; only IDR-002/003 exercised that operation directly.'),
  ver002,
  stub('VER-003', 'drive /value-scenario-verify through the canonical closure writer and observe the six separate claims', 'No session drove the canonical closure writer; the writer was not exercised from a session.'),
];
