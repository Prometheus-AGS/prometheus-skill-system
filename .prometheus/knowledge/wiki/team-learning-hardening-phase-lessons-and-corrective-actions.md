---
type: Lesson
id: team-learning-hardening-phase-lessons-and-corrective-actions
title: Team Learning Hardening Phase Lessons and Corrective Actions
tags:
- vis:project
- kbd:reflect
- phase:phase-team-learning-hardening
- team-learning
- phase-reflection
- test-gates
- installer-dispatch
- performance-measurement
- cache-behavior
- spec-review
- codex-memory
links:
- treat-one-build-guard-blocked-phase-gates-as-rerunnable
sources:
- id: learning
  resource: learning:5d348f3c3f06b48f
generated:
  by: pk/1.11.0
  at: 2026-10-05T09:22:32.902672+00:00
created_at: 2026-10-05T09:22:32.902672+00:00
updated_at: 2026-10-05T09:22:32.902672+00:00
revision: 0
content_hash: d9e4ad5b61889e05f6754615321a413fe62f5a357420ff049a1f70d97b7623d8
---

## Key deltas

- A **CRITICAL** shipped despite all per-change gates passing: Change 02 ran 22 + 20 green checks, but `install-skills-flat.sh` never applied the Codex memory setting. The call was in `install_to_codex()`'s `else` arm, which is reached on uninstall, not the install path. The test sourced the helper directly and never exercised installer dispatch. Fix #154 added a negative-controlled structural regression check that fails against the old installer.[^learning]
- Change 05 initially failed 4 of 6 cache tests. `add_memory` performed duplicate detection by calling `search_memories`, which embedded every written memory twice and populated the query cache from the write path. The spec criterion that “the write path never touches the cache” exposed the defect. Fix `f919469` removed the write-path cache mutation and a redundant embedding on every write.[^learning]
- Change 05's performance premise was weaker than assumed. One query embedding cost 0.014 s idle and 0.340 s at p95 under 8 concurrent writers. The cache saved about 1 s of a 2.8 s recall at loaded p95, but embeddings were not the dominant cause of the previous phase's watchdog misses, which occurred near load 250.[^learning]
- Change 07's first real-service gate was **BLOCKED** by a bad test glob. The test expected `cache/cortex/<version>`, while the real layout is `cache/cortex/cortex/<version>`. The real run then exposed a product defect hidden by the stub: Cortex exits when stdin closes, so the mirror could lose lessons. A detached feeder fixed the issue.[^learning]
- Change 09 failed on the phase-boundary gate because the test read uninitialised `tools/*` submodule gitlinks as stray staged paths, making the verdict depend on checkout state. Fix #155 corrected this.[^learning]
- Spec review required two rounds. Round 1 found three critical issues: Change 06's reconcile feature had no gate; Change 04's check read live `:23001`; Change 02 symlinked the real Codex `auth.json`.[^learning]
- Branch cleanup broke machine refresh when a worktree was removed. Claude Code's `prometheus-skill-pack` marketplace references in `~/.claude/settings.json` and `~/.claude/plugins/known_marketplaces.json` pointed at the `dist-ship-script-lib` worktree, and Codex's `~/.codex/config.toml` pointed at `deploy-main`. Removing `dist-ship-script-lib` made `claude plugin marketplace update` fail with `ENOENT`. Repointing to the main checkout failed because the updater refuses a dirty source tree and the KBD runtime writes tracked files in main. A detached HEAD also failed because `update-skill-pack.sh` runs `git pull --ff-only`. Final arrangement: `deploy-main` is the single install worktree, on a local-only `deploy/main` branch that tracks `origin/main` and is never pushed; all three references point at it. Config backups were kept as `*.bak-20261005T*`. This is the one exception to “main only”.[^learning]
- Codex regenerated `memory_summary.md` despite `generate_memories = false`. The live `codex.memories` doctor check found a 7.8 KB summary written at 00:48. After archiving the file, the doctor passed. The setting alone is not sufficient; the doctor check is the effective guard. Investigate whether Codex consolidation also requires `use_memories` or another key.[^learning]
- Change 02's gate was blocked once by a concurrent Cargo build from outside the session. The one-build guard worked as designed; treat this as a rerunnable transient, consistent with [Treat One-Build Guard BLOCKED Phase Gates as Rerunnable](/treat-one-build-guard-blocked-phase-gates-as-rerunnable.md).[^learning]

## Root causes

- Acceptance tests exercised components—the helper, doctor, and script—but not the production entry point's control flow. Under the integration-only rule, the claim “both installers apply it” required a test through installer dispatch, or an equivalent structural dispatch check.[^learning]
- Read and write paths shared `search_memories`, so a read-path optimisation leaked into writes. The worker could not detect this without compiling; the spec's write-path criterion surfaced it.[^learning]
- Performance analysis inferred the bottleneck from code shape (“4 embeddings per recall”) instead of measuring first. The spec's measurement-first task converted the wrong assumption into a recorded finding rather than an oversized build.[^learning]
- Tests encoded environment assumptions, including cache layout and submodule initialisation, instead of discovering the real environment.[^learning]
- The first spec draft copied prior-phase gate patterns that touched live services and real credentials.[^learning]

## Corrective actions

1. **Entry-point and installer features must be tested through real dispatch.** If a full install is too heavy, require a structural check on dispatch, as in #154. Add this rule to the spec template's acceptance criteria.[^learning]
2. **Read-path caches must declare allowed callers.** Tests must assert that write-path counters remain unchanged. Change 05's test is the model.[^learning]
3. **Performance changes must measure before sizing.** Keep “measure first, with a stop rule” as task 1 whenever analysis names a bottleneck.[^learning]
4. **Tests must discover environment shape.** Glob the real layout, skip gitlinks, and avoid assertions on shared live state that another session can mutate. Change 07's `~/.cortex/memory.db` mtime check remains a known flake risk.[^learning]
5. **Spec review must continue checking for live services and real credentials in gates.** This review step caught the Round 1 critical issues.[^learning]
6. **Ledger discipline worked.** The phase used `begin-task`/`end-task` throughout and had no drift. Change 06 now makes `mark-done` sync and adds `kbd-apply reconcile`, which `/kbd-reflect` runs at step 4.[^learning]

[^learning]: phase team-learning-hardening reflection lesson