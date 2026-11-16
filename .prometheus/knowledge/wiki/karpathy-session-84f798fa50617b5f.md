---
type: SessionRecord
id: karpathy-session-84f798fa50617b5f
title: Karpathy session 84f798fa5061
tags:
- karpathy
- session-learning
sources:
- resource: session:5789740d-cbcd-4a6a-8e3e-182eb55399dd
generated:
  by: pk/1.11.0
  at: 2026-11-16T14:52:46.835982+00:00
created_at: 2026-11-16T14:52:46.835982+00:00
updated_at: 2026-11-16T14:52:46.835982+00:00
revision: 0
---

## Delta

The liter-llm gateway is restarted and now listens on loopback only: `lsof` shows `127.0.0.1:4000` where it showed `*:4000` before.

- **Process:** the new process runs with `--host 127.0.0.1`, which the fixed launch wrapper supplies. The service definition itself did not need to change.
- **Still working:** `/health` returns 200, an unauthenticated `/v1/models` returns 401 as it should, and an authenticated `/v1/models` returns 200 with 8 models.
- **Not tested:** I could not probe from a non-loopback address, because the Mac had no address on `en0` or `en1` at the time. The listener address from `lsof` is the evidence that other machines can no longer reach it.

The restart dropped any request that was in flight at 08:52; the log shows a clean shutdown.

The documentation changes from the previous turn are still uncommitted.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 5789740d-cbcd-4a6a-8e3e-182eb55399dd
- Captured: 2026-11-16T14:52:44.977702Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- CHANGELOG.md
- CLAUDE.md
- docs/guide/09b-liter-llm.md
- docs/guide/14-rust-toolchain.md
- docs/guide/16-cli-and-scripts.md
- docs/guide/26-service-operations.md
- docs/guide/value-scenario-workflow.md
- evals/vlbv-protocol/README.md
- site/docs/execution/installation-doctor-and-recovery.md
- site/docs/operations/doctors-and-mac-certification.md
- site/docs/operations/installation-and-upgrades.md
- site/sidebars.js
- site/docs/kbd/boundary-authorization.md
- site/docs/kbd/protocol-evals-and-capability-evidence.md
