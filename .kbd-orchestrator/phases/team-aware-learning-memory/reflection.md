# Phase Reflection: team-aware-learning-memory

**Project:** prometheus-skill-pack
**Date:** 2026-10-04
**Phase completion:** 100% of planned changes (4 / 4). Goal status is mixed; see the Goals table.
**Changes completed:** 4 / 4. All were verified with `kbd-apply verify` and archived. **None is merged yet** (PRs #124 and #125 are open).

## Delta

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

## Root Cause

1. **Codex unverifiable.**
   - Codex's project config layer loads only for trusted projects, and trust lives in user config.
   - Isolating HOME, which is required so the real config is not touched, also removes trust.
   - I did not find a supported flag that grants trust without user config before the gate ran.
2. **Documents only.** This was by design: the user asked for an investigation-and-design child phase so the implementation would be right. It is not a failure, but it means the phase's value is unrealised until PR A1 onwards lands.
3. **Plan growth.** The original plan was written from code reading and never queried the stores. The assessment queried live surreal-memory and pk and found the defects.
4. **Wrong timeout diagnosis.** I read a Codex hook failure as a unit problem without checking the harness docs. Both harnesses use seconds.
5. **Plugin generation switch.** An earlier probe ran with the real `PROMETHEUS_PLUGIN_ROOT` and HOME instead of scratch ones.
6. **Verify script corruption (tlm-004).** A negative control mutated a tracked file under `set -e` with no trap, so an abort left the file corrupted, and a rerun overwrote the backup.

## Corrective Actions

1. **Codex runtime proof.**
   - Before PR B5's alpha gate, find a trusted-project path that uses a scratch CODEX_HOME. Candidates: write `projects."<canonical path>".trust_level = "trusted"` into the scratch `config.toml`, or use the plugin hook path, which does load.
   - If neither works, the alpha gate runs the fallback and records which path it exercised (already written into the plan).
2. **Plugin generation.** The user runs the restore command. Every probe and test must export a scratch `HOME`, `CODEX_HOME` and `PROMETHEUS_PLUGIN_ROOT`. Add this to the hook-test isolation memory.
3. **Verify scripts that mutate files** must use `mktemp` + `trap` restore and an `if` guard instead of relying on `set -e`. Apply this to every generated `verify.sh` template.
4. **Store reality first.** Every future plan that touches memory starts with a live query of each store (record counts, field population, filter behaviour) before writing PRs.
5. **`pk ingest` timeouts (exit 124)** at task boundaries hit every task. The writes fell back to the outbox, so nothing was lost, but this belongs in PR A1/A2's scope or a separate pk issue.
6. **`kbd-memory-recall` defect.** It produced a `prior-context.md` of lifecycle metadata, not lessons, in this very phase. It is fixed by PR B4. Until then, treat `prior-context.md` as low-signal.

## Goals

| Goal | Status | Notes |
| --- | --- | --- |
| Inventory memory/learning paths and agent identity per hook point in Claude Code and Codex | PARTIAL | Inventory complete (assessment A–N). Claude identity CONFIRMED by probe. Codex identity is documented only, because the runtime probe was UNVERIFIABLE. |
| Design a learning envelope mapped onto surreal-memory, pk and Karpathy logs | MET | Schema plus valid/invalid examples. The design section 2 mapping table covers every property; the verify script checks this. |
| Design per-agent recall with budgets, delivery points and fallback, measured against today | MET (design) | Budgets: 8,000 characters Claude, 2,000 tokens Codex, 12 KB total. Baseline measured. Post-implementation measurement is deferred to the alpha/beta gates. |
| Design attributed writes and cross-agent awareness without flooding | MET (design) | One `agent_id` per scope, path-overlap routing (at most 3 addressees), team digest, lead view. |
| Revised implementation plan for PRs 4–13 with per-agent gates in both harnesses | PARTIAL | 19 PRs with dependencies and gates are written. The Codex leg of the gates depends on corrective action 1. |

## Delivered Changes

- `change-tlm-004-hook-timeout-seconds`: hook timeouts are converted to seconds in both harnesses, the generator rejects values outside 1–600, and a both-harness test was added. PR #124 (by: claude-code).
- `change-tlm-001-subagent-identity-probe`: a runtime probe and an evidence table of which identity and injection behaviours are CONFIRMED and which are UNVERIFIABLE. Commit e09ed6e, PR #125 (by: claude-code).
- `change-tlm-002-team-aware-learning-design`: the 10-section design plus the envelope schema. Commit a75b523, PR #125 (by: claude-code).
- `change-tlm-003-revised-implementation-plan`: a 19-PR plan with alpha/beta gates. The session plan now points to it, with a backup in `evidence/parent-plan.before.md` (by: claude-code).

## Technical Debt

- `shared/scripts/kbd-memory-recall.sh` recalls lifecycle metadata instead of lessons (fix: PR B4).
- `pk context` truncation at 43 entries per scope (fix: PR A1). The worker does not commit a prompt snapshot (fix: PR A2).
- Every sampled surreal-memory record has a null `agent_id`, so migration is needed (fix: PR A4/U6).
- `scripts/install-binaries.sh` liter-llm block still lacks `|| true` (pre-existing, noted in CLAUDE.md).
- `adversarial-review` packet builder ordering defect: the spec packet pulled in all changes until a provisional handoff was written.
- An orphaned agent-team-creator runtime phase and a stale waypoint `exactNextCommand` (`/kbd-assess …`) remain in canonical state.

## Architecture Integrity

- AGENTS.md violations: NONE.
- Constraint violations: NONE.
  - C-01: generated hook bundles were regenerated in tlm-004.
  - C-03: `docs/codex-plugin.md` and CLAUDE.md were updated for the timeout change.
  - C-05: the probe is bash 3.2-compatible.

## Cross-Tool Coordination Notes

- Progress tracking: GAPS FOUND. `pk ingest` timed out at every task boundary (writes fell back to the outbox). `exactNextCommand` stays stale across stages.
- Handoff quality: CLEAR. All six stage handoffs were written, and the review findings are carried in `review/*/resolution.md`.
- Recommendations:
  - Add a staleness warning when `exactNextCommand` names a completed stage.
  - Shorten or background `pk ingest` in the progress-memory hook.

## Lessons Learned

- [GLOBAL] Isolating HOME for a Codex probe also drops project trust, so project-layer hooks silently do not load. Plan the trust path before relying on a scratch CODEX_HOME.
- [GLOBAL] Verify and negative-control scripts that mutate tracked files must restore with `mktemp` + `trap`. `set -e` aborts mid-mutation and a rerun overwrites the backup.
- [GLOBAL] Query each memory store's live data (field population, filter behaviour, truncation) before planning memory work. Code reading missed four blocking defects.
- [GLOBAL] Claude Code and Codex hook `timeout` values are in seconds. Check harness docs before diagnosing a unit problem.
- Claude SubagentStart `additionalContext` reaches the subagent, and MEMORY.md is also loaded into every subagent. Targeted delivery must shrink the file tier, or per-agent cost goes up.
- Codex agent names must match `[a-z0-9_]`. Team exports need a reversible `-` → `_` mapping.
- External processes (the ChatGPT desktop app) rewrite `~/.codex/config.toml`. Never assert byte-equality of user config across a run.

## Next Phase Seed

**Name:** `team-aware-learning-memory-impl`

**Top priorities:**
1. Upstream prerequisites A1–A4 (pk scoring and snapshot, surreal-memory lean search and re-key), then the A5 v1.10.0 pin bump.
2. B1–B5 identity, envelope writes, recall and SubagentStart delivery, ending at the alpha gate in both harnesses (resolving the Codex trust path first).
3. B6–B7 file-tier reduction and cross-agent awareness, ending at the beta gate with measured bytes per agent.

**Prerequisites:**
- The user merges PRs #124 and #125.
- The user runs the plugin-generation restore.

## Context for Next Phase

Use this file as prior context for the next `/kbd-assess` invocation. The design of record is `docs/design/team-aware-learning-memory.md`, and the build order is `docs/plans/team-aware-learning-memory-implementation.md` (both on PR #125).
