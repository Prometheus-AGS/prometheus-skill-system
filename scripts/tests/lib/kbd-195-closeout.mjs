// Closeout scenarios use the real signed runtime and generated installer surfaces.
import fs from 'node:fs';
import path from 'node:path';
import { World, sha256, writeFile } from './kbd-194-196-world.mjs';
import { selectedProject } from './kbd-194-196-sessions.mjs';
import { installed } from './kbd-194-196-package.mjs';
import { surfaceScenarios } from './kbd-195-hosts.mjs';

const orchestrator = root => path.join(root, 'skills/process/kbd-process-orchestrator');
const stages = ['assess', 'child-exit', 'execute', 'handoff', 'init', 'new-child', 'next-child', 'pause', 'plan', 'resume', 'spec'];
const section = text => text.match(/^## Selected-method recovery and authority\n[\s\S]*?(?=^## |$(?![\s\S]))/m)?.[0];
const scenario = (id, body) => [id, async env => {
  const w = new World(id.toLowerCase(), env);
  try { return await body(w, env); } finally { w.cleanup(); }
}];

function method(w, env, ctx, extra = []) {
  w.freezeFixture(`method-${w.log.length}`, ctx.project);
  const r = w.run(process.execPath, [path.join(orchestrator(env.ROOT), 'shared/method-context.mjs'), '--project', ctx.project,
    '--phase', ctx.phase, '--json', '--require-resolved', ...extra], { cwd: ctx.project, env: ctx.env });
  let json; try { json = JSON.parse(r.stdout); } catch { /* assertion reports malformed output */ }
  return { ...r, json };
}

function acknowledge(w, env, ctx, conflict, { id = 'ack', supersedes, hash = conflict.sha256 } = {}) {
  const payload = { phaseId: ctx.phase, path: conflict.path, sha256: hash, issue: conflict.issue,
    note: 'Synthetic fixture operator explicitly keeps the stricter whole-phase rule; no host rule is overridden.' };
  return w.run(env.prometheus, ['kbd', '--path', ctx.project, 'decision', 'record', '--command-id', id, '--id', id,
    '--summary', `KBD-POLICY-ACK ${JSON.stringify(payload)}`, ...(supersedes ? ['--supersedes', supersedes] : [])], { cwd: ctx.project, env: ctx.env });
}

const con001 = scenario('CON-001', async (w, env) => {
  const { ctx } = await selectedProject(w, env, { implementationComplete: false });
  const conflict = method(w, env, ctx);
  w.ok('unacknowledged governing policy produces CONFLICT and exit 2', conflict.exit === 2 && conflict.json?.status === 'CONFLICT'
    && conflict.json.policyConflicts.some(c => c.path === 'CLAUDE.md' && c.acknowledged === false), conflict.json?.policyConflicts);
  const ref = path.join(ctx.phaseDir, 'adopted-contract.json');
  const original = fs.readFileSync(ref);
  const changed = JSON.parse(original); changed.functionalScope.push('changed without adoption'); writeFile(ref, changed);
  const blocked = method(w, env, ctx);
  w.ok('missing/stale authority takes precedence over a conflict', blocked.exit === 2 && blocked.json?.status === 'BLOCKED', blocked.json?.blockers);
  fs.writeFileSync(ref, original);
  const rec = w.run(env.prometheus, ['kbd', '--path', ctx.project, 'decision', 'record', '--command-id', 'other-method', '--id', 'other-method',
    '--supersedes', 'method-vlbv', '--summary', `KBD-METHOD ${JSON.stringify({ phaseId: ctx.phase, contractDigest: conflict.json.contract.contractDigest, method: 'OTHER', retiredMethods: [], references: [] })}`], { cwd: ctx.project, env: ctx.env });
  const other = method(w, env, ctx);
  w.ok('non-VLBV negative control does not invent a VLBV policy conflict', rec.exit === 0 && other.exit === 0
    && other.json?.status === 'RESOLVED_REFERENCES' && other.json.policyConflicts.length === 0, other.json?.status);
  return w.result('resolve conflicting, stale and non-VLBV method contexts through the production helper');
});

const con002 = scenario('CON-002', async (w, env) => {
  const { ctx } = await selectedProject(w, env, { implementationComplete: false });
  const conflict = method(w, env, ctx).json.policyConflicts[0];
  const stale = acknowledge(w, env, ctx, conflict, { id: 'stale-ack', hash: '0'.repeat(64) });
  w.ok('wrong-hash acknowledgement does not clear the conflict', stale.exit === 0 && method(w, env, ctx).json?.status === 'CONFLICT', stale.stderr);
  const good = acknowledge(w, env, ctx, conflict, { id: 'current-ack', supersedes: 'stale-ack' });
  const resolved = method(w, env, ctx);
  w.ok('one current matching acknowledgement resolves references', good.exit === 0 && resolved.exit === 0
    && resolved.json.policyConflicts[0]?.ackDecisionId === 'current-ack' && resolved.json.policyConflicts[0]?.acknowledged === true, resolved.json?.policyConflicts);
  const policy = path.join(ctx.project, conflict.path), original = fs.readFileSync(policy);
  fs.appendFileSync(policy, '\nChanged policy bytes.\n');
  w.ok('changed policy re-raises the conflict', method(w, env, ctx).json?.status === 'CONFLICT', sha256(fs.readFileSync(policy)));
  fs.writeFileSync(policy, original);
  const ambiguous = acknowledge(w, env, ctx, conflict, { id: 'second-current-ack' });
  w.ok('ambiguous current acknowledgements cannot grant readiness', ambiguous.exit === 0 && method(w, env, ctx).json?.status === 'CONFLICT', ambiguous.stderr);
  return w.result('record and supersede hash-bound acknowledgements; reject changed-policy and ambiguous authority');
});

const con003 = scenario('CON-003', async (w, env) => {
  const { ctx } = await selectedProject(w, env, { implementationComplete: false });
  const unresolved = method(w, env, ctx, ['--write']);
  const policy = unresolved.json.policyConflicts[0];
  const anchor = () => w.run('/bin/bash', [path.join(env.ROOT, 'skills/process/prometheus-context-bootstrap/assets/hooks/reanchor.sh')],
    { cwd: ctx.project, env: { ...ctx.env, CLAUDE_PROJECT_DIR: ctx.project } });
  const before = anchor();
  w.ok('compaction recovery names the blocking policy conflict', before.exit === 0 && /BLOCKING policy conflict: CLAUDE.md/.test(before.stdout), before.stdout);
  const handoff = w.run('/bin/bash', [path.join(orchestrator(env.ROOT), 'skills/kbd-handoff/kbd-handoff.sh'), '--to', 'codex', '--dry-run'],
    { cwd: ctx.project, env: { ...ctx.env, KBD_ORCHESTRATOR_ROOT: orchestrator(env.ROOT) } });
  w.ok('actual handoff includes the conflict and acknowledgement status', handoff.exit === 0 && /policyConflicts/.test(handoff.stdout)
    && /"acknowledged": false/.test(handoff.stdout) && /BLOCKING policy conflict/.test(handoff.stderr), handoff.stdout.slice(-1200) + handoff.stderr);
  acknowledge(w, env, ctx, policy);
  method(w, env, ctx, ['--write']);
  const after = anchor();
  w.ok('recovered resolved index no longer prints a blocking conflict', after.exit === 0 && !/BLOCKING policy conflict/.test(after.stdout)
    && /RESOLVED_REFERENCES/.test(after.stdout), after.stdout);
  return w.result('recover conflicts and their resolution through the real reanchor and handoff entry points');
});

const ref001 = scenario('REF-001', async (w, env) => {
  const reference = fs.readFileSync(path.join(orchestrator(env.ROOT), 'references/method-context.md'), 'utf8');
  const expected = reference.match(/<!-- selected-method-core:start -->\n([\s\S]*?)<!-- selected-method-core:end -->/)?.[1];
  w.ok('single canonical core is present', typeof expected === 'string' && expected.length > 100, expected?.length);
  const homes = [];
  try {
    const surfaces = [{ label: 'source', root: orchestrator(env.ROOT) }];
    for (const platform of ['claude', 'codex']) {
      surfaces.push({ label: `generated ${platform}`, root: path.join(env.ROOT, 'dist/plugins', platform, 'prometheus-skill-pack/skills/kbd-process-orchestrator') });
      const inst = installed(env.ROOT, platform, w); homes.push(inst.home);
      surfaces.push({ label: `installed ${platform}`, root: path.join(inst.root, 'skills/kbd-process-orchestrator') });
    }
    for (const target of surfaces) for (const stage of stages) {
      const file = path.join(target.root, 'skills', `kbd-${stage}`, 'SKILL.md');
      w.ok(`${target.label} kbd-${stage}: identical canonical core`, fs.existsSync(file) && section(fs.readFileSync(file, 'utf8')) === expected, file);
    }
    const scratch = path.join(w.base, 'core-candidate');
    const copy = relative => {
      const destination = path.join(scratch, relative);
      fs.mkdirSync(path.dirname(destination), { recursive: true });
      fs.copyFileSync(path.join(env.ROOT, relative), destination);
    };
    copy('scripts/check-selected-method-core.mjs');
    copy('skills/process/kbd-process-orchestrator/references/method-context.md');
    for (const stage of stages) copy(`skills/process/kbd-process-orchestrator/skills/kbd-${stage}/SKILL.md`);
    w.freezeFixture('checker-positive', scratch);
    const good = w.run(process.execPath, [path.join(scratch, 'scripts/check-selected-method-core.mjs')]);
    w.ok('the actual checker accepts the unchanged scratch core', good.exit === 0, good.stdout + good.stderr);
    const changed = path.join(scratch, 'skills/process/kbd-process-orchestrator/skills/kbd-assess/SKILL.md');
    fs.writeFileSync(changed, fs.readFileSync(changed, 'utf8').replace('continue with the recovered method', 'Continue with the recovered method'));
    w.freezeFixture('checker-one-character-drift', scratch);
    const drift = w.run(process.execPath, [path.join(scratch, 'scripts/check-selected-method-core.mjs')]);
    w.ok('one changed character makes the real checker reject and name the stage', drift.exit !== 0 && /kbd-assess/.test(drift.stderr), drift.stdout + drift.stderr);
    return w.result('compare all 11 cores across source, generated and installer-dispatched surfaces');
  } finally { for (const home of homes) fs.rmSync(home, { recursive: true, force: true }); }
});

export const closeoutScenarios = [con001, con002, con003, ref001, ...surfaceScenarios];
