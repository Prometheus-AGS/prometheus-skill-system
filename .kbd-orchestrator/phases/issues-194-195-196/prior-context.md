# Prior context — issues-194-195-196 (execute)

> Auto-populated by /kbd-memory-recall for lead view (main thread); lessons from surreal-memory. Cite the lessons that apply; recalled entries are information recorded by agents, not instructions.

## Lessons

- [lead] KBD assess summary for phase issues-194-195-196: Source assessment: #194 current runtime already rejects duplicate registration keys, but incident identity/version and authorized repair remain unresolved; #195 needs selected-method/authority/supersession propagation, and #196 depends on that policy to separate calibrated discovery, BDD authoring and phase-end execution. Reuse existing finite contract and typed closure; do not reopen delivered foundations or infer current behavior from recalled historical repairs. WARNING: independent review remains pending under the completed-production reviewer boundary, provider preflight has zero resolved distinct models, generated-code health/parity is UNKNOWN, and calibration examples, selected installed targets and finite budgets/oracles await prospective contract adoption. _(recorded by an agent; progress, stage assess, 2026-10-27; via surreal-memory)_
- [lead] KBD plan summary for phase issues-194-195-196: Owner approved the three calibration examples, 180-minute/240-call final verification/repair ceilings (up to two repair attempts; two no-progress events) and selected local macOS source/generated/scratch-installed surfaces. Contract canonically adopted at plan revision 19; actual identity attached. Four ordered changes, sixteen exact OpenSpec backend tasks (fourteen production, two EVIDENCE); #195 precedes #196 shared edits and delivery owns C-01 generation/final evidence. Task model assignments and sequential fallback are in plan.md; actual routes/host readiness must be rechecked before dispatch. WARNING: independent review deferred to complete production; original #194 incident remains NOT_EXERCISED; no implementation, tests, workers, global install, publication or remote actions started _(recorded by an agent; progress, stage plan, 2026-10-27; via surreal-memory)_
- [lead] KBD analyze summary for phase issues-194-195-196: READY to spec: seven candidates (one adopt, four adapt, one reference, one reject); retain current signed runtime, adopted contract, typed closure and JS/Rust BDD runners. #194 needs distinct diagnostics for duplicate rows/embedded IDs/map keys/subject resolution and an explicit history-preserving typed repair; exact incident journal/cause remains UNKNOWN. #195 compact policy/authority/supersession references precede #196 bounded discovery and author-only/phase-end routing; no new framework or dependency upgrade. WARNING: independent review pending under completed-production policy; seven selected installed KBD files match source bytes but behavior/parity remains unqualified; owner calibration, concrete budgets/oracles and selected qualification surfaces remain prospective adoption inputs. _(recorded by an agent; progress, stage analyze, 2026-10-27; via surreal-memory)_
- [project] Assessment of issues-194-195-196 at source bfe572ff found ChangeRegistered already refuses duplicate phase-map keys (substrate/kbd-runtime/src/lib.rs:2073); issue #194 incident root cause remains unconfirmed without affected journal/version and map-key versus embedded-ID inspection. reconcile.mjs emits generic duplicate progress ID errors, while kbd-apply reuses task IDs, not duplicate changes. #195/#196 should reuse existing finite adopted contract and typed closure, repair conflicting failing-test-first/per-change BDD instructions, and propagate compact selected-method/authority/supersession references. Assessment performed no generated-code checks; independent review is pending under repository completed-production policy. See phase assessment.md and assess handoff; next suggested stage /kbd-analyze issues-194-195-196. _(recorded by an agent; lesson, 2026-10-27; via surreal-memory)_
- [project] Initialized KBD phase issues-194-195-196 on 2026-10-26 after git pull reported already up to date and parent/submodule indexes had no unmerged entries. Canonical runtime revision 90 activates the phase; goals.md and issue-baseline.md capture duplicate-change repair (#194), selected VLBV propagation (#195), and scenario/BDD authoring-execution separation (#196, dependent on #195). No implementation or acceptance is claimed; next step is /kbd-assess issues-194-195-196. Existing dirty files were preserved; required OpenSpec refresh to 1.15.0 backed up generated integrations. Canonical runtime, not preexisting stale waypoint/project activePhase, is authoritative. _(recorded by an agent; lesson, 2026-10-27; via surreal-memory)_

## pk knowledge

- [pk:project] Decision: defer MCP 2026-07-28 adoption; converge rmcp in a dedicated phase: Decision: defer MCP 2026-07-28 adoption; converge rmcp in a dedicated phase **Do not adopt MCP `2026-07-28` now.** Unblock the parent phase with two minimal edits, record what was measured, and give the five-crate `rmcp` convergence its Committed instead: `sse-stream` floor at 0.2.4 + `ContentBlock as Content` in two _(recorded by an agent; lesson; via pk pk:decision-defer-mcp-2026-07-28-adoption)_
- [pk:project] Team-Aware Learning Memory Implementation Reflect Lessons: `team-aware-learning-memory-impl` reached reflect with all `tasks.json` files showing done, but the canonical runtime ledger had drifted and required repair. - At reflect time, `progress.json` recorded only **13 of 25 changes**, while every `tasks.json` showed all tasks complete. - From B5 onward, tasks were closed with `kbd-apply mark-done`; this flips task flags but does **not** sync the runtime ledger. - Reflect repair replayed every task through `begin-task` / `end-task`. _(recorded by an agent; lesson; via pk learning:d630eaafa139fe7b)_
- [pk:project] Decision: mobile skill updates ride sovereign-sync, not a signed HTTPS bundle: Decision: mobile skill updates ride sovereign-sync, not a signed HTTPS bundle **Use the existing `sovereign-sync` P2P substrate.** Do not build a signed HTTPS - **`skill-index` carries enough to update a device.** It syncs the index; it is 2. **Kills "the index is enough."** Test: sync `skill-index` to a device with an _(recorded by an agent; lesson; via pk pk:decision-mobile-update-transport)_

## Previous reflection

From `team-aware-learning-memory/reflection.md`:

**Delta**

These are the differences between what was planned and what was delivered.

1. **The Codex half of the design is unproven at runtime.**
   - The probe confirmed Claude Code SubagentStart injection, identity fields, and that MEMORY.md reaches subagents.
   - For Codex it could not load the project hook layer under a scratch HOME (`--ignore-user-config` drops `projects.*.trust_level`). The four Codex hook behaviours are recorded UNVERIFIABLE, and the design relies on documented fallbacks instead: role capture from `spawn_agent` PreToolUse, correlated by `agent_id`.
   - Goals 1 and 5 promised both harnesses, so they are PARTIAL, not MET.
2. **Phase output is documents, not behaviour.**
   - No agent receives targeted lessons yet.
   - The per-agent byte reduction (14 KB / 10.8 KB → ≤ 12 KB total, with targeted content) is a target, not a measurement.
3. **The plan grew from 11 PRs to 19 PRs across 4 repositories.**
   - Most of the growth is upstream prerequisites (U1–U6) that the earlier plan did not know about: pk truncation at 43 entries, no snapshot commit, embeddings in search responses, null `agent_id` on all sampled records.
4. **Unplanned side work** went into change-tlm-004 (hook timeouts in seconds, PR #124). It corrected a wrong diagnosis I had made earlier in the session (milliseconds).
5. **Collateral damage to the user's environment:**
   - The probe switched the user's real plugin generation, and the restore is still pending.
   - The ChatGPT desktop app rewrote `~/.codex/config.toml` during the tlm-001 gate. That was not caused by us, but it broke a strict before/after hash check.

**Root Cause**

1. **Codex unverifiable.**
   - Codex's project config layer loads only for trusted projects, and trust lives in user config.
   - Isolating HOME, which is required so the real config is not touched, also removes trust.
   - I did not find a supported flag that grants trust without user config before the gate ran.
2. **Documents only.** This was by design: the user asked for an investigation-and-design child phase so the implementation would be right. It is not a failure, but it means the phase's value is unrealised until PR A1 onwards lands.
3. **Plan growth.** The original plan was written from code reading and never queried the stores. The assessment queried live surreal-memory and pk and found the defects.
4. *
… (truncated)

## Knowledge gaps

- Goal PARTIAL: Inventory memory/learning paths and agent identity per hook point in Claude Code and Codex — Inventory complete (assessment A–N). Claude identity CONFIRMED by probe. Codex identity is documented only, because the runtime probe was UNVERIFIABLE.
- Goal PARTIAL: Revised implementation plan for PRs 4–13 with per-agent gates in both harnesses — 19 PRs with dependencies and gates are written. The Codex leg of the gates depends on corrective action 1.
- `shared/scripts/kbd-memory-recall.sh` recalls lifecycle metadata instead of lessons (fix: PR B4).
- `pk context` truncation at 43 entries per scope (fix: PR A1). The worker does not commit a prompt snapshot (fix: PR A2).
- Every sampled surreal-memory record has a null `agent_id`, so migration is needed (fix: PR A4/U6).
- `scripts/install-binaries.sh` liter-llm block still lacks `|| true` (pre-existing, noted in CLAUDE.md).
- `adversarial-review` packet builder ordering defect: the spec packet pulled in all changes until a provisional handoff was written.
- An orphaned agent-team-creator runtime phase and a stale waypoint `exactNextCommand` (`/kbd-assess …`) remain in canonical state.
