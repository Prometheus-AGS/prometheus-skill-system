# change-prui-005-smoke-test

## Summary

Add `substrate/prometheus-research/scripts/smoke-test.sh` — a portable shell script that
exercises the full HTTP API of `prometheus-research`: start server, poll `/health`, create
a job, read its status, open the SSE stream and read one event, cancel the job, and verify
clean shutdown.

## Goal

G-05: Integration smoke test (start → /health → job → SSE → cancel)

## Files Changed

- `substrate/prometheus-research/scripts/smoke-test.sh` — new smoke test script

## Acceptance Criteria

- [ ] Script is executable (`chmod +x`)
- [ ] Starts `prometheus-research --mode server` in background, captures PID
- [ ] Polls `GET /health` with retries (max 5s / 10 attempts × 500ms) before proceeding
- [ ] POSTs to `/api/v1/jobs` with `{"query":"smoke test query"}` and asserts `job_id` is present in response
- [ ] GETs `/api/v1/jobs/{job_id}` and asserts `status` field is present
- [ ] Opens SSE stream at `/api/v1/jobs/{job_id}/events` via `curl --no-buffer`, reads at least 1 `data:` line within 10s
- [ ] DELETEs `/api/v1/jobs/{job_id}` and asserts response contains `"cancelled":true` or HTTP 200
- [ ] Kills server PID; verifies process is no longer running
- [ ] Exits 0 on full pass, 1 on any step failure
- [ ] Works without pre-installed `prometheus-research` — uses `cargo run --manifest-path ... -- --mode server` as fallback

## Risk

Low. New file only. Requires binary or cargo build to test locally.
