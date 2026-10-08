---
type: Lesson
id: team-learning-hardening-phase-reflection
title: Team Learning Hardening Phase Reflection
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
sources:
- id: learning
  resource: learning:10fc1e6bc266f681
generated:
  by: pk/1.11.0
  at: 2026-10-05T07:07:35.700107+00:00
created_at: 2026-10-05T07:07:35.700107+00:00
updated_at: 2026-10-05T07:07:35.700107+00:00
revision: 0
content_hash: b297aa191ea23c1f343179250a1795f7f184aa82fd392580167a6e1981123757
---

## Phase outcomes

- A shipped **CRITICAL** escaped all per-change gates: Change 02 ran 22 + 20 green checks, but `install-skills-flat.sh` never applied the Codex memory setting because the relevant call was in `install_to_codex()`'s `else` arm, a path reached only on uninstall. The test sourced the helper directly and did not exercise installer dispatch. Fix #154 added a negative-controlled structural regression check that fails against the old installer.[^learning]
- Change 05 initially failed 4 of 6 cache tests. `add_memory` checked duplicates by calling `search_memories`, which embedded every written memory twice and populated the query cache from the write path. The fix in `f919469` removed the write-path cache mutation and a redundant embedding on every write.[^learning]
- Change 05's performance premise was overstated: one query embedding cost 0.014 s idle and 0.340 s at p95 under 8 concurrent writers. The cache saved about 1 s from a 2.8 s recall at loaded p95, but embeddings were not the dominant cause of the prior phase's watchdog misses, which occurred near load 250.[^learning]
- Change 07's first real-service gate was blocked by a bad glob: the test expected `cache/cortex/<version>`, while the real layout was `cache/cortex/cortex/<version>`. The real run then exposed a stub-hidden product defect: Cortex exited when stdin closed, so the mirror could lose lessons. The fix used a detached feeder.[^learning]
- Change 09 failed at the phase-boundary gate because it treated uninitialized `tools/*` submodule gitlinks as stray staged paths, making the verdict checkout-state dependent. Fixed in #155.[^learning]
- Spec review required 2 rounds. Round 1 found 3 critical issues: Change 06's reconcile feature had no gate, Change 04's check read live `:23001`, and Change 02 symlinked the real Codex `auth.json`.[^learning]
- Change 02 was blocked once by a concurrent external Cargo build; the one-build guard behaved as designed.[^learning]

## Root causes

- **Entry-point coverage gap:** acceptance tests covered components—the helper, doctor, and script—but not production installer control flow. Under the repository's integration-only rule, the claim that "both installers apply it" required a dispatch-level test or equivalent structural check.[^learning]
- **Read/write path coupling:** `search_memories` was shared between read and write paths, so a read-path cache optimization leaked into writes. The spec's explicit "write path never touches cache" criterion surfaced the issue only after compilation.[^learning]
- **Unmeasured performance assumption:** analysis inferred the bottleneck from code shape (`4` embeddings per recall) before measuring. Making measurement task 1 converted the wrong assumption into a recorded finding rather than an oversized build.[^learning]
- **Environment assumptions in tests:** tests hard-coded cache layout and submodule initialization assumptions instead of discovering environment state.[^learning]
- **Unsafe inherited gate patterns:** the first spec draft reused prior patterns that touched live services and real credentials.[^learning]

## Corrective actions

1. **Installer and entry-point features must test real dispatch.** If full install is too heavy, use a structural dispatch check like #154. Add this rule to the spec template's acceptance criteria.[^learning]
2. **Read-path caches must declare allowed callers.** Tests must assert write-path counters remain unchanged; Change 05's test is the model.[^learning]
3. **Performance bottleneck claims require measurement before sizing.** Keep "measure first, with a stop rule" as task 1 for performance changes.[^learning]
4. **Tests should discover environment state.** Glob the actual layout, skip gitlinks, and avoid assertions against shared live state that other sessions can mutate. Change 07's `~/.cortex/memory.db` mtime check remains a known flake risk.[^learning]
5. **Spec review must continue checking for live services and real credentials in gates.** This review practice caught the critical issues in the first spec-review round.[^learning]
6. **Ledger discipline remains effective.** The phase used `begin-task`/`end-task` throughout without drift. Change 06 now makes `mark-done` sync and adds `kbd-apply reconcile`, which `/kbd-reflect` runs at step 4.[^learning]

[^learning]: Phase `phase-team-learning-hardening` reflection lesson.