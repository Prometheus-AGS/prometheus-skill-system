---
type: SessionRecord
id: karpathy-session-d36396632cc9ba19
title: Karpathy session d36396632cc9
tags:
- karpathy
- session-learning
sources:
- session:cebcfbb5-668f-49de-8c67-ea6fae9d262c
timestamp: 2026-08-06T09:04:40.140380+00:00
created_at: 2026-08-06T09:04:40.140380+00:00
updated_at: 2026-08-06T09:04:40.140380+00:00
revision: 0
---

## Delta

Pushed as `3c31581`.

## Neither service was actually broken

Both were **misdiagnosed by their probes** — and in sovereign-sync's case, the bad probe was also what kept breaking it.

**forge-rs (422):** the health checker sent a malformed JSON-RPC body — no `jsonrpc` field, empty `params`. The server's own error said it outright: `missing field 'jsonrpc'`. forge-mcp correctly rejects that; `pk-cherry` happens to be lenient and accepts it, which is exactly why one looked broken and the other didn't. With a spec-compliant `initialize`, forge-mcp returns `200`.

**sovereign-sync (UNREACHABLE):** 1.7.0 serves HTTP on a **unix socket** and binds no TCP port unless given `--tcp` — which the LaunchAgent doesn't pass. So `http://127.0.0.1:7892/health` could never succeed. Over its socket it answers `{"status":"ok","version":"1.7.0"}`.

## Why it "seemed broken a lot"

This is the part that actually matters. `install-mcp-services.sh` used that same dead `:7892` probe to decide *whether the service was already running*. It always failed, so **every installer run concluded the daemon was down and restarted a healthy process.** The installer was the thing breaking it. That's why sovereign-sync's PID kept changing across my earlier runs (996 → 83323).

## How it stays fixed

- `service-probe.sh` — `check_running_service` now accepts `unix:<path>` and probes HTTP over the socket, so *any* caller reuses a healthy socket daemon
- `install-mcp-services.sh` — sovereign-sync probes its socket; `prometheus-exec` got the same reuse-if-healthy check, since its delegated installer also restarted unconditionally
- `check-mcp-health.sh` — `unix:` mode does a real HTTP request, not just a "socket file exists" check
- `detect-toolchain.sh` — probes the socket before `:7892`

**Verified:** all 8 daemons green; two consecutive *live* installer runs left both PIDs untouched (83323 and 59046); `--restart` still forces a restart when you want one. Five test suites pass, including `test-detect-toolchain-sovereign-sync` at 4/4.

One transparency note: changing a status string broke an assertion in that test. I updated the test deliberately — the old string named a port the check never probes — rather than reverting the fix to keep it green.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cebcfbb5-668f-49de-8c67-ea6fae9d262c
- Captured: 2026-08-06T09:04:36.773490Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
