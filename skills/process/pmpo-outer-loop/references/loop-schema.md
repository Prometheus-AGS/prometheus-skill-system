# Loop Definition Schema

Human-readable reference for `loop.json` — the file written by `/loop-define` to
`.kbd-orchestrator/loops/<name>/loop.json`.

Machine schema: [`schemas/loop-definition.schema.json`](schemas/loop-definition.schema.json)

---

## Six Required Parameter Groups

Every loop.json must address all six canonical loop parameters:

| Parameter Group | Field(s) | Required |
|----------------|----------|----------|
| **Goal** | `goal.description`, `goal.measurable_criteria` | yes |
| **Feedback** | `feedback_sources[]` | recommended |
| **Termination** | `termination.max_ticks` | yes |
| **Escalation** | `escalation_points[]` | recommended |
| **Cadence** | `cadence.mode` | recommended |
| **Evolution** | `evolution_name` | yes |

---

## Field Reference

### `name` (string, required)
Kebab-case identifier for the loop. Becomes the directory name under
`.kbd-orchestrator/loops/`.

### `goal` (object, required)
```json
{
  "description": "Ship the authentication module to production",
  "measurable_criteria": [
    "all tests pass (exit 0)",
    "open bug count == 0 in gh issues with label 'auth'",
    "deployment to prod verified by smoke test"
  ]
}
```
`measurable_criteria` are machine-checkable strings. Each should name the
check command or data source so the tick can verify them automatically.

### `feedback_sources` (array)
Each source is polled once per tick:

| `type` | Required field | What it does |
|--------|---------------|-------------|
| `command` | `run` | Shell command; exit code or stdout parsed by `interpret` |
| `gh-query` | `run` | `gh` CLI query (e.g. `gh issue list --label auth --state open --json number`) |
| `file` | `path` | Read a file (e.g. test report, coverage JSON) |
| `url` | `fetch` | HTTP GET the URL; response body parsed by `interpret` |

`interpret` (optional string) describes how to read the result:
- `"exit-code"` — 0 = progress/pass, non-zero = fail
- `"count-delta"` — compare count to prior tick; decrease = progress
- `"jsonpath:<expr>"` — extract a value from JSON response
- Free text describing the human/model judgment call

### `termination` (object, required)
```json
{
  "goal_satisfied": true,
  "max_ticks": 20,
  "max_no_progress_ticks": 3,
  "budget": { "max_minutes_per_tick": 30 }
}
```

- `goal_satisfied` — terminate when all measurable_criteria hold (default `true`)
- `max_ticks` — hard ceiling; loop stops even if goal not met
- `max_no_progress_ticks` — escalate after N consecutive ticks with no improvement (default 2)
- `budget.max_minutes_per_tick` — wall-clock limit per tick

### `escalation_points` (array of strings)
Conditions that pause the loop and consult the human via `/pmpo-elicit`:

```json
[
  "zeespec NO-GO on a capability",
  "feedback regression (metric worse than prior tick)",
  "capability gap FAILED after 2 retries",
  "max_no_progress_ticks reached",
  "termination: human decides continue/stop"
]
```

### `cadence` (object)
```json
{ "mode": "manual", "schedule": null }
```
- `manual` — human runs `/loop-tick <name>` each time
- `background` — Claude Code background task re-invokes after each tick
- `cron` — scheduled agent fires on `schedule` (ISO 8601 duration or cron expr)

### `evolution_name` (string, required)
The backing iterative-evolver key. `/loop-tick` runs one `/evolve "<name>"` cycle.
Must match an entry in `.evolver/evolutions/`.

### Runtime fields (written by `/loop-tick`, not `/loop-define`)
| Field | Type | Purpose |
|-------|------|---------|
| `current_tick` | integer | How many ticks have run |
| `no_progress_ticks` | integer | Consecutive ticks without improvement |
| `status` | enum | `active` `paused` `completed` `escalated` |
| `last_tick_at` | ISO8601 | Timestamp of last tick |
| `created_at` | ISO8601 | When `/loop-define` wrote this file |

---

## Minimal Example

```json
{
  "name": "ship-auth",
  "goal": {
    "description": "Ship authentication module to production",
    "measurable_criteria": [
      "npm test exits 0",
      "gh issue list --label auth --state open --json number | jq length == 0"
    ]
  },
  "feedback_sources": [
    { "type": "command", "run": "npm test", "interpret": "exit-code" },
    { "type": "gh-query", "run": "gh issue list --label auth --state open --json number", "interpret": "count-delta" }
  ],
  "termination": {
    "goal_satisfied": true,
    "max_ticks": 15,
    "max_no_progress_ticks": 2
  },
  "escalation_points": [
    "feedback regression",
    "max_no_progress_ticks reached"
  ],
  "cadence": { "mode": "manual", "schedule": null },
  "evolution_name": "ship-auth",
  "current_tick": 0,
  "no_progress_ticks": 0,
  "status": "active",
  "last_tick_at": null,
  "created_at": "2026-06-23T00:00:00Z"
}
```
