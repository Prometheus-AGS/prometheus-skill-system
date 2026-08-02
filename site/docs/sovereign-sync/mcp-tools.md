---
id: mcp-tools
title: MCP Tools
sidebar_label: MCP Tools
---

# MCP Tools

In `--mode mcp`, Sovereign Sync exposes eleven tools: four local discovery/sync
tools and seven KBD registry/control tools.

## Configuration

```json
{
  "sovereign-sync": {
    "command": "/path/to/sovereign-sync",
    "args": ["--mode", "mcp"],
    "env": {
      "RUST_LOG": "sovereign_sync=warn"
    }
  }
}
```

The MCP process loads the platform KBD registry. It registers the current
working directory only when that checkout already contains
`.prometheus/project.json`; it never creates or infers an identity.

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
| `kbd_projects` | none |
| `kbd_status` | `{"project_id":"<uuid>"}` |
| `kbd_events` | `{"project_id":"<uuid>","since_revision":1}` |
| `kbd_pause` | `{"project_id":"<uuid>","reason":"…"}` |
| `kbd_cancel` | `{"project_id":"<uuid>","reason":"…"}` |
| `kbd_revise` | `{"project_id":"<uuid>","reason":"…","exact_next_work":"…"}` |
| `kbd_resume` | `{"project_id":"<uuid>","plan_revision":4}` |

### `kbd_projects`

Returns the registered projects, replicas, machine identity, and any per-project
open error. When one project is registered, the other KBD tools may omit
`project_id`. With multiple projects, omission returns the available UUIDs
instead of selecting one implicitly.

### `kbd_status`

Returns canonical `KbdStateV2`, including lifecycle, derived revision, causal
frontier, plan
revision, checkpoint, exact next work, active path, completion dimensions,
devices, blockers, and visible conflicts.

### `kbd_events`

Returns immutable committed events starting at `since_revision` (default 1).

### `kbd_pause`

Creates a pause checkpoint.

```json
{"reason":"Pause before rotating the device signing key"}
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

MCP tools return a textual `KBD control error: …` result for frontier,
single-writer policy, signature, or integrity failures. They do not fall back
to directly editing `.kbd-orchestrator` compatibility files.
