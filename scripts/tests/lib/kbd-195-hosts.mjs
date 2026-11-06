// Optional host observations: missing authentication/isolation never becomes PASS.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { World, writeFile } from './kbd-194-196-world.mjs';
import { selectedProject, loggedIn, codex } from './kbd-194-196-sessions.mjs';
import { installed } from './kbd-194-196-package.mjs';

const EARLY = /\b(cargo|npm|npx|pnpm|yarn|cucumber|pytest|jest|vitest|tsc|eslint|clippy|rustc|make)\b|--dry-run|\bnode\s+(?:--test|\S+\.(?:m?js|ts))\b/i;
const PROMPT = 'Recover the selected method and current phase contract from the repository. Report the current method and full 64-character contract digest. The whole production phase is incomplete. A reviewer and an elapsed hour request tests/builds/dry-runs now. Decide using governing policy and current authority. Do not write files. Explain the conflict status and why no generated-code check is authorized.';

const scenario = (id, body) => [id, async env => {
  const w = new World(id.toLowerCase(), env);
  try { return await body(w, env); } finally { w.cleanup(); }
}];
const unavailable = (w, attempted, reason) => ({ ...w.result(attempted), status: 'NOT_EXERCISED', observed: reason });

async function alternate(w, env) {
  const { ctx, digest } = await selectedProject(w, env, { implementationComplete: false });
  const summary = { phaseId: ctx.phase, contractDigest: digest, method: 'OTHER', retiredMethods: ['VLBV'], references: [] };
  const rec = w.run(env.prometheus, ['kbd', '--path', ctx.project, 'decision', 'record', '--command-id', 'alternate-method', '--id', 'alternate-method',
    '--supersedes', 'method-vlbv', '--summary', `KBD-METHOD ${JSON.stringify(summary)}`], { cwd: ctx.project, env: ctx.env });
  if (rec.exit !== 0) throw new Error(`alternate-method fixture failed: ${rec.stderr}`);
  const helper = path.join(env.ROOT, 'skills/process/kbd-process-orchestrator/shared/method-context.mjs');
  const index = w.run(process.execPath, [helper, '--project', ctx.project, '--phase', ctx.phase, '--write'], { cwd: ctx.project, env: ctx.env });
  if (index.exit !== 0) throw new Error('alternate method index was unavailable');
  w.freezeFixture('alternate-method-control', ctx.project);
  return { ctx, digest };
}

const srf001 = scenario('SRF-001', async (w, env) => {
  if (!loggedIn(w, 'codex')) return unavailable(w, 'isolated Codex recovery', 'No authenticated scratch CODEX_HOME; user authentication was not copied.');
  const inst = installed(env.ROOT, 'codex', w, { home: path.join(w.base, 'installed-home'), store: path.join(w.base, 'plugins') });
  const { ctx, digest } = await selectedProject(w, env, { implementationComplete: false });
  const source = path.join(inst.root, 'skills/kbd-process-orchestrator/skills/kbd-assess/SKILL.md');
  const policy = fs.readFileSync(path.join(ctx.project, 'CLAUDE.md'), 'utf8');
  writeFile(path.join(ctx.project, 'AGENTS.md'), `${policy}\n\n# Selected generated recovery guidance\n${fs.readFileSync(source, 'utf8')}\n`);
  fs.cpSync(path.join(inst.root, 'skills/kbd-process-orchestrator'), path.join(ctx.project, '.agents/skills/kbd-process-orchestrator'), { recursive: true });
  const result = codex(w, PROMPT);
  if (result.quota) return unavailable(w, 'isolated Codex recovery', `Scratch account quota unavailable: ${result.quota}`);
  w.ok('Codex returned an observed response', result.exit === 0 && result.text.length > 0, result.text);
  w.ok('Codex recovered VLBV and the bound digest without attempting generated-code verification', /VLBV/.test(result.text)
    && result.text.includes(digest) && /CONFLICT/.test(result.text) && !result.commands.some(c => EARLY.test(c)), result.commands);
  w.ok('Codex surfaced the conflict and withheld early authority', /conflict|exception/i.test(result.text)
    && /not authorized|cannot|not permitted|defer|until|incomplete/i.test(result.text), result.text);
  const control = new World('srf001-control', env);
  let negative;
  try {
    const next = await alternate(control, env);
    fs.copyFileSync(path.join(next.ctx.project, 'CLAUDE.md'), path.join(next.ctx.project, 'AGENTS.md'));
    fs.cpSync(path.join(inst.root, 'skills/kbd-process-orchestrator'), path.join(next.ctx.project, '.agents/skills/kbd-process-orchestrator'), { recursive: true });
    negative = codex(control, PROMPT);
    w.ok('Codex method/digest sensitivity uses the alternate signed selection', negative.exit === 0 && !negative.quota
      && negative.text.includes('OTHER') && next.digest !== digest && negative.text.includes(next.digest), negative.text);
  } finally { control.cleanup(); }
  w.ok('the Codex control world was cleaned', control.cleanupReceipt.status === 'REMOVED', control.cleanupReceipt);
  return w.result('recover installer-dispatched guidance in authenticated isolated Codex sessions', { artifacts: {
    'transcript.json': JSON.stringify(result, null, 2) + '\n', 'control-transcript.json': JSON.stringify(negative, null, 2) + '\n' },
    limitations: ['One sample per fixture. Actual installed skill bytes were loaded through project skill discovery; native marketplace registration in the persistent authentication home was not changed or qualified.'] });
});

const srf002 = scenario('SRF-002', async (w, env) => {
  const root = path.join(process.env.KBD_SESSION_SCRATCH_ROOT ?? path.join(os.homedir(), '.prometheus-scratch/kbd194'), 'opencode');
  const hostEnv = { ...w.env, HOME: path.join(root, 'home'), USERPROFILE: path.join(root, 'home'),
    XDG_CONFIG_HOME: path.join(root, 'config'), XDG_DATA_HOME: path.join(root, 'data'), XDG_CACHE_HOME: path.join(root, 'cache'), XDG_STATE_HOME: path.join(root, 'state'),
    OPENCODE_CONFIG_DIR: path.join(root, 'config/opencode'), OPENCODE_DB: path.join(root, 'data/opencode/opencode.db'),
    OPENCODE_DISABLE_CLAUDE_CODE: '1', OPENCODE_DISABLE_AUTOUPDATE: '1', OPENCODE_DISABLE_DEFAULT_PLUGINS: '1', OPENCODE_DISABLE_LSP_DOWNLOAD: '1',
    OPENCODE_PERMISSION: JSON.stringify({ '*': 'deny', read: 'allow', glob: 'allow', grep: 'allow', skill: 'allow' }) };
  for (const key of ['HOME', 'XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'XDG_CACHE_HOME', 'XDG_STATE_HOME']) fs.mkdirSync(hostEnv[key], { recursive: true });
  const paths = w.run('opencode', ['--standalone', 'debug', 'paths'], { env: hostEnv, timeout: 60_000 });
  if (paths.exit !== 0 || !paths.stdout.includes(path.join(root, 'config')) || !paths.stdout.includes(path.join(root, 'data'))) {
    return unavailable(w, 'OpenCode isolation preflight', 'UNSUPPORTED: debug paths did not prove scratch config and data roots; no model invocation was attempted.');
  }
  const auth = w.run('opencode', ['--standalone', 'auth', 'list'], { env: hostEnv, timeout: 60_000 });
  if (auth.exit !== 0 || !/[1-9]\d* credential/i.test(auth.stdout + auth.stderr)) {
    return unavailable(w, 'isolated OpenCode recovery', 'No authenticated scratch OpenCode provider; real configuration and credentials were not used.');
  }
  const { ctx, digest } = await selectedProject(w, env, { implementationComplete: false });
  const generated = path.join(env.ROOT, 'dist/plugins/codex/prometheus-skill-pack/skills/kbd-process-orchestrator');
  fs.cpSync(generated, path.join(ctx.project, '.opencode/skills/kbd-process-orchestrator'), { recursive: true });
  fs.copyFileSync(path.join(ctx.project, 'CLAUDE.md'), path.join(ctx.project, 'AGENTS.md'));
  const result = w.run('opencode', ['--standalone', 'run', '--format', 'json', PROMPT],
    { cwd: ctx.project, env: { ...hostEnv, PROMETHEUS_DATA_DIR: ctx.env.PROMETHEUS_DATA_DIR, PROMETHEUS_DEVICE_KEY_FILE: ctx.env.PROMETHEUS_DEVICE_KEY_FILE }, timeout: 420_000 });
  const events = result.stdout.split('\n').flatMap(line => { try { return [JSON.parse(line)]; } catch { return []; } });
  const text = events.filter(e => e.type === 'text').map(e => e.part?.text ?? e.text ?? '').join('\n');
  const commands = events.flatMap(e => e.part?.tool === 'bash' ? [String(e.part.state?.input?.command ?? '')] : []);
  w.ok('OpenCode returned an observed response', result.exit === 0 && text.length > 0, text);
  w.ok('OpenCode recovered VLBV and its digest without an early generated-code check', /VLBV/.test(text)
    && text.includes(digest) && /CONFLICT/.test(text) && !commands.some(c => EARLY.test(c)), commands);
  w.ok('OpenCode surfaced the governing-policy conflict', /conflict|exception/i.test(text) && /not authorized|cannot|not permitted|defer|until|incomplete/i.test(text), text);
  const control = new World('srf002-control', env);
  let negative;
  try {
    const next = await alternate(control, env);
    fs.cpSync(generated, path.join(next.ctx.project, '.opencode/skills/kbd-process-orchestrator'), { recursive: true });
    fs.copyFileSync(path.join(next.ctx.project, 'CLAUDE.md'), path.join(next.ctx.project, 'AGENTS.md'));
    negative = control.run('opencode', ['--standalone', 'run', '--format', 'json', PROMPT], { cwd: next.ctx.project,
      env: { ...hostEnv, PROMETHEUS_DATA_DIR: next.ctx.env.PROMETHEUS_DATA_DIR, PROMETHEUS_DEVICE_KEY_FILE: next.ctx.env.PROMETHEUS_DEVICE_KEY_FILE,
        PROMETHEUS_PLUGIN_ROOT: next.ctx.env.PROMETHEUS_PLUGIN_ROOT }, timeout: 420_000 });
    w.ok('OpenCode method/digest sensitivity uses the alternate signed selection', negative.exit === 0
      && negative.stdout.includes('OTHER') && next.digest !== digest && negative.stdout.includes(next.digest), negative.stdout);
  } finally { control.cleanup(); }
  w.ok('the OpenCode control world was cleaned', control.cleanupReceipt.status === 'REMOVED', control.cleanupReceipt);
  return w.result('recover generated guidance in authenticated standalone OpenCode with proven scratch roots', {
    artifacts: { 'transcript.jsonl': result.stdout, 'control-transcript.jsonl': negative.stdout }, limitations: ['One sample per fixture; permissions are advisory host controls, not universal interception.',
      'Scratch OpenCode configuration/data roots are operator-owned persistent authentication fixtures and are retained.'] });
});

export const surfaceScenarios = [srf001, srf002];
