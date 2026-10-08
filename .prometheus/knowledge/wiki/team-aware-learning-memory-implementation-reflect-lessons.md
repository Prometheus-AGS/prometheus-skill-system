---
type: Lesson
id: team-aware-learning-memory-implementation-reflect-lessons
title: Team-Aware Learning Memory Implementation Reflect Lessons
tags:
- vis:project
- kbd:reflect
- phase:team-aware-learning-memory-impl
- team-aware-learning
- memory-system
- phase-reflect
- project-tracking
- merge-conflicts
- cadence-engine
- integration-testing
sources:
- id: learning
  resource: learning:d630eaafa139fe7b
generated:
  by: pk/1.11.0
  at: 2026-10-04T22:36:53.583844+00:00
created_at: 2026-10-04T22:36:53.583844+00:00
updated_at: 2026-10-04T22:36:53.583844+00:00
revision: 0
content_hash: 536091808c15033a7c32a117aeccb140ce455ae765cd3af0af3c0f8267339a41
---

## Phase

`team-aware-learning-memory-impl` reached reflect with all `tasks.json` files showing done, but the canonical runtime ledger had drifted and required repair.[^learning]

## Key findings

### Ledger drift

- At reflect time, `progress.json` recorded only **13 of 25 changes**, while every `tasks.json` showed all tasks complete.[^learning]
- From B5 onward, tasks were closed with `kbd-apply mark-done`; this flips task flags but does **not** sync the runtime ledger.[^learning]
- Reflect repair replayed every task through `begin-task` / `end-task`.[^learning]
- Root cause: `mark-done` was used as a shortcut because `end-task` fires hooks and is slow; no check compared `tasks.json` with canonical progress before reflect.[^learning]

### Merge and rebase churn

Six PRs required rebases after a sibling merge: **B5, B6, C2, B7, C3b, C4**.[^learning]

Most conflicts were in generated files:

- `hooks/*.json`
- release manifests
- `dist/**`

Two conflicts were real source conflicts:

- `learning_recall.py` imports: C2 `prompt_gap` vs. B7 `learning_route`
- `team.schema.json` / `validation.mts`: B6 `agentMemory` vs. C4 `card`

Contributing factors:

- Every change regenerated the same shared outputs, so parallel branches conflicted regardless of source scope.[^learning]
- The no-stacked-PRs rule, adopted after auto-closure of #33/#127, serialized delivery: each merge forced the next branch to rebase.[^learning]

### Stale-head merge

- mini #34 merged at stale pre-rebase commit `5a74398`.[^learning]
- Follow-up commit `9c08636`—Codex exclusion of D1a's Claude-only hook plus its test—never reached `main`.[^learning]
- mini `main` carried a failing test until fix PR mini #38.[^learning]
- Root cause: no post-merge check verified that `main` contained the PR branch's final head.[^learning]

### Live and integration defects missed by mocks

Live and integration runs found defects earlier gates and mocked tests missed:[^learning]

- Doctor rejected every Codex hook as "not pinned".
  - Generated hooks had moved to single-string commands in `abf0ade`.
  - `collect_hook_commands` still read `command` + `args`.
  - Fixed in #137.
- `team-request` failed on the first request to any team because `gh issue create --label` errors when the label is missing.
  - Fixed in #144.
- Codex forks the parent thread into every spawned child, so the `SessionStart` lead view leaked `@lead` text into `ui_dev`.
  - The Codex main-thread view was changed to digest-only.

Root causes:

- The doctor's hook-graph fixture had not been exercised since `abf0ade` changed the emitter shape; its failure on `main` was incorrectly dismissed as pre-existing.[^learning]
- `team-request` had only fake-`gh` coverage.[^learning]
- Codex context-forking behavior is undocumented and required a real two-harness run with inherited-history assertions to reveal.[^learning]

### Cadence engine delivery friction

The cadence engine conflicted with the delivery procedure in four ways:[^learning]

1. Iteration 1 froze the constantly mutated `main` checkout and was refused three times with `Source changed after freeze`.
2. After a failed `finish`, every `start` is refused; the only repair path is `ready` on the failed iteration.
3. `finish` refuses a candidate whose sources are unchanged, forcing skipped hours.
4. `--auto` parity called the lock-holding cadence CLI from inside its own checkpoint; the call failed silently and fell back to `N=1`, so every `--auto` run was a full refresh until iteration 4.

Root causes:

- The procedure was written against the `main` checkout before freeze semantics were understood.[^learning]
- The parity fallback `|| echo 1` hid lock failures instead of failing loudly.[^learning]

### Broken `main` blocked delivery

`main` was broken twice by other work, blocking delivery:[^learning]

- Stale generated `dist/` for delivery-cadence—16 files with source at 1.2.1—made `update-skill-pack.sh` refuse installation until #142.
- A fast-forwarded submodule gitlink left the deploy worktree dirty.

Root cause: generated-output drift was checked only when someone regenerated outputs, so a source-only commit could land with stale `dist/`; the refresh procedure also did not sync submodules after fast-forwarding.[^learning]

### Gate amendments

Four gates were amended mid-phase, each with a recorded reason:[^learning]

- A5a: `prometheus-exec` path-dependent hash, later fixed upstream by #132.
- B3: hook-matrix count.
- D1b: pin check, after mini #32 was retargeted from v1.10.0 to v1.11.0.
- D2: mini 19-test baseline.

Root cause: gates encoded exact versions and counts that legitimate later decisions changed.[^learning]

### Operator-dependent critical path

Operator-only steps defined the critical path but were requested late:[^learning]

- mini pin A5b waited on the operator.
- C4 task 4 waited on a sandbox repo that did not exist until operator approval to create it.
- B6 task 3, live `MEMORY.md` partition, and the Codex memory decision waited on approval.

Root cause: the plan treated operator steps as normal tasks and assigned no lead time.[^learning]

## Corrective actions

1. Reconcile `tasks.json` against canonical progress at every cadence iteration, not only at reflect.[^learning]
2. Close tasks with `begin-task` / `end-task`, not `mark-done`, when canonical progress must be updated.[^learning]
3. Add a `rebase-regenerate` helper to the pack:
   - If every conflicted path is generated, take the base side, rerun both generators, and continue the rebase.
   - Stop on any source conflict.
4. After every merge, confirm `main` contains the PR's final head:

```bash
git merge-base --is-ancestor <pr-head> origin/main
```

   If it does not, open the missing commits as a fix PR immediately.[^learning]
5. Keep one emitter and one validator per generated format, and fail the validator's own test when the emitter changes shape.[^learning]
6. Treat a pre-existing failure on `main` as a defect to fix, not a reason to skip validation.[^learning]
7. For every external integration, run at least one test against the real service before claiming done; fakes hide preconditions such as required labels and inherited context.[^learning]
8. Version the cadence refresh procedure in the repo instead of keeping it local in `.prometheus/cadence/procedures`.[^learning]
9. Make cadence refresh fail loudly on parity or lock errors.[^learning]
10. Always run `git submodule update` after fast-forwarding.[^learning]
11. Add `check:distribution` to the merge gate for changes that touch generated sources, preventing source-only commits from leaving stale `dist/` on `main`.[^learning]
12. Write gates against ranges and invariants—such as `pk >= 1.10.0` or "an ancestor of v1.10.0"—rather than exact pins or counts.[^learning]
13. Raise operator-dependent work—pins, repos, live-data approvals—at plan time with lead times.[^learning]

[^learning]: Reflection lesson from `learning:d630eaafa139fe7b`.