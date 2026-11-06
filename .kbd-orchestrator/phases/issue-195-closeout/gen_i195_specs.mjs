// Generates the issue-195-closeout native-kbd change specs and the draft adopted contract.
// Re-run from the repository root: node .kbd-orchestrator/phases/issue-195-closeout/gen_i195_specs.mjs
import fs from 'node:fs';
import path from 'node:path';

const ROOT = process.cwd();
const PHASE = 'issue-195-closeout';
const PHASE_DIR = path.join(ROOT, '.kbd-orchestrator/phases', PHASE);
const CHANGES = path.join(ROOT, '.kbd-orchestrator/changes');
const ORCH = 'skills/process/kbd-process-orchestrator';
const STAGES = ['kbd-assess', 'kbd-child-exit', 'kbd-execute', 'kbd-handoff', 'kbd-init', 'kbd-new-child',
  'kbd-next-child', 'kbd-pause', 'kbd-plan', 'kbd-resume', 'kbd-spec'];
const STAGE_FILES = STAGES.map(s => `${ORCH}/skills/${s}/SKILL.md`);
const ISOLATION = 'Owned mkdtemp project; scratch PROMETHEUS_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, XDG_CONFIG_HOME, XDG_DATA_HOME, TMPDIR, device key and hook roots; KBD_MEMORY_MCP_URL/UAR_MEMORY_MCP_URL at http://127.0.0.1:1; no write to the real journal, ~/.prometheus/plugins, ~/.claude, ~/.codex or ~/.local/share/opencode. Cleanup owner: the integration driver World.';
const GATE = 'node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json --prometheus "$(command -v prometheus)" --evidence-root .kbd-orchestrator/phases/issue-195-closeout/evidence (repository root, after every production task is done)';
const LESSONS = {
  ledger: 'Recalled lesson: "`kbd-apply mark-done` updates the task flag but does not synchronize the canonical KBD ledger" — close tasks only with kbd-apply begin-task/end-task.',
  install: 'Recalled lesson: "A CRITICAL shipped despite all per-change gates passing … the installer never applied the Codex memory setting" — surface checks observe the installer-dispatched payload, never source text alone.',
  regen: 'Recalled lesson: "Use rebase-regenerate.sh for Generated-Only Rebase Conflicts" — resolve generated-only dist/ conflicts with scripts/rebase-regenerate.sh.',
};

const CORE = [
  '## Selected-method recovery and authority',
  '',
  'Before dependent work, run `shared/method-context.mjs --project <project-root> --phase <canonical-phase-id> --write` from the resolved orchestrator root (or read the phase-local `method-context.md` when Node is unavailable) and act on its `status`:',
  '',
  '- `RESOLVED_REFERENCES` — continue with the recovered method, contract and stage authority; preserve the same references in role inputs, dispatch and handoff read order.',
  '- `BLOCKED` — a method decision, contract, calibration or supersession is missing or changed; resolve it explicitly. Prior commands, memory and tool inventories are not fresh authority.',
  '- `CONFLICT` — governing host policy conflicts with the selected method. Stop before dependent work and show the conflict; continue only after the operator records a `KBD-POLICY-ACK` decision for that file\'s current hash. Never rewrite operator-owned policy.',
  '',
  'For selected VLBV, define acceptance before generation and finish the whole agreed value phase before any generated-code verification; tasks, hours, commits, reviewer requests and compiler problems create no boundary. Preserve original evidence, consumed budgets and unresolved effects across resume and supersession; reconcile an uncertain effect before any retry. Full rules: `references/method-context.md`.',
].join('\n');

function write(rel, text) {
  const file = path.join(ROOT, rel);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, text.endsWith('\n') ? text : `${text}\n`);
}

function acceptance(b) {
  return [
    `## Behavior: ${b.behavior}`,
    '',
    '| Field | Required declaration |',
    '|---|---|',
    `| Canonical identity | ${PHASE} / ${b.change} / task ${b.task}; scenario ${b.scenario} |`,
    '| Source identity | prometheus-skill-pack main at the completed-production candidate tree; tracked-diff and untracked SHA-256 recorded by the driver |',
    `| Production entry point | ${b.entry} |`,
    `| Real collaborators | ${b.collaborators} |`,
    `| Boundary exercised | ${b.boundary} |`,
    `| Observable result | ${b.observable} |`,
    `| Negative control | ${b.negative} |`,
    `| Isolation | ${b.isolation ?? ISOLATION} |`,
    `| Prerequisites | ${b.prereq} |`,
    `| Final local gate | ${GATE} |`,
    `| Evidence | .kbd-orchestrator/phases/${PHASE}/evidence/<attempt>/${b.scenario}/ — pending; nothing has run |`,
    `| Limitations | ${b.limits} |`,
    '',
  ].join('\n');
}

const changes = [
  {
    id: 'i195-01-policy-conflict-ack',
    title: 'Governing-policy conflicts block dependent work until acknowledged',
    why: 'Issue #195 acceptance 2 requires a governing-policy conflict to be surfaced before dependent execution. Today `method-context.mjs` lists conflicts but `status` (line 128) and `--require-resolved` (line 182) ignore them, so a recovered session can proceed past the conflict. This repository\'s own CLAUDE.md:102 / AGENTS.md:104 compiler-check exception triggers it once VLBV is selected.',
    what: [
      'Reuse the existing signed decision surface: an acknowledgement is a decision whose summary is `KBD-POLICY-ACK {"phaseId","path","sha256","issue","note"}`, recorded with `prometheus kbd decision record` and replaced through `--supersedes` (same pattern as `KBD-METHOD`, method-context.mjs:70-80). No new event kind, CLI command or schema.',
      'A conflict is `acknowledged` only when exactly one current (non-superseded) ack matches phaseId, path and the file\'s current SHA-256. A changed policy file therefore re-raises the conflict.',
      'Any unacknowledged conflict sets `status: "CONFLICT"` (blockers still win: BLOCKED > CONFLICT > RESOLVED_REFERENCES); `--require-resolved` exits 2 for both BLOCKED and CONFLICT. Each conflict entry gains `acknowledged` and `ackDecisionId`.',
      'reanchor.sh prints a blocking line naming each unacknowledged conflict; kbd-handoff.sh includes `policyConflicts` in its method summary. references/method-context.md documents the payload, the recording command and the trust model.',
      'Operator-owned policy files are never edited. Control is described honestly: canonical Decision state carries no actor, so acknowledgement authority equals the existing KBD-METHOD trust model (instruction-level; not a new guarantee).',
    ],
    tasks: [
      { id: '1', title: 'Resolve KBD-POLICY-ACK decisions and add CONFLICT status and exit semantics to method-context.mjs', files: [`${ORCH}/shared/method-context.mjs`] },
      { id: '2', title: 'Surface unacknowledged conflicts in reanchor.sh and kbd-handoff.sh; document acknowledgement in references/method-context.md', files: ['skills/process/prometheus-context-bootstrap/assets/hooks/reanchor.sh', `${ORCH}/skills/kbd-handoff/kbd-handoff.sh`, `${ORCH}/references/method-context.md`] },
    ],
    lessons: [LESSONS.ledger],
    behaviors: [
      { scenario: 'CON-001', task: '1', behavior: 'An unacknowledged governing-policy conflict blocks dependent work',
        entry: `node ${ORCH}/shared/method-context.mjs --project <scratch> --phase <phase> --require-resolved --json`,
        collaborators: 'installed prometheus CLI (kbd status/disposition, decision record), scratch canonical journal, scratch CLAUDE.md copied byte-for-byte from this repository',
        boundary: 'process (node -> prometheus subprocess), filesystem (scratch project and data dir)',
        observable: 'exit code 2; JSON status "CONFLICT"; policyConflicts[] names CLAUDE.md with acknowledged=false',
        negative: 'same project with a KBD-METHOD selecting a non-VLBV method: status RESOLVED_REFERENCES, exit 0, policyConflicts empty. Fixtures seed their own scratch adopted contract and signed KBD-METHOD decision; the real project\'s method decision is not used',
        prereq: 'prometheus CLI built from the candidate; Node 22+',
        limits: 'Detection is a text pattern over three governing files; wording outside the pattern is not detected.' },
      { scenario: 'CON-002', task: '1', behavior: 'A hash-bound acknowledgement clears the conflict without editing policy; a policy change re-raises it',
        entry: `prometheus kbd decision record --id <ack-id> --summary 'KBD-POLICY-ACK {...}' then method-context.mjs --require-resolved`,
        collaborators: 'installed prometheus CLI, scratch canonical journal, scratch CLAUDE.md',
        boundary: 'process, filesystem, signed journal append',
        observable: 'after the ack: exit 0, status RESOLVED_REFERENCES, conflict acknowledged=true with ackDecisionId; CLAUDE.md SHA-256 unchanged before and after',
        negative: 'append one byte to scratch CLAUDE.md: exit 2 and CONFLICT again; an ack superseded by a later decision, or bound to a different phase, does not clear it',
        prereq: 'as CON-001',
        limits: 'Does not establish who recorded the ack (Decision state has no actor).' },
      { scenario: 'CON-003', task: '2', behavior: 'Recovery and handoff surfaces show an unacknowledged conflict as blocking',
        entry: 'bash skills/process/prometheus-context-bootstrap/assets/hooks/reanchor.sh and kbd-handoff.sh in the scratch CON-001 project',
        collaborators: 'method-context.mjs, prometheus CLI, scratch phase files',
        boundary: 'process (bash -> node -> prometheus), filesystem',
        observable: 'reanchor output contains a blocking conflict line naming CLAUDE.md; the handoff method summary includes policyConflicts with acknowledged=false',
        negative: 'after CON-002\'s acknowledgement the blocking line is absent',
        prereq: 'as CON-001; bash 3.2-compatible scripts (/bin/bash)',
        limits: 'Observes printed instructions only; no tool interception is claimed.' },
      { scenario: 'POL-002', task: '2', behavior: 'A pressured session runs no generated-code check before production completes and surfaces the compiler-exception conflict',
        entry: 'claude -p with the generated Claude payload (--plugin-dir dist/plugins/claude/prometheus-skill-pack) under a scratch CLAUDE_CONFIG_DIR, via the existing session driver',
        collaborators: 'Claude Code CLI, generated payload, scratch project, method-context.mjs',
        boundary: 'process, stream-json tool-call observation',
        observable: 'no attempted test/build/lint/dry-run tool call while production is incomplete; the reply names the CONFLICT and does not claim compiler-check permission',
        negative: 'existing POL-002 pressure prompts (task, hour, commit, reviewer, stale command)',
        isolation: `${ISOLATION} CLAUDE_CONFIG_DIR=~/.prometheus-scratch/kbd194/claude (operator login).`,
        prereq: 'operator-logged-in scratch Claude config; Bash never pre-approved',
        limits: 'n=1 sample; shows the behavior can occur, not its reliability.' },
    ],
  },
  {
    id: 'i195-02-single-method-reference',
    title: 'One identical selected-method core in all stage skills',
    why: 'The `## Selected-method recovery and authority` section is hand-copied into 11 stage skills (two byte variants today, differing by one trailing blank line). Issue #195 acceptance 5 needs source, generated and installed surfaces to keep the same semantics; copies drift.',
    what: [
      `Replace the section in the 11 stage skills (${STAGES.join(', ')}) with one identical short core (below) that keeps the load-bearing rules inline and links \`references/method-context.md\` for the rest. A bare link is rejected: a model may not open it.`,
      'C-01 reconciliation (constraints.md:8-19): run `npm run build:distribution` and `npm run generate:skills-index` twice, record the SHA-256 of every tracked output after each run in `.kbd-orchestrator/phases/issue-195-closeout/evidence/reconciliation.json` and require them byte-identical; then run `npm run check:distribution`, `npm run check:skills-index` and `npm run validate:harness-adapters`. No distribution certification is claimed before this.',
      'Add `scripts/check-selected-method-core.mjs` (wired as `npm run check:selected-method`) that fails when any stage skill\'s section differs byte-for-byte from the canonical core or is missing.',
      'Deviation from goal B\'s literal wording ("one reference file"): the reference file stays single, but each stage skill keeps an identical short inline core because a bare link may not be opened; the #195 mapping must report goal B as satisfied in this adapted form, not literally.',
      '`dist/**` is tracked (4,482 files per `git ls-files dist` at 6bb66d5c). Regenerate and commit tracked outputs in the same change: `npm run build:distribution` (dist/plugins/claude/**, dist/plugins/codex/**, .claude-plugin/marketplace.json, .agents/plugins/marketplace.json) and `npm run generate:skills-index` (SKILLS.md).',
      '',
      'Canonical core:',
      '',
      '```markdown',
      CORE,
      '```',
    ],
    tasks: [
      { id: '1', title: 'Replace the selected-method section in the 11 stage skills with the identical canonical core', files: STAGE_FILES },
      { id: '2', title: 'Add check-selected-method-core.mjs and npm script; C-01 reconciliation: run the generators twice, prove byte-identical tracked hashes, run drift validators', files: ['scripts/check-selected-method-core.mjs', 'package.json', 'dist/plugins/claude/**', 'dist/plugins/codex/**', '.claude-plugin/marketplace.json', '.agents/plugins/marketplace.json', 'SKILLS.md'] },
    ],
    lessons: [LESSONS.regen, LESSONS.install, LESSONS.ledger],
    dependsOn: ['i195-01-policy-conflict-ack'],
    behaviors: [
      { scenario: 'REF-001', task: '2', behavior: 'All stage skills carry the identical core in source, generated payloads and an installer-dispatched scratch generation',
        entry: 'node scripts/check-selected-method-core.mjs; then, from the integration driver without any CLI session, node scripts/install-plugin-generation.js --source-root <candidate> --home <scratch> --plugin-root <scratch>/plugins after the World has created the owned scratch roots',
        collaborators: 'distribution generator, installer generation writer, scratch plugin root',
        boundary: 'filesystem (source, dist/, scratch installed generation)',
        observable: 'the section bytes are identical across the 11 source skills, both generated payloads and the scratch-installed generation; check exits 0',
        negative: 'a scratch copy with one stage section altered by one character makes the check exit non-zero naming that file',
        prereq: 'completed production; generator outputs regenerated',
        limits: 'Proves byte identity, not that a model reads the section.' },
      { scenario: 'POL-001', task: '1', behavior: 'Fresh, resumed and delegated sessions recover the same method, contract, value and stage authority from the rewritten core (rerun)',
        entry: 'existing POL-001 session implementation in scripts/tests/lib/kbd-194-196-sessions.mjs with the regenerated Claude payload',
        collaborators: 'Claude Code CLI, generated payload, method-context.mjs, scratch project',
        boundary: 'process, stream-json observation',
        observable: 'all observations report identical method, contract digest, included value/exclusions and stage authority',
        negative: 'existing POL-001 negative cases',
        isolation: `${ISOLATION} CLAUDE_CONFIG_DIR=~/.prometheus-scratch/kbd194/claude (operator login).`,
        prereq: 'operator-logged-in scratch Claude config',
        limits: 'n=1 sample.' },
      { scenario: 'POL-003', task: '1', behavior: 'Resume after supersession keeps budgets and uncertain effects with the rewritten core (rerun)',
        entry: 'existing POL-003 session implementation in scripts/tests/lib/kbd-194-196-sessions.mjs with the regenerated Claude payload',
        collaborators: 'Claude Code CLI, generated payload, installed prometheus CLI, scratch journal',
        boundary: 'process, signed journal, stream-json observation',
        observable: 'latest signed selection wins; retired workflow not resurrected; consumed budget retained; uncertain effect not retried',
        negative: 'existing POL-003 stale handoff',
        isolation: `${ISOLATION} CLAUDE_CONFIG_DIR=~/.prometheus-scratch/kbd194/claude (operator login).`,
        prereq: 'operator-logged-in scratch Claude config',
        limits: 'n=1 sample.' },
      { scenario: 'PKG-001', task: '2', behavior: 'Finite source/generated/scratch-installed parity (rerun of the prior phase scenario)',
        entry: 'existing PKG-001 implementation in scripts/tests/lib/kbd-194-196-package.mjs',
        collaborators: 'generator (two runs), installer dispatch, scratch plugin root',
        boundary: 'filesystem, installer process',
        observable: 'two generator runs byte-identical; actual installer dispatch used; method-context --write works through a symlinked skill dir',
        negative: 'existing PKG-001 negative cases',
        prereq: 'completed production',
        limits: 'macOS only.' },
    ],
  },
  {
    id: 'i195-03-delivery-qualification',
    title: 'Author and run the issue #195 closeout evidence; close honestly',
    why: 'VER-001 and VER-003 are stubs (scripts/tests/lib/kbd-194-196-sessions.mjs:314-316), so #195 acceptance 4 and 6 have no evidence; Codex and OpenCode were never exercised; the session driver leaves PROMETHEUS_PLUGIN_ROOT unset. This change owns all test authoring and the single final boundary, after changes 01 and 02 are complete.',
    what: [
      'Task 1 runs only after every production task of changes 01 and 02 is done (host policy: implementation first).',
      'Session driver isolation: set a scratch PROMETHEUS_PLUGIN_ROOT for every harness. Add an OpenCode harness: `opencode run --format json --standalone` with scratch XDG_CONFIG_HOME/XDG_DATA_HOME, OPENCODE_DISABLE_CLAUDE_CODE=1, OPENCODE_DISABLE_AUTOUPDATE=1, preceded by an isolation preflight that confirms config and auth resolve under the scratch roots; otherwise the scenario is NOT_EXERCISED (unsupported isolation), never run against real config.',
      'Implement VER-001 and VER-003 bodies in place of stub(); add CON-001..003, REF-001, SRF-001, SRF-002 and register them so the driver iterates this phase\'s contract.',
      'Task 2 runs the boundary packet once; fixes go in coherent batches with one confirming rerun, within the adopted budget.',
      'Task 3: cross-model review of the cumulative diff via liter-llm `dispatch-judge.sh` with the per-run judge override (`PROMETHEUS_KBD_JUDGE_*`, e.g. glm-5.3), because `~/.prometheus/kbd/models.toml` sets judge = critic and refuses dispatch; this cumulative review is the review receipt that i195-01 and i195-02 need before archive (judge not Claude-family; label recorded honestly), `npm run validate:strict`, `npm run check:distribution`, protected-test certification from the committed candidate.',
      'Task 4: typed canonical closure of this phase with the six claims kept distinct and no successor; record a partial disposition for issues-194-195-196 that references this phase; draft the #195 acceptance-to-evidence comment. Posting the comment and closing the issue are outward-facing and need explicit owner authorization at that time; record the request and its outcome (posted, or declined with reason) in closure-report.md.',
    ],
    tasks: [
      { id: '1', title: 'After production completes: author driver isolation fix, OpenCode harness, VER-001/VER-003 bodies and CON/REF/SRF scenarios', files: ['scripts/tests/lib/kbd-194-196-sessions.mjs', 'scripts/tests/lib/kbd-195-closeout.mjs', 'scripts/tests/kbd-194-196.integration.mjs'] },
      { id: '2', title: 'Run the consolidated boundary packet once; batch fixes; one confirming rerun', files: [`.kbd-orchestrator/phases/${PHASE}/evidence/**`] },
      { id: '3', title: 'Cross-model review, strict validation, distribution check and protected-test certification', files: [`.kbd-orchestrator/phases/${PHASE}/review/final/**`] },
      { id: '4', title: 'Typed closure, partial disposition of issues-194-195-196, draft #195 evidence comment (posting needs owner authorization)', files: [`.kbd-orchestrator/phases/${PHASE}/closure-report.md`] },
    ],
    lessons: [LESSONS.install, LESSONS.ledger],
    dependsOn: ['i195-01-policy-conflict-ack', 'i195-02-single-method-reference'],
    behaviors: [
      { scenario: 'VER-001', task: '1', behavior: 'value-scenario-verify drives the real repair-and-resume operation and its oracle is sensitive',
        entry: '/value-scenario-verify invoked by a scratch Claude session against the IDR-002/IDR-003 repair-and-resume operation as runner',
        collaborators: 'Claude Code CLI with generated payload, installed prometheus CLI (change repair-identity, kbd-apply begin/end-task), scratch journal',
        boundary: 'process, signed journal, stream-json observation',
        observable: 'verdict PASS with the observed resumed task state; the session reports authored, measured and demonstrated value separately',
        negative: 'a deliberately wrong expected value yields FAIL; a removed required assertion yields NOT_EXERCISED, never PASS',
        isolation: `${ISOLATION} CLAUDE_CONFIG_DIR=~/.prometheus-scratch/kbd194/claude (operator login).`,
        prereq: 'operator-logged-in scratch Claude config; value-scenario-verify in the generated payload',
        limits: 'n=1 sample.' },
      { scenario: 'VER-003', task: '1', behavior: 'Canonical closure keeps six claims distinct and starts no successor',
        entry: 'scratch session drives the canonical phase closure writer through value-scenario-verify (closure adapter fixture/step/disposition)',
        collaborators: 'Claude Code CLI, installed prometheus CLI, scratch journal',
        boundary: 'process, signed journal',
        observable: 'typed disposition persists; authored, measured, demonstrated, gaps, release eligibility and release authority reported separately; eventCounts show no new phase, task or publication events',
        negative: 'a request to "also start the next phase" produces no phase/task events',
        isolation: `${ISOLATION} CLAUDE_CONFIG_DIR=~/.prometheus-scratch/kbd194/claude (operator login).`,
        prereq: 'operator-logged-in scratch Claude config',
        limits: 'n=1 sample.' },
      { scenario: 'SRF-001', task: '1', behavior: 'A Codex session recovers the selected method and reports the conflict',
        entry: 'codex exec --json -s read-only --ephemeral with the installed Codex payload under scratch CODEX_HOME and PROMETHEUS_PLUGIN_ROOT',
        collaborators: 'codex-cli, installed Codex payload, method-context.mjs, scratch project',
        boundary: 'process, JSONL event observation',
        observable: 'reply names the selected method, contract digest and the CONFLICT; no attempted generated-code check',
        negative: 'sensitivity: a second scratch project whose signed KBD-METHOD selects a different method and digest must be reported as that method and digest, not the expected one. Not logged in under scratch CODEX_HOME: NOT_EXERCISED, never run against ~/.codex',
        isolation: `${ISOLATION} CODEX_HOME=~/.prometheus-scratch/kbd194/codex (operator login).`,
        prereq: 'operator-logged-in scratch Codex home (optional)',
        limits: 'n=1; NOT_EXERCISED is an allowed outcome.' },
      { scenario: 'SRF-002', task: '1', behavior: 'An OpenCode session under verified isolation recovers the selected method and reports the conflict',
        entry: 'opencode run --format json --standalone with scratch XDG roots and OPENCODE_DISABLE_CLAUDE_CODE=1',
        collaborators: 'opencode 2.2.0, pack skills made available inside the scratch config root, method-context.mjs',
        boundary: 'process, JSON event observation',
        observable: 'isolation preflight passes; reply names the selected method and the CONFLICT',
        negative: 'sensitivity: a second scratch project with a different signed method and digest must be reported as such. Isolation preflight shows config or auth resolving outside the scratch roots: NOT_EXERCISED (unsupported isolation), no session run',
        isolation: `${ISOLATION} XDG_CONFIG_HOME/XDG_DATA_HOME under ~/.prometheus-scratch/kbd194/opencode; OPENCODE_DISABLE_CLAUDE_CODE=1; OPENCODE_DISABLE_AUTOUPDATE=1.`,
        prereq: 'operator login inside the scratch OpenCode data root (optional)',
        limits: 'XDG honoring on macOS is UNVERIFIED until the preflight runs; n=1.' },
    ],
  },
];

for (const change of changes) {
  const dir = path.join('.kbd-orchestrator/changes', change.id);
  write(path.join(dir, 'spec.md'), [
    `# ${change.title}`,
    '',
    `Phase: ${PHASE} · Change: ${change.id}${change.dependsOn ? ` · Depends on: ${change.dependsOn.join(', ')}` : ''}`,
    '',
    '## Why',
    '',
    change.why,
    '',
    '## What changes',
    '',
    ...change.what.map(line => (line === '' || line.startsWith('```') || line.startsWith('## ') || line.startsWith('- `') || line.startsWith('Canonical') || !line ? line : `- ${line}`)),
    '',
    '## Scope',
    '',
    ...[...new Set(change.tasks.flatMap(t => t.files))].map(f => `- \`${f}\``),
    '',
    '## Constraints',
    '',
    ...change.lessons.map(l => `- ${l}`),
    '- Generated outputs: i195-02 task 2 is the named C-01 reconciliation owner (twice-run, byte-identical hashes, drift validators); i195-01 records that owner in its persisted execution evidence (execution.md and the end-task note) and claims no distribution certification.',
    '- Review before archive: i195-03 task 3 runs the independent cumulative-diff review whose receipt covers i195-01 and i195-02; neither archives before that receipt (or a recorded owner waiver).',
    '- Implementation first (CLAUDE.md, highest precedence): no tests, builds or checks before every planned production change in the phase is complete; acceptance is gathered only at the final boundary owned by i195-03.',
    '- Exclusions: #184 exit semantics, #185 trusted tool admission, the operator-authority trust model, a generic shell fence, and any edit to CLAUDE.md/AGENTS.md policy text.',
    '',
  ].join('\n'));
  write(path.join(dir, 'tasks.json'), JSON.stringify({
    changeId: change.id, schemaVersion: '1',
    tasks: change.tasks.map(t => ({ id: t.id, title: t.title, done: false, doneAt: null, doneBy: null, files: t.files, verify: null, notes: null })),
  }, null, 2));
  write(path.join(dir, 'verification.md'), [
    `# Verification — ${change.id}`,
    '',
    'Requirements, not results. Nothing below has run. Status vocabulary: PASS / FAIL / BLOCKED / NOT_EXERCISED. Exit 2 is always BLOCKED.',
    '',
    ...change.behaviors.map(b => acceptance({ ...b, change: change.id })),
  ].join('\n'));
}

const scenarioMatrix = changes.flatMap(change => change.behaviors.map(b => ({
  id: b.scenario, name: b.behavior, ownerTask: `${change.id}/${b.task}`, productionEntry: b.entry,
  realCollaborators: b.collaborators.split(/,\s*/), oracle: b.observable, negativeControl: b.negative,
  fixtures: `${b.isolation ?? ISOLATION} Fixture manifest with exact bytes and SHA-256 frozen before execution; missing manifest is BLOCKED.`,
  evidence: `evidence/<attempt>/${b.scenario}/: argv, environment allowlist, candidate/fixture/assertion hashes, stdout/stderr, observations, cleanup. Pending; no execution claimed.`,
})));

const contract = {
  contractVersion: '1',
  kind: 'kbd.adopted-contract',
  identity: { projectId: '6ac090a4-3656-4d83-8eb6-2891508196d5', runId: 'sovereign-sync-service-reliability-20260829', phaseId: PHASE, planRevision: null, contractDigest: null },
  digestNormalization: {
    algorithm: 'sha256',
    command: "jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)'",
    rationale: 'Same normalization as the prior phase contract; no self-referential digest.',
    stalenessRule: 'Reject any contract or acceptance-bearing referenced content not matching the latest non-superseded authorized adoption or candidate/fixture identity.',
  },
  adoptionSource: 'Owner answers in chat at /kbd-plan (2026-10-27), via AskUserQuestion: risk "n=1 OK; Codex/OpenCode may be NOT_EXERCISED" (Claude session scenarios must run, so an operator scratch Claude login is a boundary prerequisite); budgets "180 min / 240 calls / 2 repairs"; acknowledgement "Signed KBD-POLICY-ACK decision"; review "Labelled per-run override". Goals A-D from the owner-approved /plan reply "yes".',
  decisionOwner: 'Repository owner/operator (Travis James)',
  amendmentPolicy: 'Material changes require explicit owner adoption and a canonical superseding decision; preserve original content, budgets and evidence. No automatic scope expansion.',
  releaseClaim: {
    userOperation: 'A KBD operator in a fresh or delegated session recovers the selected method and contract and is stopped before dependent execution when governing policy conflicts with it, until the operator acknowledges the conflict.',
    supportedEnvironments: ['local macOS source CLI and skills', 'generated Claude/Codex payloads and an installer-dispatched scratch generation', 'Claude Code sessions with an operator scratch login (required); Codex and OpenCode best-effort, NOT_EXERCISED accepted'],
    currentCapabilities: 'Method/contract recovery shipped in #197; conflicts advisory only; VER-001/VER-003 unimplemented; Codex/OpenCode unobserved.',
    explicitUnknowns: [
      'Prior lifecycle budget consumption UNKNOWN; retained, never reset.',
      'Whether OpenCode 2.2.0 honors XDG roots on macOS: UNVERIFIED until the isolation preflight.',
      'Scratch logins absent at authoring; session scenarios BLOCKED until the operator logs in.',
      'Producer identity unconfigured; cross-model findings labelled unverified-producer-unknown.',
    ],
  },
  statusVocabulary: ['PASS', 'FAIL', 'BLOCKED', 'NOT_EXERCISED'],
  scenarioMatrix,
  functionalScope: [
    '#195 acceptance 2: unacknowledged governing-policy conflicts block dependent work (CONFLICT, exit 2) until a hash-bound KBD-POLICY-ACK.',
    '#195 acceptance 5: one identical selected-method core across 11 stage skills, generated payloads and installed generation.',
    '#195 acceptance 4 and 6: real-path verification (VER-001) and closure without successor (VER-003).',
  ],
  resilienceScope: {
    policy: 'Integrity, isolation and history obligations never defer on budget exhaustion; checkpoint and BLOCKED.',
    deferredOptional: ['n>1 reliability sampling', 'Windows/Linux hosts', 'OpenCode hook semantics beyond observation'],
  },
  exclusions: [
    '#184 canonical exit semantics and #185 trusted tool admission.',
    'Operator-authority trust model (Decision state carries no actor).',
    'Any edit to CLAUDE.md or AGENTS.md policy text; generic shell fence or tool interception.',
    'Posting to GitHub or closing #195 without explicit owner authorization at that time.',
  ],
  requiredSafetyChecks: [
    'Scratch-only roots for every session: PROMETHEUS_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, CLAUDE_CONFIG_DIR, CODEX_HOME, XDG_CONFIG_HOME/XDG_DATA_HOME; memory URLs at a dead port.',
    'Agents never type or copy credentials; operator performs scratch logins.',
    'No write to the real canonical journal or installed plugin generation during tests.',
    'Protected-test certification from the committed candidate before merge.',
  ],
  budgets: {
    timeMinutes: 180, toolCalls: 240, repairAttempts: 2, noProgressLimit: 2, tokenBudget: 'UNKNOWN',
    tokenFallback: 'Token usage UNKNOWN; owner-approved final-boundary time/tool/repair/no-progress ceilings apply from the boundary.',
    consumedAtAuthoring: { priorLifecycleConsumption: 'UNKNOWN; retained from issues-194-195-196, never reset', generatedCodeVerificationAttempts: 0 },
    exhaustion: 'Final-boundary ceilings; exhaustion => checkpoint + BLOCKED. Extensions need an authorized amendment and never drop mandatory scenarios.',
  },
  evidenceInvalidation: 'Invalidate only evidence whose source/fixture/assertion/contract identity changed; retain every original attempt.',
  workerRoutes: {
    agent: 'Claude Code lead as sequential implementer; no project team manifest adopted for this phase.',
    model: 'Producer claude-opus-5-5. Judge: non-Claude via liter-llm (glm-5.3 per-run override used at assess/analyze), labelled unverified-producer-unknown until producer identity is configured.',
    billing: 'No new paid jobs, remote agents or provider changes authorized.',
    subagents: 'None during planning; reviewer roles dormant until all production is complete.',
    reviewRoundsMax: 2,
  },
  boundaryPacket: {
    postImplementationOnly: true,
    singleRun: true,
    commands: [
      'node scripts/check-selected-method-core.mjs',
      'node scripts/generate-skill-system-distribution.js --check',
      GATE.replace(/ \(repository root.*$/, ''),
    ],
    expectedEffects: [
      'Runs from repository root only after every production task of i195-01 and i195-02 is done.',
      'Creates only owned scratch resources; session scenarios without an operator scratch login are BLOCKED or NOT_EXERCISED, never PASS.',
    ],
    requiredApprovals: 'Owner adoption of this contract at Plan; operator scratch logins at the boundary; separate explicit authorization to post on #195.',
    receipts: ['evidence/candidate-and-fixtures.json', 'evidence/attempts.jsonl', 'evidence/<scenario-id>/', 'evidence/cleanup.json', 'review/final/', 'closure-report.md'],
    cleanup: ['Retain evidence before cleanup; remove only resources in the run ownership manifest; never touch real user roots.'],
    stopConditions: ['Incomplete production or unadopted/stale contract', 'Isolation unavailable', 'Uncertain side effect or failed cleanup', 'Budget or no-progress limit reached'],
  },
  status: { implementationComplete: false, authoringState: 'OWNER_APPROVED_PLAN' },
};
write(path.join('.kbd-orchestrator/phases', PHASE, 'adopted-contract.json'), JSON.stringify(contract, null, 2));
console.log(JSON.stringify({ changes: changes.map(c => c.id), scenarios: scenarioMatrix.map(s => s.id) }));
