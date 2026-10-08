# Analysis — team-aware-learning-memory-impl

**Mode:** stack specified (Rust for pk and surreal-memory, bash/python for pack hooks, TypeScript for the agent-team runtime, Node for mini).

**Research budget:** tier 1: 9 queries (repo inspection plus one runtime spike); tier 2: 0 (prior phase covered the docs); tier 3: 1; tier 4: 0. Within caps.

**Inputs:**
- `assessment.md`, including both review rounds;
- the prior phase's `analysis.md` (D-1…D-5) and `library-candidates.json`;
- the design and plan on `main`.

This phase is almost entirely **build**. The external landscape was researched last phase: no external memory runtime is adopted, and mem0, CrewAI, LangMem, Graphiti and Letta patterns are copied (cand-005/006). What remained were six internal decisions the assessment raised, plus one empirical unknown.

## Spike: Codex SubagentStart delivery (resolves the largest risk)

**Setup:**
- codex-cli 0.158.0;
- a fully scratch `CODEX_HOME`, with `auth.json` symlinked and the real `~/.codex/config.toml` hash unchanged before and after;
- a scratch `config.toml` with `[projects."<canonical path>"] trust_level = "trusted"`, `[features] hooks = true` and `multi_agent = true`;
- a project `.codex/hooks.json` with SessionStart and SubagentStart hooks;
- `codex exec --dangerously-bypass-hook-trust`.

| Attempt | Result |
|---|---|
| 1: `[features].codex_hooks` (deprecated name), no hook-trust bypass | Subagent ran, replied `NONE`; no hook fired |
| 2: `[features].hooks`, `--dangerously-bypass-hook-trust`, project hooks, matcher `*` | SessionStart fired. **SubagentStart fired** with `agent_type: "tlm_spike_role"` and an `agent_id` UUID. The child replied **`NONCE:SPIKE-7f3a`**. The rollouts prove the injection reached the child: the nonce first appears in the **child** thread as a `role: developer` message, before its reply. The parent holds it only as the child's relayed `agent_message` |
| 3: same, but hooks shipped in an **installed plugin** (`codex plugin marketplace add` + `codex plugin add` into the scratch CODEX_HOME), anchored matcher `^tlm_spike_role$` | Fired; the child replied **`NONCE:PLUGIN-9c21`**. This is the exact delivery path B5 ships |

Evidence: `evidence/codex-subagentstart-spike/` (the README records the prompt, isolation, per-run results, and the rollout lines that show the injection in the child thread).

**Consequences:**
- The prior phase's "UNVERIFIABLE" was caused by `--ignore-user-config` discarding trust, not by Codex lacking the feature.
- The design's Codex fallback (spawn_agent PreToolUse capture) is kept only as a **specced contingency**. It is not built. **Trigger:** the B5 alpha gate's Codex leg fails to show the role's token in the child thread through the plugin path on a future codex-cli version.
- Hooks also need **hook trust**. In normal interactive use, Codex shows its one-time hook trust prompt. Installation docs must say so; the gate uses the bypass flag on a scratch home only.

## Decisions

### D-1 Codex delivery path → native SubagentStart (adopt cand-002)
B5 ships the same `subagentstart-learning` hook to both harnesses through the generated hook bundles from `shared/harnesses/hook-contract.json`. The alpha gate's Codex leg uses the spike's harness, with a scratch CODEX_HOME, trust entry, `features.hooks` and the bypass flag.

**Rejected:**
- Building the PreToolUse-correlation fallback: unnecessary now, and it adds a stateful correlation file.
- Running the gate against the real `~/.codex`: it pollutes user config, as the prior lesson showed.

### D-2 Incumbent team-memory writer → adapt in place (cand-001)
`memory.mts` keeps its outbox, digest ids and idempotency, and changes three things:
1. The record content becomes the learning envelope, with `kind` mapped from its `scope`. The old `kind:'agent-team-memory'` wrapper is retired. Recall reads both forms during migration, recognising legacy records by the wrapper.
2. `agent_id` follows the design table (`<team>/<role>`, `<team>/@team`, `@project`).
3. `user_id` is no longer allowed to default to null. memory.mts stays input-driven: the **caller** (the team CLI entry `memory-queue`/`memory-publish`, or the orchestrator that invokes it) resolves the project id once through B1's resolver and passes it in the existing `scopeMapping.userId` / `kbd.projectId` inputs (memory.mts:15, :56-58). memory.mts **fails closed** when it is absent. No subprocess, and no layout-dependent path inside the runtime.

The `mapped-http` provider is unchanged and out of scope. Its records are not recalled.

**Rejected:**
- Replacing `memory.mts` with calls to `learning_write.py`: it puts a python subprocess in the TS runtime's hot path and discards a tested outbox.
- Leaving it as is: two incompatible attributed write paths, and recall misses team memories.

**New plan item:** this becomes PR **B3b**, after B3, in the agent-team-creator runtime.

### D-3 Mini v1.10.0 pin → operator-gated
- **A5 splits** into:
  - **A5a**, the skill-pack pin bump, done by an agent, with gate `install-binaries.sh`;
  - **A5b**, the mini gitlinks plus `versions.toml`, done by the operator in one commit. The agent files a pin request containing the exact commits and the `versions.toml` diff.
- **D1 splits** into:
  - **D1a**, the file-tier port, the envelope on mini's existing `deliverToPk` records, and the identity resolver. None of it needs v1.10.0, so it is not blocked.
  - **D1b**, the pk `--tag` / `--type` use in mini, blocked on A5b.
- G1's mini half is **not** satisfied by filing a request. It is tracked as **BLOCKED-ON-OPERATOR**, owned by the operator, and re-checked at reflect. G1 is MET only when A5b lands.

### D-4 Matcher collisions → per-role anchored matchers per harness, never rename agents
`shared/harnesses/hook-contract.json` has five separate SubagentStop groups, events[5..9]. Each carries its own phase args (`{evolution} assess|analyze|plan|…`). `scripts/generate-harness-adapters.js` emits one `matcher` per group and supports a per-group `harnesses` filter (lines 84-92).

**B2 does this:**
- **Keeps the five groups and their args.**
- **Claude Code and Kimi:** each matcher becomes anchored and requires the prefix: `^iterative-evolver:assessor$`, `^iterative-evolver:analyst$`, `^iterative-evolver:planner$`, `^iterative-evolver:executor$`, `^iterative-evolver:reflector$`. A bare team role named `planner` can no longer match.
- **Codex:** payloads carry the bare TOML agent name, with no prefix (spike payload). B2 therefore adds a Codex-only copy of each group (`harnesses: ["codex"]`) with matcher `^<role>$`. Its handler first runs B1's `agent_identity.py --is-team-role <agent_type>`, and exits 0 silently when the name resolves to a role in an active team. Codex keeps its current behaviour for genuine iterative-evolver agents, and team roles stop triggering it.
- **The executor group's Karpathy learning hook** moves to a matcher-less SubagentStop group that runs only when B1 resolves a team role, so it no longer keys on the literal `executor`.

The agents in `skills/process/iterative-evolver/agents/` keep their names, because their public ids belong to a separately distributed plugin.

**B2 gate:**
- for every generated output (claude, kimi, codex), no non-Codex group matcher matches any bare role id from `.agent-team/*/team.json` or the probe fixtures;
- the per-phase args survive regeneration unchanged;
- a SubagentStop payload with `agent_type: planner` from a team role triggers no iterative-evolver handler in either harness, while `iterative-evolver:planner` (Claude) or `planner` with no team (Codex) still does.

**Rejected:**
- one combined regex group: it loses the per-phase args;
- an optional prefix: it still matches bare names;
- renaming the agents: it breaks the plugin's consumers.

### D-5 pk context (A1) → score all, then budget
- In `run_context`, score every snapshot entry in each successful scope, and apply `max_candidates` **after** scoring as a cap on the merged candidate list.
- A failing scope's share is redistributed (assessment, line 708).
- Snapshot sizes are bounded by the KB, which today is in the hundreds of entries, so scoring is linear and cheap.
- **Order after scoring:**
  - score descending;
  - then scope priority, project before shared before global;
  - then entry id;
  - deterministic tie-breaking.
- A failed scope contributes nothing, and because the cap applies to the merged list, the surviving scopes use the whole budget.
- Paths: `pk-cli/src/main.rs::run_context` (lines 683-830). There is no `crates/` prefix.

**Rejected:** keeping per-scope `take` with a larger default. It still silently drops entries.

### D-6 surreal-memory (A4) → lean response DTO, categories in both legs, re-key as a v2 operation
- **Lean at the response boundary, not in storage.** `search_memories` re-ranks with `filter_map(|m| m.embedding.as_deref()?…)` (`crates/surreal-memory/src/storage/surreal.rs:2139-2146`). Loading records without embeddings would drop every result. The REST (`src/api/search.rs`) and MCP search handlers therefore map `Memory` to a lean DTO without `embedding` before serialising. `include_embeddings: true` opts back in.
- **Categories (any-match) in both hybrid legs.**
  - `search_memories` stops ignoring `_categories`. It adds `categories CONTAINSANY $categories` inside the KNN query, so the HNSW limit applies after filtering.
  - `hybrid_search_memories` (line 2533, which today passes `None`) threads categories into both the vector leg and the BM25 leg.
  - REST `SearchBody` gains `categories`.
- **Re-key** is a new operation kind in the existing `/api/v2/operations` framework (`src/api/mod.rs:120`). It reuses that framework's idempotency.
  - Request: `{kind: "rekey_agent_id", from: null, to_agent_id, user_id?, dry_run}`.
  - Predicate: `agent_id IS NONE OR agent_id = NULL`, so absent and null fields both match. Covers memories and task-step records.
  - **Authorization: loopback-only**, decided. The handler refuses non-loopback peers, consistent with the server's local deployment.
- **A4 gate**, against SurrealDB 3.3.0:
  - seeds records with both a NONE and a NULL `agent_id`;
  - asserts a non-empty lean result with no `embedding` key on both the REST and MCP paths;
  - asserts the categories filter on both hybrid legs;
  - asserts a dry-run count, then the re-key, then idempotent replay.

### D-7 Mini hook timeouts (D2) → seconds + budget test rewrite; no Codex hooks in mini
D2 makes these changes:
- converts `hooks/hooks.json` to seconds;
- unifies `lib/kbd/hooks.mjs`;
- rewrites `lib/karpathy/hooks-budget.test.mjs` to a seconds threshold;
- tests with `hooks/hooks.test.mjs`.

Mini has no Codex hook path today (`package-builder.mjs` copies hooks only into the Claude package). Adding one is **out of scope** for this phase, so D1a/D2 make no Codex assertions and the design's §9 Codex note is corrected.

## Corrections carried to spec

The following must be used verbatim in change files:
- assessment section D path table;
- A2 also fixes the `normalize_payload` `add_task_step` branch;
- A3 changes `pk-librarian/src/librarian.rs:413` and `pk-store/src/store.rs:394`;
- the broken `reflect:after` path in the orchestrator hooks is fixed in B4. The orchestrator ships in the pack under `skills/process/kbd-process-orchestrator/hooks/`, so the fix is a source change plus regeneration.

## Revised PR list (deltas against the plan)

| Plan PR | Change |
|---|---|
| A5 | → A5a (agent, skill-pack) + A5b (operator, mini) |
| — | **+ B3b**: adapt `memory.mts` to the envelope and keys (D-2) |
| B2 | namespaced matchers, no agent renames (D-4) |
| B5 | Codex leg uses the native path (D-1); fallback removed |
| D1 | → D1a (unblocked) + D1b (after A5b) |
| D2 | adds the budget-test rewrite and the `lib/kbd/hooks.mjs` unit; drops the Codex assertion |

The result is 22 PRs, one of them operator-owned.

**Dependencies, corrected:**
- B1 (resolver and identity) and B2 (matchers) need no v1.10.0, so they start immediately, in parallel with A1–A4. B2 depends on B1.
- A5a depends on all of A1–A4.
- The v1.10.0 consumers are B3 (live writes with attribution) and B4 (tag- and category-filtered recall).

**Critical path to the alpha gate:**

**max(A1 → {A2, A3}, A4) → A5a → B3 → B4 → B5**

B1 → B2 runs alongside, off the critical path.

## Open questions (none block spec)

1. D-4: does a pack hook see the plugin-prefixed agent type for a separately installed plugin? To be measured in B2.
2. ~~surreal-memory re-key auth~~: decided, loopback-only (D-6).
3. Codex hook trust in interactive use: document the one-time prompt in the B5 install notes.

## Review resolution

**Round 1** used a harness-native judge (`same-model-collision`) and returned BLOCK: 1 CRITICAL, 6 WARNING, 1 SUGGESTION. Each finding was verified and addressed:
- **CRITICAL (D-4 regex):** rewritten to per-role anchored, prefix-required matchers that keep the five groups and their args. Codex gets its own groups guarded by a team-role check, and the gate is extended.
- **Spike evidence:** run 2's rollouts now show the injection as a developer message in the child thread. Run 3 exercised the plugin-bundled hook path with an anchored matcher. The fallback is kept as a specced, unbuilt contingency with a trigger.
- **Critical path:** B1 and B2 re-rooted to the start, A5a depends on A1–A4, and the v1.10.0 dependency moved to B3 and B4.
- **D-2:** the caller passes the project id, memory.mts fails closed, and no subprocess is spawned in the runtime.
- **D-6:** a v2 operation, loopback-only, with a NONE-or-NULL predicate, a lean DTO at the response boundary, and categories in both legs.
- **D-3:** G1's mini half is BLOCKED-ON-OPERATOR, not MET.
- **D-5:** ordering, redistribution and the correct path are stated.

**Round 2:** harness-native judge, PASS, 0 CRITICAL, 7 WARNING, 2 SUGGESTION. All are incorporated as amendments that bind spec.

- **Kimi is removed from D-4 and the B2 gate.** The contract's `harnesses` list is `['claude-code','codex']`. Kimi hooks come from `capabilities.json` and have no SubagentStop and no matchers, so there is no collision to fix.
- **The Claude plugin-agent payload is measured, not assumed.** The prior phase's probe (behaviour 2, plugin form) recorded `agent_type: "tlmprobe:tlm-plug-role"`. Plugin agents report `<plugin>:<name>`. B2 still captures one installed `iterative-evolver` SubagentStop payload as its first gate step, before the matcher strings change.
- **The Codex SubagentStop payload is measured (spike run 4).** It fires through plugin hooks with an anchored matcher. It carries the bare `agent_type`, `agent_id`, `last_assistant_message` and `agent_transcript_path`.
  - B3's attributed-write hook uses `last_assistant_message` and `agent_transcript_path` directly.
- **The guard mechanism is now concrete.** Each hook in a group runs independently, so a check inside one hook cannot stop the others. The mechanism is a **wrapper target**, `shared/scripts/team-role-guard.sh <real-target> <args…>`, which every hook in the Codex copies points at. It exits 0 silently when `agent_type` resolves to an active team role, and otherwise execs the real target.
  - The Codex copies get new unique hook ids (`subagent-<role>-…-codex`).
  - The five original groups gain `harnesses: ["claude-code"]`.
- **No behaviour regression for the executor.** The iterative-evolver executor keeps its Karpathy learning hook in its own executor groups. Team-role learning is an **additional** group (matcher-less, guard inverted: it runs only for resolved team roles). Nothing that fires today stops firing.
- **D-6 re-key scope** is `memory` and `task_stream` records. `DbTaskStep` (surreal.rs:431) has no `agent_id`.
- **D-6 loopback-only needs wiring.** The server binds `0.0.0.0` by default and has no `ConnectInfo`. A4 therefore wires `into_make_service_with_connect_info::<SocketAddr>()` in `src/main.rs` and adds a per-kind peer check in `submit_operation`/`validate_payload` of **`src/operations.rs`** (the framework file; `src/api/operations.rs` does not exist). The re-key is documented as unavailable from container bridge networks; the operator runs it on the host.
- **The D-6 categories change** alters the storage trait signature of `hybrid_search_memories` (surreal.rs:2521) and every implementation and caller (REST, MCP, A2A). Spec lists them. The A4 gate also asserts that a categories-filtered KNN still returns `limit` rows when matches sit outside the unfiltered top-k. If SurrealDB 3.3.0 applies the HNSW limit before WHERE, the implementation over-fetches and post-filters instead.
- **Run 3/4 evidence** now includes `hook.sh` and the child rollout lines. The B5 gate must run the **generated** `codex-hooks.json` entry (`hook-dispatch-v1.sh … --harness codex`), not a raw script.

**Unresolved review findings:** none. Both rounds are done and every finding is resolved or carried as a binding spec amendment.
