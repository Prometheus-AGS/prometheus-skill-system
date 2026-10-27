// PKG-001: finite source / generated / scratch-installed parity through production entry points.
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { World, PHASE, sha256, writeFile } from './kbd-194-196-world.mjs';
import { fixture, step, surface } from './kbd-phase-closure-adapter.mjs';

const sh = (f) => sha256(fs.readFileSync(f));

/** Install a generated payload into a scratch home with the real installer dispatch. */
function installed(ROOT, platform, w) {
  const payload = path.join(ROOT, 'dist/plugins', platform, 'prometheus-skill-pack');
  const home = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), `kbd194-install-${platform}-`)));
  const store = path.join(home, '.prometheus/plugins/prometheus-skill-pack');
  const env = { PATH: process.env.PATH, HOME: home, USERPROFILE: home, CODEX_HOME: path.join(home, '.codex'), CLAUDE_CONFIG_DIR: path.join(home, '.claude'),
    XDG_CONFIG_HOME: path.join(home, '.config'), XDG_DATA_HOME: path.join(home, '.local/share'), PROMETHEUS_PLUGIN_ROOT: store, PYTHONDONTWRITEBYTECODE: '1' };
  const r = w.run(process.execPath, [path.join(payload, 'scripts/install-plugin-generation.js'), '--source-root', payload, '--home', home, '--plugin-root', store, '--targets', 'codex'], { env, timeout: 600_000 });
  if (r.exit !== 0) throw new Error(`installer for ${platform} failed:\n${r.stdout}\n${r.stderr}`);
  return { home, root: fs.realpathSync(path.join(store, 'current')) };
}

const orchestrator = (root, source) => path.join(root, source ? 'skills/process/kbd-process-orchestrator' : 'skills/kbd-process-orchestrator');

async function pkg001({ prometheus, seeder, ROOT }) {
  const w = new World('pkg001', { prometheus, ROOT });
  const surfaces = [
    { label: 'source', root: ROOT, source: true },
    { label: 'generated claude', root: path.join(ROOT, 'dist/plugins/claude/prometheus-skill-pack') },
    { label: 'generated codex', root: path.join(ROOT, 'dist/plugins/codex/prometheus-skill-pack') },
  ];
  const homes = [];
  for (const platform of ['claude', 'codex']) {
    const inst = installed(ROOT, platform, w);
    homes.push(inst.home);
    surfaces.push({ label: `installed ${platform}`, root: inst.root });
  }

  // Generator drift gate and two byte-identical outputs were taken at generation; re-check now.
  const gen = w.run('node', ['scripts/generate-skill-system-distribution.js', '--check'], { cwd: ROOT, timeout: 600_000 });
  w.ok('generator --check reports no drift', gen.exit === 0, (gen.stdout + gen.stderr).slice(-300));

  // One bound contract and one recorded method selection, observed through every surface.
  const scratch = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'kbd194-pkg-')));
  const ctx = fixture(scratch, 'pkg', prometheus);
  ctx.surface = surface('source', ROOT, { sourceLayout: true });
  const adopt = await step(ctx, { op: 'adopt', args: {} });
  w.ok('contract adoption through the production helper succeeds', adopt.exit === 0, (adopt.stdout + adopt.stderr).slice(-300));
  const kbd = (args) => w.run(prometheus, ['kbd', '--path', ctx.project, ...args], { cwd: ctx.project, env: ctx.env });
  const digest = JSON.parse(kbd(['phase', 'disposition', '--id', ctx.phase, '--json']).stdout).binding?.contractDigest;
  const reference = path.join(ctx.phaseDir, 'calibration.md');
  writeFile(reference, 'owner-reviewed calibration v1\n');
  const selection = { phaseId: ctx.phase, contractDigest: digest, method: 'VLBV', retiredMethods: [],
    references: [{ path: path.relative(ctx.project, reference), sha256: sh(reference) }] };
  const rec = kbd(['decision', 'record', '--command-id', 'pkg-method', '--id', 'method-vlbv', '--summary', `KBD-METHOD ${JSON.stringify(selection)}`]);
  w.ok('method selection is recorded as a signed decision', rec.exit === 0, (rec.stdout + rec.stderr).slice(-200));

  const normalized = {};
  for (const s of surfaces) {
    const mc = path.join(orchestrator(s.root, s.source), 'shared/method-context.mjs');
    const exists = fs.existsSync(mc);
    w.ok(`${s.label}: method-context entry point is present`, exists, mc.replace(os.homedir(), '~'));
    if (!exists) continue;
    const run = (extra = []) => w.run('node', [mc, '--project', ctx.project, '--phase', ctx.phase, '--json', ...extra], { cwd: ctx.project, env: ctx.env });
    const ok = run(['--require-resolved']);
    let doc = null; try { doc = JSON.parse(ok.stdout); } catch { /* reported below */ }
    w.ok(`${s.label}: selection resolves to the bound contract and VLBV`, ok.exit === 0 && doc?.selectedMethod === 'VLBV' && doc?.contract?.contractDigest === digest, (ok.stdout + ok.stderr).slice(0, 300));
    if (doc) { const { generatedAt, ...rest } = doc; normalized[s.label] = JSON.stringify(rest); }
    // Hooks reach the helper through symlinked skill directories; --write must still write.
    const link = path.join(scratch, `link-${s.label.replace(/\s+/g, '-')}`);
    fs.symlinkSync(orchestrator(s.root, s.source), link);
    const written = path.join(ctx.phaseDir, 'method-context.json');
    fs.rmSync(written, { force: true });
    const viaLink = w.run('node', [path.join(link, 'shared/method-context.mjs'), '--project', ctx.project, '--phase', ctx.phase, '--write'], { cwd: ctx.project, env: ctx.env });
    w.ok(`${s.label}: --write through a symlinked skill directory writes the index`, viaLink.exit === 0 && fs.existsSync(written), (viaLink.stdout + viaLink.stderr).slice(0, 200));
    fs.rmSync(written, { force: true });
    fs.writeFileSync(reference, 'calibration CHANGED after selection\n');
    const changed = run(['--require-resolved']);
    let changedDoc = null; try { changedDoc = JSON.parse(changed.stdout); } catch { /* reported below */ }
    w.ok(`${s.label}: a changed reference is reported, not guessed`, changed.exit === 2 && changedDoc?.status === 'BLOCKED'
      && changedDoc.references.some((r) => r.matches === false && r.sha256 !== r.expectedSha256), `exit ${changed.exit} ${changed.stderr.slice(0, 200)}`);
    fs.writeFileSync(reference, 'owner-reviewed calibration v1\n');
  }
  const forms = Object.values(normalized);
  w.ok('all surfaces report byte-identical method context', forms.length === surfaces.length && forms.every((f) => f === forms[0]), Object.keys(normalized));

  // Identical diagnostics for the same damaged canonical state on every surface.
  const dmg = new World('pkg001-dmg', { prometheus, ROOT }).damaged(seeder, 'equal');
  writeFile(path.join(dmg.project, '.kbd-orchestrator/phases', PHASE, 'progress.json'), { phase: PHASE, changes: [{ id: 'target', tasks_done: 0, tasks_total: 1 }] });
  const diagnostics = {};
  for (const s of surfaces) {
    const orch = orchestrator(s.root, s.source);
    const apply = path.join(orch, 'skills/kbd-apply/kbd-apply.sh');
    if (!fs.existsSync(apply)) { w.ok(`${s.label}: kbd-apply entry point is present`, false, apply); continue; }
    const r = dmg.run('bash', [apply, 'reconcile', PHASE, '--json'], { env: { ...dmg.env, KBD_ORCHESTRATOR_ROOT: orch } });
    try { diagnostics[s.label] = JSON.stringify(JSON.parse(r.stdout).errors.map((e) => e.message)); } catch { diagnostics[s.label] = r.stdout + r.stderr; }
  }
  const diag = Object.values(diagnostics);
  w.ok('all surfaces name the same duplicate canonical identity', diag.length === surfaces.length && diag.every((d) => d === diag[0]) && /duplicate canonical change ID/.test(diag[0]), diag[0]);

  // Installed payload bytes equal the generated payload bytes for the touched entry points (actual installer dispatch).
  for (const [platform, generatedIdx, installedIdx] of [['claude', 1, 3], ['codex', 2, 4]]) {
    for (const rel of ['shared/method-context.mjs', 'skills/kbd-apply/kbd-apply.sh', 'skills/kbd-apply/reconcile.mjs', 'skills/kbd-apply/reconcile-command.mjs', 'skills/kbd-handoff/kbd-handoff.sh', 'hooks/hooks.json']) {
      const a = path.join(orchestrator(surfaces[generatedIdx].root, false), rel), b = path.join(orchestrator(surfaces[installedIdx].root, false), rel);
      w.ok(`installed ${platform} ${rel} equals the generated bytes`, fs.existsSync(a) && fs.existsSync(b) && sh(a) === sh(b), { generated: fs.existsSync(a), installed: fs.existsSync(b) });
    }
  }
  const outcome = w.result('run the generator drift gate, real installer dispatch for both payloads, and compare method-context and reconcile behavior across source, generated and installed surfaces', {
    artifacts: { 'surfaces.json': `${JSON.stringify(surfaces.map((s) => ({ label: s.label, root: s.root.replace(os.homedir(), '~') })), null, 2)}\n` },
    limitations: ['Only local macOS source, generated and scratch-installed Claude/Codex payloads; no real user install was read or changed.',
      'Payload equality and identical diagnostics establish payload fidelity, not session obedience; native-session observation is covered separately and is BLOCKED.'] });
  outcome.commands = [...w.log, ...dmg.log, ...ctx.log ?? []];
  fs.rmSync(scratch, { recursive: true, force: true });
  for (const h of homes) fs.rmSync(h, { recursive: true, force: true });
  w.cleanup(); dmg.cleanup();
  return outcome;
}

export const packageScenarios = [['PKG-001', pkg001]];
