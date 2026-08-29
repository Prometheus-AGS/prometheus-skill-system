ASSESSMENT: openspec-mirror-drift-cleanup› sovereign-sync-service-reliability
Project: prometheus-skill-pack
Date: 2026-08-29
Codebase baseline: The managed daemon was alive under launchd, but transport, startup-gating, and signer mismatches made KBD behave as if it were down.
Cross-tool progress: none

IMPLEMENTATION STATUS
- launchd supervision: VERIFIED, NOT CHANGED — `~/Library/LaunchAgents/ai.prometheus.sovereign-sync.plist` has `KeepAlive=true`, `RunAtLoad=true`, and `ThrottleInterval=10`; before the repair `launchctl` reported PID 921, runs 1, and no prior exit, so the reported repeated non-restart was not reproduced.
- managed transport: DONE — `tools/prometheus-cli/crates/prometheus-cli/src/commands/control_transport.rs`, `kbd.rs`, and `doctor.rs` prefer the private same-user Unix socket and retain an explicit TCP override. The focused Unix GET/POST test and installed `prometheus kbd status --json` passed.
- partial authority availability: DONE — `substrate/sovereign-sync/src/rest_api.rs` no longer rejects the whole router when some registrations fail; `kbd_control.rs` preserves per-project errors. After restart, logs reported 27 opened and three failed while this project remained writable.
- signer convergence: DONE — `substrate/kbd-runtime/src/lib.rs` discovers the existing mode-0600 managed key only for the default canonical platform data root. The signer-scope regression test passed and child creation advanced canonical revision 5 to 7 without rejection.
- failure diagnostics: DONE — `substrate/sovereign-sync/src/kbd_control.rs` logs the project ID and concrete open error. Restart logs identified the three missing-path projects instead of only `success=false`.
- KBD child workflow: PARTIAL — canonical control was repaired, the child was created at revision 7, and this assessment records the assess stage; plan/execute/reflect/exit remain.

CROSS-TOOL PROGRESS
- NONE — no other tool-owned change was recorded in the child progress ledger.

SPEC GAP SUMMARY
- The platform registry at `~/Library/Application Support/prometheus/kbd/registry.json` contains three missing-path projects: `b22f29b4-6b30-4ff1-a47d-9d8c0b03e805` (`~/.claude/worktrees/uar-uiux-refinement-2026-08`), `0df3daa9-273b-4d18-a386-4079aaee91e1` (`~/.claude/worktrees/uar-uiux-refinement-followup-2026-08`), and `9df8c584-0aaf-43f1-aef5-c1b6c5572552` (a deleted `/private/tmp/.../scratchpad/repro5`). They remain visible as per-project errors but no longer disable healthy routes.
- Compatibility counters exceed canonical state for ten historical phases: `adversarial-review-for-creation`, `ideation-and-decision-tools`, `kimi-desktop-extensibility`, `mobile-skill-portability`, `openspec-mirror-drift-cleanup`, `prometheus-exec-code-execution-engine`, `sovereign-sync-domain-adapters`, `uar-frontend-workspace-repair`, `uar-host-execution::mcp-2026-07-28-adoption`, and `uar-host-execution`. The built-in migration refused to discard that evidence and backed it up under `~/Library/Application Support/prometheus/kbd/projects/6ac090a4-3656-4d83-8eb6-2891508196d5/migration-backups/20260829T110943Z-4db46ff9-45f7-458a-b1f0-632f55b081b0/`. Resolution ownership returns to the parent `openspec-mirror-drift-cleanup` phase; this child starts a clean successor run rather than rewriting historical evidence.
- `kbd-new-child` runtime mode referenced `child_label` before assignment, and the stage gate compared a child basename with its fully-qualified canonical phase ID. Both seams required package-source fixes.

BUILD HEALTH
- build check: PASS — release builds completed for `prometheus-cli` and `sovereign-sync`.
- tests: PASS — CLI 30/30, kbd-runtime 74 passed with 6 operator proofs ignored, sovereign-sync 73/73 across unit and integration suites.
- lint: PASS — clippy passed for all touched packages with `-D warnings`.
- formatting: PASS — both Rust workspaces pass `cargo fmt --all -- --check`.
- known violations: NONE in the touched surface.
- test coverage: PARTIAL — focused regressions fully cover the three changed behaviors (Unix GET/POST, signer discovery scoping, and partial authority availability). Six live kbd-runtime operator proofs remain intentionally ignored because they require explicit `KBD_CERT_*` or migration-proof data roots; the ordinary unit/integration paths pass.

CHANGED-FILE AND VALIDATION EVIDENCE
- `substrate/kbd-runtime/src/lib.rs` — managed signer discovery and its data-root scoping test; validated by `cargo test --manifest-path substrate/kbd-runtime/Cargo.toml` and clippy.
- `substrate/sovereign-sync/src/rest_api.rs` and `kbd_control.rs` — partial startup availability, concrete error logging, and failed-registration regression; validated by sovereign-sync unit/integration tests, clippy, release build, and two installed restarts.
- `tools/prometheus-cli/Cargo.toml`, `crates/prometheus-cli/Cargo.toml`, and `crates/prometheus-cli/src/commands/{mod.rs,control_transport.rs,kbd.rs,doctor.rs}` — Unix transport and call-site integration; validated by 30 CLI tests, clippy, release build, and installed KBD status/mutations.
- `skills/process/kbd-process-orchestrator/skills/kbd-new-child/kbd-new-child.sh` — defines `child_label` before runtime-mode use; `/bin/bash` 3.2 syntax check passes and the repaired skill created the canonical child.
- `skills/process/kbd-process-orchestrator/shared/lib/stage-gate.sh` — compares canonical child IDs using `progress.json.phaseId`; `/bin/bash` 3.2 syntax and explicit child assess gate pass.
- Launchd invokes `/Users/gqadonis/.local/bin/sovereign-sync --mode daemon` directly; neither edited shell skill is launchd-reachable. Forced restarts produced PID `75842 → 91030`, runs 4, last exit 0, and health 200.

CONSTRAINT CHECK
- AGENTS.md violations: NONE — all validation was local; no hosted CI was used.
- constraints.md status: PARTIAL PENDING REFRESH — C-01/C-04 require the skill distribution refresh and idempotency check in execute; C-02 passes because no key material or credentials are committed; C-03 does not apply because the Codex manifest/install surface is unchanged; C-05 passes because launchd invokes the Rust binary directly and both edited orchestration scripts pass `/bin/bash` 3.2 syntax checks.

GOAL PROGRESS
- Determine why sovereign-sync repeatedly exits and is not restarted: PARTIAL — the reported repeated non-restart was not reproduced. Historical logs contain explicit shutdown signals and older `Database already open. Cannot acquire lock.` bursts, but the observed launchd job was continuously running with KeepAlive enabled. The reproducible KBD block was stale TCP-only client transport plus all-or-nothing authority startup.
- Implement a durable service recovery and restart fix: MET FOR THE OBSERVED FAILURE MODE — launchd supervision required no code change; durable repair consists of Unix transport, signer convergence, partial availability, and concrete diagnostics, all installed and restart-verified. No redundant supervision change is planned.
- Verify sovereign-sync remains healthy and no longer blocks KBD: MET — forced restarts changed PID 75842 to 91030, health returned 200, and signed KBD mutations advanced revision 5 to 7.
- Return control to the parent phase and resume its exact next command: PARTIAL — child exit must restore parent `openspec-mirror-drift-cleanup`, whose saved reflected position is 6/6 with exact next command `/kbd-new-phase kbd-control-plane-recovery`.

SYCOPHANCY REVIEW
- The assessment retains the unresolved stale registrations and historical projection drift as explicit risks rather than treating successful health checks as total system health.

ASSESSMENT COMPLETE

UNRESOLVED REVIEW FINDINGS

The KBD artifact gate requires second-round CRITICAL findings to remain in the
artifact. The final revision above addresses each one, but the findings are
preserved verbatim for downstream visibility.

1. Claim: "The assessment declares the repair implementation complete without providing any verifiable code, diff, or path-level evidence for what changed."
   Evidence: "It states \"managed transport: DONE\", \"signer convergence: DONE\", and \"failure diagnostics: DONE\", but the packet also reports `\"cited_paths\": \"(no file paths cited in this artifact)\"` and `\"prior_handoffs\": null`. No modified files, commits, symbols, commands, or outputs identify the claimed repairs."
   Suggested fix: "Add a changed-file inventory and, for each DONE item, cite the relevant source path and behavior, the validation command, and the observed before/after result."
   Resolution: addressed by `CHANGED-FILE AND VALIDATION EVIDENCE` and the path-level implementation entries.

2. Claim: "The first goal is marked MET even though the assessment does not establish why sovereign-sync repeatedly exited or why launchd failed to restart it."
   Evidence: "The goal is \"Determine why sovereign-sync repeatedly exits and is not restarted\", but the assessment says \"launchd supervision was healthy\" and \"the observed daemon had not exited before this repair.\" It instead attributes the symptom to \"stale TCP assumptions and all-or-nothing authority startup\" without logs, exit statuses, restart events, or a causal link to the reported repeated exits."
   Suggested fix: "Provide launchd logs and exit/restart evidence demonstrating the original failure mechanism, or change the goal status from MET and state that the reported restart failure was not reproduced."
   Resolution: the goal is now PARTIAL and distinguishes unreproduced restart behavior from the reproduced KBD availability failure.

3. Claim: "The clean constraint check is unverifiable because the artifact does not identify the edited files or prove that the edited shell scripts cannot be invoked by launchd."
   Evidence: "The assessment concludes \"constraints.md violations: NONE\" and states \"the two edited shell scripts are orchestration skills and are not launchd entrypoints,\" but neither script is named and no launchd `ProgramArguments`, call graph, or `/bin/bash` compatibility result is provided. It also says C-01/C-04 \"do not apply\" without a changed-file list proving no generator inputs or generated artifacts were touched."
   Suggested fix: "Include the complete changed-file list, classify each file against C-01 through C-05, cite the launchd invocation path for every shell script, and record `/bin/bash` test results for any launchd-reachable script."
   Resolution: the inventory names both scripts, records `/bin/bash` 3.2 checks and launchd `ProgramArguments`, and leaves C-01/C-04 pending execute-stage refresh.
