// SubagentStop matchers must not fire iterative-evolver hooks for agent-team
// roles that happen to share a name (planner, executor, ...). Claude Code reports
// plugin agents as `<plugin>:<name>` (captured: `iterative-evolver:planner`), so
// its matchers are anchored and prefix-required. Codex reports the bare agent
// name, so Codex copies run through shared/scripts/team-role-guard.sh, which
// skips agents that resolve to a role in the active team.
//
// Design: docs/design/team-aware-learning-memory.md §1.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, writeFileSync, existsSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const read = (file) => JSON.parse(readFileSync(path.join(root, file), 'utf8'));
const ROLES = ['assessor', 'analyst', 'planner', 'executor', 'reflector'];
const PHASE_ARGS = {
  assessor: ['assess', 'phase_complete'],
  analyst: ['analyze', 'phase_complete'],
  planner: ['plan', 'phase_complete'],
  executor: ['execute', 'phase_complete'],
  reflector: ['reflect', 'phase_complete'],
};

const subagentStopGroups = (file) => read(file).hooks.SubagentStop ?? [];

// Bare role ids that a team may declare: the five names plus any role in a
// committed .agent-team manifest.
function bareRoleIds() {
  const ids = new Set([...ROLES, 'backend-dev', 'ui-dev']);
  return [...ids];
}

test('no Claude Code SubagentStop matcher matches a bare role id', () => {
  for (const group of subagentStopGroups('hooks/hooks.json')) {
    if (group.matcher === undefined) continue;
    const pattern = new RegExp(group.matcher);
    for (const id of bareRoleIds()) {
      assert.ok(!pattern.test(id), `Claude matcher ${group.matcher} matches bare role "${id}"`);
    }
  }
});

test('Claude Code matchers match exactly the captured plugin-qualified agent types', () => {
  const captured = JSON.parse(
    readFileSync(
      process.env.TLI_B2_PAYLOAD ??
        path.join(root, 'scripts/tests/fixtures/claude-iterative-evolver-subagentstop.json'),
      'utf8',
    ),
  );
  assert.equal(captured.agent_type, 'iterative-evolver:planner');
  const matchers = subagentStopGroups('hooks/hooks.json')
    .map((group) => group.matcher)
    .filter(Boolean);
  for (const role of ROLES) {
    const hits = matchers.filter((matcher) => new RegExp(matcher).test(`iterative-evolver:${role}`));
    assert.equal(hits.length, 1, `iterative-evolver:${role} must match exactly one group`);
    assert.ok(!matchers.some((matcher) => new RegExp(matcher).test(`x-iterative-evolver:${role}-y`)), 'matchers must be anchored');
  }
});

test('per-phase hook arguments survive in both harness bundles', () => {
  const contract = read('shared/harnesses/hook-contract.json');
  for (const role of ROLES) {
    const claude = contract.events.find((group) => group.matcher === `^iterative-evolver:${role}$`);
    const codex = contract.events.find((group) => group.matcher === `^${role}$`);
    assert.deepEqual(claude.harnesses, ['claude-code']);
    assert.deepEqual(codex.harnesses, ['codex']);
    const checkpoint = claude.hooks.find((hook) => hook.id === `subagent-${role}-checkpoint`);
    assert.deepEqual(checkpoint.args, ['{evolution}', ...PHASE_ARGS[role]]);
    // Codex copies wrap the same targets and args, with unique ids.
    assert.equal(codex.hooks.length, claude.hooks.length);
    for (const [index, hook] of claude.hooks.entries()) {
      const copy = codex.hooks[index];
      assert.equal(copy.id, `${hook.id}-codex`);
      assert.equal(copy.target, 'shared/scripts/team-role-guard.sh');
      assert.deepEqual(copy.args, [hook.target, ...(hook.args ?? [])]);
    }
  }
  const codexMatchers = subagentStopGroups('hooks/codex-hooks.json').map((group) => group.matcher);
  for (const role of ROLES) assert.ok(codexMatchers.includes(`^${role}$`), `codex bundle lacks ^${role}$`);
});

test('the iterative-evolver executor keeps its Karpathy learning hook', () => {
  const contract = read('shared/harnesses/hook-contract.json');
  const executor = contract.events.find((group) => group.matcher === '^iterative-evolver:executor$');
  const learning = executor.hooks.find((hook) => hook.id === 'subagent-executor-karpathy-learning');
  assert.equal(learning.target, 'shared/scripts/karpathy-hook-dispatch.sh');
  assert.deepEqual(learning.args, ['executor_complete', '{harness}']);
});

// Guard behaviour through the real generated dispatcher.
function sandbox({ teamRole }) {
  const dir = mkdtempSync(path.join(tmpdir(), 'subagent-matchers-'));
  mkdirSync(path.join(dir, 'home'));
  mkdirSync(path.join(dir, '.evolver/evolutions/default'), { recursive: true });
  writeFileSync(path.join(dir, '.evolver/evolutions/default/state.json'), '{"phases":{}}\n');
  if (teamRole) {
    mkdirSync(path.join(dir, '.agent-team/fixture'), { recursive: true });
    writeFileSync(
      path.join(dir, '.agent-team/fixture/team.json'),
      JSON.stringify({
        schemaVersion: 1,
        id: 'fixture',
        roles: [{ id: teamRole, description: 'd', prompt: 'p', skills: [], owns: [], inputs: [], outputs: [], dependsOn: [] }],
      }),
    );
  }
  return dir;
}

function dispatch(dir, hook, harness, agentType) {
  return spawnSync('bash', [path.join(root, 'shared/scripts/generated/hook-dispatch-v1.sh'), '--hook', hook, '--harness', harness], {
    cwd: dir,
    input: JSON.stringify({ hook_event_name: 'SubagentStop', agent_type: agentType, agent_id: 'a1', cwd: dir }),
    env: {
      ...process.env,
      HOME: path.join(dir, 'home'),
      EVOLVER_STATE_DIR: path.join(dir, '.evolver'),
      PROMETHEUS_PLUGIN_ROOT: path.join(dir, 'no-plugin'),
      PROMETHEUS_PROJECT_ID: 'project:fixture',
    },
    encoding: 'utf8',
  });
}

const checkpointed = (dir) => existsSync(path.join(dir, '.evolver/evolutions/default/checkpoints'));

test('Codex: a team role named planner does not run the iterative-evolver checkpoint', () => {
  const dir = sandbox({ teamRole: 'planner' });
  try {
    const result = dispatch(dir, 'subagent-planner-checkpoint-codex', 'codex', 'planner');
    assert.equal(result.status, 0, result.stderr);
    assert.equal(checkpointed(dir), false, 'guard let a team role through');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('Codex: a genuine planner agent with no team still runs the checkpoint', () => {
  const dir = sandbox({ teamRole: null });
  try {
    const result = dispatch(dir, 'subagent-planner-checkpoint-codex', 'codex', 'planner');
    assert.equal(result.status, 0, result.stderr);
    assert.equal(checkpointed(dir), true, `checkpoint did not run: ${result.stderr}`);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('Claude Code: the anchored planner group still runs its checkpoint directly', () => {
  const dir = sandbox({ teamRole: 'planner' });
  try {
    const result = dispatch(dir, 'subagent-planner-checkpoint', 'claude-code', 'iterative-evolver:planner');
    assert.equal(result.status, 0, result.stderr);
    assert.equal(checkpointed(dir), true, `checkpoint did not run: ${result.stderr}`);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
