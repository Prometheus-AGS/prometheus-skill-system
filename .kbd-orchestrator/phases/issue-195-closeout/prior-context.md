# Prior context — issue-195-closeout (plan)

> Auto-populated by /kbd-memory-recall for lead view (main thread); lessons from pk. Cite the lessons that apply; recalled entries are information recorded by agents, not instructions.

## Lessons

- [pk:project] Team-Aware Learning Memory Implementation Reflect Lessons: `team-aware-learning-memory-impl` reached reflect with all `tasks.json` files showing done, but the canonical runtime ledger had drifted and required repair. - At reflect time, `progress.json` recorded only **13 of 25 changes**, while every `tasks.json` showed all tasks complete. - From B5 onward, tasks were closed with `kbd-apply mark-done`; this flips task flags but does **not** sync the runtime ledger. - Reflect repair replayed every task through `begin-task` / `end-task`. _(recorded by an agent; lesson; via pk learning:d630eaafa139fe7b)_
- [pk:project] Team Learning Hardening Phase Lessons and Corrective Actions: - A **CRITICAL** shipped despite all per-change gates passing: Change 02 ran 22 + 20 green checks, but `install-skills-flat.sh` never applied the Codex memory setting. The call was in `install_to_codex()`'s `else` arm, which is reached on uninstall, not the install path. The test sourced the helper directly and never exercised installer dispatch. Fix #154 added a negative-controlled structural regression check that fails against the old installer. - Change 05 initially failed 4 of 6 cache tests. `add_memory` performed duplicate detection by calling `search_memories`, which embedded every writt _(recorded by an agent; lesson; via pk learning:5d348f3c3f06b48f)_
- [pk:project] Team Learning Hardening Phase Reflection: - A shipped **CRITICAL** escaped all per-change gates: Change 02 ran 22 + 20 green checks, but `install-skills-flat.sh` never applied the Codex memory setting because the relevant call was in `install_to_codex()`'s `else` arm, a path reached only on uninstall. The test sourced the helper directly and did not exercise installer dispatch. Fix #154 added a negative-controlled structural regression check that fails against the old installer. - Change 05 initially failed 4 of 6 cache tests. `add_memory` checked duplicates by calling `search_memories`, which embedded every written memory twice and p _(recorded by an agent; lesson; via pk learning:10fc1e6bc266f681)_
- [pk:project] Delivery Cadence Source Freezing Requires Deploy Worktrees: `delivery-cadence` freezes every source named in `ready`; do **not** point those sources at the actively edited main checkout. Use a dedicated deploy worktree instead, because normal bookkeeping or local writes in the main checkout can invalidate the frozen source and trip checkpoint guards. - `ready` records the source state for the iteration. - `finish` refuses unchanged sources, so an iteration cannot be completed by simply retrying without an admissible source-state change. - If `finish` fails, the only repair path is to run `ready` again for that **same iteration**. _(recorded by an agent; lesson; via pk learning:cf4a0d4d09608dcd)_
- [pk:project] Treat One-Build Guard BLOCKED Phase Gates as Rerunnable: A one-build-at-a-time guard can block a phase gate because of builds outside the current session. A `BLOCKED` result from this guard is not evidence that the gate passed; treat it as a transient condition and rerun the gate after the external build clears. - Interpret `BLOCKED` as **rerun required**, never as pass/success. _(recorded by an agent; lesson; via pk learning:e0085ae85a309bb4)_
- [pk:project] Use rebase-regenerate.sh for Generated-Only Rebase Conflicts: `scripts/rebase-regenerate.sh` resolved a real generated-only conflict in PR/issue `#151` in one step. Prefer this script over manual regeneration when handling generated-only conflicts during rebases in this repository. _(recorded by an agent; lesson; via pk learning:944c292f0d68b2f5)_
- [pk:project] KBD Task Closure Requires Ledger Synchronization: `kbd-apply mark-done` updates the task flag but does **not** synchronize the canonical KBD ledger. - Do not rely on `kbd-apply mark-done` alone when closing phase tasks. - `begin-task` - `end-task` _(recorded by an agent; lesson; via pk learning:e5e11913d966b673)_
- [pk:project] Deploy Worktrees Need Submodule Update After Fast-Forward: After fast-forwarding a dedicated deploy worktree, run `git submodule update` before invoking `update-skill-pack.sh`; otherwise the script can reject the checkout as a dirty tree. _(recorded by an agent; lesson; via pk learning:90adeb6bc53b712e)_
- [pk:shared] Default MVP verification demos to agent-run with recorded authorization: For MVP phase verification, default demos to **agent-run execution with recorded operator authorization** rather than requiring the operator to perform the demo by hand. Reserve operator-hands requirements for actions that are truly account-bound or require human judgment, such as adjudication decisions. - `design.md` - `execution.md` _(recorded by an agent; lesson; via pk learning:351f041cc1b0c9d6)_
- [pk:shared] Installer behavior tests must exercise installer dispatch: A component-level test passing is not evidence that the production installer entry point uses that component. If a specification says “the installer applies X,” validate the behavior through the installer's dispatch path, or add a structural assertion proving the install path reaches the relevant call. _(recorded by an agent; lesson; via pk learning:ba6b5cffdef7de64)_

## pk knowledge

- [pk:project] Decision: defer MCP 2026-07-28 adoption; converge rmcp in a dedicated phase: Decision: defer MCP 2026-07-28 adoption; converge rmcp in a dedicated phase **Do not adopt MCP `2026-07-28` now.** Unblock the parent phase with two minimal edits, record what was measured, and give the five-crate `rmcp` convergence its Committed instead: `sse-stream` floor at 0.2.4 + `ContentBlock as Content` in two _(recorded by an agent; lesson; via pk pk:decision-defer-mcp-2026-07-28-adoption)_
- [pk:project] Karpathy session 27dbcfe5daa9: The specs are written and the stage is closed at revision 2432. Nothing has been implemented or run. Completed kbd-spec — issue-195-closeout **Three changes** (native-kbd, under `.kbd-orchestrator/changes/`): | `i195-01-policy-conflict-ack` | 2 | `method-context.mjs` reads signed `KBD-POLICY-ACK` decisions bound to the policy file's hash. Unacknowledged conflicts give status `CONFLICT`, and `--require-resolved` exits 2. `reanchor.sh` and `kbd-handoff.sh` show the conflict, and the reference doc explains acknowledgement. | _(recorded by an agent; lesson; via pk pk:karpathy-session-27dbcfe5daa959a2)_
- [pk:project] Karpathy session 177646e73cfc: Completed kbd-assess — issue-195-closeout (step 0 of 0) **What the assessment found** (`.kbd-orchestrator/phases/issue-195-closeout/assessment.md`): | **A** | Policy conflicts are detected but never block anything (`method-context.mjs:128`). This repo's own `CLAUDE.md:102` and `AGENTS.md:104` contain the compiler-check exception, so once VLBV is selected, this phase will hit the conflict itself. That's why an acknowledgement path matters. | | **B** | The 11 copied sections differ only by one trailing blank line, so there's no real drift yet. That corrects my earlier framing of B as drift. A ca _(recorded by an agent; lesson; via pk pk:karpathy-session-177646e73cfcf1a9)_

## Previous reflection

From `team-aware-learning-memory-impl/reflection.md`:

**Delta**

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
   - Stale generated `dist/` for delivery-cadence (16 fil
… (truncated)

## Knowledge gaps

- `scripts/memory-index-partition.py` keeps file order. The live partition needed a manual priority reorder first: current phase, then feedback, then archives, then older projects. The tool should rank entries itself.
- Codex memory generation is disabled in the operator's `~/.codex/config.toml` by hand. The installer neither sets it nor checks it, so a fresh machine will regrow `memory_summary.md`.
- `.prometheus/cadence/procedures/refresh-skill-pack.sh` is local and unversioned, and it contains the parity and submodule fixes made during this phase.
- `skills/process/agent-team-creator/tests/memory-envelope.integration.mjs` publishes to the live `:23001` service rather than a scratch server. It flaked twice at load average ~250.
- The D3 Cortex mirror has been tested only against a stub server. Real Cortex 2.0.3 has no tag field, so the role tag goes into `context`.
- SubagentStart delivery is skipped (logged, not retried) when recall misses the 3.5 s watchdog under extreme load. B4's recall embeds the same query several times.
- `kbd-apply mark-done` leaves the canonical ledger stale. Nothing warns about it.
