# Headless execution: how the daemon runs a job

`prometheus-research start <query>` (REST `POST /api/v1/jobs`, MCP
`research_start`) writes a job checkpoint and re-executes the binary as
`prometheus-research --daemon-job <job_id>`. That process is the **daemon
job**. It does not run stages itself: it spawns a headless coding harness and
asks it to run the stage-contract driver (`scripts/run-research.sh`,
change-rah-003) for the job's query. This document is the contract between the
daemon, the harness, and the driver (change-rah-004; analysis D-01 option C).

## Harness resolution

The daemon scans `PATH` in order and takes the first match:

| Order | Binary | Invocation |
|---|---|---|
| 1 | `claude` | `claude -p "<prompt>" --permission-mode bypassPermissions` |
| 2 | `codex` | `codex exec --dangerously-bypass-hook-trust --full-auto "<prompt>"` |

`--dangerously-bypass-hook-trust` is the Codex flag that lets a headless
`codex exec` run the plugin's non-managed hooks without the interactive trust
prompt (`docs/codex-plugin.md`). `RESEARCH_HARNESS_ARGS` appends extra flags
to either invocation. There is no per-job `harness` field on `research_start`
(spec open question, default no): the daemon's `PATH` decides.

When neither binary is on `PATH`, the daemon records the job as
`blocked` with the reason (`no harness binary on PATH (looked for claude,
then codex)`), emits one `agent.error`, and exits 0. The daemon process
serving `/health` is unaffected: absence is a job outcome, not a crash.

## The prompt

The prompt is written to `<output_root>/<job_id>/prompt.md` before the
harness starts, so what the session was asked is always on disk. It carries
the query, depth, citation style, max sources, output root, and job id, and
the exact driver command:

```
bash <run-research.sh> --query '<query>' --depth <depth> --citation-style <style> \
     --job-id <job_id> --output-root '<output_root>'
```

The driver path is resolved from `RESEARCH_DRIVER`, then
`$CLAUDE_PLUGIN_ROOT/skills/research/deep-research/scripts/run-research.sh`,
then `~/.claude/skills/deep-research/scripts/run-research.sh`, then the source
tree the binary was built from. The prompt then states the checkpoint-mode
loop from `stage-contracts.md`: run the driver; when it exits 3 with a
`next_stage` line, run that stage skill against the package directory and
re-invoke the driver with `--resume`; stop at exit 0 (complete) or 1
(blocked). The harness is told not to run stages outside the driver and not
to edit `checkpoint.json`.

### Value normalisation

The entry points and the driver do not share a vocabulary: REST and MCP
default `depth` to `moderate` and every entry point defaults
`citation_style` to lowercase `apa`, while the driver accepts
`shallow | deep | exhaustive` and the manifest schema enumerates
`APA | MLA | Chicago | IEEE | Vancouver`. The daemon canonicalises both
before writing the prompt (`moderate` → `deep`, `apa` → `APA`, and so on).
A value it does not recognise **blocks the job** with a reason naming the
field, before any prompt is written: the driver line is executed by a
session with permissions bypassed, so a caller-supplied string is never
placed on it unvalidated, and every interpolated value on that line is
shell-quoted regardless. The job checkpoint keeps the value the caller sent;
the package checkpoint carries the canonical one. The integration test found
the vocabulary gap (its first run blocked at export with
`citation_style: 'apa' is not one of ['APA', ...]`) and asserts that a depth
of `deep; touch /tmp/...` is refused without the harness ever starting.

## Environment the harness inherits

| Variable | Value | Purpose |
|---|---|---|
| `RESEARCH_OUTPUT_DIR` | the daemon's output root | one root for the job directory and the package |
| `RESEARCH_JOB_ID` | the job id | the driver records it in the package checkpoint; the daemon finds the package by it |
| `RESEARCH_QUERY`, `RESEARCH_DEPTH` | from the job | convenience for the stage skills |
| `KBD_HOOKS_DISABLED` | `1` | the session fires no KBD lifecycle hook (below) |
| everything else | inherited from the daemon | `RESEARCH_HOOK_LOG`, `RESEARCH_STAGE_RUNNER`, `RESEARCH_JUDGE_CMD`, gateway keys, and `PATH` flow through unchanged |

The harness's stdout and stderr go to `<job_dir>/harness.log`; the daemon's
own log is `<job_dir>/daemon.log`.

## What the daemon does while the harness runs

Every 500 ms the daemon:

1. Looks for the package directory the driver created: the directory under
   the output root whose `checkpoint.json` carries `stages_planned`, the
   job's `job_id`, and a `created_at` no earlier than the job's
   `started_at`. The driver names packages after the query
   (`<slug>-<yyyymmdd>-<4hex>`), so the job id is the link, and the time
   bound means a stale package from an earlier run is never adopted even if
   it carries the same id. Once found, `package_dir` and `package_id` are
   recorded in the job checkpoint.
2. Reads the package `checkpoint.json` and mirrors it: `stage` and
   `stage_name` from `current_stage` (or the last completed stage),
   `progress` as completed stages over planned stages. For every stage newly
   present in `stages_completed` since the previous poll it emits one
   `agent.status`, so a driver that finishes several stages between two polls
   still produces one event per stage. A `blocked` block in the package
   checkpoint is mirrored as `agent.error`.
3. Checks the job checkpoint for `cancelled` (written by `research_cancel` /
   `DELETE /api/v1/jobs/{id}`) and, on `SIGTERM`, kills the harness and
   records `cancelled`.

**Who writes the job checkpoint.** `start` writes it once, `pending`, and
writes nothing after the daemon is spawned (a later parent write would race
the child and could overwrite a `blocked` the child reached within
milliseconds). The daemon records its own pid and every later status. Every
daemon write re-reads the file first and never overwrites a `cancelled`
written by the operator; the surviving window is one read-write pair, not a
poll interval.

Events are POSTed to `RESEARCH_EVENT_SINK` (`http://127.0.0.1:<port>`, set by
the HTTP server for the jobs it starts) at `/api/v1/jobs/{id}/events`, which
broadcasts them to the SSE stream and the surface bridge. The request carries
`x-research-event-token: $RESEARCH_EVENT_TOKEN`, a secret the server
generates at startup and only the daemons it spawns inherit; a request
without it is refused with 401, and one whose event names a different job
than the path with 400, so another local process that knows the port cannot
forge a job's stream. A job started from the CLI or the MCP server without a
running HTTP server has no sink; its checkpoint still carries every mirrored
field.

## Exit mapping

The harness exit code is recorded (`exit_code`) but never decides the
outcome; the package checkpoint does, because a harness can exit 0 after the
driver blocked and non-zero after it completed.

| Package `status` at harness exit | Job `status` | `error` |
|---|---|---|
| `complete` | `complete` | cleared; `stage` 10, `progress` 100 |
| `blocked` | `blocked` | the package's `blocked.stage` and `reason`, plus the harness exit |
| `awaiting_stage` | `blocked` | `harness exited (...) while the run was awaiting stage NN; resume with the driver's --resume` |
| anything else | `failed` | harness exit and the package status |
| no package found | `failed` | `harness exited ... without producing a package under <root>` |

`research_status` returns these fields; `research_export` accepts only
`complete`.

## Hook policy (resolves the analyze open question)

- **The daemon fires no hooks.** It is a supervisor.
- **The harness runs the real driver**, which fires the four deep-research
  hook scripts (`pre-research`, `post-stage`, `on-contradiction`,
  `post-export`) exactly as a foreground run does, with `RESEARCH_HOOK_LOG`
  markers when that variable is set.
- **The harness session fires no KBD lifecycle hook.** The daemon sets
  `KBD_HOOKS_DISABLED=1` in the harness environment and the prompt states it.
  This is a contract with the session, not an enforcement inside the KBD hook
  library: a research job is not a KBD phase, so nothing in the driver or the
  stage skills calls `kbd_hooks_fire`, and the variable exists so a harness
  whose plugin hooks would otherwise fire on session events can see that this
  session is a research job. The integration test asserts the variable
  reached the harness and that its fixture harness wrote no KBD marker.

## Export

`research_export` (MCP) reads the job checkpoint, requires `complete`,
validates `<package_dir>/manifest.json` against
`references/schemas/research-manifest.schema.json` with the `jsonschema`
crate (the schema is embedded in the binary at build time), and returns the
package directory, manifest path, `verification_status`,
`verification_verdict`, and the report path. An invalid manifest is refused
with the failing field named; nothing is exported.

## Test seams

All optional; production leaves them unset.

| Variable | Read by | Effect |
|---|---|---|
| `RESEARCH_DAEMON_EXE` | `spawn_job` | the binary to run as `--daemon-job` (an integration test's `current_exe` is the test harness) |
| `RESEARCH_DRIVER` | daemon | the driver path named in the prompt |
| `RESEARCH_EVENT_SINK` | daemon | where to POST events (set by `run_server`) |
| `RESEARCH_EVENT_TOKEN` | daemon, ingest handler | the ingest secret (generated by `run_server`; an operator may pin one) |
| `RESEARCH_HARNESS_ARGS` | daemon | extra harness flags |

`tests/fixtures/fake-claude.sh` is a harness that records the prompt and
environment it received and then runs the driver command the prompt names,
with `RESEARCH_STAGE_RUNNER` pointing at the deep-research fixture runner.
`tests/job_execution.rs` puts it first on `PATH` as `claude`, starts a job
through the production path, and asserts on the checkpoint, the SSE stream,
the hook markers, and `research_export`.
