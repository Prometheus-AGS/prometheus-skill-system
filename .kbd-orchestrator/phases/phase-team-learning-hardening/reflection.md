# Reflection: phase-team-learning-hardening

- **Delivered:** 9 of 9 changes and 24 of 24 tasks, plus 2 follow-up fixes from the phase review.
  - Skill pack: PRs #146–#155, all merged (main `f35fe2a`).
  - surreal-memory-server: PR #46 merged and tagged v1.10.1 (`0cf8c7e`).
  - Mini parity: mini #39 (delivery-cadence 1.2.3) and mini #40 (agent-team Codex/Claude memory settings), both merged (mini main `6044ecd`).

## Delta

1. **The phase review found a shipped CRITICAL that every per-change gate had passed.** Change 02's gate ran 22 + 20 checks green, yet `install-skills-flat.sh` never applied the Codex memory setting. The call sat in `install_to_codex()`'s else-arm, and on the install path that function is never called; it only runs on uninstall. The test sourced the helper directly and never exercised the installer's dispatch. It was fixed in #154 with a structural regression check, negative-controlled: the old installer fails it.
2. **Change 05's first compile failed 4 of 6 cache tests.** The cause was `add_memory`'s duplicate check, which searched by content through `search_memories`. It embedded every written memory twice and filled the query cache from the write path. The spec's "the write path never touches the cache" criterion caught it. The fix (`f919469`) also removed a redundant embedding on every write.
3. **Change 05's premise was weaker than the analysis assumed.** One query embedding costs 0.014 s idle and 0.340 s at p95 under 8 concurrent writers. The cache saves about 1 s of a 2.8 s recall at loaded p95, but embedding is not the dominant cost of last phase's watchdog misses, which happened at load around 250.
4. **Change 07's first real-service gate run was BLOCKED by a test glob.** The test looked for `cache/cortex/<version>`, but the real layout is `cache/cortex/cortex/<version>`. The real run then exposed a product defect the stub had hidden: Cortex exits when its stdin closes, so the mirror could lose lessons. It was fixed with a detached feeder.
5. **Change 09's test failed on the phase-boundary gate** because it read uninitialised `tools/*` submodule gitlinks as stray staged paths. Its verdict depended on checkout state. Fixed in #155.
6. **Spec review needed 2 rounds.** Round 1 found 3 critical findings:
   - 06's reconcile feature had no gate;
   - 04's check read the live `:23001`;
   - 02 symlinked the real Codex `auth.json`.
7. **Removing a worktree during branch cleanup broke the machine refresh.** Claude Code's `prometheus-skill-pack` marketplace (`~/.claude/settings.json`, `~/.claude/plugins/known_marketplaces.json`) was registered against the `dist-ship-script-lib` worktree, and Codex's (`~/.codex/config.toml`) against `deploy-main`. Removing the first made `claude plugin marketplace update` fail with ENOENT. Repointing them to the main checkout failed in turn, because the updater refuses a dirty source tree and the KBD runtime writes tracked files in the main checkout. A detached HEAD failed too, because `update-skill-pack.sh` runs its own `git pull --ff-only`. Final arrangement: `deploy-main` is the single install worktree, on a local-only `deploy/main` branch that tracks `origin/main` and is never pushed, and all three references point at it (config backups `*.bak-20261005T*`). It is the one exception to "main only".
8. **Codex regrew `memory_summary.md` despite `generate_memories = false`.** The live `codex.memories` doctor check found a 7.8 KB summary written at 00:48. It was archived, and the doctor then passed. The setting alone is not sufficient: the doctor check is the real guard. Investigate whether Codex's consolidation also needs `use_memories` or another key.
9. **Change 02's gate was BLOCKED once** by a concurrent cargo build from outside this session. The one-build guard worked as designed.

## Root Cause

- **Delta 1:** acceptance tests exercised components (the helper, the doctor, the script) but not the production entry point's control flow. Under the repo's own integration-only rule, "both installers apply it" needed a test through the installer dispatch. A component test passing was taken as the integration claim.
- **Delta 2:** the write and read paths shared `search_memories`, so a read-path optimisation silently leaked into writes. The worker could not see this without compiling, and the spec's write-path criterion was what surfaced it.
- **Delta 3:** analysis inferred the bottleneck from the code shape ("4 embeddings per recall") without measuring first. The spec did make measurement task 1, which turned a wrong assumption into a recorded finding instead of a wasted build.
- **Deltas 4 and 5:** tests encoded assumptions about the environment (the cache layout, submodule initialisation) instead of discovering it.
- **Delta 6:** the first spec draft copied last phase's test patterns, which hit the live service and real credentials.

## Corrective Actions

1. **Installer and entry-point features need a test through the real dispatch,** or a structural check on the dispatch when a full install is too heavy, as in #154. Add this to the spec template's acceptance rules.
2. **Read-path caches must state which callers may use them,** and the test must assert the write path's counters stay unchanged. Change 05's test is the model.
3. **Analysis that names a performance bottleneck must measure it before sizing the change.** Keep "measure first, with a stop rule" as task 1 for performance changes.
4. **Tests discover the environment** (glob the real layout; skip gitlinks) and never assert on shared live state that another session may touch. Change 07's `~/.cortex/memory.db` mtime check remains a known flake risk.
5. **Spec review keeps checking for live services and real credentials in gates,** which is what caught Delta 6.
6. **Ledger:** this phase used `begin-task`/`end-task` throughout and had no drift. Change 06 now makes `mark-done` sync and adds `kbd-apply reconcile`, which `/kbd-reflect` runs at step 4.

## Recalled Lessons

- **Applied:** "`kbd-apply mark-done` updates the task flag but does **not** synchronize the canonical KBD ledger". Every task was closed with `begin-task`/`end-task`; the ledger reached 24/24 with no drift. Change 06 removed the trap.
- **Applied:** "For every external integration, run at least one test against the real service". The real Cortex run found the stdin-close defect (Delta 4), and the real executor measurement reframed 05 (Delta 3).
- **Applied:** "Verify main includes PR final head after merge". Checked for every merge (#146–#155, sm #46, mini #39 and #40). Mini #39 was squash-merged, and its content was diffed against the head instead.
- **Applied:** "Avoid reentrant lock-taking CLIs and default fallbacks". Change 03's procedure exits 2 without a valid iteration and never calls the cadence CLI.
- **Recurred, in a new form:** "Generated Format Emitters and Validators Must Stay Paired". The review found `assertSharedGeneratedPaths` vacuous: it checks the outputs against a set derived from the same outputs. That is the emitter validating itself. Left as debt.

## Goals

| Goal | Status | Evidence |
|---|---|---|
| G1a ranked MEMORY.md partitioning | MET | #148; gate passes on merged main |
| G1b Codex `generate_memories=false` in installer and doctor | MET, with a caveat | #151 and #154. Live: the doctor caught a summary regrown despite the setting; after the archive it passes. The setting alone does not prevent regrowth. |
| G1c versioned, fail-loud refresh procedure | MET | #146, #153 (reconcile), #154 (bounded cargo wait) |
| G2a envelope test on a scratch server | MET | #147 |
| G2b less repeated query embedding | PARTIAL | sm #46 + v1.10.1 shipped and measured; not yet deployed, because the skill-pack and mini `tools/surreal-memory-server` pins (`versions.toml`, operator-authored) still point at 1.10.0 |
| G2c ledger/tasks reconciliation | MET | #153 |
| G3a Cortex mirror against real Cortex | MET | #150, gate against real Cortex 2.0.3 |
| G3b recall quality | MET | #152; held-out fixture: 4 of the top 5 current-or-role, 0 foreign; negative control leaks 10 |
| G3c rebase-regenerate helper | MET | #149; used for real on #151's conflicts; test fixed in #155 |

8 MET and 1 PARTIAL.

## Delivered Changes

| Change | PR | Merge |
|---|---|---|
| 01 ranked partition | #148 | `aff8005` |
| 02 Codex memories, installer and doctor | #151 (+#154 fix) | `e8f7543`, `511a3ea` |
| 03 versioned refresh procedure | #146 | `2f3d3b8` |
| 04 scratch surreal library | #147 | `483a4a4` |
| 05 query-embedding cache, 1.10.1 | sm #46, tag v1.10.1 | `0cf8c7e` |
| 06 ledger reconciliation | #153 | `9233000` |
| 07 Cortex mirror, real service | #150 | `ce0984f` |
| 08 recall scoping and evaluation | #152 | `de0c3fd` |
| 09 rebase-regenerate | #149 (+#155 test fix) | `82ead52`, `f35fe2a` |
| mini: delivery-cadence 1.2.3 | mini #39 | `d944833` |
| mini: agent-team memory parity | mini #40 | `6044ecd` |

## Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with a gate | 9/9 |
| First-run gate pass (worker or lead) | 6/9. 05 failed 4 tests on first compile; 07 was BLOCKED by its glob; 09 needed one fix rerun for gitlink pins. |
| Phase-boundary gate on merged main | 7/9 first pass. 09 failed (environment-dependent test, fixed in #155); 02 was BLOCKED by a concurrent cargo build. Both were rerun after #155; see execution.md. |
| Cumulative adversarial review | BLOCK: 1 critical, 8 warnings, 3 suggestions. The critical finding and 3 warnings were fixed in #154; the rest are recorded as debt. |
| Review isolation | Harness-native, same-family, for every review. The liter-llm preflight was `degraded` with no distinct judge model. |

## Technical Debt

1. **surreal-memory cache key:** it uses `(dimensions, text)` rather than `(model id, text)`. Two embedders of the same width would share entries. Fix in surreal-memory.
2. **Cache counters:** failed embeds are not counted, and the disabled path counts a miss before embedding. `stats()` turns a storage error into `None`.
3. **Vacuous check:** `assertSharedGeneratedPaths` in `generate-skill-system-distribution.js` can never fail.
4. **Doctor repair hint:** `codex.memories` gives the repo-relative `bash shared/scripts/codex-memories-config.sh`, which does not exist for an installed CLI.
5. **`install-system.js` overrides an operator's custom `CODEX_HOME`.**
6. **Cortex feeder:** each mirrored write spawns a detached feeder plus a node server for up to 180 s, with no concurrency cap.
7. **Flaky assertion:** the real-Cortex test asserts the live `~/.cortex/memory.db` is unchanged, which flakes if another Cortex session writes.
8. **Partition footer:** idempotence can be off by one byte at a digit-width boundary (E=9 or 99).
9. **Pins:** the `tools/surreal-memory-server` pins in the skill pack and mini still point at 1.10.0. Bumping them needs an operator-authored `versions.toml` edit.
10. **Mini baseline:** `mini-baseline-failures.txt` is incomplete. Mini main has 11 failing tests, some not listed.
11. **Hook bytecode in the installed generation:** hooks run Python from the active installed generation without `-B` or `PYTHONDONTWRITEBYTECODE`. `shared/scripts/lib/__pycache__` then appears inside the immutable payload, and the next `install-plugin-generation` verification fails with "payload entry is not manifested". This blocked the phase-end refresh. Fix: run hook Python with `-B` (or set `PYTHONDONTWRITEBYTECODE=1` in the hook wrapper), or have the verifier ignore `__pycache__`.
12. **Mini dist:** the generator drops the executable bit on `dist/.../refresh-skill-pack.sh`.

## Lessons Learned

- [GLOBAL] A component test passing is not evidence that a production entry point uses the component. When a spec says "the installer applies X", test through the installer's dispatch, or at least assert structurally that the install path reaches the call.
- [GLOBAL] When adding a cache to a read path that shares a function with the write path, assert the write path leaves the cache counters unchanged. Shared helpers leak read optimisations into writes.
- [GLOBAL] Measure a suspected bottleneck before sizing the fix, with an explicit stop rule. Here one embedding was 14 ms, so the cache saved about 1 s of a 2.8 s budget, not the whole timeout.
- [GLOBAL] Real-service tests find defects stubs hide. Real Cortex exits when its stdin closes, which silently dropped mirrored writes.
- [GLOBAL] Tests must discover their environment (real cache layouts, submodule initialisation) rather than encode one checkout's state, or the phase-boundary gate on a fresh worktree will disagree with the worker's run.
- `scripts/rebase-regenerate.sh` resolved a real generated-only conflict (#151) in one step. Prefer it to hand regeneration during rebases in this repo.
- A one-build-at-a-time guard can BLOCK a phase gate because of builds outside the session. Treat BLOCKED as "rerun", never as a pass.
- [GLOBAL] Before removing a git worktree, grep harness configs (`~/.claude/settings.json`, `~/.claude/plugins/known_marketplaces.json`, `~/.codex/config.toml`) for its path. Local plugin marketplaces are often registered against a worktree, and removing it breaks every later plugin update.
- [GLOBAL] In a repo with `tag.gpgsign=true`, `git tag -f name sha` fails without a message, and an `&&`-less cleanup script then deletes the branch it meant to archive. Create archive refs with `git update-ref refs/tags/...`, and check that the ref exists before deleting the branch. Also check worktrees for live processes (`lsof +D`) before removing them: other agent sessions create worktrees during a long run.
- [USER] The operator approves merges, tags and machine-level changes explicitly. A single up-front approval question at the phase boundary keeps the run unblocked.

## Next Phase Seed

`phase-learning-deploy-and-debt`. Top priorities:

1. **Deploy what shipped:**
   - with operator approval, bump the `tools/surreal-memory-server` pins in the skill pack and mini to v1.10.1 (`versions.toml`);
   - refresh the machine;
   - measure SubagentStart recall under load with the cache live.
2. **Close review debt:**
   - the cache key to the model id, and the counter semantics;
   - the vacuous `assertSharedGeneratedPaths`;
   - the doctor repair hint for installed CLIs;
   - `CODEX_HOME` respect;
   - the Cortex feeder concurrency cap.
3. **Make "test through the entry point" a spec-template rule,** and refresh mini's failure baseline.

## Codify as Skill?

- **Phase-boundary approval batch:** one AskUserQuestion covering merges, tags, machine refresh and local config before the boundary run. Worth a short section in `kbd-execute`.
- **Repo cleanup to main:** the inventory plus archive-tag plus worktree-removal script used at this handoff. Worth codifying as a `repo-branch-cleanup` helper in cowork-management.
