# Team-aware learning and memory — design

Status: design of record for phase `team-aware-learning-memory` (change-tlm-002). Implementation follows `docs/plans/team-aware-learning-memory-implementation.md` (change-tlm-003).

## Background

Inputs: phase assessment sections A–N (`.kbd-orchestrator/phases/team-aware-learning-memory/assessment.md`), analysis decisions D-1…D-5 and D-2a, and the runtime probe `evidence/subagent-identity-probe.md`.

The problem in one paragraph. Every agent in a team gets the same untargeted memory:
- Claude Code loads the project auto-memory `MEMORY.md` (about 14 KB) into every subagent; the probe confirmed this.
- Codex loads a user-level `memory_summary.md` (about 10.8 KB).

Meanwhile targeted lessons reach no one:
- agent identity is dropped at every write;
- KBD recall reads lifecycle metadata instead of lessons;
- `pk context` scores only the first 43 entries per scope;
- the learning worker never commits its prompt snapshot.

The goal: each agent receives only the lessons directed to it, within a bounded budget; its own lessons come back to it; relevant ones reach the roles that need them.

## 1. Identity resolution

**Decision.**

Every hook that reads or writes learning resolves `(projectId, teamId, roleId)` once per invocation.

**Sources of identity, by hook point:**

| Hook point | Claude Code 2.1.289 | Codex 0.158 |
|---|---|---|
| SubagentStart, SubagentStop | `agent_id`, `agent_type` (**CONFIRMED** by probe) | `agent_id`, `agent_type` (documented; runtime UNVERIFIABLE) |
| PreToolUse / PostToolUse inside a subagent | `agent_id`, `agent_type` (documented) | optional `agent_id`/`agent_type` in schema (UNVERIFIABLE) |
| PreToolUse on the spawn tool (main thread) | `tool_input.subagent_type`, no `agent_id` (**CONFIRMED**) | `spawn_agent` arguments (agent name) |
| SessionStart | `agent_type` only with `claude --agent <name>` | none (main thread is the operator) |
| Stop | main thread only | main thread only |

**Normalisation:**
1. Strip any `<plugin>:` prefix. The probe confirmed that plugin agents report `tlmprobe:tlm-plug-role`.
2. Map Codex agent names back to role ids by replacing `_` with `-`. Codex rejects hyphens in agent names (probe: "agent_name must use only lowercase letters, digits, and underscores"), so agent-team Codex exports must emit `name = role_id.replace('-', '_')`, and the reverse map is exact because role ids cannot contain `_`.

**Resolution order:**
1. The **project id** comes from the single resolver `shared/scripts/lib/project_id.py`. Its precedence:
   1. `PROMETHEUS_PROJECT_ID`
   2. `.prometheus/project.json` `projectId`
   3. the runtime-registered project UUID (`prometheus kbd status --json`)
   4. `project:<sha256(git common dir)>`, so all worktrees of a repository share one id

   This replaces the four current derivations (assessment E). The literal default `MEM_PROJECT=prometheus-skill-pack` goes away.
2. **Team:** `.agent-team/project-routing.json` `activeTeam`, then the sole `.agent-team/<id>/team.json`, then `unresolved`.
3. **Role:**
   - the normalised `agent_type` if it is a role id in the team manifest;
   - else the role whose `owns` globs longest-match the paths the agent touched, with ties broken by role order in the manifest and the ambiguity recorded;
   - else `unresolved`.
4. Built-in names collide with team roles. Ghostex has `planner`, and `~/.claude/agents/planner.md` exists. So the pack's SubagentStop matchers become anchored and namespaced. The iterative-evolver hooks match only `^(prometheus-skill-pack:)?iterative-evolver-(assessor|analyst|planner|executor|reflector)$` once those agents are renamed, and the Karpathy learning hook keys on any SubagentStop with a resolved role instead of on the bare name `executor`.

**Rejected alternatives**
- Environment variables for identity: neither harness exports agent identity to hooks (assessment A, E3).
- Prompt-text parsing of `Role: <id>`: fragile, and not available to hooks.
- Keeping bare SubagentStop matchers: they collide with real team roles today.

**Evidence:** probe behaviours 1–2 (CONFIRMED), 3–4 (UNVERIFIABLE, fallbacks adopted), the assessment E2 identity table, the Codex naming finding.

## 2. Learning envelope

**Decision.**

Every persisted lesson carries the envelope defined by `shared/schemas/learning-envelope.schema.json`. Required fields: `schemaVersion`, `projectId`, `visibility`, `kind`, `author`, `contentHash`, `ts`. Plus `teamId`/`roleId` whenever the author resolves.

**Scope keys.** Surreal-memory filters `agent_id` by equality only (analyze W1), so every visibility level gets its own `agent_id` value, and every recall query is an equality-filtered query:

| Visibility | `user_id` | `agent_id` |
|---|---|---|
| `agent` (private to the author role) | projectId | `<team>/<role>` |
| `role:<r>` (addressed) | projectId | `<team>/<r>` (one stored copy per addressee) |
| `lead` | projectId | `<team>/@lead` |
| `team` (digest) | projectId | `<team>/@team` |
| `project` | projectId | `@project` |
| `user` | `@user:<hash>` | `@user` |
| `global` | `@global` | `@global` |

An agent with no team resolves to `team = @solo`, so even un-teamed agents get private lessons. Legacy records with a null `agent_id` (50 of 50 sampled) are migrated by a one-off job that re-keys them to `@project`. The job runs through the surreal-memory v2 operations API under the upstream prerequisite in section 8. Until the migration runs, they are visible only to the main thread's project-scope fallback query.

**Field mapping onto the stores:**

| Envelope field | surreal-memory | pk wiki entry | Karpathy session record / learning log |
|---|---|---|---|
| `schemaVersion` | category `env:1` | frontmatter `extra.envelope.schemaVersion` | `envelope.schemaVersion` |
| `projectId` | `user_id` (or the `@user`/`@global` key above) | KB location (project KB) + `extra.envelope.projectId` | `projectId` |
| `teamId` | prefix of `agent_id` | tag `team:<t>` | `teamId` |
| `roleId` | suffix of `agent_id` | tag `role:<r>` | `roleId` |
| `visibility` | encoded by `agent_id` (table above) + category `vis:<v>` | tag `vis:<v>`; `project` → project KB, `user`/`global` → shared/global KB | `visibility` |
| `audience` | one copy per addressee, category `aud:<a>` on each | tags `aud:<a>` | `audience` |
| `kind` | category `kind:<k>` | `type` (`Lesson`, `Gotcha`, `Decision`, `Progress`, `Candidate`; needs settable type, section 8) | `kind` |
| `stage` | category `stage:<s>` | tag `stage:<s>` | `stage` |
| `paths` | category `path:<glob>` (first 5) | `extra.envelope.paths` | `paths` |
| `author` | `session_id` = `author.sessionId`; category `author:<team>/<role>`; `author.agentId` in content trailer | `extra.envelope.author` | `author` |
| `importance` | category `imp:<0-9>` (bucketed until metadata is settable) | `extra.envelope.importance` | `importance` |
| `contentHash` | category `h:<first 16>`, the de-duplication key | `id` suffix | `contentHash` |
| `ts` | `created_at` | `updated_at` | `ts` |

**Rejected alternatives**
- Storing the envelope only in surreal-memory `metadata`: there is no write path or filter for it today (assessment C).
- One `agent_id` per author, with client-side audience filtering after top-k: addressed lessons fall outside top-k and are silently dropped (analyze W1).
- Separate `user_id` per role: it breaks the per-project scoping that every other reader relies on.

**Evidence:** assessment C/D, analyze W1, store report E4, schema with valid and invalid examples.

## 3. Visibility and promotion

**Decision.**
- The default is `agent`: a lesson is private to the role that wrote it.
- Addressing (`audience: role:<r>` or `lead`) delivers a copy to that recipient. Path-overlap routing (section 7) adds addressees automatically.
- Promotion beyond the team follows the policy approved for this work:
  - Detectors propose promotion candidates (`~/.prometheus/promotion-candidates/pending`, surfaced by `kbd-open.sh` and the reflector role). A human confirms.
  - `[GLOBAL]` and `[USER]` markers in a reflection line promote that line immediately.
- Within a team, the `lead` role, or the KBD reflector, may promote a role lesson to `team`.
- Shared scopes (`@team`, `@project`, `@user`, `@global`) are read-only to individual roles. They are written only by the promotion path, the reflector, or the lead, never by an arbitrary role's hook.

**Rejected alternatives**
- Broadcast-by-default (CrewAI's crew-wide memory): reproduces today's flood.
- Automatic promotion without confirmation: contradicts the approved policy.
- Peer-to-peer writes into another role's private scope: makes attribution meaningless.

**Evidence:** analysis D-3, cand-002 (CrewAI read-only slices), cand-005 (Letta), the approved promotion policy.

## 4. Per-agent recall

**Decision.**

Recall for an agent resolved to `(P, T, R)` runs equality-filtered queries, merges them in this priority order, and stops at the budget:
1. `agent_id = T/R` (own lessons, plus lessons addressed to R)
2. `agent_id = T/@lead` (only if R is the lead role)
3. `agent_id = T/@team`, the last 50 digest entries
4. `agent_id = @project`
5. `user_id = @user:<hash>` and `user_id = @global`, top 3 each

**Scoring** inside each query: semantic 0.5 + recency 0.3 (30-day half-life) + importance 0.2, from cand-002. Results are de-duplicated on `contentHash` across queries.

**Budgets:**
- Claude SubagentStart: at most **8,000** characters.
- Codex SubagentStart: at most **2,000 tokens** (about 7,000 characters), under the 2,500-token default limit.
- Main-thread prompt context: about 2 KB.

**Total per-subagent budget across all channels (analyze W3).** The per-agent target is **≤ 12 KB total**: file-memory tier plus every SubagentStart hook. Today's baseline is about 14 KB of untargeted auto-memory and 0 B targeted. Section 5 reduces the file tier to make room, and section 10 measures each channel.

**Fallback chain:**
- surreal-memory: REST `POST /api/v1/search` with `user_id` + `agent_id`, 2 s timeout. Embeddings are stripped client-side until the upstream fix lands.
- then pk: `pk context --scope project --scope shared --scope global`, filtered client-side by `team:`/`role:` tags until the tag filter lands. This depends on the truncation fix (section 8).
- then files: the per-role sections of the project memory index (section 5).

**Measurement:** the hook writes `{agentType, bytesByChannel, entriesByScope}` to `~/.prometheus/learning-index/delivery.jsonl`, and section 10 reports from it.

**Rejected alternatives**
- One unfiltered query cut by score: leaks role-private lessons across roles.
- Task-text-keyed recall at SubagentStart: neither harness passes the task text. B12's parent-transcript scrape is fragile, and it is deferred to a later PreToolUse(Agent) `updatedInput` enhancement.

**Evidence:** analysis D/D-4, cand-002/004, store filters (E4), harness caps (E3, cand-006/007).

## 5. Delivery

**Decision.**
- **Primary channel: SubagentStart** in both harnesses, plugin hook `subagentstart-learning` with matcher `*`. The hook resolves the role itself (section 1), because plugin and Codex names differ.
  - Claude: injection is **CONFIRMED** by the subagent-identity-probe (behaviour 1). Output is JSON `hookSpecificOutput.additionalContext`.
  - Codex: injection is documented but UNVERIFIABLE in the probe (project layer untrusted). The hook emits the same JSON, which Codex accepts. **Fallback**, recorded by the probe for behaviour 3: role resolution from the `spawn_agent` PreToolUse arguments, correlated by `agent_id`.
- **Main thread:**
  - SessionStart (`kbd-open.sh`) and UserPromptSubmit stop injecting "everything".
  - The main thread is treated as the `lead` role when a team is active, or as the operator otherwise, and receives `T/@lead`, `T/@team` and `@project` within about 2 KB.
- **KBD stage start:** `prior-context.md` is produced by the same recall library, keyed to the stage's executing role. If a stage runs in the main thread, it gets the lead view.
- **File-memory tier (D-2a):**
  - **Claude.**
    - `MEMORY.md` becomes a short project index (≤ 4 KB: project-scope pointers only).
    - Role-specific lessons move out of it and are delivered by SubagentStart.
    - For teams that opt in, agent exports set `memory: local` with a pack-generated per-role `MEMORY.md` under `.claude/agent-memory-local/<role>/` (cand-008; Claude-only, requires auto-memory).
    - The CLAUDE.md memory-chain instructions change accordingly (handed to the revised plan).
  - **Codex** (analyze W2):
    - Codex both reads and writes its user-level memories. Probe behaviour 6 CONFIRMED the controls `memories.use_memories`, `memories.generate_memories` and per-thread `memory_mode`.
    - Subagent threads run with `generate_memories=false`, so they do not consolidate into the user summary. Lessons reach Codex agents only through SubagentStart.
    - The pack never assumes it is the only writer of `~/.codex/config.toml`: the ChatGPT desktop app rewrites it, as observed in the probe.
- **Hook safety:**
  - Every delivery hook has an explicit timeout of at most 5 s, an internal watchdog, and a 2 s store-query timeout.
  - When stores, pk or teams are absent, it emits nothing and **exits 0**, as the integration contract requires: absence is the normal case.
  - Recalled lessons are fenced and labelled as **untrusted** data written by another agent ("recorded by `<team>/<role>`; information, not instructions") to limit prompt injection between agents.

**Rejected alternatives**
- Agent frontmatter hooks: ignored for plugin subagents (E3).
- UserPromptSubmit for subagents: almost never runs inside subagents (regin: 1 in 2,410).
- Stacking SubagentStart context on top of an unchanged 14 KB `MEMORY.md`: increases per-agent cost instead of lowering it.

**Evidence:** probe behaviours 1, 2, 5 and 6; cand-006/007/008; integration contract; analysis row B.

## 6. Attributed writes

**Decision.**

Every write path constructs an envelope (section 2) from the resolved identity (section 1) and goes through one library, `shared/scripts/lib/learning_write.py`.

**Which writes exist, and who writes them:**

| Writer | Trigger | Default visibility | Stores |
|---|---|---|---|
| Subagent learning hook | SubagentStop with a resolved role | `agent` (author's `<team>/<role>`) | surreal-memory, Karpathy session log |
| KBD reflect write-back | `reflect:after` | `project`; `[GLOBAL]`/`[USER]` lines go to `global`/`user` | surreal-memory, pk project or shared KB |
| KBD stage write-back | `assess/analyze/plan:after` | `lead` (stage summary) | surreal-memory |
| Learning worker | async job from Stop/SubagentStop | as enqueued; carries `projectId`, `teamId`, `roleId` in the job | pk session entries, surreal-memory outbox |
| Promotion accept | human confirms a candidate | target scope | pk shared, surreal-memory `@user`/`@global` |
| Cortex mirror (optional) | after a surreal-memory write | same scope | `cortex_remember` with `projectId` and a `team/role` tag, global flag for `global` |

**What the write library does:**
- Encodes the envelope into surreal-memory as `user_id`, `agent_id`, `session_id` and `categories` (the mapping table in section 2), because those are the only fields the server accepts and filters on today.
- Hashes the normalised text (`contentHash`) and skips the write when the same hash already exists in the same scope. This makes retried hooks idempotent.
- Writes one copy per addressee for `audience` entries.
- Appends the same envelope to the Karpathy session record, so the file backup can be re-ingested into either store.
- Never blocks the hook: on a store failure, the write goes to the existing outbox (`~/.prometheus/memory-outbox/`) and the hook exits 0.

**Learning worker changes:**
- `LearningJob` gains `project_id`, `team_id` and `role_id` (serde default).
- `enqueue_memory` uses them instead of re-deriving the project.
- After `store.upsert`, the worker calls `commit_prompt_snapshot`, so session entries become visible to `pk context`.

**Rejected alternatives**
- Letting each hook format its own surreal-memory payload: this is how four project-id derivations arose; one library is the fix.
- Asking the model in the subagent to tag its own lessons: unreliable, costs tokens, and the hook already knows the identity.
- Writing the subagent's full final message as a lesson: it floods recall. The worker extracts lessons; the hook writes only the lesson lines.

**Evidence:** assessment B (write paths), C (store fields), the learning-worker job shape, Cortex availability (analyze W4).

## 7. Cross-agent awareness

**Decision.**

Lessons reach other roles in three bounded ways; none of them broadcasts.

1. **Path-overlap routing.**
   - At write time, the library compares the lesson's `paths` with every role's `owns` globs in the team manifest.
   - Each role whose globs match, other than the author, gets an addressed copy (`role:<r>`).
   - At most 3 addressees per lesson. With more matches, the copy goes to `lead` instead.
2. **Team digest.**
   - Every `agent`-visibility lesson also appends a one-line summary (≤ 160 characters, author, paths, contentHash) to `~/.prometheus/team-digest/<projectId>/<team>.jsonl`.
   - It is mirrored to surreal-memory under `<team>/@team`.
   - Recall takes the last 50 digest lines for the team, so every role sees what changed but not the full lesson text. The full text stays with the author and the addressees.
3. **Lead view.**
   - The lead role (or the main thread) gets `T/@lead`: stage summaries, lessons with more than 3 path matches, and promotion candidates for the team.
   - The lead decides whether to promote to `team`.

**Bounding:**
- Digest lines count against the recipient's budget (section 4).
- Duplicates (same `contentHash`) are delivered once.
- Digest files rotate at 1,000 lines.

**Rejected alternatives**
- A shared blackboard every agent reads in full (Letta shared blocks, CrewAI crew memory): this is the flood we are removing.
- An LLM router that decides recipients per lesson: adds latency and cost on every write, and is non-deterministic. Path ownership is already declared in `team.json`.
- Peer-to-peer messages between running subagents: neither harness supports them for subagents.

**Evidence:** analysis D-3/D-4, the agent-team schema (`owns` globs), cand-002/005.

## 8. Upstream prerequisites

These must land before the design works end to end. Each is a separate PR in the revised plan.

| # | Repository | Prerequisite | Why |
|---|---|---|---|
| U1 | prometheus-knowledge-rs | `pk context` scores every entry in a scope, not the first 43 by id | Without it, targeted pk recall silently misses most lessons |
| U2 | prometheus-knowledge-rs | Learning worker commits the prompt snapshot after upsert | Session entries are invisible to `pk context` today |
| U3 | prometheus-knowledge-rs | `pk context --tag` filter, and settable `type` on ingest | Role and visibility filtering in pk without client-side post-filtering |
| U4 | surreal-memory-server | Search responses omit embeddings unless requested | Responses are many times larger than the text; the hook strips them until then |
| U5 | surreal-memory-server | Settable and filterable `metadata` (or categories filter) | Lets envelope fields move out of the `categories` encoding |
| U6 | surreal-memory-server | v2 operations API for re-keying legacy `agent_id = null` records | Migration in section 2 |
| U7 | skill-pack | Single `project_id.py` resolver; namespaced SubagentStop matchers | Section 1 |
| U8 | Codex (upstream, not ours) | Project-layer hooks load under a trusted project | Runtime proof of Codex SubagentStart injection; until then the fallbacks apply |

U1–U2 and U7 block delivery. U3–U6 are optimisations with stated workarounds. U8 is outside our control and is tracked as a residual risk.

**Rejected alternatives**
- Adopting an external memory library (mem0, Graphiti, LangMem) in place of the existing stores: rejected in analysis D-5. We copy their scoping and scoring patterns, not their runtimes.
- Working around U1 by querying pk with many narrower queries: it multiplies latency in a hook with a 5 s budget.

**Evidence:** assessment D (pk truncation), E4 (surreal-memory filters), analysis D-5, the subagent-identity-probe residual blocks.

## 9. Mini scope

`prometheus-skills-mini` carries a reduced pack without the Rust stores. Its scope:

- **Carries:**
  - the envelope schema;
  - the identity resolver (Node port);
  - the file-tier recall: per-role sections of the memory index and the team digest file;
  - the SubagentStart hook in its file-only form.
- **Does not carry:**
  - surreal-memory or pk writes;
  - the learning worker;
  - promotion detectors.
  Mini must behave exactly as today when they are absent, with no warning.
- **Budgets:** the same as section 4. Mini's per-agent payload is file-tier only, so it fits well under them.
- **Gate:** mini's `carried-payload` test asserts that the carried files match the skill-pack's copies byte for byte, and that the hook exits 0 with an empty HOME.

**Rejected alternatives**
- Porting the full store stack to mini: contradicts mini's purpose as a dependency-free pack.
- Leaving mini untouched: its agents would still get the untargeted flood, and the two packs' hook behaviour would diverge.

**Evidence:** mini repository layout (`versions.toml`, `skills/carried-payload.test.mjs`), integration contract (absence is normal).

## 10. Measurement

**Decision.**

Success is measured as context bytes delivered per agent, by channel, against today's baseline, plus targeting precision.

**Baseline** (assessment, measured 2026-10-03):

| Channel | Claude subagent | Codex subagent |
|---|---|---|
| File-memory tier | ~14 KB `MEMORY.md` (untargeted) | ~10.8 KB `memory_summary.md` (untargeted) |
| Targeted lessons | 0 B | 0 B |

**Targets:**
- The total per-subagent learning context is ≤ 12 KB across all channels.
- File tier ≤ 4 KB.
- SubagentStart ≤ 8,000 characters (Claude) or ≤ 2,000 tokens (Codex).
- Targeting precision: in the gate fixture, every delivered lesson is addressed to the receiving role, its team, or a promoted scope. No role-private lesson of another role is delivered (0 leaks).
- Recall: a lesson written by role R in cycle N appears in R's SubagentStart context in cycle N+1.

**How it is measured:**
- The delivery log `~/.prometheus/learning-index/delivery.jsonl` (section 4) records bytes by channel per agent.
- `scripts/report-learning-delivery.py` summarises it per role and per harness.
- The integration gate (plan, alpha and beta gates) runs a two-role fixture team in both harnesses:
  1. role A writes a private lesson and a path-routed lesson;
  2. the next cycle spawns A and B;
  3. assert A receives its private lesson, B receives only the routed one, and both stay under budget.

**Rejected alternatives**
- Measuring tokens with a tokenizer at hook time: adds a dependency and latency. Characters are the harness's own unit for Claude, and a 3.5 characters-per-token estimate is used for Codex.
- Judging success by lesson count delivered: rewards flooding.

**Evidence:** assessment baseline table, probe behaviour 5 (MEMORY.md reaches subagents), cand-006/007 (harness caps).
