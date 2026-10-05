# Memory tiers: the file tier and the learning store

Design: `docs/design/team-aware-learning-memory.md` sections 5 and 10.

Agents get memory from two places. They are sized differently on purpose.

| Tier | What it holds | Who sees it | Budget |
|---|---|---|---|
| File tier | Claude auto-memory `MEMORY.md` (and, for Codex, the user-level memory summary) | Every agent, untargeted | MEMORY.md at most 4,096 bytes |
| Learning store | Role, team, project and user lessons, written through `learning_write` | Delivered at SubagentStart to the agent the lesson is addressed to | 8,000 characters (Claude) / 2,000 tokens (Codex) |

Before this change every Claude subagent loaded a ~14 KB index and every Codex agent a ~10.8 KB summary, none of it targeted. The file tier is now a short project index of pointers; role-specific lessons live in the store.

## Partitioning an existing index

`scripts/memory-index-partition.py MEMORY_MD` splits an index into a project index of at most 4,096 bytes plus queued lessons. The default is a **dry run** that prints a JSON plan and changes nothing.

```bash
# Preview: bytes before/after and every line that would move
python3 scripts/memory-index-partition.py ~/.claude/projects/<project>/memory/MEMORY.md --cwd <repo>

# Apply: queue the lessons first, then replace the index (a MEMORY.md.bak-<UTC> copy is kept)
python3 scripts/memory-index-partition.py <path>/MEMORY.md --cwd <repo> --apply
```

The tool reads and writes only the path you give it, plus its backup next to it. Applying to a live index is a deliberate step: preview it first, and check the backup afterwards.

### Deterministic ranking and byte budgeting

1. Non-bullet lines (title, blank lines, prose) stay.
2. A bullet with `role:<id>`, `agent:<id>` or `@role/<id>` is removed and written with visibility `role:<id>`.
3. A bullet with `team:<id>`, `team:*` or `[team]` is removed and written with visibility `team`.
4. Remaining bullets are ranked: project pointers matching the active phase first, feedback and GLOBAL entries second, other project/unclassified pointers newest first by filename date third, and archive pointers last. Ties retain file order. The longest prefix of that priority order that fits is kept; omitted bullets move at project scope.
5. Budgeting includes the actual UTF-8 bytes of structure, kept bullets, final newlines and the footer that will be emitted. The footer counts earlier moves plus new moves, including increases in the count's decimal width. No footer space is reserved when none is needed.

Nothing is removed unless it was queued. `--apply` queues every lesson first and replaces the index (atomically, after the backup) only if all queue writes succeeded; if one fails the index is untouched and the exit code is 1. If structure or structure plus the required footer cannot fit, the tool reports `overflow_reason` and exits 1 before queueing or replacing anything. Dry-run JSON includes `structure_bytes`, `footer_bytes`, `prior_moved`, `cumulative_moved` and each line's rank and decision.

Reapplying an unchanged partitioned index preserves its existing footer and queues no new lessons. When the serialized bytes already match, it returns `unchanged: true` without loading the writer, creating another backup or replacing the file. Later additions are budgeted against the cumulative move count. Queue de-duplication remains based on content hash and scope.

Add a `role:<id>` marker to an index line when you want that lesson to follow one role. Without a marker the tool never guesses a role: unmarked lines are project-wide or stay in the index.

Exit codes: 0 ok, 1 error (nothing modified), 2 BLOCKED (`learning_write.py` could not be loaded).

Queue, log and index locations follow `PROMETHEUS_LEARNING_QUEUE`, `PROMETHEUS_LEARNING_LOG_DIR` and `PROMETHEUS_LEARNING_INDEX_DIR`; point them at scratch directories to rehearse safely.

## Exported agents

`agent-team-creator` exports follow the same split.

**Codex.** Exported agent names are `role_id.replace('-', '_')` and must match `^[a-z0-9_]+$` (`mobile-specialist` becomes `mobile_specialist`; the file is `.codex/agents/mobile_specialist.toml`). Each role file sets

```toml
"memories" = { "generate_memories" = false }
```

This disables generation eligibility for newly created agent threads. It does not stop a root session's global consolidation of retained extraction outputs. In codex-cli 0.158.0, `memories.use_memories` controls memory usage instructions, while `features.memories` gates startup of the memory pipeline. A role's `native.codex.memories` override changes its exported settings; inspect effective project, profile, agent and CLI overrides before relying on defaults. SubagentStart delivers the learning store's role view separately from Codex's user-level memory machinery.

**Claude.** `memory: local` is opt-in. Set it on the team manifest:

```json
{ "agentMemory": { "claude": "local" } }
```

Claude exports then set `memory: local` in each agent's frontmatter and ship a small `.claude/agent-memory-local/<role>/MEMORY.md`. It requires Claude auto-memory. Without the flag nothing changes. `agentMemory` is validated: only `claude`, only the value `local`.

### Codex user configuration policy

Selected Codex installs invoke the memory helper through the installer entry point, including a missing config. The persisted policy is:

```toml
[features]
memories = false

[memories]
generate_memories = false
use_memories = false
```

The 0.158.0 release [startup source](https://github.com/openai/codex/blob/064c6b8c737f5b41d171fdda80bd9ef10ad06eb3/codex-rs/memories/write/src/start.rs) checks the memory feature, ephemeral mode and root-session eligibility before spawning extraction and consolidation; generation/use flags do not guard that entry point. Retained outputs from earlier eligible threads can feed consolidation even when new threads have generation disabled. The [Codex plugin guide](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/codex-plugin.md#codex-memory-policy-and-startup-consolidation) links the exact release configuration and consolidation sources.

`codex-memories-config.sh --check` is read-only and reports the three settings and known v1/v2 summaries independently. Apply mode requires stdlib `tomllib`, validates before replacement, preserves unrelated TOML/comments and creates collision-safe backups. Unsupported target inline tables or malformed configuration leave the original intact and return a failure. It archives only `memories/memory_summary.md` and `memories_v2/memory_summary.md` under `memories-archive/`, with version-distinguishable names; `MEMORY.md`, `raw_memories.md`, extensions and database state are preserved. Uninstall does not apply this policy.

Doctor passes only when all three settings are false and both known summaries are absent. This is a static user-config predicate, not proof that effective profiles/CLI overrides are disabled or running sessions have stopped. Sessions/jobs may retain old configuration. Applying to the real machine and restarting/draining sessions remain a later owner-approved operation; repository implementation and scratch acceptance do not establish live application or the desktop engine's behavior.

## Checking it

```bash
/bin/bash scripts/tests/test-memory-partition.sh        # partitions a copy of a ~14 KB index in a scratch HOME
cd skills/process/agent-team-creator/runtime && npm ci && npm run build && npm run build:tests && node --test ../tests/export.integration.mjs
```

## Main-thread view at SessionStart

The `sessionstart-learning` hook hands the main thread of a team project its team view as fenced, untrusted context. On Claude Code that is the `<team>/@lead` scope plus the team digest. On Codex it is the team digest only (author, paths, contentHash; no lesson text): Codex forks the parent thread's history into every spawned agent, so anything injected into the parent is visible to every child role, and lead-scoped text must not reach a role it was not addressed to. The hook prints nothing for subagent sessions, outside a team, or when nothing is recalled.

## Optional Cortex mirror

Cortex is an optional third copy of each lesson, never a requirement and not part of the lookup chain (surreal-memory, then pk, then the file fallback). `learning_write.py` mirrors a lesson after it has queued it; the mirror follows the integration-contract rule that capability is discovered, never assumed.

**Discovery** (in order):

1. `PROMETHEUS_LEARNING_CORTEX=0` disables the mirror.
2. `PROMETHEUS_CORTEX_MCP` is a Cortex MCP stdio server command, shell-split (used by tests and non-standard installs).
3. Otherwise the newest installed plugin, `~/.claude/plugins/cache/cortex/cortex/<version>/dist/mcp-server.js`, run with `node`.

When no server is found, the primary lesson write continues normally. Explicit invalid server commands and admission/spawn failures appear in the writer's JSON summary; hooks remain quiet. Optional mirror outcomes do not change the durable queue result or turn a successful lesson write into a failure.

**Admission and lifetime.** `PROMETHEUS_CORTEX_MAX_FEEDERS` bounds simultaneous feeder/server pairs across processes sharing a learning queue. Its default is 4; 0 disables mirroring. Invalid values fall back to 4 and report a non-secret diagnostic. Admission is nonblocking: a busy admission lock or full capacity skips the optional mirror without another pending queue or retry loop.

Slots live under `${PROMETHEUS_LEARNING_QUEUE}/cortex-feeders`, defaulting to `~/.prometheus/learning-queue/cortex-feeders`, outside immutable plugin generations. POSIX OS file locks are acquired before spawning and inherited by the feeder and server. Slot files are permanent and must not be deleted while workers are live. A writer or feeder crash cannot free a slot still held by its server; lowering the configured capacity counts existing live slots before admitting more. Platforms without inherited POSIX locks skip the optional mirror with `inherited-locks-unavailable` rather than using unsafe PID or age-based leases.

**What is written.** Each admitted primary lesson gets one detached `cortex_remember` JSON-RPC request (initialize, then `tools/call`); addressed copies and duplicates are not mirrored. The writer transfers input without waiting for model work. The feeder keeps server stdin open until the matching reply or `PROMETHEUS_CORTEX_FEED_TIMEOUT` (default 180 seconds, finite positive values only). It then closes stdin, waits up to five seconds for shutdown, and kills and reaps a server that remains alive before releasing its slot.

The JSON result retains `cortex: true|false` and adds `cortex_mirror` with `status`, `reason`, and, when admission is considered, `limit`. Status is `accepted`, `disabled`, `absent`, `saturated`, `failed`, or `not_attempted`. `accepted` means the detached feeder accepted input; it does not prove that the server later started or saved the memory. Only real Cortex storage and recall establish delivery. A duplicate or failed primary write reports `not_attempted`.

| `cortex_remember` argument | Value |
|---|---|
| `content` | the lesson text (no envelope trailer) |
| `context` | `prometheus-learning team/role:<team>/<role> visibility:<v> kind:<k> h:<hash16>` (the `team/role` part only for a team role) |
| `projectId` | the scope's project id (`@<user>` for user visibility) |
| `global` | `true`, with no `projectId`, for `global` visibility |

Cortex 2.0 stores only `content`, `context` and `projectId`, so the `team/role` tag lives in `context` and is found by `cortex_recall` as a keyword; recall filters by `projectId`. Cortex never delivers lessons to agents: SubagentStart and SessionStart read the learning store only.

For local acceptance, use the production lesson writer and a real Cortex MCP server with scratch `HOME`, `CODEX_HOME`, `CORTEX_DATA_DIR`, learning queue/log/index roots and isolated ports. Check storage/recall and concurrency across actual processes. A stub protocol probe does not establish delivery. These integration checks run after the complete phase implementation.
