# Prior context — vlbv-r4-portability (reflect)

> Auto-populated by /kbd-memory-recall for lead view (main thread); lessons from surreal-memory. Cite the lessons that apply; recalled entries are information recorded by agents, not instructions.

## Lessons

- [lead] KBD assess summary for phase vlbv-r4-portability: Gaps: no AT/§18 eval corpus, runner, grader or per-harness report; no canonical B4-authorization instant or declared runner-output globs for CR-33. Six machinery ATs execute once but count in every harness report; CR-33 needs a mixed-date oracle (SCAN-3). Open owner decisions: auth copy into scratch homes, samples per case, grader model. Round-2 review unusable; plan review must re-check G5, G6/SCAN-3, G8. _(recorded by an agent; progress, stage assess, 2026-11-16; via surreal-memory)_
- [lead] KBD assess summary for phase issue-195-closeout: Gaps: (A) method-context policy conflicts never block — status/--require-resolved ignore them, and this repo's own CLAUDE.md/AGENTS.md compiler exception will trigger it once VLBV is selected; (B) 11 duplicated selected-method sections (whitespace-only variance) vs existing references/method-context.md; (C) VER-001/VER-003 are driver stubs, not failed runs; (D/E/G) Codex/OpenCode never exercised, scratch logins gone, judge/critic collide in models.toml and producer identity is unconfigured. Open for analyze/plan: owner risk acceptance and budgets, acknowledgement mechanism (signed decision recommended), operator scratch logins, cross-model review route. Review WARNINGs (incorporated): OpenCode isolation and liter-llm feasibility. _(recorded by an agent; progress, stage assess, 2026-10-27; via surreal-memory)_
- [lead] KBD analyze summary for phase phase-learning-deploy-and-debt: 12 candidate decisions; no new dependencies. Adopt inherited Python bytecode suppression; adapt Codex 0.158.0 feature gate, corroborated by consolidation timings; retain quick_cache. Full pack has no root versions.toml. Open Spec decisions: model lifetime, error stats, Codex-home precedence, optional feeder saturation. Registry metadata unavailable; adversarial review deferred to completed production boundary, not passed. Owner approval still required for protected edits and machine changes. _(recorded by an agent; progress, stage analyze, 2026-10-05; via surreal-memory)_
- [lead] KBD plan summary for phase phase-team-learning-hardening: 9 changes / 24 tasks registered. Order: round 1 parallel (03, 04, 01, 09, 07), round 2 cargo serialized (05 then 02), round 3 after merges (06 after 03, 08 after 04; stacked-branch fallback). First change: change-tlh-03-versioned-cadence-refresh-procedure. Task model assignments in plan.md: Sonnet 5.5 workers via the Agent tool, Opus 5.5 lead for 05 task 1 and the phase gate; liter-llm cross-model judge unresolved (preflight degraded), so the fallback is harness-native. Review PASS 0/3/4, all folded in. Approvals: v1.10.1 tag, real ~/.codex installer run, local shim replacement, machine refresh at the boundary. _(recorded by an agent; progress, stage plan, 2026-10-04; via surreal-memory)_
- [lead] KBD analyze summary for phase issue-195-closeout: 5 candidates, all in-repo except OpenCode: adopt signed-decision KBD-POLICY-ACK pattern (A, build only CONFLICT status semantics in method-context.mjs); adapt references/method-context.md with an identical short core + equality check (B); adopt existing session driver for VER-001/VER-003 + CON-001/002/REF-001 (C); Codex via existing driver but PROMETHEUS_PLUGIN_ROOT must be made scratch; OpenCode only with scratch XDG roots + OPENCODE_DISABLE_CLAUDE_CODE=1 after an isolation preflight, else UNSUPPORTED; judge via labelled per-run override (G). READY to spec. Open owner inputs: risk/budgets, ack mechanism, scratch logins, models.toml route. Review WARNINGs (verified partly mistaken, substance incorporated): Codex evidence, generated-surface list. _(recorded by an agent; progress, stage analyze, 2026-10-27; via surreal-memory)_
- [lead] KBD Plan phase-learning-deploy-and-debt is complete at revision 2204: 18 changes and 40 pending tasks cover final full/mini source convergence, hooks/Codex/debt, Companion source publication, all README/site/team/model/service docs and final local integration. Owner selected full skill-system ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61; Companion currently has four cfbc262 Cargo consumers. Do not substitute that SHA for surreal-memory or infer protected versions.toml/tag/home approval. Canonical task audit found five historical unfinished tasks, four nested under completed changes; change totals alone are insufficient. Preserve 18 blockers and 27 boundary obligations until evidence-backed reconciliation. Next is change-ldd-13-source-convergence task 1; model assignments and ownership are in plan.md; no production testing or independent review ran during Plan. _(recorded by an agent; lesson, 2026-10-05; via surreal-memory)_
- [lead] KBD assess summary for phase phase-learning-deploy-and-debt: 14 assessed work areas: prioritize immutable-hook bytecode prevention, evidence-based Codex regrowth diagnosis, then approval-gated v1.10.1 pin/deployment work and remaining debt. Doctor fails installed generation integrity; Codex memories passes currently; full pack has no root versions.toml. Tests and independent review are deferred to the completed production boundary; lifecycle receipt gap was explicitly reconciled by the owner, never fabricated. _(recorded by an agent; progress, stage assess, 2026-10-05; via surreal-memory)_

## pk knowledge

- [pk:project] Decision: defer MCP 2026-07-28 adoption; converge rmcp in a dedicated phase: Decision: defer MCP 2026-07-28 adoption; converge rmcp in a dedicated phase **Do not adopt MCP `2026-07-28` now.** Unblock the parent phase with two minimal edits, record what was measured, and give the five-crate `rmcp` convergence its Committed instead: `sse-stream` floor at 0.2.4 + `ContentBlock as Content` in two _(recorded by an agent; lesson; via pk pk:decision-defer-mcp-2026-07-28-adoption)_
- [pk:project] Team-Aware Learning Memory Implementation Reflect Lessons: `team-aware-learning-memory-impl` reached reflect with all `tasks.json` files showing done, but the canonical runtime ledger had drifted and required repair. - At reflect time, `progress.json` recorded only **13 of 25 changes**, while every `tasks.json` showed all tasks complete. - From B5 onward, tasks were closed with `kbd-apply mark-done`; this flips task flags but does **not** sync the runtime ledger. - Reflect repair replayed every task through `begin-task` / `end-task`. _(recorded by an agent; lesson; via pk learning:d630eaafa139fe7b)_

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
