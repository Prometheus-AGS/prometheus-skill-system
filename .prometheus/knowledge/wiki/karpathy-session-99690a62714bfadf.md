---
type: SessionRecord
id: karpathy-session-99690a62714bfadf
title: Karpathy session 99690a62714b
tags:
- karpathy
- session-learning
sources:
- resource: session:5789740d-cbcd-4a6a-8e3e-182eb55399dd
generated:
  by: pk/1.11.0
  at: 2026-11-16T11:48:23.151361+00:00
created_at: 2026-11-16T11:48:23.151361+00:00
updated_at: 2026-11-16T11:48:23.151361+00:00
revision: 0
---

## Delta

The doctor exits 0 with no required failures, but it is not fully clean: 18 checks pass, 4 are skipped as optional, and 2 warnings remain. Both warnings were there before today's work.

**What today's work changed**

- **KBD runtime authority:** the 503 warning is gone. With the old daemon unloaded, the doctor reports "no control endpoint discovered; local KBD runtime is authoritative", which the pack treats as normal.
- **Prometheus Exec runtime:** passes.
- **Managed binaries:** 6 of 6 are executable, hashed and signed, and the CLI now reports 1.15.0.
- **Native plugin cache:** matches the active generation at 1.15.0.

**The two remaining warnings**

- **KBD production rollout gate:** shadow mode is active and promotion is blocked. It needs seven consecutive successful days and at least 100 real mutations; it has 1 day and 2.
- **Harness discovery budgets:** not measured for any of the four harnesses (Claude Code, Codex, OpenCode, Kimi).

**Skipped as optional:** managed MCP configuration, the control-plane extension, evolver state and the trace store.

One thing I noticed in the output: the doctor's own "source sha256" figures are wrong on this machine. It reads the hardcoded in-repo `target/release/` path, the same mistake the installer had. So it shows the stale 1.14.0 hash as the source for `prometheus` and "not-built" for `pk`. The check still passes because it only requires the installed binary to be hashed and signed. That lookup is in the Rust CLI (`doctor.rs`), which I have not touched.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 5789740d-cbcd-4a6a-8e3e-182eb55399dd
- Captured: 2026-11-16T11:48:21.352864Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- No changed paths detected.
