---
id: mcp-tools
title: MCP Tools
sidebar_label: MCP Tools
---

# MCP Tools

In `--mode mcp`, Sovereign Sync exposes ten tools: four local discovery/sync
tools and six KBD control tools.

## Configuration

```json
{
  "sovereign-sync": {
    "command": "/path/to/sovereign-sync",
    "args": ["--mode", "mcp"],
    "env": {
      "RUST_LOG": "sovereign_sync=warn",
      "KBD_FOCUS_PROJECT_PATH": "/path/to/project"
    }
  }
}
```

The MCP process discovers its KBD project from `KBD_FOCUS_PROJECT_PATH`, then
the current working directory, then the parent of the skills directory.

The CLI accepts `--prefix-tools`, but the current tool router still exposes the
unprefixed names documented below. Do not configure clients with
`sovereign:*` names yet.

## Discovery and sync tools

### `search-skills`

Input:

```json
{"query":"feynman grading","limit":5}
```

Performs local keyword search over installed skill names and descriptions.

### `sync-status`

Input:

```json
{"domain":"learner-model"}
```

`domain` is optional. The current MCP response is a bounded local status
summary and points clients to REST. Neither MCP nor REST reads live
`P2PNode` state in `0.1.0`.

### `sync-push`

Input:

```json
{"domain":"learner-model"}
```

Acknowledges a queued push request. It does not call the CRDT exporter or P2P
broadcaster and is not delivery confirmation.

### `sync-peers`

Input: none.

Returns a fixed no-peers summary. The tool is not connected to the daemon’s
live gossip neighbors. See [Pair two machines](./pair-two-machines).

## KBD read and operator tools

| Tool | Input |
|---|---|
| `kbd_status` | none |
| `kbd_events` | `{"since_revision": 1}` |
| `kbd_pause` | `{"reason":"…"}` |
| `kbd_cancel` | `{"reason":"…"}` |
| `kbd_revise` | `{"reason":"…","exact_next_work":"…"}` |
| `kbd_resume` | `{"plan_revision":4}` |

### `kbd_status`

Returns canonical `KbdStateV2`, including lifecycle, committed revision, plan
revision, checkpoint, exact next work, active path, completion dimensions,
devices, and blockers.

### `kbd_events`

Returns immutable committed events starting at `since_revision` (default 1).

### `kbd_pause`

Creates a pause checkpoint.

```json
{"reason":"Pause before rotating the control token"}
```

### `kbd_cancel`

Transitions the run to terminal `cancelled` while preserving history.

```json
{"reason":"Operator abandoned this run"}
```

### `kbd_revise`

Records immutable plan revision `N+1` and optionally replaces exact next work.

```json
{
  "reason":"Upstream interface changed",
  "exact_next_work":"Adopt the supported endpoint"
}
```

### `kbd_resume`

Resumes a suspended lifecycle at the supplied or current plan revision.

```json
{"plan_revision":4}
```

## Error behavior

MCP tools return a textual `KBD control error: …` result for revision,
single-writer policy, signature, or integrity failures. They do not fall back
to directly editing `.kbd-orchestrator` compatibility files.
