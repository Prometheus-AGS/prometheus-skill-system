## 1. Local Control and Identity

- [x] 1.1 Add managed Unix-socket HTTP transport with explicit endpoint override and verify focused GET/POST transport tests pass.
- [x] 1.2 Route KBD commands and doctor diagnostics through the shared control transport and verify the complete CLI test suite passes.
- [x] 1.3 Scope managed device-signer discovery to the default canonical data root and verify managed and custom-root signer tests pass.
- [x] 1.4 Clarify that KBD runtime authority is hosted by sovereign-sync and verify live doctor health plus the unreachable-diagnostic wording regression pass.

## 2. Partial Authority Availability

- [x] 2.1 Preserve routes for successfully opened registered projects when other registrations fail and verify the partial-authority regression test passes.
- [x] 2.2 Log concrete per-project open failures without key material and verify restart logs identify stale registrations while healthy routes remain writable.

## 3. KBD Orchestration Compatibility

- [x] 3.1 Initialize the runtime child label before activation and verify `/bin/bash` 3.2 syntax plus canonical child creation succeed.
- [x] 3.2 Compare fully-qualified child IDs from `progress.json.phaseId` in stage gates and verify the explicit child assess/plan gate succeeds.

## 4. Distribution and Runtime Certification

- [x] 4.1 Refresh generated Codex plugin distributions twice and verify the second generation is byte-identical and `npm run validate:codex` passes.
- [x] 4.2 Run local formatting, clippy with warnings denied, full touched-package tests, release builds, protected-test verification, and `git diff --check`, recording every command and result.
- [x] 4.3 Install and sign release binaries, force two supervised launchd restarts, and verify socket health plus signed KBD mutations after each restart.

## 5. Completion and Handoff

- [x] 5.1 Establish validated archive readiness and write an execute handoff that retains stale-registry and historical-projection risks for KBD reflection and parent restoration to exact next command `/kbd-new-phase kbd-control-plane-recovery`.
