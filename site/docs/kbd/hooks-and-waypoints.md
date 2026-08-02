---
id: hooks-and-waypoints
title: Hooks & Waypoints
---

# Hooks & Waypoints

**Hooks** — every stage fires `<stage>:before` / `<stage>:after` via
`shared/lib/hooks.sh`; per-task boundaries fire `task:before`/`task:after`
from `/kbd-apply`. Projects wire custom hooks per phase without touching the
skills.

**Waypoints** — `current-waypoint.json` records phase, status, and the next
command, so a session (or a different tool entirely) resumes with one read.
`position-reminder.txt` mirrors it as plain text for prompt injection.

Waypoints are compatibility projections. A projection is accepted as current
only when its `sourceRevision` matches the canonical runtime revision. File
mtime is not causal authority, and direct writes do not append an event.

**Stage gates** — each stage requires its predecessor's handoff
(`kbd_stage_gate`), and writes its own (`kbd_stage_handoff_write`) — a
1–3-sentence summary plus artifact list the next stage reads first.

## Cross-harness adapter events

One capability manifest generates the Claude Code, Codex, OpenCode, and Kimi
adapters. The native event names differ, but they map to the same KBD events:

| Purpose | Claude Code | Codex | OpenCode | Kimi |
|---|---|---|---|---|
| Session start | `SessionStart` | `session_start` | `session.created` | `SessionStart` |
| Post-compact re-anchor | `SessionStart:compact` | `post_compact` | `session.compacted` | `PostCompact` |
| Prompt | `UserPromptSubmit` | `user_prompt_submit` | `chat.message` | `UserPromptSubmit` |
| Interrupt | — | `turn_cancelled` | `session.status:cancelled` | `Interrupt` |

The adapter no longer intercepts tool calls on any harness. The pre-mutation
fence that once guarded `Bash`, `Write`, `Edit`, and `MultiEdit` was removed;
these events are observational and always exit successfully. See
[Tool guards](./bash-mutation-guard).

## Stop and interrupt semantics

Stop hooks are advisory and fail open. They may record a reminder, but they
never reinterpret missing prose or an incomplete footer as authority to keep
an agent running.

An explicit interrupt is different: it creates `.kbd-orchestrator/PAUSE`
immediately, queues a deferred event, and asks the control plane to create a
durable pause checkpoint. Local operator intent is honored before network,
memory, or parsing work.

## Re-anchor after compaction

Session-start and post-compact events render a bounded status block containing
the committed revision, plan revision, lifecycle, active path, and exact next
work. The renderer has a 4,800-character ceiling so it
cannot flood the new context window.

*Canonical source: [`shared/lib`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/skills/process/kbd-process-orchestrator/shared/lib) (`hooks.sh`,
`stage-gate.sh`, `waypoint.sh`, `progress.sh`).*
