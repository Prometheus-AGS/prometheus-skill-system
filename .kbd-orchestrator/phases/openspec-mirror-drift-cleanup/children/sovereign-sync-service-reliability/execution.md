EXECUTION: openspec-mirror-drift-cleanup› sovereign-sync-service-reliability
Project: prometheus-skill-pack
Date: 2026-08-29
Selected backend: openspec
Dispatched to: SELF through kbd-apply
Backend rationale: The phase has one medium, cross-module, spec-backed change whose implementation and verification must remain traceable to canonical KBD task transitions.
Backend entrypoint: `/kbd-apply repair-sovereign-sync-kbd-availability`
OpenSpec available: YES
Source plan: `.kbd-orchestrator/phases/openspec-mirror-drift-cleanup/children/sovereign-sync-service-reliability/plan.md`

EXECUTION SCOPE

- repair-sovereign-sync-kbd-availability: Keep managed KBD usable across daemon startup degradation and supervised restarts.

DISPATCH CONTRACTS

- repair-sovereign-sync-kbd-availability → kbd-apply / OpenSpec
  Entry: `/kbd-apply repair-sovereign-sync-kbd-availability`
  Model class: medium
  Concrete model: session default (the project has no `model_policy.registry` configuration)
  Model rationale: The change crosses the runtime, daemon, CLI, and generated skill boundary but introduces no new external protocol or state schema.
  Progress file: `.kbd-orchestrator/phases/openspec-mirror-drift-cleanup/children/sovereign-sync-service-reliability/progress.json`
  Handoff: Record every task through kbd-apply, transition the canonical change only after local QA, and retain the restart evidence in reflection.

APPROVAL GATES

- NONE — all work is local and within the user-authorized repository/service scope.

FALLBACK CONDITIONS

- If the OpenSpec adapter cannot enumerate or transition tasks, stop and repair the KBD wrapper rather than invoking bare `/opsx:apply`.
- If installed runtime checks contradict source tests, keep the change in progress and diagnose the deployed binary/service boundary.

VERIFICATION REQUIREMENTS

- `npm run build:codex` twice with identical distribution hash, then `npm run validate:codex`.
- Rust formatting, clippy with `-D warnings`, full tests, and release builds for the CLI, kbd-runtime, and sovereign-sync workspaces.
- `/bin/bash` 3.2 syntax checks for both edited orchestration scripts.
- `node scripts/verify-protected-tests.mjs` and `git diff --check` locally.
- Two launchd restarts, Unix-socket health 200, and accepted signed KBD mutations.

PROGRESS LEDGER

- [DONE] repair-sovereign-sync-kbd-availability — kbd-apply / OpenSpec; 12/12 implementation tasks complete, local QA passes, deployed runtime healthy, and distinct-model review PASS (0 critical / 3 warnings / 1 suggestion).

OUTPUTS

- OpenSpec change `repair-sovereign-sync-kbd-availability`
- Locally installed release binaries and supervised-restart evidence
- Refreshed generated skill distributions

BLOCKERS

- NONE

REVIEW DISPOSITION

- Corrected the default Unix-socket unlink/bind race, separated reachable HTTP failures from transport unreachability in doctor, and gated the Unix-only `anyhow!` import.
- Verified the managed-key warning was a false positive: `load_device_key` uses `symlink_metadata`, rejects symlinks/non-regular files, and enforces mode 0600 on Unix for explicit and discovered keys alike.
- The installed service returned Unix health 200 and `control.kbd-runtime` PASS at canonical revision 22 after the final supervised restart. Its initial authority replay exceeded ten seconds and correctly surfaced as an initializing/reachable state rather than a separate `kbd-runtime` daemon failure.

REFLECTION HANDOFF

- Distinguish the unreproduced historical shutdown report from the reproduced transport/startup/signer availability failure.
- Retain the three stale registry paths and ten-phase compatibility projection drift as explicit parent-owned follow-up.
- Resume the parent with exact next command `/kbd-new-phase kbd-control-plane-recovery`.

EXECUTION READY
