---
name: karpathy-progress-memory
description: >
  Record a successful KBD task, change, or phase boundary as a canonical,
  idempotent progress event. Use when a boundary transition succeeds, when
  recovering work after context loss, or when KBD progress must survive a
  memory-service outage without editing generated projections. Do NOT use for
  changing KBD position or status; use kbd-process-orchestrator instead.
version: '1.0.0'
license: MIT
compatibility: python >=3.10, git, prometheus CLI; pk optional
allowed-tools: file_system
metadata:
  author: Prometheus AGS
  version: '1.0.0'
  category: process
  tags: [process, kbd, progress, memory, recovery]
---

# Karpathy Progress Memory

This skill records one versioned event at a successful KBD boundary. The
recorder checks the event against `prometheus kbd status --json` before it
writes anything. A disagreement is an error to report; the recorder never
repairs or edits `progress.json`, `tasks.md`, a waypoint, or another generated
projection.

The record is written once to `.prometheus/session-log.md` and sent to the
project-scoped knowledge store through `pk ingest`. If `pk` or its backing
service is unavailable, the same bounded record enters the existing durable
memory outbox. This degraded state is reported and does not block later work.

## Automatic boundary use

KBD invokes the recorder after successful task, change, and phase transitions.
The hook form derives its stable event identity from the canonical project and
run, boundary, qualified subject, and terminal status. A successor run can
reuse phase/change/task IDs without colliding with earlier receipts. Unrelated
later KBD revisions therefore replay as the same event. `observedAt` is receipt
metadata and is not part of semantic collision detection:

```sh
python3 scripts/record-progress.py \
  --project-root "$PWD" \
  --from-hook \
  --boundary task
```

`KBD_HOOK_NAME` must identify a task as `change-id:task-id` or
`change-id/task-id`. Change hooks carry the change ID. A failed or partial
transition cannot pass canonical validation and therefore cannot produce a
completion record.

Optional hook metadata:

- `KBD_TASK_CLASS`: `product`, `research`, `evidence`, `integration`, or
  `release`; defaults to `product`.
- `KBD_TASK_ELAPSED_HOURS`: decimal elapsed hours; defaults to `0`.
- `PROMETHEUS_BIN`: explicit Prometheus CLI path for an installed or test
  runtime.
- `PK_BIN`: explicit `pk` path. A missing or failing command activates the
  durable outbox.
- `KPM_PK_TIMEOUT_SECONDS`: bounded `pk` submission timeout from `0.1` to `10`
  seconds; defaults to `5`.

## Explicit event use

For a manually assembled evidence boundary, validate an event matching
`references/progress-event.schema.json`:

```sh
python3 scripts/record-progress.py \
  --project-root /path/to/project \
  --input /path/to/progress-event.json
```

The event contains a stable ID; boundary status; canonical phase, change, and
task identity; task class and elapsed hours; touched files; verification
commands with exit codes and summaries; commit hash; blocker; and exact next
work. Payloads are bounded to 256 KiB and rejected when they contain common
secret forms or unsafe paths.

## Result contract

The command prints one JSON object:

- `recorded`: the session log and `pk` accepted the record.
- `queued`: the session log was written and memory delivery is in the durable
  outbox.
- `degraded`: the session log was written but neither memory path accepted the
  record. Continue work and report this result.
- `duplicate`: the stable event already has a complete receipt; no second
  append or memory write occurs.

Each receipt stores the original bounded event. If a process exits after the
session append and pending receipt but before memory submission, replay resumes
delivery from that snapshot under the same event lock. Hook replay compares
only canonical boundary identity; later HEAD, touched-file, exact-next, and
wall-clock changes do not rewrite the original record.

Exit `2` means the event, canonical identity, or required local write failed.
Do not change generated KBD files to make such an event pass.
