ASSESSMENT: team-aware-learning-memory
Project: prometheus-skill-pack (runtime project 6ac090a4-3656-4d83-8eb6-2891508196d5)
Date: 2026-10-03
Codebase baseline: skill-pack origin/main 20d97f2 (after PRs #111, #121, #122, #123); prometheus-skills-mini origin/main after PRs #27/#28; surreal-memory-server 777cf72 (v1.9.0); prometheus-knowledge 1bbaecc (v1.9.0); Claude Code 2.1.289; codex-cli 0.158.0. Paths are skill-pack repo-relative unless prefixed: `orch/` = `skills/process/kbd-process-orchestrator/`, `atc/` = `skills/process/agent-team-creator/runtime/src/`, `pkw/` = prometheus-knowledge `pk-learning-worker/src/`, `sm/` = surreal-memory-server.
Cross-tool progress: none. This phase was created at runtime revision 1366 and has no registered changes. The runtime's previous active phase, `agent-team-creator` (reflect stage), belongs to a worktree that no longer exists. This checkout's waypoint projection had been stale since revision 1292.

## Evidence sources
- Four read-only investigation reports, produced in this session and summarized below:
  - E1: memory/Karpathy/Feynman wiring
  - E2: agent-team runtime identity
  - E3: harness hook identity and delivery, from official docs plus the codex 0.158 binary schemas
  - E4: store targeting and baseline cost, from source plus live read-only queries against :23001 and `pk context`
- Live observation from the `assess:before` hook for this phase. `prior-context.md` was written with project `unknown`, five lifecycle events and zero lessons.
- Probe results from PR #121 (Codex hook execution form).

## IMPLEMENTATION STATUS

### A. Agent identity at the point of read and write
**Status: MISSING (capture). The identity is available from the harnesses, but the pack never reads it.**

What the harnesses expose:
- Claude Code: every hook that fires inside a subagent carries `agent_id` and `agent_type` (E3, hooks.md).
- Codex: SubagentStart and SubagentStop carry `agent_id` and `agent_type`. PreToolUse, UserPromptSubmit and PreCompact declare them as optional fields (binary schema). Codex SessionStart has no agent fields.

How names map to roles:
- Claude Code: `agent_type` is the agent's frontmatter `name`. For plugin agents it is `plugin:name`.
- Codex: `agent_type` is the role `name`, with no plugin prefix (inferred from docs and binary; to be confirmed at runtime).
- Agent-team native exports name each agent after `role.id` (atc/adapters-local.mts:25-49), so `agent_type` resolves to `(team, role)` through `.agent-team/project-routing.json` (activeTeam) and then `.agent-team/<team>/team.json`.

What the pack does with it:
- `enqueue-learning-job.py` (148-159) drops `agent_id`/`agent_type`. `LearningJob` (pk-learning-worker main.rs:89-103) has no agent field.
- `record-progress.py` uses an allow-list (207-215) that rejects any identity field.
- The SubagentStop fallback reads `SUBAGENT_NAME`, which is never set.
- No hook script reads `agent_type`.

Inside an agent:
- The team id is not visible in a running agent's prompt, frontmatter or environment (E2). Only `Role: <id>` appears in the generated prompt.

### B. Per-agent delivery channel
**Status: MISSING. A channel exists in both harnesses.**

- SubagentStart can inject into only that subagent:
  - Claude: `additionalContext` is "added to the subagent's context ... before its first prompt", and is deduplicated on resume.
  - Codex: plain stdout or `additionalContext` is added as developer context, with a default limit of 2500 tokens.
- The pack registers no SubagentStart hook.
- Subagents get no SessionStart or UserPromptSubmit hook context today (observed in E4's subagent; consistent with the docs). They do receive the **untargeted file-memory tier** (section N): the full project auto-memory `MEMORY.md` (13,928 B, indexing 115 files), plus global and project CLAUDE.md files. So today every subagent gets the same about 14 KB of memory regardless of its role, and none of the pack's targeted lessons.
- PreToolUse on `Agent` / `spawn_agent` with `updatedInput` can rewrite a single spawn's prompt. This is documented generically and has not been tested on the spawn tool.

### C. Learning envelope: the fields every write carries
**Status: MISSING.**

surreal-memory:
- It persists `agent_id`, `session_id` and `categories`.
- `scope`, `memory_type`, `metadata` and `importance` are hard-coded and cannot be set through any API (`sm/src/contracts.rs:93-119`, `sm/crates/surreal-memory/src/memory.rs:164-193`).
- Live data, 50 of 50 sampled records: `agent_id` null, `metadata` null, `scope` global.

pk:
- `WikiEntry` has tags, sources, `entry_type` and `extra`.
- `pk ingest` cannot set tags or type. `entry_type` is hard-coded to "Reference" (prometheus-knowledge `pk-librarian/src/librarian.rs:413`).

Karpathy session records and the learning log:
- No agent, role or team field anywhere.

### D. Targeted recall
**Status: MISSING (filters). Partial store support.**

surreal-memory:
- Filters by `user_id`, `agent_id` and `session_id` in the database query, for both vector and BM25 search.
- `categories` is accepted by MCP `search_memories` and ignored (`sm/crates/surreal-memory/src/storage/surreal.rs:2096`). No API filters on categories or metadata.
- No byte or token budget is applied.
- Every response includes a 384-float embedding: 18.7 KB for 3 hits.

pk:
- `pk context` filters only by scope.
- Scoring is substring counts. Tags are ignored, and there is no IDF or length normalization.
- **Defect:** only the first `ceil(max_candidates/#scopes)` entries per scope, in id order, are ever scored (prometheus-knowledge `pk-cli/src/main.rs:706-743`, with snapshots sorted by id in `pk-store/src/prompt_snapshot.rs:34`). With the defaults that is 43 per scope. The project KB has 548 entries and shared has 805, so only ids starting roughly a–c are reachable. None of the 288 `karpathy-session-*` entries is ever returned.
- This is why every UserPromptSubmit context in this session has shown unrelated `android-*`, `agent-*` and `bible-*` entries.

KBD recall:
- `kbd-memory-recall.sh` queries `/api/v1/entities/search?q=kbd_lifecycle_event`, which returns metadata, not memories. No script in the pack calls `/api/v1/search` or `search_memories`.

### E. Reflect write-back to recall closure (per E1)
**Status: BROKEN.**

- `memory-writeback.sh` extracts only Delta, Root Cause and Corrective Actions. The template `orch/prompts/reflect.md` emits Lessons Learned and Next Phase Focus. The sycophancy gate forces the three sections on retry, but Lessons Learned is never captured.
- Project-id derivation differs across four writers:
  1. `shared/scripts/lib/memory-bridge.sh:16`: `PROMETHEUS_PROJECT_ID`, else the literal `prometheus-skill-pack`.
  2. `orch/skills/kbd-memory-recall/kbd-memory-recall.sh`: `.kbd-orchestrator/project.json` `.project // .projectId`, else `unknown`. That fallback is the `unknown` seen live.
  3. `pkw/main.rs:1537` `project_scope()`: `.prometheus/project.json` `projectId`, else `project:<sha256(canonical path)>`.
  4. `shared/scripts/evaluate-session.sh:61-63`: hard-coded `user_id:"prometheus-skill-pack"`. It also sends a `metadata` object that `AddMemoryRequest` (`sm/src/contracts.rs:93-102`) silently drops.
- The worker never commits a prompt snapshot after upserting its session entry (`pkw/main.rs:504-562`). Session entries therefore stay invisible to `pk context` even without the truncation defect.
- Next-phase seeding matches the boilerplate "Context for Next Phase" heading.

### F. Cross-agent awareness
**Status: MISSING.**

- Agent-team handoffs carry `memoryRefs`.
- The team memory outbox publishes with a team-level `agent_id` and categories `['agent-team', scope]`. Nothing reads it back.
- All 13 team state files on this machine are empty (0 tasks, handoffs or outbox entries).
- No mechanism routes one role's lesson to another role, by owned path or otherwise, and there is no team-lead rollup.

### G. Role metadata usable for routing
**Status: PARTIAL.**

- `team.json` roles carry `owns`, `skills`, `inputs`, `outputs` and `dependsOn`, validated by schema. Every role in the roughly 17 real teams found on this machine has non-empty `owns`.
- `owns` is advisory and can overlap. sansaba-team has 147 overlapping pattern pairs. A matcher with a longest-match tie-break is needed.
- One `activeTeam` per repository. There is no per-component routing, and team ids repeat across repos (sansaba-team appears in three), so keys need `projectRoot`.

### H. Matcher hygiene
**Status: DEFECT.**

- SubagentStop matchers are bare names: `assessor`, `analyst`, `planner`, `executor`, `reflector` (shared/harnesses/hook-contract.json:118-251).
- They do not match plugin-shipped Claude agents (`plugin:name`).
- They collide with real team roles: Ghostex has `planner`, and `~/.claude/agents/planner.md` exists. A team role named `executor` would fire the Karpathy learning hook.
- The iterative-evolver agents these matchers target are never dispatched.

### I. Hook timeout units
**Status: DEFECT, partly introduced by this session.**

- Both harnesses read `timeout` as seconds (Claude docs; Codex docs and binary: "timeout is in seconds", default 600).
- The contract uses 1000–35000, which works out to roughly 17 minutes to 10 hours.
- The same unit defect exists in prometheus-skills-mini `hooks/hooks.json` (1000–15000).
- PR #121 added `CODEX_MIN_HOOK_TIMEOUT_MS` = 5000 and documented Codex as using milliseconds. That was a misdiagnosis: the failing hook was the JSON-stdout one, and the failure persisted after the floor was added. The code, CLAUDE.md and docs/codex-plugin.md need correcting.

### J. Candidate queues: promotion, new-skill, knowledge-gap (Goal 1 inventory)
**Status: MISSING (producers). A consumer exists.**

| Queue | Path | Producer today | Consumer today | Live state |
|---|---|---|---|---|
| promotion candidates | `~/.prometheus/promotion-candidates/pending` | none | `shared/scripts/kbd-open.sh` (PR #123) | absent |
| new-skill candidates | `~/.prometheus/skill-candidates/pending` | none | `kbd-open.sh` | absent |
| skill-update candidates | `~/.prometheus/skill-updates/` | `shared/scripts/propose-skill-update.sh`, which nothing invokes | `kbd-open.sh`, `pmpo-skill-creator --update` | empty |
| knowledge gaps | `~/.prometheus/knowledge-gaps/gaps.jsonl` | none | `kbd-open.sh` | absent |
| learning jobs | `~/.prometheus/learning-queue/pending` | Stop and SubagentStop[executor] → `enqueue-learning-job.py` | `prometheus-learning-worker` (launchd) | drained (0) |
| memory operations | `~/.prometheus/learning-queue/memory/pending` | `memory-bridge.sh`, `record-progress.py`, worker | worker → `sm` `/api/v2/operations` | drained (0) |

None of these queues carries agent, role or team, and the kbd-open renderer has no audience filter. Every candidate would be shown to every session.

Naming collision: `skills/process/cowork-management/scripts/discover-skills.sh` already emits a `skill-candidates` document (schema `skills/process/cowork-management/assets/schemas/skill-candidates.schema.json`). Those are external skills found for a capability gap, unevaluated and never installed. The planned `~/.prometheus/skill-candidates/` queue holds new skills inferred from transcripts. The two need distinct names or a shared `kind` discriminator.

### K. Other readers missed in the first draft
- surreal-memory palace store: `POST /api/v1/palace/recall` is called by `shared/scripts/content-grounding.sh:219,243`, `shared/scripts/content-grounding-kb.sh:382,410` and `skills/learn/learn-goal/scripts/content-grounding.sh`. Palace drawers are a separate store, partitioned by wing and room. They are a candidate home for role-partitioned or team-partitioned knowledge, and they need evaluating in analyze.
- learner-model `GET /api/v1/due` (FSRS) in `kbd-open.sh`. This is per learner, not per agent.

### L. Agent identity at the remaining hook points (Goal 1)
Per E3, Claude Code docs:
- **Stop:** fires for the main thread. Its input has no `agent_id`. Subagent completion is SubagentStop (with `agent_type`, `agent_transcript_path` and `last_assistant_message`). The current Stop learning job therefore always describes the main session. Subagent work is attributed to the parent `session_id` (E4: `karpathy-session-7629f9706cdb9fd5`).
- **PostToolUse** (memory-writeback, scope-record and the others): inside a subagent the input carries `agent_id` and `agent_type`, so a reflection written by a subagent can be attributed. Today the scripts ignore these fields.
- **TaskCompleted** (Claude-only, `kbd-task-completed-gate.sh`): the docs list it among the events, but E3 did not establish its agent fields. UNVERIFIED.

Codex:
- Stop, SessionStart and PostToolUse inputs have no agent fields in the 0.158 schema.
- PreToolUse, UserPromptSubmit and PreCompact declare optional `agent_id` and `agent_type`.
- SubagentStop has them, but its output cannot carry additionalContext.

### M. Mini repo baseline (prometheus-skills-mini origin/main 56cbf12)
- Hooks (`hooks/hooks.json`):
  - SessionStart: kbd-control, project context
  - PostToolUse (Write|Edit): position reminder
  - **SubagentStop `*`: `subagent-fallback-checkpoint`**, which reads `subagent_name`. Nothing populates it, the same defect as the skill-pack's `SUBAGENT_NAME`.
  - TaskCompleted and PreCompact
  - There is no Stop learning-job hook and no pk-context prompt hook.
  - Timeouts are 1000–15000, the same unit defect as section I.
- Learning capture **exists**:
  - `skills/karpathy-progress-memory` plus `lib/karpathy/*` (event, canonical-state, fixtures) port `record-progress.py` and write to `pk ingest`.
  - `skills/agent-team-creator/scripts/memory.mjs` posts the identity/content envelope to surreal-memory.
  - Neither carries a per-role agent identity, and nothing reads per role.
- KBD recall is ported to JavaScript (`scripts/kbd-memory-recall.mjs`, `lib/kbd/memory.mjs`). It has the same lifecycle-metadata recall defect.
- agent-team-creator is byte-identical to the skill-pack (PR #27).

### N. File-memory tier: Claude auto-memory, CLAUDE.md memory chain, Cortex
**Status: PRESENT, UNTARGETED.**

- Claude Code auto-memory: the project memory directory `~/.claude/projects/<project>/memory/` holds 115 files. Its `MEMORY.md` index (13,928 B) is loaded into every session and every subagent. The round-2 reviewer subagent observed it in its own context.
- CLAUDE.md makes a lookup chain mandatory: surreal-memory MCP, then Cortex MCP, then file memory. Agents write file memories after features (types user, feedback, project, reference; "GLOBAL" naming for cross-project lessons).
- This is currently the **largest and only memory channel that reaches subagents**. It is keyed by project directory only: no role, no team, no budget beyond the 24.4 KB index limit. It is written by whichever agent does the work, and it is read by all of them.
- Cortex MCP (`cortex_recall`/`cortex_remember`) is configured in the harness. No pack script uses it.
- Implications for the design:
  - The file tier is the de facto "file" fallback that goal 3 names.
  - Any per-agent targeting must either stay within it (for example per-role index sections, or Claude's per-subagent `memory:` frontmatter directories) or reduce what it injects into subagents. Otherwise targeted SubagentStart context only adds to the about 14 KB every agent already receives.
  - The file tier is Claude-specific. Codex has no equivalent auto-loaded memory index (cand-008 analysis).

## CROSS-TOOL PROGRESS
NONE. No cross-tool activity has been recorded for this phase. The predecessor runtime phase `agent-team-creator` is orphaned (its worktree is deleted) and still at its reflect stage.

## SPEC GAP SUMMARY
- No OpenSpec capability covers learning or memory targeting. The integration contract (`docs/integration-contract.md`) has no learning-envelope seam.
- The approved plan, PRs 4–13 in `~/.claude/plans/yes-fix-it-in-logical-mccarthy.md`, assumes project/user/global scopes only. It has no agent or role dimension, so PRs 4–13 must be revised against this phase's design before implementation.
- The `pk context` truncation defect and the missing worker snapshot commit sit upstream of every recall path. Any targeting design is ineffective until both are fixed.

## Baseline cost (measured, E4)
| Surface | Who receives it | Bytes | Relevance |
|---|---|---|---|
| UserPromptSubmit `pk context` | main thread, every prompt | ~4.0 KB (3,949–4,029 over three prompts) | Low. The truncation defect returns alphabetically-first entries |
| SessionStart kbd-open | main thread, once | ~8.5 KB (last real snapshot) | Phase-level, not role-level |
| SubagentStart | each subagent | 0 B (no hook registered) | n/a |
| Claude auto-memory `MEMORY.md` (file tier, N) | main thread **and every subagent** | 13,928 B | Untargeted. Same for every role. |
| KBD stage start `prior-context.md` (`orch/skills/kbd-memory-recall/kbd-memory-recall.sh`) | the stage agent, only when its SKILL.md says to read it (analyze does; assess, plan, execute and reflect do not) | 622 B for this phase | Lifecycle metadata only, project `unknown`, no lessons |
| surreal-memory search payload | n/a (no reader) | 18.7 KB for 3 hits, mostly embeddings | n/a |

Per-agent targeted context today: **0 bytes targeted. Every subagent gets about 14 KB of untargeted file memory. The main thread additionally gets about 4 KB of mostly irrelevant pk context per prompt and about 8.5 KB at session start.**

## BUILD HEALTH
- build check: UNKNOWN for this phase, since no code has changed. PRs #121–#123 passed their local gates (`check:distribution`, `validate:harness-adapters`, `validate:codex`, and `cargo check --locked` for prometheus-cli and forge-rs).
- known violations: hook timeout units (I) and matcher collisions (H).
- test coverage: MINIMAL for learning and memory. There is no integration test proving write → recall closure. The memory-bridge shell test covers queueing only.

## CONSTRAINT CHECK
- AGENTS.md / CLAUDE.md violations:
  - `shared/scripts/evaluate-session.sh:58-64` POSTs directly to surreal-memory, contrary to the rule that hooks never call the memory service (stated in `shared/scripts/lib/memory-bridge.sh:12-14`).
  - The memory-bridge default `MEM_PROJECT=prometheus-skill-pack` mis-scopes writes from every other project.
- Integration contract: any new delivery path must stay silent when surreal-memory, pk or agent teams are absent, as the "capability is discovered, never assumed" rule requires.
- `.kbd-orchestrator/constraints.md` (tracked on origin/main) applies to every change in this phase:
  - C-01: regenerate generated outputs (hook-contract → `hooks/hooks.json`, `hooks/codex-hooks.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/hook-dispatch-v1.sh`, `dist/plugins/*`) in the same change, or name one reconciliation change. `npm run validate:codex` must pass after touching `hooks/hooks.json`.
  - C-03: any change to Codex hooks updates `docs/codex-plugin.md` and the CLAUDE.md Codex section.
  - C-04: generators stay idempotent.
  - C-05: launchd-invoked scripts must be bash 3.2 compatible.
  - Every change also needs QA plus an independent diff review before archive.
  - These rules bind fixes H (matchers), I (timeout units) and the new SubagentStart hook.

## GOAL PROGRESS
1. Inventory read/write paths and identity at each point: **PARTIAL**. Write paths, read paths (including palace and learner-model, section K), candidate queues (J) and identity per hook (A, L) are inventoried. Still unconfirmed at runtime: Codex `agent_type` equals the role name; Codex PreToolUse inside a subagent carries `agent_id`; `updatedInput` on the spawn tool is honored.
2. Learning envelope design: **NOT MET**. Inputs are gathered: store fields in C, gaps in D.
3. Per-agent recall design with byte measurement: **NOT MET**. The baseline is measured (see table).
4. Agent-attributed writes and cross-agent awareness design: **NOT MET**. Inputs are in F and G.
5. Revised plan for PRs 4–13 with per-agent integration gates: **NOT MET**.

## Risks and open questions for analyze
- Runtime confirmation (Codex): run a probe with a custom `.codex/agents/<role>.toml`, isolated via HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT, to confirm `agent_type` and SubagentStart injection. Claude has documentation evidence only.
- Visibility model: what should be private to one agent, shared with a role, the team, the project, the user, or global? And who promotes between levels? This interacts with the approved "auto-propose, human confirms" promotion policy.
- Budget: the per-agent byte cap and its default (Codex caps SubagentStart context at 2500 tokens by default).
- Upstream changes needed in surreal-memory (category filter, settable metadata/scope, no embeddings in search responses) and pk (fix candidate truncation, tag/source filters, settable type and tags). These must land and be pinned (v1.10.0) before the pack can rely on them, or the pack needs a client-side fallback.
- Main thread versus subagent: when the main session is itself acting as a role (`claude --agent <name>`, or the operator), SessionStart carries `agent_type` in Claude but not in Codex.
- Unconfigured repos: about a third of the real teams have no `project-routing.json`, so role resolution needs a fallback such as the sole team or `owns` matching.

## Review resolution (adversarial-review round 1, harness-native, same model family)
- CRITICAL constraints.md "absent": fixed. The constraint check now applies C-01, C-03, C-04 and C-05.
- CRITICAL candidate queues not inventoried: fixed in section J, including the cowork `skill-candidates` naming collision.
- WARNING missed readers and hook identity: sections K and L.
- WARNING unnamed project-id derivations and dropped `metadata`: named in section E.
- WARNING MINI omitted: section M, plus the baseline line.
- SUGGESTION ambiguous paths: path prefixes are declared in the header.

## Review resolution (adversarial-review round 2, harness-native, same model family)
Round-1 findings: 4 resolved and 2 partly resolved (MINI, paths), per the round-2 reviewer. Round-2 findings and the fixes applied after it:
- CRITICAL: file-memory tier omitted, so "subagents receive nothing" was wrong. Fixed: section N added, and section B and the baseline corrected (about 14 KB untargeted per subagent).
- WARNING: section M facts. Fixed: mini SubagentStop fallback, mini Karpathy port and team memory writer, mini timeout defect, and the SHA 56cbf12.
- SUGGESTION: stage-start delivery missing from the baseline. Fixed: `prior-context.md` row added.
- SUGGESTION: path prefixes incomplete and rule misattributed. Fixed for every cited source file. The memory-bridge rule is now attributed to `memory-bridge.sh:12-14`.

## Unresolved review findings
The two-round adversarial cap was reached. The round-2 fixes above were applied without a third review, so the analyze stage must treat sections B, M, N and the baseline table as **reviewed-once**. Both rounds ran harness-native (same model family). The REST judge returned `JUDGE_MODEL_COLLISION` (exit 4) because no distinct model is configured. The cross-model guarantee is therefore **not** met, and a `pending_review` for the cross-model judge remains open until a distinct judge is configured (`/liter-llm-bridge configure`).

ASSESSMENT COMPLETE
