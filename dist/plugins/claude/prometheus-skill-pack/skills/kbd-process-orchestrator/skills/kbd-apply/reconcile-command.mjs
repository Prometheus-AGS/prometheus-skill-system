import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { runReconcile } from './reconcile.mjs';
import { readReconcileState } from './reconcile-state.mjs';
import { activePhase, complete, cancelled } from './reconcile-artifacts.mjs';

const directory = path.dirname(fileURLToPath(import.meta.url));
const driver = path.join(directory, 'kbd-apply.sh');
const nestedOrchestrator = path.resolve(directory, '../..');
const orchestrator = fs.existsSync(path.join(nestedOrchestrator, 'shared/openspec/cli.mjs'))
  ? nestedOrchestrator : path.resolve(directory, '../kbd-process-orchestrator');

const adapters = {
  readCanonical: root => readReconcileState(root).state,
  repairTask(root, phase, task) {
    const check = () => {
      const current = readReconcileState(root).state;
      if (current.activePath?.phaseId !== phase) throw new Error('active phase changed before repair');
      const change = current.phases?.[phase]?.changes?.[task.change];
      const status = change?.tasks?.[task.runtimeId]?.status;
      if (cancelled(status) || cancelled(change?.status) || cancelled(current.phases?.[phase]?.status)) throw new Error('task or phase was cancelled before repair');
      return status;
    };
    let status = check();
    if (complete(status)) return;
    const call = command => {
      const result = spawnSync('bash', [driver, command, task.change, task.id, String(task.sequence), String(task.total), task.title], {
        cwd: root, encoding: 'utf8', maxBuffer: 8 * 1024 * 1024,
        env: { ...process.env, KBD_ORCHESTRATOR_ROOT: process.env.KBD_ORCHESTRATOR_ROOT || orchestrator, KBD_RECONCILE_REPAIR: '1', KBD_RECONCILE_PHASE: phase },
      });
      if (result.stderr) process.stderr.write(result.stderr);
      if (result.error || result.status !== 0) throw new Error(`${command} failed for ${task.change}/${task.id}: ${result.error?.message || result.status}`);
    };
    if (!['in-progress', 'in_progress'].includes(status)) call('begin-task');
    status = check();
    if (!complete(status)) call('end-task');
  },
  repairCounts(context, rows) {
    const current = JSON.parse(fs.readFileSync(path.join(context.root, '.kbd-orchestrator/current-waypoint.json'), 'utf8'));
    if (current.generatedBy === 'kbd-runtime') throw new Error('project became runtime-owned before count repair');
    if (activePhase(current) !== context.active) throw new Error('active phase changed before count repair');
    const progress = JSON.parse(fs.readFileSync(context.progressFile, 'utf8'));
    for (const row of rows.filter(item => item.kind === 'count')) {
      const change = progress.changes.find(item => item.id === row.change);
      const tasks = context.artifacts[row.change].tasks;
      change.tasks_done = tasks.filter(task => task.done).length;
      change.tasks_total = tasks.length;
    }
    if (rows.some(row => row.kind === 'phase-count')) {
      for (const [done, total] of [['changes_completed', 'changes_total'], ['implementation_completed', 'implementation_total']]) {
        if (progress[done] !== undefined || progress[total] !== undefined) {
          progress[done] = context.phaseCounts.completed;
          progress[total] = context.phaseCounts.total;
        }
      }
      if (progress.completion?.implementation) Object.assign(progress.completion.implementation, context.phaseCounts);
    }
    fs.writeFileSync(context.progressFile, `${JSON.stringify(progress, null, 2)}\n`);
  },
};

process.exitCode = await runReconcile(process.argv.slice(2), adapters);
