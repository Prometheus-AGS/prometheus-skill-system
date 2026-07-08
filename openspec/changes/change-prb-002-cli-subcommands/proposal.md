---
id: change-prb-002-cli-subcommands
title: Implement start, status, cancel subcommands with job checkpoint system
phase: phase-prometheus-research-binary
priority: P0
effort: L
wave: 1
agent: general-purpose
status: pending
gap_id: G-02,G-03,G-04
verdict: BUILD
depends_on: change-prb-001-scaffold-crate
scope:
  - substrate/prometheus-research/src/job/mod.rs
  - substrate/prometheus-research/src/job/checkpoint.rs
  - substrate/prometheus-research/src/job/spawn.rs
  - substrate/prometheus-research/src/job/cancel.rs
---

# Change: CLI subcommands + job checkpoint system

## Problem

No background job execution or persistence. Research jobs die with the session.

## Solution

Add three CLI subcommands and a checkpoint system:

### `start` subcommand
- Accepts: `--query <str>`, `--depth shallow|deep|exhaustive`, `--max-sources <n>`, `--citation-style apa|mla|chicago|ieee`
- Generates `job-<timestamp>-<rand>` job ID
- Creates `~/.research-jobs/<job-id>/checkpoint.json` with initial state
- Spawns background process (self re-exec with `--daemon-job <job-id>`) and writes PID
- Prints job ID to stdout

### `status` subcommand
- Accepts: `<job-id>`
- Reads `~/.research-jobs/<job-id>/checkpoint.json`
- Prints: stage, stage_name, progress %, elapsed time, status, tokens_used

### `cancel` subcommand
- Accepts: `<job-id>`
- Reads PID from checkpoint, sends `SIGTERM` via `nix::sys::signal`
- Updates checkpoint `status` to `"cancelled"`

### Checkpoint format
```json
{
  "job_id": "job-20260708-abc123",
  "query": "...",
  "depth": "deep",
  "max_sources": 20,
  "citation_style": "apa",
  "status": "running|completed|cancelled|failed",
  "stage": 3,
  "stage_name": "retrieve",
  "progress": 35,
  "pid": 12345,
  "started_at": "...",
  "last_updated_at": "...",
  "tokens_used": 0,
  "sources_found": 0,
  "output_dir": "~/.research-jobs/<job-id>/"
}
```

## Acceptance Criteria

- [ ] `prometheus-research start "test query"` prints a job ID and returns immediately
- [ ] `prometheus-research status <job-id>` prints checkpoint fields without error
- [ ] `prometheus-research cancel <job-id>` sends SIGTERM and updates checkpoint to `cancelled`
- [ ] Checkpoint file exists at `~/.research-jobs/<job-id>/checkpoint.json` after `start`
- [ ] `cargo test --lib` passes for `job::` module unit tests
