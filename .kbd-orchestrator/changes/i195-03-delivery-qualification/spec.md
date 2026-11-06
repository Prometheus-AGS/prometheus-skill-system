# Author and run the issue #195 closeout evidence; close honestly

Phase: issue-195-closeout · Change: i195-03-delivery-qualification · Depends on: i195-01-policy-conflict-ack, i195-02-single-method-reference

## Why

VER-001 and VER-003 are stubs (scripts/tests/lib/kbd-194-196-sessions.mjs:314-316), so #195 acceptance 4 and 6 have no evidence; Codex and OpenCode were never exercised; the session driver leaves PROMETHEUS_PLUGIN_ROOT unset. This change owns all test authoring and the single final boundary, after changes 01 and 02 are complete.

## What changes

- Task 1 runs only after every production task of changes 01 and 02 is done (host policy: implementation first).
- Session driver isolation: set a scratch PROMETHEUS_PLUGIN_ROOT for every harness. Add an OpenCode harness: `opencode run --format json --standalone` with scratch XDG_CONFIG_HOME/XDG_DATA_HOME, OPENCODE_DISABLE_CLAUDE_CODE=1, OPENCODE_DISABLE_AUTOUPDATE=1, preceded by an isolation preflight that confirms config and auth resolve under the scratch roots; otherwise the scenario is NOT_EXERCISED (unsupported isolation), never run against real config.
- Implement VER-001 and VER-003 bodies in place of stub(); add CON-001..003, REF-001, SRF-001, SRF-002 and register them so the driver iterates this phase's contract.
- Task 2 runs the boundary packet once; fixes go in coherent batches with one confirming rerun, within the adopted budget.
- Task 3: cross-model review of the cumulative diff via liter-llm `dispatch-judge.sh` with the per-run judge override (`PROMETHEUS_KBD_JUDGE_*`, e.g. glm-5.3), because `~/.prometheus/kbd/models.toml` sets judge = critic and refuses dispatch; this cumulative review is the review receipt that i195-01 and i195-02 need before archive (judge not Claude-family; label recorded honestly), `npm run validate:strict`, `npm run check:distribution`, protected-test certification from the committed candidate.
- Task 4: typed canonical closure of this phase with the six claims kept distinct and no successor; record a partial disposition for issues-194-195-196 that references this phase; draft the #195 acceptance-to-evidence comment. Posting the comment and closing the issue are outward-facing and need explicit owner authorization at that time; record the request and its outcome (posted, or declined with reason) in closure-report.md.

## Scope

- `scripts/tests/lib/kbd-194-196-sessions.mjs`
- `scripts/tests/lib/kbd-195-closeout.mjs`
- `scripts/tests/kbd-194-196.integration.mjs`
- `.kbd-orchestrator/phases/issue-195-closeout/evidence/**`
- `.kbd-orchestrator/phases/issue-195-closeout/review/final/**`
- `.kbd-orchestrator/phases/issue-195-closeout/closure-report.md`

## Constraints

- Recalled lesson: "A CRITICAL shipped despite all per-change gates passing … the installer never applied the Codex memory setting" — surface checks observe the installer-dispatched payload, never source text alone.
- Recalled lesson: "`kbd-apply mark-done` updates the task flag but does not synchronize the canonical KBD ledger" — close tasks only with kbd-apply begin-task/end-task.
- Generated outputs: i195-02 task 2 is the named C-01 reconciliation owner (twice-run, byte-identical hashes, drift validators); i195-01 records that owner in its persisted execution evidence (execution.md and the end-task note) and claims no distribution certification.
- Review before archive: i195-03 task 3 runs the independent cumulative-diff review whose receipt covers i195-01 and i195-02; neither archives before that receipt (or a recorded owner waiver).
- Implementation first (CLAUDE.md, highest precedence): no tests, builds or checks before every planned production change in the phase is complete; acceptance is gathered only at the final boundary owned by i195-03.
- Exclusions: #184 exit semantics, #185 trusted tool admission, the operator-authority trust model, a generic shell fence, and any edit to CLAUDE.md/AGENTS.md policy text.
