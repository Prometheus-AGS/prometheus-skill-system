# Assessment — team-aware-learning-memory-impl

**Date:** 2026-10-04 (UTC; local 2026-10-03)
**Assessor:** claude-code (claude-opus-5-5)
**Inputs:**
- design of record `docs/design/team-aware-learning-memory.md` (on skill-pack `main` at 206ebbd, PR #125 merged);
- plan `docs/plans/team-aware-learning-memory-implementation.md`;
- prior reflection `.kbd-orchestrator/phases/team-aware-learning-memory/reflection.md`.

**Repositories inspected (upstream `origin/main`, fetched today):**

| Repo | origin/main | Notes |
|---|---|---|
| prometheus-skill-system | 206ebbd | PRs #124 and #125 merged |
| prometheus-knowledge-rs | 1bbaecc (v1.9.0) | skill-pack gitlink = 1bbaecc; local submodule checkout is stale at 01a1dbe (1.8.0) |
| surreal-memory-server | 777cf72 (v1.9.0) | live server on 127.0.0.1:23001 reports 1.9.0 |
| prometheus-skills-mini | 56cbf12 | `versions.toml` pins pk 1bbaecc and surreal-memory 777cf72 |

**Overall: 0 of 5 goals met. 0 of 19 planned PRs landed.**

## Goal status

| # | Goal | Status | Evidence |
|---|---|---|---|
| G1 | Upstream prerequisites A1–A4 + A5 v1.10.0 pin bump | NOT MET | Every defect is still present; see A below |
| G2 | Identity, envelope writes, per-agent recall (B1–B4) | NOT MET | None of the 6 new libraries/scripts exist on `main` (B below) |
| G3 | SubagentStart delivery passes alpha in both harnesses | NOT MET | No SubagentStart hook exists; Codex trust path unsolved (C) |
| G4 | File-tier reduction + awareness pass beta | NOT MET | MEMORY.md is unpartitioned; no routing or digest code |
| G5 | C1–C4, D1–D3 | NOT MET | Mini hooks still in milliseconds; no promotion, gap, discovery or team-request code |

## A. Upstream defects (verified on origin/main)

**A1. pk context truncation — PRESENT.** In `pk-cli/src/main.rs`, `run_context` (line 683 onwards) does this:
- computes `candidates_per_scope = max_candidates.div_ceil(scopes)`;
- then takes `snapshot.entries.into_iter().take(remaining)` **before** `snapshot_score` runs.

With 3 scopes, only the first ⌈max_candidates/3⌉ entries per scope, in snapshot order, are ever scored. The fix belongs in `run_context`, not in `pk-librarian`.

A related defect: the per-scope budget is fixed up front (line 708). A scope that fails (no project root or no snapshot) does not give its share to the other scopes. Fold this into the A1 gate.

**A2. Worker attribution and snapshot — PRESENT.**
- `pk-learning-worker/src/main.rs` has no call to `pk_store::commit_prompt_snapshot`. The function exists at `pk-store/src/prompt_snapshot.rs:28`.
- There are no `team_id` or `role_id` fields. `project_id` appears only as a fallback read in `project_scope` (around line 1543).
- `enqueue_memory` (lines 727-734) sends `"agent_id": null`. This is the source of the null-`agent_id` records. A2 must stop producing them, and A4's re-key repairs the existing ones.

**A3. pk tag filter / settable type — ABSENT.**
- `pk-cli/src/main.rs` has no `--tag` or `--type` arguments.
- `pk-librarian/src/librarian.rs:413` hard-codes `entry_type = Some("Reference")` on every compile, and `pk-store/src/store.rs:394` defaults it to the same value. A settable type must change both places.
- The pk-mcp `knowledge_search` and `knowledge_ingest` tools (`pk-mcp/src/tools.rs:65-99`) have no tag or type parameters. They are **out of A3 scope**; the hooks call the CLI.

**A4. surreal-memory lean search / category filter / re-key — ABSENT.**
- `src/api/search.rs` returns `Json<Vec<Memory>>`, which includes embeddings.
- `SearchBody` accepts only `user_id`, `agent_id` and `session_id`.
- The categories filter is **advertised but silently dropped**:
  - The MCP `search_memories` tool passes `categories` (`src/mcp/handlers.rs:83`, `:766`), but storage takes `_categories` and ignores it (`crates/surreal-memory/src/storage/surreal.rs:2090-2096`).
  - REST `hybrid_search_memories` has no categories parameter at all.
- The fix belongs in `crates/surreal-memory/src/storage/surreal.rs` (`search_memories` and `hybrid_search_memories`) as well as `src/api/search.rs`. The A4 gate must cover both the MCP and REST paths.
- There is no re-key operation.

## B. Skill-pack baseline (origin/main 206ebbd)

| Planned artifact | State |
|---|---|
| `shared/scripts/lib/project_id.py`, `agent_identity.py`, `learning_write.py`, `learning_recall.py` | ABSENT |
| `shared/scripts/subagentstart-learning.sh`, `kbd-stage-writeback.sh` | ABSENT |
| `shared/schemas/learning-envelope.schema.json` | present (PR #125) |
| `shared/scripts/lib/memory-bridge.sh:16` | still `MEM_PROJECT="${PROMETHEUS_PROJECT_ID:-prometheus-skill-pack}"` |
| `hooks/hooks.json` SubagentStop matchers | bare `assessor`, `analyst`, `planner`, `executor`, `reflector` plus a matcher-less group. They collide with team roles (design §1) |
| SubagentStart hooks | none |
| **Existing team-memory writer** | **Present and not mentioned in the design.** `skills/process/agent-team-creator/runtime/src/memory.mts` (`queueMemory` at :21, `publishMemory` at :102; CLI `memory-queue` / `memory-publish`) already writes team memories. It uses an explicit `agent_id`, `provenance.teamId`, digest-based ids, an outbox, and categories `['agent-team', scope]`, inside its own envelope `{schemaVersion:1, kind:'agent-team-memory'}`, which differs from `learning-envelope.schema.json`. Analyze must decide replace / wrap / migrate. Recall (B4/B5) must read these records, or two incompatible attributed write paths will exist. |

## C. Codex trust path (alpha-gate risk)

This is unsolved and has the largest schedule risk. The prior probe showed that a scratch CODEX_HOME loses `projects.*.trust_level`, so project `.codex/` hooks never load.

Two untested candidates:
1. Write `[projects."<canonical path>"] trust_level = "trusted"` into the scratch `config.toml`.
2. Exercise delivery through the **plugin** hook path, which PR #121 showed fires under `codex exec --dangerously-bypass-hook-trust`. This one does not depend on project trust at all and is the more promising path.

**Open question for analyze:** run a short spike on candidate 2 before spec, so the alpha gate is designed against a path that works.

## D. Corrections to the plan and design (facts that changed or were wrong)

The plan was written from memory of the layouts and gets several paths wrong. Spec must use the paths below.

| Plan / design says | Actual |
|---|---|
| pk paths `crates/pk-cli/...`, `crates/pk-librarian/src/context.rs` | No `crates/` dir. `pk-cli/src/main.rs::run_context`, `pk-learning-worker/src/main.rs`, `pk-store/src/prompt_snapshot.rs` |
| `hooks/hook-contract.json` | `shared/harnesses/hook-contract.json` (the generator source) |
| `shared/scripts/kbd-memory-recall.sh` | `skills/process/kbd-process-orchestrator/skills/kbd-memory-recall/kbd-memory-recall.sh` (flat copies under `dist/` are generated; never edit them) |
| mini `skills/carried-payload.test.mjs` | Does not exist. Mini tests live under `lib/**/*.test.mjs` and `hooks/hooks.test.mjs` |
| design §9: "mini does not carry pk writes" | **Wrong.** `lib/karpathy/transport.mjs::deliverToPk` already spawns `pk ingest` when pk is present. D1 must attach the envelope to those records instead of excluding pk. |
| mini hook timeouts | Confirmed still milliseconds: three hooks at 1000, two at 5000, one at 15000. D2 is valid, but its file list is infeasible: mini has only `hooks/hooks.json`, with no `codex-hooks.json` and no generator. D2 must either create the Codex adapter or drop the cross-harness identity assertion. Its test is `hooks/hooks.test.mjs`. |
| plan A5 edits mini `versions.toml` | **Agents may not write it.** `versions.toml` line 1 and `docs/versions-toml.md:5` both state that the operator writes it. A5 splits into (a) the skill-pack pin bump, which an agent can do, and (b) an operator-authored mini pin change. B1 depends only on (a). |
| plan B2 `agents/iterative-evolver-*.md` | The agents are at `skills/process/iterative-evolver/agents/{planner,assessor,…}.md`, inside a separately distributed plugin with public ids `iterative-evolver:planner` and so on. Renaming them breaks that plugin's consumers. **Namespace the matchers instead of renaming the agents.** |
| plan `shared/scripts/memory-bridge.sh` | `shared/scripts/lib/memory-bridge.sh` |
| plan B6 `runtime/src/export-claude.mts`, `export-codex.mts` | `runtime/src/export-files.mts` |
| plan C4 `runtime/src/registry.mts` | new file (does not exist yet) |
| plan C1 `skills/process/kbd-reflect/SKILL.md` | `skills/process/kbd-process-orchestrator/skills/kbd-reflect/SKILL.md` |
| plan A4 `src/store/memory.rs` | `crates/surreal-memory/src/storage/surreal.rs` |
| plan pk `pk-cli/src/context.rs`, `ingest.rs`, `pk-store/src/scope.rs`, `entry.rs` | All CLI code is in `pk-cli/src/main.rs`; the entry type is `pk-core/src/types.rs:225` |

## E. Other defects carried forward

- **Broken `reflect:after` hook path.** The installed orchestrator's `reflect:after` hook runs `${KBD_ORCHESTRATOR_ROOT}/../../../shared/scripts/memory-writeback.sh`, which resolves outside the install. Reflect write-back never runs in the flat install. Fix within B4.
- **`pk ingest` timeouts.** It returned exit 124 at every kbd-apply task boundary last phase (fell back to the outbox). Investigate alongside A2.
- **Stale local pk checkout.** The local submodule checkout lags its gitlink (01a1dbe vs 1bbaecc). Any local build must check out the gitlink first.

## F. Constraints that bind this phase

- Implement fully, then run integration gates only.
- One cargo build at a time.
- Generated hooks and dist are regenerated from `shared/harnesses/hook-contract.json`, never hand-edited.
- Every hook exits 0 and stays silent when a dependency is absent.
- The user merges all PRs.
- Every probe or test uses a scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT.
- Model preflight status is `degraded`, so adversarial review uses harness-native fresh-context judges (`cross_model_check: same-model-collision`).

## Review resolution

The adversarial review used a harness-native fresh-context judge (`cross_model_check: same-model-collision`). Round 1 returned BLOCK: 2 CRITICAL, 4 WARNING, 3 SUGGESTION.

All findings were verified and incorporated:
- **C1:** the existing team-memory writer is now a B row, and the replace/wrap/migrate decision goes to analyze.
- **C2:** the A5 operator split is recorded in D.
- **Warnings:** the categories filter that MCP advertises but storage drops; the extended path corrections; D2 infeasibility; the A3 type-overwrite sites.
- **Suggestions:** the A1 budget redistribution, the A2 null `agent_id` link, and the date and dist notes.

Findings are recorded in `review/assess/findings.json`.

### Round 2 (PASS: 0 critical, 3 warnings, 2 suggestions), incorporated

- **The mini half of A5 is entirely operator-gated, not only `versions.toml`.** Mini's `versions.toml` requires each gitlink to equal its pinned commit, and `rules/test/versions-toml.test.mjs` fails on any disagreement. The gitlink bump and the operator's `versions.toml` edit therefore have to land in the same commit.
  - G1's mini half becomes "operator pin request filed".
  - D1, and any mini gate that needs v1.10.0, is blocked on the operator step.
- **D2 must also rewrite `lib/karpathy/hooks-budget.test.mjs`.** At line 51 it treats timeouts above 1000 as milliseconds, so after the change to seconds it would silently stop guarding anything. D2 must also unify the unit with `lib/kbd/hooks.mjs:78`, which already defaults to 15 seconds.
- **A2 must also fix `normalize_payload`'s `add_task_step` branch** (`pk-learning-worker/src/main.rs:1315`), which emits a null `agent_id` and `user_id`. A4's re-key must cover task-step records too.
- **The team-memory writer row needs two more items.** `user_id` and `session_id` default to null (`memory.mts:58-60`), and a `mapped-http` provider (`:72`) writes outside surreal-memory. The replace/wrap/migrate decision covers both. Recall of `['agent-team', scope]` categories depends on A4.
- **Mini has no Codex hook path today.** Its `.codex-plugin/plugin.json` declares no hooks, and `lib/distribution/package-builder.mjs:82-99` copies hooks into the Claude package only. Any Codex assertion in D1 or D2 means adding Codex hook emission there.

**Unresolved review findings:** none.
