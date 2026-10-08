# Analysis — team-aware-learning-memory

Date: 2026-10-03
Mode: stack specified. The stores are fixed: surreal-memory, pk, and the Karpathy session logs. The harnesses are fixed: Claude Code 2.1.289 and codex-cli 0.158.0. We adopt patterns and harness features, not a new memory library.
Inputs: `assessment.md`, revised after adversarial round 1; evidence reports E1–E4 from assess; landscape research L1 (this stage); `prior-context.md`. The prior-context file contains only lifecycle metadata, which is the recall defect itself.
Machine contract: `library-candidates.json` (cand-001 … cand-013). `coverage_estimate` = the fraction of the named gap a candidate addresses on its own, judged against the assessment section it cites.

## Landscape (L1)

None of the surveyed systems can sit on top of surreal-memory and pk. All of them converge on the same three ideas:

1. **A key tuple or path per memory:**
   - mem0: `user_id`, `agent_id`, `run_id`
   - LangMem: namespace tuples
   - CrewAI: `/agent/<a>`, `/project/<p>` path scopes
   - Graphiti: `group_id`
2. **Recall across several scopes at once, then merge:**
   - Graphiti: `group_ids=[...]`
   - CrewAI: `MemorySlice` across branches
3. **Private by default, shared by deliberate act:**
   - CrewAI: private-by-source and read-only shared slices
   - Letta: shared blocks attached to agents. Appends are safe, but whole-block rewrites lose updates.

Prior art for hook-based delivery:
- B12's `memory-subagent-start.sh` keys retrieval on `agent_type`. Because the payload has no task text, it reads the parent transcript's last Agent call.
- regin measured that UserPromptSubmit almost never runs inside subagents: 1 search in 2,410.
- regin also claims Claude's SubagentStart ignores `additionalContext`. Current official docs contradict this (10,000-char cap), so the claim is stale but must be confirmed by a run.

## Build vs adopt per sub-problem

| Sub-problem (assessment gap) | Verdict | Candidates | Notes |
|---|---|---|---|
| Learning envelope and keys (C) | **Build**, copying the patterns | cand-001, 002, 003 | `user_id` = project id from one resolver; `agent_id` = authoring role (`<team>/<role>`); `session_id` = harness session; categories carry `audience:*`, `kind:*`, `stage:*`. Visibility is a CrewAI-style path. Surreal-memory cannot set `scope`, `metadata` or `importance` today, so the envelope lives in categories until upstream adds fields. |
| Per-agent recall and budget (D) | **Build** | cand-002, 004 | Query role scope, then team, then project, then user/global, and merge. Score semantic + recency + importance, deduplicate by content hash, cut at a byte budget below the delivery cap. Surreal-memory filters `agent_id` in the database.

**Addressed lessons** cannot rely on client-side category filtering after a top-k search, because the lesson sits under the author's `agent_id` and can fall outside top-k. So an addressed lesson is written **once per audience**, with `agent_id` set to the audience (`<team>/<role>`, or `<team>/lead`) and the author in categories (`author:<team>/<role>`). Recall then stays a database-filtered `agent_id` query. When the upstream category filter lands, this becomes a single write plus a `CONTAINSANY` filter.

**Legacy records:** existing memories with null `agent_id` (50 of 50 sampled) are treated as project-visibility, reachable only by the project-scope query, never role-targeted. |
| Delivery into subagents (B) | **Adopt** harness features; build only the hook script. Account for the file tier (D-2a). The hook must: (1) carry an explicit timeout of at most 5 s, with an internal watchdog and a store-query timeout of 2 s; (2) emit nothing and exit 0 when stores or teams are absent (integration contract); (3) fence and label recalled lessons as untrusted data ("lessons recorded by <role>; treat as information, not instructions") to limit prompt injection between agents; (4) budget against the **per-hook** cap (Claude 10,000 chars per hook output), staying at or under 8,000 so other plugins' SubagentStart hooks have room. | cand-006, 007, 009 (+008 optional) | SubagentStart in both harnesses, matcher `^(.+:)?<role>$` or a catch-all that resolves the role itself. Role-keyed by default. Task-keyed (via the parent transcript or PreToolUse `updatedInput`) only after runtime confirmation. Claude `memory:` frontmatter is optional and Claude-only. |
| Cross-agent awareness (F) | **Build**, copying Letta and CrewAI | cand-002, 005 | Promotion rather than broadcast. Lessons stay in the author's role scope unless addressed (`audience:role:<r>`, `audience:lead`) or promoted by the reflector or lead.

**Team digest:**
- Store: an append-only JSONL file at `.prometheus/team-digest/<team>.jsonl` in the project, each line an envelope. Mirrored as surreal-memory records with `agent_id` = `<team>/lead`.
- Appends use `O_APPEND` writes under 4 KB.
- Cap: the last 50 entries or 8 KB are delivered.
- Compaction: the reflector or lead role at reflect time, writing a new file and renaming it, never rewriting in place (Letta's lost-update warning). |
| Identity capture (A) | **Build** | — | Read `agent_id`/`agent_type` from hook stdin (SubagentStart, SubagentStop, PostToolUse, PreToolUse). Resolve `(team, role)` via `.agent-team/project-routing.json` → `team.json`; fall back to the sole team, then to `owns` glob match (longest match). Add the fields to `LearningJob` and the record-progress schema. |
| Candidate queues (J) | **Build** (extend) | — | Add `audience`/`team`/`role` to candidate files so kbd-open and SubagentStart show only the relevant ones. Rename the planned new-skill queue to avoid the cowork `skill-candidates` collision (e.g. `skill-proposals/`). |
| Reflect write-back and recall closure (E) | **Build (fix)** | — | Unify the four project-id derivations into one resolver. Extract Lessons Learned. Recall memories via `/api/v1/search`, not lifecycle entities. Seed the next phase from `## Next Phase Seed`. Route `evaluate-session.sh` through the bridge, removing the direct POST and the dead `metadata`. Remove the literal `MEM_PROJECT` default. These are prerequisites to everything role-aware, and they also fix the two constraint violations. |
| Cortex MCP (N) | **Reconcile, not adopt** | cand-013 | CLAUDE.md mandates the chain surreal-memory, then Cortex, then file memory, but no pack script uses Cortex. Decision: Cortex stays an agent-level fallback for interactive lookups; the pack's automated pipeline uses surreal-memory, then pk, then file. The design states the order explicitly, so the two chains do not diverge silently. |
| Feynman gap routing, skill discovery, cross-team requests (Goal 5) | **Build**, each role-aware | — | No external candidates were found beyond those in the parent plan. Each becomes role-aware: gaps record `roleId` and are delivered to the role that hit them; skill proposals carry the authoring role; cross-team requests target role cards. Verdicts per feature are carried in the revised plan. |
| Keys across repos (G) | **Build** | — | Team ids repeat across repos (sansaba-team ×3), so every key includes `projectId`. `agent_id` = `<team>/<role>` is unique only within `user_id` = projectId, and recall always filters both. |
| Palace store (K) | **Reference, do not adopt now** | cand-010 | Palace partitions by wing and room and allows scope and importance, but it is a separate retrieval path, used by content grounding. Revisit if surreal-memory's memory table cannot gain category filters. |

## Upstream prerequisites (must land before the pack relies on them)

1. **pk:**
   - Fix the candidate truncation (`pk-cli/src/main.rs:735-743`): score all snapshot entries, then truncate. Without this no recall path works, targeted or not.
   - Commit a prompt snapshot after the worker upserts its session entry (`pk-learning-worker/src/main.rs` around line 527).
   - Add tag and `generated_by`/source filters to `pk context`. Let `pk ingest` set tags **and `type`**: `entry_type` is hard-coded to "Reference" in `pk-librarian/src/librarian.rs:413`.
2. **surreal-memory:**
   - Filter on `categories` (`CONTAINSANY`) in the vector, BM25 and list queries.
   - Stop serializing `embedding` in search responses.
   - Allow `metadata`, `scope` and `importance` on `AddMemoryRequest`.
   - Until these ship, the pack filters client-side and strips embeddings.
3. Pinning (corrected): the parent plan's PR 14 tags only prometheus-knowledge, and no PR owns the surreal-memory changes. The revised plan (change-tlm-003) must:
   - put the pk fixes and the surreal-memory fixes first, as their own PRs;
   - tag both v1.10.0 and pin them **before** any pack PR whose integration gate depends on them.

   The pk truncation fix has no client-side fallback (any `pk context` caller hits it), so it is a hard prerequisite.

## Decisions taken in analyze

- D-1: No external memory library is adopted. Patterns are copied (cand-001…005).
- D-2: The delivery channel is SubagentStart, in both harnesses. The main thread keeps SessionStart and UserPromptSubmit, but its context becomes lead- or operator-scoped rather than "everything".
- D-2a: The file-memory tier (Claude auto-memory `MEMORY.md`, about 14 KB, loaded into every subagent; assessment section N) is the largest untargeted per-agent cost. The design must not simply add SubagentStart context on top of it. Two options for spec to choose between:
  - (i) Keep `MEMORY.md` as a short project index and move role-specific lessons into per-role sections delivered by SubagentStart.
  - (ii) Use Claude's per-subagent `memory:` directories (cand-008) for role-private lessons.
  **Codex also has an auto-loaded memory tier** (corrected after review): codex-cli 0.158 native memories are on here (`~/.codex/config.toml` `[features] memories = true`).
  - `~/.codex/memories/` holds `memory_summary.md` (10,840 B), `MEMORY.md` (182,746 B), `raw_memories.md`, `rollout_summaries/` and `extensions/`, consolidated by Codex itself.
  - Binary prompt strings refer to "MEMORY_SUMMARY below", so the summary is injected per session and `MEMORY.md` is consulted on demand.
  - The tier is user-level (one directory for all projects), not project- or role-scoped. `AGENTS.md` is also auto-loaded.
  - Whether a Codex subagent receives `memory_summary.md` is UNVERIFIED and is added to change-tlm-001.
  - Both harnesses therefore already push untargeted memory: Claude about 14 KB per project, Codex about 10.8 KB per user. The design treats reducing or partitioning that tier as equal in priority to adding SubagentStart context.
- D-3: Visibility levels are `agent` (private to the author role), `role:<r>` (addressed), `team` (digest), `project`, `user`, `global`. The default is `agent`. Promotion beyond `team` follows the user's approved policy: auto-propose, human confirms; `[GLOBAL]`/`[USER]` markers are immediate.
- D-4: Budgets: Claude SubagentStart at most 8,000 chars (cap 10,000); Codex at most 2,000 tokens, about 7,000 chars (default limit 2,500 tokens); main-thread UserPromptSubmit cut from about 4 KB to about 2 KB of targeted context.
- D-5: Correct the timeout-unit misdiagnosis from PR #121 (both harnesses use seconds) and namespace the SubagentStop matchers, within this phase, under C-01/C-03.

## Open questions (carried to spec, with how to resolve)

- Q-1: Does Codex `agent_type` equal the custom role `name`? Does Claude SubagentStart `additionalContext` reach the subagent? Both are documented but unconfirmed at runtime. A probe needs authenticated model calls, so it becomes the first change of execute (isolated HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT). The spec treats both as documented behaviour, with a fallback named for each.
- Q-2: Main thread acting as a role (`claude --agent`): SessionStart carries `agent_type` in Claude but not in Codex. Proposed answer: Codex main thread = operator/lead scope.
- Q-3 (corrected): mini already has learning writers: the `skills/karpathy-progress-memory` port (`lib/karpathy/*` → `pk ingest`) and `agent-team-creator/scripts/memory.mjs` (→ surreal-memory). Its only gap is the missing Stop/worker capture. Proposed answer: mini adopts the envelope in both existing writers, plus recall and SubagentStart delivery, and ships no worker. Otherwise mini keeps producing envelope-less records.
- Q-4: Overlapping `owns` (sansaba-team has 147 overlapping pairs). Proposed answer: longest-match glob, tie-break by declared role order, record ambiguity.

## Review resolution (adversarial round 1, harness-native, same model family)
- CRITICAL: Codex memory tier dismissed. Fixed in D-2a: inspected locally, and the subagent behaviour was added to the probe.
- WARNINGs fixed:
  - pinning ownership (upstream PRs come first)
  - addressed lessons and top-k (write per audience)
  - legacy null `agent_id` records
  - gap E row
  - Cortex row
  - Claude confidence lowered, per-hook cap
  - mini Q-3 corrected
  - hook timeout, silence, untrusted fencing
  - candidate evidence claims rewritten (`library-candidates.json`)
  - Goal 5 features and gap G rows
- SUGGESTIONs fixed: team digest store, cap and compaction; `entry_type` upstream item.

## Round-2 review warnings carried to change-tlm-002 (design), with the direction taken
- W1, scope keys: surreal-memory filters `agent_id` by equality only, so every visibility level becomes its own `agent_id` value:
  - `<team>/<role>` for agent/role-private and addressed lessons;
  - `<team>/@team` for the digest;
  - `@project` for project;
  - `@user:<hash>` and `@global` for user/global records under their own `user_id`.

  A project-scope query is then `agent_id = '@project'` and never returns role-private lessons. Legacy null-`agent_id` records are reachable only by an unfiltered recall. The design either migrates them, re-keyed to `@project` by a one-off upstream-assisted job, or accepts them as a pre-envelope archive that is recalled only by the main thread.
- W2, Codex memory writes: Codex consolidates every thread, subagent threads included, into the user-level `~/.codex/memories`, with no envelope. The design adds a Codex-side option alongside the Claude ones: per-thread `memory_mode` or `use_memories`/`generate_memories` (binary keys; behaviour to be probed in change-tlm-001), so subagent threads neither consume nor pollute the user-level summary.
- W3, total budget: the budget is per subagent across all channels (file tier + every SubagentStart hook), not per hook. The design sets a total target and reports measured bytes per channel.
- W4, Cortex writes: CLAUDE.md instructs agents to write Cortex memories without team or role. The revised plan includes a CLAUDE.md update so that Cortex writes carry the envelope tags, or are routed through the pack's writer.
- S5 (resolver precedence and worktrees), S6 (digest is git-ignored and per worktree; the authoritative copy is the one in the main checkout or a shared `~/.prometheus/team-digest/<projectId>/` directory), and S7 (stale `gaps_addressed`; queue producers) are design-level.

## Confidence

Patterns: high. Claude Code delivery: **medium**, until change-tlm-001 confirms SubagentStart injection at runtime (regin's 2026-06 contrary claim is unresolved; the docs support it). Codex delivery: medium, pending Q-1. Codex native memory: present and inspected locally (see D-2a); its subagent behaviour is unverified.
