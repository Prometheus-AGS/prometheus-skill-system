# KBD Cross-Harness Handoff

This protocol applies to Claude Code, Codex, OpenCode, Kimi, and any adapter
implementing the KBD control contract.

## Authority

The append-only journal in `.kbd-orchestrator/runtime/events.jsonl` is the
durable authority. `current-waypoint.json`, `progress.json`, and
`position.json` are compatibility projections and MUST NOT be used for
mutual exclusion or causal ordering. Git is for review, recovery, and signed
snapshots—not live coordination.

Operator pause and cancellation always outrank continuation policy. The local
`.kbd-orchestrator/PAUSE` emergency valve disables steering before any state is
parsed.

## Handoff protocol

1. The current writer runs `prometheus kbd pause --reason <text>` if an audit
   checkpoint is required.
2. It records a course correction, when needed, with `prometheus kbd revise
   --reason <text> --exact-next-work <text>`.
3. It runs `prometheus kbd handoff --to <harness>`.
4. The runtime atomically replaces the old lease, increments the fencing
   token, and records the target harness. There is no unleased write window.
5. The receiving harness runs `prometheus kbd status --json`, verifies the
   checkpoint and plan revision, then `prometheus kbd resume
   --plan-revision <n>`.
6. All other harnesses remain observers. A stale lease ID, fencing token, or
   expected revision is rejected.

The destination device and session are resolved only after the replicated
handoff reaches a trusted peer. They are never copied from the sending
session.

## Pause checkpoint

Every graceful pause contains:

- reason;
- previous lifecycle state;
- last completed and exact next work;
- decisions and blockers;
- dirty-work summary;
- current plan revision.

`pause_requested`, `paused`, and `blocked` are suspended states. Stop hooks
and prompt hooks must remain silent while any of them is active.

## Completion invariant

Implementation and post-implementation evidence remain independent.
`completion.implementation` is the primary N/N counter. Evidence,
certification, and publication can be pending without making completed code
unfinished.

## Recovery

- `prometheus kbd audit --since <revision-or-event>` inspects immutable history.
- `prometheus kbd migrate --check` is read-only.
- `prometheus kbd migrate --apply` creates backups before normalization.
- A missing daemon degrades to the same local CLI contract; it never changes
  lifecycle semantics.
- Direct edits to compatibility JSON are legacy-only and are rejected after
  migration enforcement is enabled.
