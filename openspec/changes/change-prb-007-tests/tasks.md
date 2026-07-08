# Tasks — change-prb-007-tests

- [x] Create `tests/job_lifecycle.rs` with 3 tests: creates_checkpoint_on_start, status_reads_checkpoint, cancel_updates_checkpoint_to_cancelled
- [x] Create `tests/mcp_tools.rs` with 3 tests: research_start_returns_job_id, research_status_returns_stage_fields, research_cancel_returns_cancelled_true
- [x] Create `tests/sse_stream.rs` with 2 tests: sse_endpoint_returns_event_stream_content_type, broadcast_event_appears_in_sse_stream
- [x] Ensure test servers bind on port 0 (no hardcoded ports)
- [x] Run `cargo test` — all 8 tests pass
- [x] Run `cargo test -- --nocapture` — no panics visible in output
