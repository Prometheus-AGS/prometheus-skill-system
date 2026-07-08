# Tasks — change-prui-005-smoke-test

- [ ] task-001: Create `substrate/prometheus-research/scripts/` directory
- [ ] task-002: Write `smoke-test.sh` with full lifecycle: start → /health poll → POST job → GET status → SSE read → DELETE → kill server
- [ ] task-003: `chmod +x substrate/prometheus-research/scripts/smoke-test.sh`
- [ ] task-004: Test locally: `bash substrate/prometheus-research/scripts/smoke-test.sh` — confirm exit 0
- [ ] task-005: Commit with message `test(prometheus-research): add smoke-test.sh for full HTTP API lifecycle`
