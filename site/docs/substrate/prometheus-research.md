---
id: prometheus-research
title: prometheus-research
---

# prometheus-research

Background deep-research daemon on `127.0.0.1:7891` (crate version 0.1.0, the value `/health` reports; it is versioned independently of the pack release): five MCP tools
(`research_start/status/cancel/export`, `render_component`), an AG-UI SSE
event stream, and an A2UI registry of eight server-rendered HTMX fragments
(HTMX 2.0.8 + Alpine.js vendored).

Auto-starts via `com.prometheus.research.plist` launchd service; installed by
`scripts/install-binaries.sh`.

## Headless job execution

`research_start` (and the equivalent `POST /api/v1/jobs`) does not run the
pipeline in-process. It spawns a **headless harness child** — `claude`, falling
back to `codex` — and that child drives the deep-research stage contract to
completion. The daemon supervises: it records the package directory, mirrors
per-stage events onto the SSE stream, and maps the child's exit code onto the
job status.

Resolution order, both first-match-wins:

| What | Order |
|---|---|
| Harness | `claude`, then `codex`, on the daemon's `PATH` |
| Driver | `RESEARCH_DRIVER`, then `CLAUDE_PLUGIN_ROOT/skills/research/deep-research/scripts/run-research.sh`, then `~/.claude/skills/deep-research/scripts/run-research.sh`, then the in-tree path |

Relevant environment: `RESEARCH_DRIVER` (driver path override),
`RESEARCH_HARNESS_ARGS` (extra harness flags), `RESEARCH_EVENT_SINK` and
`RESEARCH_EVENT_TOKEN` (the token-gated ingest endpoint the child posts stage
events to), `RESEARCH_OUTPUT_DIR` (package root, default
`~/.prometheus/research`).

### The daemon never reports a success it did not achieve

A child that exits 0 without leaving a package is recorded `failed` with
`harness exit code 0 without producing a package under <root>`, not `complete`.
A missing harness is `blocked` with an actionable message. Exit codes map to
`unavailable` (3), `refused` (2), and `failed` (anything else). Job status,
`package_dir`, `harness`, `harness_pid`, `exit_code`, and `error` are all
readable from `GET /api/v1/jobs/{id}`.

`research_export` refuses a job that is not `complete`, validates the package
before answering, and returns `package_dir`, `output_path`, `report_path`,
`verification_status`, and `verification_verdict`.

### Execution self-check

The daemon reports what it would actually run — at startup and at `/health` —
so a broken install is visible before a job is spent discovering it:

```json
{
  "execution": {
    "harness": { "status": "ok", "name": "claude", "path": "…", "size_bytes": 123456 },
    "driver":  { "status": "ok", "path": "…", "size_bytes": 30718,
                 "missing_stage_contract_markers": [] }
  }
}
```

`driver.status` is `stale` when the resolved script does not implement the stage
contract — the check looks for `--resume`, `checkpoint`, `next_stage`, and
`RESEARCH_STAGE_RUNNER`, and names whichever are missing. Counting markers beats
comparing file size: size drifts with every edit, but a driver either speaks the
contract or it does not. A stale driver would run a job to exit 0 having produced
no package.

`harness.status` is `missing` when nothing on `PATH` resolves. Under launchd that
almost always means the plist granted no `PATH`.

The daemon **warns and keeps serving** rather than refusing to start: refusing
would make a broken install harder to inspect, and `/health` is how an operator
inspects it.

### Deployment requirements

The daemon needs a harness binary **on its own `PATH`** and a driver that
implements the stage contract. Under launchd the process inherits only what its
plist grants, so a plist without an `EnvironmentVariables`/`PATH` entry leaves
the daemon unable to find `claude` or `codex` even when both are installed.
Likewise, if the installed plugin generation predates the stage-contract driver,
`~/.claude/skills/deep-research/scripts/run-research.sh` resolves to an older
script; set `RESEARCH_DRIVER`, or reinstall so the current generation is
published.

*Canonical source: [`substrate/prometheus-research`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/substrate/prometheus-research) — modules: `a2ui`, `agui`, `config`, `job`.*
