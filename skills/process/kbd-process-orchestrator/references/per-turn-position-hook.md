# Per-turn position reporting

The KBD requirement is: **every turn**, the user sees where they are in the
process — outer phase chain *and* any active child loop — as
`starting/completing <phase|task> <name>, <i> of <n>`.

There are two layers. Do not confuse them.

## Layer 1 — the guarantee (plain-text Progress Signals)

Plain-text lines emitted into the assistant's normal response are the **only**
channel Claude Code reliably surfaces to the user. Every KBD skill MUST emit
them, and `/kbd-apply` emits them on every task boundary:

```
Starting task 3 of 10: wire MCP servers into opencode config
Completed task 3 of 10: wire MCP servers into opencode config
```

For the full chain (outer + child), render via `waypoint_chain`:

```
Starting task 3 of 10: parent-phase › child-auth › migrate sessions
```

This layer does not depend on any settings hook or on the model remembering to
source a shell library — `/kbd-apply`'s `begin-task`/`end-task` print it
directly, and the orchestrator skills print it in their Progress Signals
sections.

## Layer 2 — the extension point (shell hooks, stderr)

The `report-progress` hook (`*:*`, `hooks/hooks.json`) writes
`starting/ending <kind> <name> [i/n]` to **stderr**. This is for logs, the
memory mirror (`kbd-memory-log`), and user-defined reporters/overrides — **not**
a user-facing guarantee. Claude Code does not surface hook stderr into the
conversation. Treat it as telemetry/extensibility only.

> **Correction (2026-06-03):** earlier docs implied the stderr reporter was what
> the user sees each turn. It is not. Layer 1 is the guarantee; Layer 2 is the
> extension point. Overriding `report-progress` changes telemetry, not what the
> user reads.

## Optional Layer 2b — inject position via a Claude Code settings hook

If you want the position injected into context automatically each turn
(belt-and-suspenders, independent of which skill is active), add a `Stop` hook
to `~/.claude/settings.json` (verified key shape: top-level `hooks` object).
This is **opt-in** — KBD does not install it for you.

```jsonc
// ~/.claude/settings.json  (merge into existing keys; do not overwrite)
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "jq -r 'if .phase then \"KBD position: \" + ((.parentPhase // \"\") + \" › \" + .phase + (if .childPointer then \" › \" + .childPointer else \"\" end)) else empty end' .kbd-orchestrator/current-waypoint.json 2>/dev/null || true"
          }
        ]
      }
    ]
  }
}
```

Notes:
- Use `/update-config` (or the `update-config` skill) to add this safely —
  settings hooks are executed by the harness, not by the model.
- The command must be cwd-tolerant and never exit non-zero (`|| true`) so it
  cannot block the Stop chain.
- This reports the *outer/child phase chain*; per-task counts come from Layer 1
  (`/kbd-apply`), since task index lives in the loop, not the waypoint.

## Summary

| Need | Use |
|---|---|
| User sees position every turn (guaranteed) | Layer 1 — plain-text signals (always on) |
| Memory mirror / custom telemetry | Layer 2 — `report-progress` + augment hooks |
| Replace the default reporter | Layer 2 — `mode:"override"` hooks-config entry |
| Auto-inject chain regardless of active skill | Layer 2b — opt-in `Stop` settings hook |
