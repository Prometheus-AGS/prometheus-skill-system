# change-tlm-002-team-aware-learning-design

**Title:** Design agent-team-aware memory and Karpathy learning: envelope, identity, visibility, per-agent recall, delivery, attribution, cross-agent awareness
**Repository:** `prometheus-skill-pack`
**Phase:** team-aware-learning-memory
**Depends on:** change-tlm-001-subagent-identity-probe (its verdicts select primary or fallback delivery)
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` at or after `20d97f2` (contains PRs #111, #121–#123), at `/Users/gqadonis/Projects/prometheus/worktrees/tlm-design`. Never this checkout's `codex/delivery-cadence-recovery` branch, which lacks #121. Delivered through a pull request.

## Why

Every agent in a team gets the same untargeted memory today: about 14 KB of Claude auto-memory per subagent, with no role awareness. Targeted lessons never reach anyone: agent identity is dropped at every write, recall reads lifecycle metadata, and `pk context` scores only 43 entries per scope. Phase goals 2–4 require a design that the implementation PRs (change-tlm-003) are built on (assessment A–N; analysis D-1…D-5, D-2a).

## What Changes

Write `docs/design/team-aware-learning-memory.md` with these sections, each stating the decision, the rejected alternatives and the evidence:

1. **Identity resolution.**
   - Sources by hook point and harness: SubagentStart, SubagentStop, PreToolUse and PostToolUse stdin `agent_id`/`agent_type`; Claude SessionStart `agent_type`; Codex main thread is the operator.
   - Normalisation: strip `<plugin>:`.
   - Resolution of `(projectId, teamId, roleId)` through the single project-id resolver, `.agent-team/project-routing.json`, then `team.json`. Fallbacks: the sole team, then an `owns` longest-match glob with tie-break, then `unresolved`.
   - Built-in agent names (iterative-evolver, kbd-*) and namespacing of the SubagentStop matchers.
2. **Learning envelope.**
   - Fields: `projectId`, `teamId`, `roleId`, `author{harness, agentId, sessionId}`, `visibility` (`agent` | `role:<r>` | `team` | `project` | `user` | `global`), `audience[]`, `kind` (lesson | gotcha | decision | progress | candidate), `stage`, `paths[]`, `importance`, `contentHash`, `ts`.
   - Exact mapping onto surreal-memory (`user_id` = projectId; `agent_id` = `<team>/<role>`; `session_id`; categories `vis:*`, `aud:*`, `kind:*`, `stage:*`), onto pk entries (tags plus frontmatter `extra`), onto Karpathy session records, and onto the learning-log line.
   - **Scope keys (analyze W1):** surreal-memory filters `agent_id` by equality only, so each visibility level is its own `agent_id`: `<team>/<role>` (private and addressed), `<team>/@team` (digest), `@project`, `@user:<hash>` and `@global` (under their own `user_id`). No recall query ever runs unfiltered. Legacy null-`agent_id` records: migrate (re-key to `@project`) or archive status, with the decision and the migration mechanism stated.
   - Published as `shared/schemas/learning-envelope.schema.json`.
3. **Visibility and promotion.**
   - Default `agent`. Addressing a lesson (`aud:role:<r>`, `aud:lead`) delivers it to that audience.
   - Promotion to team, project, user or global follows the approved policy: auto-propose, human confirms; `[GLOBAL]`/`[USER]` markers act immediately.
   - The lead or reflector role can promote within the team. Shared scopes are read-only to individual roles.
4. **Per-agent recall.**
   - Query order: own role, then addressed-to-role, then team digest, then project, then user/global.
   - Score: semantic 0.5, recency 0.3 (30-day half-life), importance 0.2. Deduplicate by `contentHash`.
   - Budgets: Claude SubagentStart at most 8,000 chars; Codex at most 2,000 tokens; main-thread prompt about 2 KB.
   - **Total per-subagent budget across all channels (analyze W3):** file tier plus every SubagentStart hook, with a target and the measured bytes per channel.
   - The surreal-memory → pk → file fallback, with client-side category filtering and embedding stripping until upstream support lands.
   - Byte measurement method.
5. **Delivery.**
   - SubagentStart in both harnesses, with the matcher strategy. Main-thread SessionStart and prompt context become lead/operator-scoped.
   - The KBD stage-start `prior-context.md` becomes role-aware.
   - The file-memory tier decision (D-2a): per-role sections versus Claude per-subagent `memory:` directories, and how `MEMORY.md` injection into subagents is reduced.
   - **Codex native memories (analyze W2):** Codex both reads (`memory_summary.md` injected per session) and writes (consolidating every thread, subagent threads included) a user-level store with no envelope. State the per-thread control (`memory_mode`, or `use_memories`/`generate_memories`, per the change-tlm-001 probe) used to keep subagent threads from consuming or polluting it.
   - The fallbacks chosen by change-tlm-001's verdicts.
   - **Hook safety (analysis row B):** every delivery hook carries an explicit timeout of at most 5 s, with an internal watchdog and a 2 s store-query timeout. It emits nothing and exits 0 when stores or teams are absent (integration contract). Recalled lessons are fenced and labelled as untrusted data written by another agent.
6. **Attributed writes.**
   - `enqueue-learning-job.py` and `LearningJob` gain `agentId`, `agentType`, `teamId` and `roleId`, filled from SubagentStop (subagent work) and Stop (main thread).
   - The record-progress schema gains the same fields. Reflection write-back attributes lines to the writing role.
   - The worker writes the envelope to all three stores.
7. **Cross-agent awareness.**
   - Path-overlap routing: a lesson touching paths owned by another role is addressed to that role.
   - An append-only team digest with a size cap, delivered to the lead role and the main thread.
   - Handoff `memoryRefs` filled from recall.
8. **Upstream prerequisites.**
   - pk: score all candidates, then truncate; worker snapshot commit; tag and source filters; settable tags on ingest.
   - surreal-memory: category filter in the vector, BM25 and list queries; no embeddings in responses; settable metadata, scope and importance.
   - Each with the interim pack-side behaviour.
   - **Cortex (analyze W4):** CLAUDE.md tells agents to write Cortex memories with no team or role. State how Cortex writes carry the envelope tags (or are routed through the pack writer). The CLAUDE.md change is scheduled in change-tlm-003.
   - **Resolver and worktrees (S5/S6):** name the single project-id resolver and its precedence (`PROMETHEUS_PROJECT_ID` > `.prometheus/project.json` projectId > runtime-registered project UUID > `project:<sha256(git common dir)>`, so worktrees share one id). The team digest lives in `~/.prometheus/team-digest/<projectId>/<team>.jsonl`, not in a git-ignored per-worktree file.
9. **Mini scope.** Recall and delivery ship in mini; worker-dependent capture stays in the skill-pack.
10. **Measurement.** Bytes per agent before and after, for the baseline table in the assessment.

Machine-checkable companion: `shared/schemas/learning-envelope.schema.json` (JSON Schema 2020-12) plus one valid and one invalid example under `shared/schemas/examples/`. The invalid example must fail on its `visibility` value (for example `"visibility": "everyone"`), so the schema's central rule is what is exercised.

## Scope

- `docs/design/team-aware-learning-memory.md`
- `shared/schemas/learning-envelope.schema.json`
- `shared/schemas/examples/learning-envelope.valid.json`
- `shared/schemas/examples/learning-envelope.invalid.json`

## Capabilities

- `learning-envelope` (new)
- `per-agent-recall` (new)

## ADDED Requirements

### Requirement: every learning write carries the envelope
Every lesson persisted by the pack SHALL carry `projectId`, `visibility` and `author.harness`, and SHALL carry `teamId`/`roleId` when the author resolves to a team role.

### Requirement: a subagent receives only lessons visible to its role
For a subagent resolved to `(team T, role R)`, delivered context SHALL contain only lessons whose visibility is `agent` and authored by R, `role:R`, `team`, `project`, `user` or `global`, within the per-harness budget.
