# Phase Reflection: team-aware-learning-memory-impl

**Project:** prometheus-skill-pack (with prometheus-knowledge-rs, surreal-memory-server, prometheus-skills-mini)
**Date:** 2026-10-04
**Phase completion:** 100% (25 of 25 changes, 82 of 82 tasks, each with an integration gate)
**Changes completed:** 25 / 25

## Delta

1. **The canonical ledger drifted from the change files.** At reflect time `progress.json` recorded 13 of 25 changes, while every `tasks.json` showed all tasks done. From B5 onward, tasks were closed with `kbd-apply mark-done`, which flips the flag but does not sync the runtime ledger. Fixed at reflect by replaying every task through `begin-task`/`end-task`.
2. **Merge conflicts took far more time than planned.** Six PRs needed rebases after a sibling merged: B5, B6, C2, B7, C3b and C4. Most conflicts were in generated files (hook bundles, release manifests, `dist/**`). Two were real source conflicts:
   - `learning_recall.py` imports: C2's `prompt_gap` against B7's `learning_route`.
   - `team.schema.json` / `validation.mts`: B6's `agentMemory` against C4's `card`.
3. **A PR merged at a stale head.** mini #34 merged at its pre-rebase commit `5a74398`. The follow-up `9c08636` (Codex exclusion of D1a's Claude-only hook, plus its test) never reached main, so mini main carried a failing test until fix PR mini #38.
4. **Live and integration runs found defects that mocked and earlier gates had missed:**
   - The doctor rejected every Codex hook as "not pinned". Generated hooks moved to single-string commands in `abf0ade`, but `collect_hook_commands` still read `command`+`args`. Fixed in #137.
   - `team-request` failed on the first request to any team because `gh issue create --label` errors when the label is missing. Fixed in #144.
   - Codex forks the parent thread into every spawned child, so the SessionStart lead view leaked `@lead` text into ui_dev. The Codex main-thread view was made digest-only.
5. **The cadence engine fought the delivery procedure. Four distinct failures:**
   - Iteration 1 froze the constantly mutated main checkout and was refused three times ("Source changed after freeze").
   - After a failed finish, every `start` is refused. The only repair path is `ready` on the failed iteration.
   - `finish` refuses a candidate whose sources are unchanged, which forced skipped hours.
   - My `--auto` parity called the lock-holding cadence CLI from inside its own checkpoint. The call failed silently and fell back to N=1, so every `--auto` run was a full refresh until iteration 4.
6. **Main itself was broken twice by other work, and each break blocked delivery:**
   - Stale generated `dist/` for delivery-cadence (16 files, source at 1.2.1) made `update-skill-pack.sh` refuse to install until #142.
   - A fast-forwarded submodule gitlink left the deploy worktree "dirty".
7. **Four gates were amended mid-phase, each with a recorded reason:**
   - A5a: the prometheus-exec path-dependent hash, later fixed upstream by #132.
   - B3: the hook-matrix count.
   - D1b: the pin check, after mini#32 was retarget from v1.10.0 to v1.11.0.
   - D2: the mini 19-test baseline.
8. **Operator-only steps defined the critical path:**
   - The mini pin (A5b) waited on the operator.
   - C4 task 4 waited on a sandbox repo that did not exist until the operator approved creating it.
   - B6 task 3 (live MEMORY.md partition) and the Codex memory decision waited on approval.

## Root Cause

1. **Ledger drift (1).** `mark-done` was used as a shortcut because `end-task` fires hooks and is slow. Nothing checks tasks.json against canonical progress before reflect, so the drift was invisible. Recurrence of the earlier lesson "query live stores first, don't trust a projection" (feedback_GLOBAL_mutating_verify_and_store_reality).
2. **Conflict churn (2).**
   - Every change regenerates the same shared generated files: `hooks/*.json`, the release manifest and `dist/**`. Two changes developed in parallel therefore always conflict, whatever their source scope.
   - The no-stacked-PRs rule, adopted after #33/#127 auto-closed, made the work serial: each merge forces a rebase of the next branch.
3. **Stale-head merge (3).** The rebase was pushed after the PR was already approvable, and no post-merge check compared main with the branch's final head.
4. **Defects missed before live runs (4).**
   - The doctor's hook-graph test fixture had not been exercised since `abf0ade` changed the emitter; it failed on main and was dismissed as pre-existing.
   - `team-request` had only fake-`gh` coverage.
   - The Codex context-forking behaviour is undocumented, and only a real two-harness run with inherited-history assertions revealed it.
5. **Cadence friction (5).**
   - The procedure was written against the main checkout before the engine's freeze semantics were understood.
   - The parity fallback (`|| echo 1`) was written as a safety net and hid the lock failure.
   - Recurrence: "silent fallbacks mask failures" (feedback_GLOBAL_set_e_chains_and_hash_bound_payloads).
6. **Broken main (6).** Generated-output drift is checked by a gate that only runs when someone regenerates, so a source-only commit can land unchecked. The refresh procedure did not sync submodules after a fast-forward.
7. **Gate amendments (7).** The gates encoded exact versions and counts that legitimate later decisions changed, such as the retarget to v1.11.0.
8. **Operator path (8).** The plan treated operator steps as tasks, but none of them had a lead time, so they were asked for late.

## Corrective Actions

1. Reconcile `tasks.json` against canonical progress at every cadence iteration, not only at reflect. Use `begin-task`/`end-task` (not `mark-done`) when closing tasks.
2. Add a `rebase-regenerate` helper to the pack. When every conflicted path is generated, it takes the base side, reruns both generators, and continues the rebase. It stops on any source conflict.
3. After every merge, confirm main contains the PR's final head: `git merge-base --is-ancestor <pr-head> origin/main`. If it doesn't, open the missing commits as a fix PR immediately.
4. [GLOBAL] Keep one emitter and one validator per generated format, and fail the validator's own test when the emitter changes shape. A "pre-existing failure" on main is a defect to fix, not to skip.
5. [GLOBAL] For every external integration, run at least one test against the real service before claiming done. Fakes hide preconditions such as "label must exist" or "context is inherited".
6. Version the cadence refresh procedure in the repo instead of keeping it local in `.prometheus/cadence/procedures`. Make it fail loudly on any parity or lock error, and always run `git submodule update` after fast-forwarding.
7. Make `check:distribution` part of the merge gate for any change that touches a generated source, so source-only commits cannot leave `dist/` stale on main.
8. Write gates against ranges and invariants (for example "pk ≥ 1.10.0", "an ancestor of v1.10.0") rather than exact pins or counts.
9. Raise operator-dependent steps (pins, repos, live-data approvals) at plan time with lead times, not when the change reaches them.

## Recalled Lessons

- Recurred: "don't trust a projection; query the live store" — `progress.json` lagged the change files (Delta 1).
- Recurred: "silent fallbacks and `&&` chains hide failures" — the cadence `--auto` parity fallback (Delta 5).
- Recurred: "stacked PRs auto-close when the base branch is deleted". It was applied all phase as the no-stacked-PRs rule, and that rule caused the rebase churn (Delta 2).
- Applied: "one cargo build at a time" — refreshes and agents serialised cargo. One agent overlapped a single `cargo check` with an unrelated process; it finished cleanly.
- Applied: "hook stdin is a socket / Codex ignores args" — mini's Codex hooks were shipped as single-string commands (mini #34) from the start.
- Applied: "never edit plugin caches" — every install went through `update-skill-pack.sh` from the deploy worktree.
- (prior-context.md itself held no applicable lessons; it recalled five generic "task" events. The recall pipeline did not surface the memory-file lessons above. That is itself a delta for the next phase.)

## Goals

| Goal | Status | Notes |
| ---- | ------ | ----- |
| A1–A4 upstream prerequisites and the A5 v1.10.0 pin in both repos | MET | pk v1.10.0 and surreal-memory v1.10.0 were tagged. The skill pack was pinned in A5a. Mini was pinned in A5b by the operator at v1.11.0 (pk) and v1.10.0 (surreal), mini #37. |
| B1–B4: identity, envelope writes, per-agent recall, KBD memory loop | MET | #128–#133 merged. `test-learning-write` 6/6 and `test-kbd-memory-loop` 13/13 against live services. |
| SubagentStart alpha gate (Claude Code + Codex, 0 leaks, budgets, Codex trust path) | MET | `test-subagent-delivery` 7/7 across both harnesses, through the Codex native plugin path with no fallback needed. Claude delivered ~7.9k chars (8k budget); Codex ~6.7k chars / 1,910 tokens. Under extreme load a delivery can time out; it is now logged as `timedOut`. |
| Beta gate: MEMORY.md ≤ 4 KB, path routing, team digest, bytes vs baseline | MET | MEMORY.md went from 15,580 to 3,974 B. `--require-reduction`: Claude 3,974 B (baseline 14,336), Codex 0 B (baseline 11,059) after Codex memory generation was disabled with operator approval. `test-team-awareness` 11/11 across both harnesses. |
| C1–C4, D1–D3 each with an integration gate | MET | All verify.sh gates rc 0. C4 ran live against `Prometheus-AGS/team-sandbox`, 8/8. D1b passed on mini main after mini #38. |

## Delivered Changes

- A1–A4 (pk context scoring, worker attribution, tags/type; surreal lean search/categories/rekey) — prometheus-knowledge-rs, surreal-memory-server
- `change-tli-a5a` pin v1.10.0 — skill pack; `change-tli-a5b` mini pin — operator (mini #37)
- `change-tli-b1`…`b4` — identity resolver, namespaced matchers, learning_write, learning_recall plus the KBD loop (#128–#133)
- `change-tli-b5` SubagentStart delivery (#134); `b6` file-tier reduction (#135) plus the live partition; `b7` routing, digest and main-thread view (#138)
- `change-tli-c1a` / `c3a` pk promotion and skill discovery (pk #36, #38, release v1.11.0 in pk #39); `c1b` routing and review (#142); `c2` gap routing (#136); `c3b` skill-candidate surfacing (#143); `c4` team cards and cross-repo requests (#144)
- `change-tli-d1a` / `d1b` / `d2` mini port, pk tags and timeouts (mini #33, #36, earlier #29); `d3` memory chain and Cortex mirror (#140)
- `change-tli-e1` pk v1.11.0 tag and skill-pack pin (#139)
- Related fixes during the phase: doctor learning checks for issue #118 (#137); mini Codex hooks (mini #34) and exclusion fix (mini #38); dropping the duplicate kbd-open section (#145)
- Delivered by: Claude Code (Opus orchestrating, Sonnet agents implementing), with the operator merging every PR

## Technical Debt

- `scripts/memory-index-partition.py` keeps file order. The live partition needed a manual priority reorder first: current phase, then feedback, then archives, then older projects. The tool should rank entries itself.
- Codex memory generation is disabled in the operator's `~/.codex/config.toml` by hand. The installer neither sets it nor checks it, so a fresh machine will regrow `memory_summary.md`.
- `.prometheus/cadence/procedures/refresh-skill-pack.sh` is local and unversioned, and it contains the parity and submodule fixes made during this phase.
- `skills/process/agent-team-creator/tests/memory-envelope.integration.mjs` publishes to the live `:23001` service rather than a scratch server. It flaked twice at load average ~250.
- The D3 Cortex mirror has been tested only against a stub server. Real Cortex 2.0.3 has no tag field, so the role tag goes into `context`.
- SubagentStart delivery is skipped (logged, not retried) when recall misses the 3.5 s watchdog under extreme load. B4's recall embeds the same query several times.
- `kbd-apply mark-done` leaves the canonical ledger stale. Nothing warns about it.

## Architecture Integrity

- AGENTS.md violations: NONE found. Generated files were always regenerated, never hand-edited. Bash 3.2 was kept for launchd scripts.
- Constraint violations:
  - One overlapping `cargo check`, from an agent, caught and reported.
  - The no-stacked-PRs rule was held, except that C3b was built on C1b's branch, and its PR was opened only after #142 merged.
  - Implementation-first and integration-only gates were followed throughout.

## Cross-Tool Coordination Notes

- Progress tracking: GAPS FOUND. `progress.json` lagged at 13/25 while the change files were at 25/25 (Delta 1), and was reconciled at reflect. `execution.md` was the reliable record.
- Handoff quality: CLEAR, with four caveats:
  - One agent's report repeated three times.
  - One agent's PR merged before its rebase push landed (mini #34).
  - Agents' "pre-existing failure" claims needed checking; one was a real main defect (doctor hook parsing).
  - Agents twice left orphaned scratch surreal servers. The root cause was fixed by making `test-learning-write.sh` `exec` its server.

## Lessons Learned

- `kbd-apply mark-done` flips the task flag but does not sync the canonical KBD ledger. Close tasks with `begin-task`/`end-task`, or reconcile before reflect.
- delivery-cadence freezes the sources named in `ready`. Point them at a dedicated deploy worktree, never the main checkout. A failed `finish` can only be repaired with `ready` on that same iteration, and `finish` refuses unchanged sources.
- The deploy worktree needs `git submodule update` after every fast-forward, or `update-skill-pack.sh` refuses a "dirty" tree.
- [GLOBAL] Codex forks the parent thread's history into every spawned child. Anything injected into the parent at SessionStart reaches every subagent, so keep parent injections free of role-private or lead-only text.
- [GLOBAL] Codex ignores a hook's `args`. Ship the whole invocation as one `command` string, and update every validator and doctor that parses hooks in the same change.
- [GLOBAL] `gh issue create --label X` fails when X does not exist. Ensure the label first with `gh label create X --force`, which is idempotent.
- [GLOBAL] Tests that replace HOME cannot use gh credentials, because both the gh config and the macOS keychain are found through HOME. Resolve `gh auth token` under the real environment and pass it as `GH_TOKEN`.
- [GLOBAL] GitHub's list-by-label lags a just-created issue by seconds. Poll instead of asserting immediately.
- [GLOBAL] After a PR merges, verify main contains the PR's final head commit. A push that lands after approval can be left out of the merge.
- [GLOBAL] A stray `__pycache__` makes byte-exact generated-output checks report "stale". Delete caches before checking.
- [GLOBAL] Never call a CLI that takes a lock from inside an operation that already holds that lock, and never paper over such a failure with a default value.
- [GLOBAL] When truncating an always-loaded index to a byte budget, rank entries by importance first. File order keeps the oldest entries and drops the newest rules.
- [USER] The operator merges every PR, wants the full KBD status at the end of every turn, and wants operator-only actions (tags, repos, live-data changes) asked explicitly.

## Next Phase Seed

`phase-team-learning-hardening`. Top three priorities:

1. **Install-time automation of this phase's manual steps:**
   - priority-ranked MEMORY.md partitioning in `memory-index-partition.py`;
   - the Codex `[memories] generate_memories = false` setting applied and checked by the installer and the doctor;
   - a versioned, fail-loud cadence refresh procedure.
2. **Test isolation and load robustness:**
   - move `memory-envelope.integration` to a scratch server;
   - reduce B4's repeated query embedding so SubagentStart recall meets its watchdog under load;
   - add a ledger/tasks reconciliation check to the cadence iteration.
3. **Real-world verification:**
   - the Cortex mirror against real Cortex;
   - recall quality in `prior-context.md` (it surfaced no applicable lessons this phase);
   - a `rebase-regenerate` helper for generated-only conflicts.

## Codify as Skill?

- **rebase-regenerate:** resolve a rebase whose conflicts are all in generated outputs. Take the base side, rerun both generators, continue, and stop on any source conflict. This happened at least six times this phase.
- **post-merge head check:** after each merge, compare the PR head with main and open the missing commits as a fix PR. Small enough to fold into the cadence iteration rather than a standalone skill.

## Context for Next Phase

Use this file as prior context for the next `/kbd-assess` invocation.
