import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fixture, team as baseTeam } from './fixture.mjs';

type Fixture = ReturnType<typeof fixture>;
type Any = any; // eslint-disable-line @typescript-eslint/no-explicit-any

function cardTeam(id: string, repo: string, component: string, capabilities: string[], owns: string[], rules: { when: string; route: 'handoff' | 'issue' }[] = []): Any {
  const base = baseTeam();
  return { ...base, id, outcome: `${component} outcome`,
    roles: [{ ...base.roles[0], id: 'lead', owns }, { ...base.roles[1], id: 'worker' }],
    card: { repo, component, owns, capabilities, intake: { intakeRole: 'lead', label: `team:${id}`, rules } } };
}

const teams = {
  api: cardTeam('api-core', 'acme/platform', 'api', ['auth', 'billing'], ['services/api/**']),
  data: cardTeam('data-pipeline', 'acme/platform', 'data', ['etl', 'warehouse'], ['pipelines/']),
  web: cardTeam('web-ui', 'acme/web', 'web', ['frontend', 'design-system'], ['apps/web/**']),
};

/** Scratch HOME, and a PATH holding only a fake gh plus git: the real gh and pk are never reachable. */
function sandbox(f: Fixture, cannedIssues: unknown[] = []) {
  const home = path.join(f.root, 'home'), bin = path.join(f.root, 'bin'), log = path.join(f.root, 'gh.log'), canned = path.join(f.root, 'issues.json');
  fs.mkdirSync(home, { recursive: true }); fs.mkdirSync(bin, { recursive: true });
  fs.writeFileSync(canned, JSON.stringify(cannedIssues));
  const gh = path.join(bin, 'gh');
  fs.writeFileSync(gh, `#!/bin/sh
if [ "$1" = "--version" ]; then echo "gh fake"; exit 0; fi
printf '%s\\n' "$*" >> "${log}"
case "$1 $2" in
  "issue list") cat "${canned}" ;;
  "issue create") echo "https://github.com/fake/issues/1" ;;
  "issue comment") echo ok ;;
esac
`, { mode: 0o755 });
  const env = { ...process.env, HOME: home, PATH: `${bin}:/usr/bin:/bin` };
  return { env, home, canned, ghCalls: () => (fs.existsSync(log) ? fs.readFileSync(log, 'utf8').trim().split('\n') : []),
    setIssues: (issues: unknown[]) => fs.writeFileSync(canned, JSON.stringify(issues)) };
}

function stateFor(f: Fixture, env: NodeJS.ProcessEnv, name: string, t: Any): string {
  const file = path.join(f.root, `${name}.json`);
  f.call('init', { state: file, team: t }, 0, env);
  return file;
}
const read = (file: string): Any => JSON.parse(fs.readFileSync(file, 'utf8'));

function publishAll(f: Fixture, env: NodeJS.ProcessEnv) {
  for (const t of Object.values(teams)) {
    const out = f.call('team-publish', { team: t }, 0, env);
    assert.equal(out.pk, 'skipped', 'pk is absent from the sandbox PATH, so publishing skips it silently');
    assert.ok(fs.existsSync(out.file));
    assert.ok(out.file.startsWith(path.join(env.HOME!, '.prometheus', 'knowledge', 'shared', 'teams')));
  }
}

test('card validation: intakeRole must be a role and the label must match the team', t => {
  const f = fixture(); t.after(f.close);
  const bad = structuredClone(teams.api); bad.card.intake.intakeRole = 'ghost';
  assert.match(f.call('validate', { team: bad }, 1).error, /intakeRole ghost is not a role/);
  const label = structuredClone(teams.api); label.card.intake.label = 'team:other';
  assert.match(f.call('validate', { team: label }, 1).error, /label must be team:api-core/);
  assert.equal(f.call('validate', { team: teams.api }).valid, true);
});

test('discovery ranks the right team first across two repos', t => {
  const f = fixture(); t.after(f.close);
  const { env } = sandbox(f);
  publishAll(f, env);
  const first = (input: Any) => f.call('team-discover', input, 0, env).matches.map((m: Any) => m.teamId);
  assert.deepEqual(first({ capabilities: ['auth'], paths: ['services/api/login.ts'] }), ['api-core']);
  assert.equal(first({ capabilities: ['ETL'] })[0], 'data-pipeline');
  assert.equal(first({ capabilities: ['design-system', 'auth'], paths: ['apps/web/src/a.tsx'] })[0], 'web-ui');
  assert.deepEqual(first({ paths: ['pipelines/daily/job.py'] }), ['data-pipeline']);
  assert.deepEqual(first({ capabilities: ['nothing-matches'] }), []);
});

test('a same-repo request creates exactly one intake task with the packet and both events', t => {
  const f = fixture(); t.after(f.close);
  const sb = sandbox(f);
  publishAll(f, sb.env);
  const state = stateFor(f, sb.env, 'api', teams.api);
  const request = { target: { teamId: 'api-core' }, from: { repo: 'acme/platform', team: 'data-pipeline', role: 'lead' },
    title: 'Expose billing export endpoint', context: 'The pipeline needs a paged export.', capabilities: ['billing'], paths: ['services/api/billing.ts'],
    evidence: ['design.md'], remaining: ['Add endpoint'], state, cwd: f.root };
  const first = f.call('team-request', { ...request, expectedRevision: 0 }, 0, sb.env);
  assert.equal(first.route, 'handoff');
  let s = read(state);
  assert.equal(s.tasks.length, 1);
  assert.equal(s.tasks[0].owner, 'lead');
  assert.equal(s.tasks[0].status, 'pending');
  assert.equal(s.handoffs.length, 1);
  assert.equal(s.handoffs[0].taskId, s.tasks[0].id);
  assert.match(s.handoffs[0].prompt, /Cross-team request for role lead of team api-core/);
  assert.match(s.handoffs[0].prompt, /Expose billing export endpoint/);
  assert.deepEqual(s.events.filter((e: Any) => e.kind.startsWith('request.')).map((e: Any) => e.kind), ['request.sent', 'request.received']);
  assert.deepEqual(sb.ghCalls(), [], 'the handoff route never calls gh');
  f.call('team-request', { ...request, expectedRevision: s.revision }, 0, sb.env);
  s = read(state);
  assert.equal(s.tasks.length, 1, 'repeating the same request must not add a second task');
  assert.equal(s.handoffs.length, 1);
});

test('a rule that forces issue routes a same-repo request to GitHub', t => {
  const f = fixture(); t.after(f.close);
  const sb = sandbox(f);
  const forced = cardTeam('forced', 'acme/platform', 'forced', ['auth'], ['x/'], [{ when: 'auth', route: 'issue' }]);
  f.call('team-publish', { team: forced }, 0, sb.env);
  const out = f.call('team-request', { target: { teamId: 'forced' }, from: { repo: 'acme/platform', team: 'other', role: 'lead' },
    title: 'T', context: 'C', capabilities: ['auth'], dryRun: true }, 0, sb.env);
  assert.equal(out.route, 'issue');
  assert.match(out.reason, /rule forces an issue/);
});

test('a cross-repo dry-run prints the exact gh command and the packet, creating nothing', t => {
  const f = fixture(); t.after(f.close);
  const sb = sandbox(f);
  publishAll(f, sb.env);
  const out = f.call('team-request', { target: { teamId: 'web-ui' }, from: { repo: 'acme/platform', team: 'api-core', role: 'lead' },
    title: "Add dark mode (it's urgent)", context: 'Dashboards need it.', capabilities: ['frontend'], dryRun: true }, 0, sb.env);
  assert.equal(out.route, 'issue');
  assert.equal(out.dryRun, true);
  assert.equal(out.command, `gh issue create --repo acme/web --title 'Add dark mode (it'\\''s urgent)' --label team:web-ui --body ${shq(out.packet)}`);
  assert.match(out.packet, /Cross-team request for role lead of team web-ui/);
  assert.deepEqual(sb.ghCalls(), [], 'dry-run must not invoke gh');
  const live = f.call('team-request', { target: { teamId: 'web-ui' }, from: { repo: 'acme/platform', team: 'api-core', role: 'lead' },
    title: 'Add dark mode', context: 'Dashboards need it.', capabilities: ['frontend'] }, 0, sb.env);
  assert.equal(live.issueUrl, 'https://github.com/fake/issues/1');
  assert.match(sb.ghCalls()[0]!, /^issue create --repo acme\/web --title Add dark mode --label team:web-ui --body /);
});

test('without gh a cross-repo request reports the command instead of failing', t => {
  const f = fixture(); t.after(f.close);
  const sb = sandbox(f);
  publishAll(f, sb.env);
  fs.rmSync(path.join(f.root, 'bin', 'gh'));
  const out = f.call('team-request', { target: { teamId: 'web-ui' }, from: { repo: 'acme/platform', team: 'api-core', role: 'lead' },
    title: 'T', context: 'C', capabilities: ['frontend'] }, 0, sb.env);
  assert.equal(out.dryRun, true);
  assert.match(out.reason, /gh is not installed/);
  assert.match(out.command, /^gh issue create --repo acme\/web/);
});

test('intake imports labelled issues idempotently, keyed by issue number, and comments only with ack', t => {
  const f = fixture(); t.after(f.close);
  const sb = sandbox(f, [{ number: 7, title: 'Add dark mode', body: 'b', url: 'https://github.com/acme/web/issues/7' }]);
  const state = stateFor(f, sb.env, 'web', teams.web);
  let out = f.call('team-intake', { state, expectedRevision: 0 }, 0, sb.env);
  assert.deepEqual(out.imported, [7]);
  assert.match(sb.ghCalls()[0]!, /^issue list --repo acme\/web --label team:web-ui --state open --json /);
  out = f.call('team-intake', { state, expectedRevision: read(state).revision }, 0, sb.env);
  assert.deepEqual(out.imported, []);
  assert.equal(out.skipped, 1);
  const s = read(state);
  assert.equal(s.tasks.length, 1, 'two imports of the same issue yield exactly one task');
  assert.equal(s.tasks[0].id, 'issue-7');
  assert.equal(s.tasks[0].owner, 'lead');
  assert.equal(s.events.filter((e: Any) => e.kind === 'request.received').length, 1);
  assert.equal(sb.ghCalls().some(c => c.startsWith('issue comment')), false, 'no comment without --ack');
  sb.setIssues([{ number: 7, title: 'Add dark mode', url: 'u7' }, { number: 9, title: 'Fix nav', url: 'u9' }]);
  out = f.call('team-intake', { state, expectedRevision: read(state).revision, ack: true }, 0, sb.env);
  assert.deepEqual(out.imported, [9]);
  assert.deepEqual(sb.ghCalls().filter(c => c.startsWith('issue comment')).map(c => c.split(' ')[2]), ['9'], 'ack comments only on newly imported issues');
  assert.equal(read(state).tasks.length, 2);
});

const sandboxRepo = process.env.PROMETHEUS_TEAM_TEST_REPO;
test('real GitHub: create one labelled issue, import it once, close it', {
  skip: sandboxRepo ? false : 'BLOCKED: PROMETHEUS_TEAM_TEST_REPO unset (sandbox repo for issue creation)',
}, t => {
  const f = fixture(); t.after(f.close);
  const home = path.join(f.root, 'home'); fs.mkdirSync(home, { recursive: true });
  const env = { ...process.env, HOME: home };
  const remote = cardTeam('team-test', sandboxRepo!, 'sandbox', ['sandbox'], ['x/']);
  f.call('team-publish', { team: remote, pk: false }, 0, env);
  const created = f.call('team-request', { target: { teamId: 'team-test' }, from: { repo: 'prometheus/elsewhere', team: 'origin', role: 'lead' },
    title: `team-request integration ${Date.now()}`, context: 'Automated sandbox test; safe to close.', capabilities: ['sandbox'] }, 0, env);
  const number = Number(String(created.issueUrl).split('/').pop());
  assert.ok(Number.isSafeInteger(number));
  try {
    const state = stateFor(f, env, 'remote', remote);
    const first = f.call('team-intake', { state, expectedRevision: 0 }, 0, env);
    assert.ok(first.imported.includes(number));
    const second = f.call('team-intake', { state, expectedRevision: read(state).revision }, 0, env);
    assert.equal(second.imported.includes(number), false);
    assert.equal(read(state).tasks.filter((x: Any) => x.id === `issue-${number}`).length, 1);
  } finally {
    spawnSync('gh', ['issue', 'close', String(number), '--repo', sandboxRepo!], { encoding: 'utf8' });
  }
});

function shq(value: string): string { return `'${value.replace(/'/g, `'\\''`)}'`; }
