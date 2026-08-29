PLAN: openspec-mirror-drift-cleanup› sovereign-sync-service-reliability
Project: prometheus-skill-pack
Date: 2026-08-29
OpenSpec available: YES
Changes to implement: 1

CHANGE LIST (ordered)
1. repair-sovereign-sync-kbd-availability: Keep managed KBD usable across daemon startup degradation and restarts
   - Scope: `substrate/kbd-runtime/src/lib.rs` | `substrate/sovereign-sync/src/{rest_api.rs,kbd_control.rs}` | `tools/prometheus-cli/` transport and call sites | `skills/process/kbd-process-orchestrator/{skills/kbd-new-child,shared/lib/stage-gate.sh}` | generated `dist/plugins/` copies
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH
   - Details: Prefer the managed Unix socket, converge interactive and daemon signing identities, and preserve healthy project routes when stale registrations fail. Repair `kbd-new-child.sh` so `child_label` exists before runtime-mode activation, and repair `stage-gate.sh` so fully-qualified child `phaseId` values match canonical state. Refresh generated distributions idempotently, install release binaries, and record local restart/certification evidence.
   - Acceptance: all touched package tests, clippy with warnings denied, formatting, protected-test verification, distribution validation/idempotency, `/bin/bash` 3.2 syntax checks for both edited orchestration scripts, two launchd restarts, socket health, and signed KBD mutations pass locally. Confirm C-03 is not triggered because no Codex manifest, MCP, hook, or install surface changed; otherwise update the required docs before completion.
   - Parent resumption: after archive/reflection, `/kbd-child-exit` must restore `openspec-mirror-drift-cleanup`, preserve the historical-drift handoff, and resume its saved exact command `/kbd-new-phase kbd-control-plane-recovery`.

EXECUTION ROUND ORDER
Round 1: repair-sovereign-sync-kbd-availability

EXPLICIT SCOPE CUTS AND DEFERRALS
- Do not remove or rewrite the three stale registry entries automatically; they are now isolated and require an explicit operator registry-cleanup decision.
- Do not overwrite the ten historical compatibility projections whose counters exceed canonical state. Return their recoverable migration backup and resolution decision to the parent `openspec-mirror-drift-cleanup` phase.
- Do not add redundant launchd supervision behavior; the existing KeepAlive/RunAtLoad configuration is verified healthy.

COMMANDS TO RUN
/opsx:new repair-sovereign-sync-kbd-availability

PLAN COMPLETE
