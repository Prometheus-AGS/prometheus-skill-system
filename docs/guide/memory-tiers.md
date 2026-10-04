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

### The rules (deterministic, in file order)

1. Non-bullet lines (title, blank lines, prose) stay.
2. A bullet with `role:<id>`, `agent:<id>` or `@role/<id>` is removed and written with visibility `role:<id>`.
3. A bullet with `team:<id>`, `team:*` or `[team]` is removed and written with visibility `team`.
4. Every other bullet stays while the byte budget lasts. The first bullet that would exceed it, and every bullet after it, is removed and written at project scope.
5. When anything moved, a one-line footer is appended and counted against the budget.

Nothing is removed unless it was queued. `--apply` queues every lesson first and replaces the index (atomically, after the backup) only if all queue writes succeeded; if one fails the index is untouched and the exit code is 1. Re-running on the same text queues nothing new: `learning_write` de-duplicates by content hash and scope.

Add a `role:<id>` marker to an index line when you want that lesson to follow one role. Without a marker the tool never guesses a role: unmarked lines are project-wide or stay in the index.

Exit codes: 0 ok, 1 error (nothing modified), 2 BLOCKED (`learning_write.py` could not be loaded).

Queue, log and index locations follow `PROMETHEUS_LEARNING_QUEUE`, `PROMETHEUS_LEARNING_LOG_DIR` and `PROMETHEUS_LEARNING_INDEX_DIR`; point them at scratch directories to rehearse safely.

## Exported agents

`agent-team-creator` exports follow the same split.

**Codex.** Exported agent names are `role_id.replace('-', '_')` and must match `^[a-z0-9_]+$` (`mobile-specialist` becomes `mobile_specialist`; the file is `.codex/agents/mobile_specialist.toml`). Each role file sets

```toml
"memories" = { "generate_memories" = false }
```

so subagent threads do not consolidate into the user-level memory summary. The key is `memories.generate_memories` in codex-cli 0.158 (confirmed in the binary's `MemoriesToml`; `use_memories` is the read side). Lessons reach Codex agents only through SubagentStart. A role's `native.codex.memories` override wins if you need different behaviour. The pack does not edit `~/.codex/config.toml`: other apps rewrite it.

**Claude.** `memory: local` is opt-in. Set it on the team manifest:

```json
{ "agentMemory": { "claude": "local" } }
```

Claude exports then set `memory: local` in each agent's frontmatter and ship a small `.claude/agent-memory-local/<role>/MEMORY.md`. It requires Claude auto-memory. Without the flag nothing changes. `agentMemory` is validated: only `claude`, only the value `local`.

## Checking it

```bash
/bin/bash scripts/tests/test-memory-partition.sh        # partitions a copy of a ~14 KB index in a scratch HOME
cd skills/process/agent-team-creator/runtime && npm ci && npm run build && npm run build:tests && node --test ../tests/export.integration.mjs
```

## Main-thread view at SessionStart

The `sessionstart-learning` hook hands the main thread of a team project its team view as fenced, untrusted context. On Claude Code that is the `<team>/@lead` scope plus the team digest. On Codex it is the team digest only (author, paths, contentHash; no lesson text): Codex forks the parent thread's history into every spawned agent, so anything injected into the parent is visible to every child role, and lead-scoped text must not reach a role it was not addressed to. The hook prints nothing for subagent sessions, outside a team, or when nothing is recalled.
