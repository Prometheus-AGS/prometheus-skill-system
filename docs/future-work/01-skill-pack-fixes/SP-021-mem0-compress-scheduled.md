---
id: SP-021
title: mem0 compress_memories scheduled job
status: ready
priority: P2
estimated_effort: 1d
agent_role: hooks-engineer
depends_on: []
unblocks: []
related: [SP-009, SP-020]
created_from_conversation_turn: 3-4
---

# SP-021 — mem0 compress_memories scheduled job

## Problem

The skill-pack ships `mem0-compress-on-stop.sh` to compress (summarize/deduplicate) accumulated memories via the mem0 library. Like SP-009's `pk-lint-cron.sh`, the script exists but is unwired — no Stop hook entry runs it on schedule.

## Evidence

```
$ ls shared/scripts/mem0-compress-on-stop.sh   # exists
$ grep -r 'mem0-compress' .claude-plugin/ hooks/   # no operational reference
```

## Why it matters

Without compression, episodic memory grows monotonically. Embedding indexes degrade. Retrieval quality drops as relevant memories get crowded out by stale ones. The compression step is the Karpathy-loop's "Compress" operation in operational form.

## Proposed fix

Register `mem0-compress-on-stop.sh` in the Stop chain to run *at most once per N days per project* (default: 14 days). The script:

1. Reads `.prometheus/last-compress.txt` for the last-run timestamp.
2. Exits early if the last run was within N days.
3. Otherwise: invokes mem0 `compress_memories` on the project-scoped episodic store (per SP-020 separation).
4. Writes the new timestamp.
5. Logs to `~/.prometheus/hooks.log` (per SP-006).

Compression is idempotent and conservative: it summarizes clusters of similar episodic memories into single records, preserving the originals as `superseded_by` relations.

## Trade-offs and risks

- **Risk: compression loses signal.** Mitigation: `superseded_by` relations preserve the originals. A user can always traverse back. Compression is reversible at the cost of resurfacing detail.
- **Risk: long-running cron.** mem0 compression on a busy store can take minutes. Stop-chain timing is acceptable (post-session); not a real concern.
- **Risk: compresses across project boundaries.** Mitigation: per-project scoping per SP-008 and SP-020.

## Acceptance criteria

- [ ] `mem0-compress-on-stop.sh` is registered in the Stop chain.
- [ ] It runs at most once per 14 days per project.
- [ ] Compression is project-scoped (no cross-project compaction).
- [ ] Originals remain accessible via `superseded_by` relations.
- [ ] Logs to hooks.log.
- [ ] Test: synthetic 14-day-old timestamp triggers run; 1-day-old skips.

## Implementation steps

1. Add the timestamp gate to the script.
2. Register in `.claude-plugin/hooks/hooks.json` (and via SP-015 symlink).
3. Verify mem0's `compress_memories` API and configure for per-project scoping.
4. Test.

## Dependencies

None functional. Recommended after SP-006 (hook log) and SP-020 (per-store separation).

## Open questions

- Cadence default of 14 days — is that right? Tunable via `MEM0_COMPRESS_INTERVAL_DAYS`. Default 14 is conservative.
- Should compression also touch the KG store? Generally no — KG entries are already deduplicated by the librarian. Compression is for episodic only.
