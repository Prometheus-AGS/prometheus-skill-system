---
id: change-prb-007-tests
title: Write integration tests for job lifecycle, MCP tools, and SSE stream
phase: phase-prometheus-research-binary
priority: P1
effort: M
wave: 4
agent: general-purpose
status: pending
gap_id: G-01,G-02,G-03,G-04,G-05,G-06
verdict: BUILD
depends_on: change-prb-005-a2ui-components
scope:
  - substrate/prometheus-research/tests/job_lifecycle.rs
  - substrate/prometheus-research/tests/mcp_tools.rs
  - substrate/prometheus-research/tests/sse_stream.rs
---

# Change: Integration tests

## Problem

No test coverage. Changes 1-6 are implemented but untested end-to-end.

## Solution

Three integration test files covering the critical paths:

### `tests/job_lifecycle.rs`
- `creates_checkpoint_on_start` — start a job, verify checkpoint exists at expected path
- `status_reads_checkpoint` — write a test checkpoint, verify `status` reads it correctly
- `cancel_updates_checkpoint_to_cancelled` — start a job, cancel it, verify status = "cancelled"

### `tests/mcp_tools.rs`
- `research_start_returns_job_id` — call tool handler directly, verify JSON response has `job_id`
- `research_status_returns_stage_fields` — write checkpoint, call status handler, verify fields
- `research_cancel_returns_cancelled_true` — start job, cancel via tool, verify response

### `tests/sse_stream.rs`
- `sse_endpoint_returns_event_stream_content_type` — spawn test server, GET /events, check header
- `broadcast_event_appears_in_sse_stream` — emit AguiEvent, verify SSE client receives it

Use `tokio::test` for all async tests. Bind test servers on port 0 (OS assigns free port).

## Acceptance Criteria

- [ ] `cargo test` runs all 8 integration tests without errors
- [ ] All 8 tests pass
- [ ] No test uses sleep-based timing — all use channel-based or deterministic waits
- [ ] Test output is clean (`cargo test -- --nocapture` shows no panics)
