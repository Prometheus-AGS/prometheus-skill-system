# Phase Reflection: openspec-mirror-drift-cleanup::sovereign-sync-service-reliability

**Project:** prometheus-skill-pack
**Date:** 2026-08-29
**Phase completion:** 100%
**Changes completed:** 1 / 1

The historical claim that `sovereign-sync` repeatedly shut down was not reproduced. Launchd supervision was already configured with `KeepAlive` and `RunAtLoad`; the observed KBD outage instead came from a legacy TCP assumption, all-or-nothing authority startup, divergent signing identities, and diagnostics that described embedded `kbd-runtime` authority as a separate daemon. The corrective work addresses those reproduced failures, but the stale registry and historical projection debt remain parent-owned.

## Goals

| Goal | Status | Notes |
| --- | --- | --- |
| Determine why sovereign-sync repeatedly exits and is not restarted | PARTIAL | No unsupported shutdown or failed launchd restart was reproduced. The false-down and blocked-KBD symptoms were reproduced at the transport, authority-loading, signer, and diagnostic boundaries. |
| Implement a durable service recovery and restart fix | MET | Managed clients retain the canonical Unix socket through unlink/bind windows, healthy authorities survive partial registry failure, all-authority failure blocks startup, canonical clients share the enrolled signer, and reachable HTTP errors are distinct from transport failure. |
| Verify sovereign-sync remains healthy and no longer blocks KBD | MET | Local Rust/shell/distribution gates passed; installed signed binaries survived supervised restarts; Unix health returned 200; signed mutations advanced the canonical runtime to revision 23; doctor reported writer/projection/signature health. |
| Return control to the parent phase and resume its exact next command | PARTIAL | Reflection is complete; `/kbd-child-exit` still must roll up the child and restore `/kbd-new-phase kbd-control-plane-recovery`. |

## Delivered Changes

- `repair-sovereign-sync-kbd-availability` — managed Unix transport, restart-window target retention, signer convergence, partial authority availability, fail-closed all-authority startup, concrete project errors, embedded `kbd-runtime` doctor classification, KBD child/stage-gate compatibility repairs, regenerated distributions, and installed runtime certification (by: Codex through kbd-apply/OpenSpec).

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with QA | 1 / 1 |
| Deterministic QA pass rate | 1 / 1 (100%) |
| Distinct-model reviews | 1 final PASS |
| Final critical findings | 0 |
| Final warning dispositions | 3 corrected or disproved |

The final review's socket-race and HTTP-classification warnings produced bounded corrections. Its managed-key warning was disproved by the shared `load_device_key` validator, which uses `symlink_metadata`, rejects symlinks/non-regular files, and enforces mode 0600 on Unix. Generated Claude/Codex distributions were byte-identical across consecutive runs at combined SHA-256 `0b4ab45771b497d3724a1a3da758080581800eb76ab2b8b9e1035eec9d8b214e`.

## Technical Debt

- Three registered project paths no longer exist. They are isolated as per-project failures and no longer disable healthy routes, but removal requires an explicit operator registry-cleanup decision.
- Ten historical compatibility projections contain counters ahead of canonical runtime state. The runtime preserved the evidence and backup; the parent `openspec-mirror-drift-cleanup` phase owns reconciliation.
- Authority replay after restart exceeded the initial ten-second observation window and emitted extensive projection-ownership warnings for stale worktrees. The service recovered without a crash, but startup latency/noise should be reduced in a later control-plane recovery phase.
- A Unix request timeout after dispatch remains conservatively classified as ambiguous because the command may have committed before the response was lost.

## Architecture Integrity

- AGENTS.md violations: NONE. All validation and certification ran locally; no hosted CI was used.
- Constraint violations: NONE. Generated outputs are synchronized and idempotent, staged secret scanning is clean, no protected tests changed, and edited shell paths pass Bash syntax/regression checks.
- Scope integrity: historical projections and stale registry entries were deliberately not rewritten or deleted.

## Cross-Tool Coordination Notes

- Progress tracking: GAPS FOUND — canonical KBD recorded the task transitions performed after runtime-authority repair (tasks 9, 11, and 12), while the archived OpenSpec checklist is the complete 12/12 implementation record. The change-level implementation counter remained correct at 1/1.
- Handoff quality: CLEAR — assessment, plan, execute handoffs, refinement evidence, and review receipts consistently retained parent ownership of stale registry/projection cleanup and the exact parent command.
- Recommendations: initialize the runtime-authoritative child before any OpenSpec task is completed so every backend task transition appears in the canonical event ledger; treat compatibility files only as generated projections.

## Lessons Learned

- A supervised service can be alive while clients report it down when discovery targets a transport the supervisor never exposes.
- `kbd-runtime` is an embedded library hosted by `sovereign-sync`; health messages must name that hosting boundary to avoid invented restart advice.
- During Unix-socket unlink/bind windows, managed clients must retain the expected socket target rather than silently fall back to a legacy TCP default.
- Partial registry degradation should preserve healthy authorities, while an all-failed authority set should fail startup explicitly.
- Shared signer discovery must be limited to canonical managed data roots and reuse the same permission/symlink validator as explicit key configuration.
- Startup health and authority readiness are different signals; diagnostics should distinguish reachable-but-initializing HTTP responses from transport unreachability.

## Next Phase Focus

Resume `openspec-mirror-drift-cleanup` with `/kbd-new-phase kbd-control-plane-recovery`, prioritizing:

1. Explicit review and cleanup of the three stale registry entries.
2. Reconciliation of the ten preserved historical projection/canonical counter disagreements without discarding evidence.
3. Reduction and observability of authority replay latency and stale-worktree projection warnings during supervised startup.

## Context for Next Phase

Use this file as prior context for the parent phase. The service availability repair is archived and locally deployed; remaining work is control-plane data hygiene and startup observability, not creation of a separate `kbd-runtime` daemon.
