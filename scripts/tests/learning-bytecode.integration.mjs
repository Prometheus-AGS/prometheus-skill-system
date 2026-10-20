// Real subprocesses, durable learning queues and file-tier recall; no service mocks.
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const supplied = process.argv.slice(2);
assert.ok(supplied.length === 0 || (supplied.length === 2 && supplied[0] === '--payload'), 'usage: node learning-bytecode.integration.mjs [--payload PACKAGE_ROOT]');
const payloads = supplied.length ? [path.resolve(supplied[1])] : [root,
  path.join(root, 'dist/plugins/claude/prometheus-skill-pack'),
  path.join(root, 'dist/plugins/codex/prometheus-skill-pack')];
const scratch = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'learning bytecode integration ')));
const read = file => fs.readFileSync(file, 'utf8');
const json = file => JSON.parse(read(file));
function write(file, value) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, typeof value === 'string' ? value : JSON.stringify(value));
}
function snapshot(directory) {
  const result = {};
  function walk(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const file = path.join(dir, entry.name), relative = path.relative(directory, file);
      assert.ok(entry.name !== '__pycache__' && !entry.name.endsWith('.pyc'), `bytecode contamination: ${file}`);
      if (entry.isDirectory()) walk(file);
      else result[relative] = crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
    }
  }
  walk(directory);
  return result;
}
const python = spawnSync('/bin/bash', ['-c', 'command -v python3'], { encoding: 'utf8' }).stdout.trim();
assert.ok(path.isAbsolute(python), 'python3 must be installed');
let commands = 0;
try {
  for (const [index, original] of payloads.entries()) {
    const box = path.join(scratch, `surface ${index}`), payload = path.join(box, 'package with spaces');
    const originalOrchestrator = fs.existsSync(path.join(original, 'skills/process/kbd-process-orchestrator'))
      ? 'skills/process/kbd-process-orchestrator' : 'skills/kbd-process-orchestrator';
    for (const relative of ['shared/scripts', 'shared/schemas', `${originalOrchestrator}/skills/kbd-memory-recall`]) {
      fs.cpSync(path.join(original, relative), path.join(payload, relative), { recursive: true,
        filter: file => !['__pycache__', 'tests'].includes(path.basename(file)) && !file.endsWith('.pyc') });
    }
    const partition = path.join(original, 'scripts/memory-index-partition.py');
    if (fs.existsSync(partition)) {
      fs.mkdirSync(path.join(payload, 'scripts'), { recursive: true });
      fs.cpSync(partition, path.join(payload, 'scripts/memory-index-partition.py'));
    }
    const bin = path.join(box, 'bin'); fs.mkdirSync(bin);
    // Real interpreter plus OS utilities only: pk/service clients are intentionally unavailable.
    fs.symlinkSync(python, path.join(bin, 'python3'));
    const before = snapshot(payload);
    for (const inherited of ['unset', '0']) {
      const consumer = path.join(box, `consumer ${inherited}`), home = path.join(box, `home ${inherited}`);
      const phase = 'bytecode-repair', phaseDir = path.join(consumer, '.kbd-orchestrator/phases', phase);
      fs.mkdirSync(home, { recursive: true });
      write(path.join(consumer, 'package.json'), { name: 'disposable-bytecode-consumer', private: true });
      write(path.join(consumer, '.kbd-orchestrator/current-waypoint.json'), { phase });
      write(path.join(phaseDir, 'handoffs/plan.handoff.json'), { summaryForNext: 'Bytecode stage writeback proves durable state.' });
      write(path.join(phaseDir, 'reflection.md'), '# Reflection\n\n## Lessons Learned\n- Bytecode reflection writeback proves durable state.\n');
      write(path.join(consumer, '.agent-team/bytecode/team.json'), { id: 'bytecode', roles: [{ id: 'lead', owns: ['**'] }] });
      const env = { PATH: `${bin}:/usr/bin:/bin:/usr/sbin:/sbin`, HOME: home,
        CODEX_HOME: path.join(home, '.codex'), CLAUDE_CONFIG_DIR: path.join(home, '.claude'),
        XDG_CONFIG_HOME: path.join(home, '.config'), XDG_DATA_HOME: path.join(home, '.local/share'), XDG_CACHE_HOME: path.join(home, '.cache'),
        TMPDIR: path.join(home, 'tmp'), PROMETHEUS_DATA_DIR: path.join(home, 'data'),
        PROMETHEUS_LEARNING_QUEUE: path.join(home, 'queue'), PROMETHEUS_LEARNING_LOG_DIR: path.join(home, 'learning-log'),
        PROMETHEUS_LEARNING_INDEX_DIR: path.join(home, 'learning-index'), PROMETHEUS_PROJECT_ID: `bytecode-project-${inherited}`,
        PROMETHEUS_USER_ID: 'bytecode-user', PROMETHEUS_LEARNING_CORTEX: '0', PROMETHEUS_LEARNING_PK: '0',
        PROMETHEUS_KBD_CONTROL_PLANE: '0', PROMETHEUS_CONTROL_ENDPOINT: 'http://127.0.0.1:1',
        SURREAL_MEMORY_URL: 'none', PROMETHEUS_MEMORY_URL: 'http://127.0.0.1:1',
        GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: os.devNull,
        KBD_PACK_ROOT: payload, KBD_ORCHESTRATOR_ROOT: path.join(payload, originalOrchestrator) };
      if (inherited !== 'unset') env.PYTHONDONTWRITEBYTECODE = inherited;
      fs.mkdirSync(env.TMPDIR);
      const scripts = path.join(payload, 'shared/scripts'), lib = path.join(scripts, 'lib');
      function run(exe, args, input = '', extra = {}) {
        const result = spawnSync(exe, args, { cwd: consumer, env: { ...env, ...extra }, input, encoding: 'utf8', timeout: 30_000 });
        commands++;
        assert.equal(result.status, 0, `${exe} ${args.join(' ')}\n${result.error ?? ''}\n${result.stdout}\n${result.stderr}`);
        assert.deepEqual(snapshot(payload), before, `package changed after ${args.join(' ')}`);
        return result.stdout;
      }
      if (fs.existsSync(partition)) {
        const indexFile = path.join(consumer, 'MEMORY.md');
        write(indexFile, '# Memory index\n- role:lead Bytecode index partition writes its lesson durably.\n');
        const result = JSON.parse(run('python3', [path.join(payload, 'scripts/memory-index-partition.py'), indexFile, '--apply', '--cwd', consumer]));
        assert.equal(result.applied, true);
        assert.ok(result.queued.some(entry => entry.queued), JSON.stringify(result));
        assert.match(read(path.join(env.PROMETHEUS_LEARNING_LOG_DIR, 'lessons.jsonl')), /Bytecode index partition writes its lesson durably/);
      }
      const lesson = 'Bytecode direct writer produces a durable, recallable lesson.';
      const written = JSON.parse(run('python3', [path.join(lib, 'learning_write.py'), '--cwd', consumer, '--text', lesson]));
      assert.ok(written.written > 0, JSON.stringify(written));
      const operations = path.join(env.PROMETHEUS_LEARNING_QUEUE, 'memory/pending');
      assert.ok(fs.readdirSync(operations).some(file => json(path.join(operations, file)).arguments.content.includes(lesson)));
      const recalled = JSON.parse(run('python3', [path.join(lib, 'learning_recall.py'), '--cwd', consumer, '--main-thread', '--no-pk', '--memory-url', 'none']));
      assert.equal(recalled.lessonSource, 'file');
      assert.ok(recalled.lessons.some(entry => JSON.stringify(entry).includes(lesson)), JSON.stringify(recalled));
      const identity = JSON.parse(run('python3', [path.join(lib, 'agent_identity.py'), '--cwd', consumer], '{}'));
      assert.equal(identity.projectId, env.PROMETHEUS_PROJECT_ID);
      const routed = JSON.parse(run('python3', [path.join(lib, 'learning_route.py'), '--cwd', consumer, '--author', 'lead', '--paths', 'src/example.js']));
      assert.equal(routed.teamId, 'bytecode');
      assert.deepEqual(routed.audience, []);
      const gap = run('python3', [path.join(lib, 'prompt_gap.py')], '{"results":[]}', {
        PROMPT_GAP_PROMPT: 'Why does bytecode corrupt immutable generation signatures?',
        PROMPT_GAP_PAYLOAD: JSON.stringify({ cwd: consumer, session_id: 'bytecode-gap' }) });
      assert.match(gap, /\[prometheus-gap\]/);
      assert.ok(read(path.join(home, '.prometheus/knowledge-gaps/gaps.jsonl')).includes(env.PROMETHEUS_PROJECT_ID));
      const hookPayload = JSON.stringify({ cwd: consumer, session_id: 'bytecode-stop' });
      run('python3', [path.join(scripts, 'enqueue-learning-job.py'), 'stop', 'claude'], hookPayload);
      const jobs = path.join(env.PROMETHEUS_LEARNING_QUEUE, 'pending');
      assert.equal(fs.readdirSync(jobs).length, 1);
      assert.equal(json(path.join(jobs, fs.readdirSync(jobs)[0])).projectRoot, consumer);
      run('bash', [path.join(scripts, 'karpathy-hook-dispatch.sh'), 'stop', 'claude'], hookPayload);
      assert.equal(fs.readdirSync(jobs).length, 1, 'same turn remains idempotent');
      run('bash', [path.join(scripts, 'kbd-stage-writeback.sh'), 'plan', phase]);
      run('bash', [path.join(scripts, 'memory-writeback.sh')]);
      const lessons = read(path.join(env.PROMETHEUS_LEARNING_LOG_DIR, 'lessons.jsonl'));
      assert.match(lessons, /Bytecode stage writeback proves durable state/);
      assert.match(lessons, /Bytecode reflection writeback proves durable state/);
      assert.ok(read(path.join(phaseDir, '.memory-writeback.tsv')).trim());
      run('bash', [path.join(payload, originalOrchestrator, 'skills/kbd-memory-recall/kbd-memory-recall.sh'), phase, 'plan']);
      assert.match(read(path.join(phaseDir, 'prior-context.md')), /Bytecode direct writer produces a durable/);
      assert.ok(read(path.join(env.PROMETHEUS_LEARNING_INDEX_DIR, 'delivery.jsonl')).trim());
      assert.ok(!fs.existsSync(path.join(consumer, 'shared')), 'consumer must not acquire package scripts');
      console.log(`PASS ${original} (PYTHONDONTWRITEBYTECODE=${inherited}): direct CLI, gap, enqueue, dispatcher, writeback and KBD recall; immutable package`);
    }
  }
  console.log(`PASS ${commands} real subprocesses across ${payloads.length} package surfaces; no bytecode writes`);
} finally {
  fs.rmSync(scratch, { recursive: true, force: true });
}
