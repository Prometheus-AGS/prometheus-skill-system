# Final local verification contract

Status: specified, not run. This contract belongs to change-ldd-12-integration-rollout. All production in production changes 01-11 and 13-18, including protected pin edits after approval, must be complete before test authoring. Generated reconciliation is the last production batch; tests follow it. No task-level verify command may run an early subset gate.

## Planned entry point

The final task will author `scripts/tests/test-learning-deploy-and-debt.sh`. It must accept explicit absolute `--full`, `--mini`, `--surreal` and `--evidence` paths and coordinate actual production programs and real services. It does not exist yet and has not passed. Planned invocation, only at the final boundary:

```sh
bash /absolute/full-worktree/scripts/tests/test-learning-deploy-and-debt.sh \
  --full /absolute/full-worktree \
  --mini /absolute/mini-worktree \
  --surreal /absolute/surreal-worktree \
  --evidence /absolute/phase-evidence
```

The coordinator must use Bash 3.2-compatible orchestration on macOS, allocate its own private scratch root, set HOME/CODEX_HOME/CORTEX_DATA_DIR/PROMETHEUS_PLUGIN_ROOT and all learning outbox/log/index roots under it, and clear inherited service/auth/plugin settings that would escape it. Tool binaries may be referenced read-only from declared paths; no real auth, configuration, model-account state or databases may be symlinked/copied into the test. Local preprovisioned model assets may be explicit read-only inputs. Run actual service instances on owned ports and databases, never :23001. Do not inspect real ~/.cortex as a test assertion. Clean only owned process trees/files.

Inventory the real integration scenarios before choosing commands. Existing package scripts may include unit/module suites and therefore cannot be blindly reused. Run direct production distribution --check commands as supplemental drift checks, actual installer/CLI integration scripts for acceptance, and an explicitly selected real-server integration target (new `query_cache_production`) for Rust. No bare cargo test, cargo test --lib, npm test, node --test without a specific accepted integration file, fake embedding provider or stub Cortex acceptance. Do not launch a real Codex session unless its scratch isolation/provider configuration is established.

Immediately before each Cargo command, inspect both `pgrep -x cargo` and `pgrep -x rustc`; a busy machine is BLOCKED or waited on, not bypassed with a new target directory. Use normal per-worktree target directories. Keep Rust compilation serialized across repositories and external agent sessions. All gates/reviews run locally; no GitHub Actions.

## Matrix and result record

- Full: copied/installed immutable hook generation, compiled and shell dispatch, direct wrapper/feeder, bytecode negative control; installer memory controls, real Codex consolidation control and restart; installed doctor repair; custom-home install/check/uninstall/rollback/prune; output ownership/rebase negative controls; partition byte boundaries; real Cortex concurrency/delivery/crash; packaged spec guidance examples.
- Mini: actual generator/package execution and executable-mode drift; selected production installer checks; published pin alignment; new baseline with each historical failure classified mapped, fixed, still failing, blocked or unverified. Historical `.kbd-orchestrator/phases/team-aware-learning-memory-impl/evidence/mini-baseline-failures.txt` remains immutable evidence; write the current version-bound baseline under this phase evidence directory. A legacy unit-only failure without production reproduction remains unverified and cannot be called fixed.
- Server: the final frozen service release for deployment/load comparison; recorded historical baseline with real local executor/SurrealDB for namespace, counters, coalescing, write isolation and failure behavior. Keep identities distinct.
- Cross-cutting: twice-generated identical bytes/modes; manifest and selected harness/release checks; protected-test integrity from committed candidate state; independent cumulative review receipt with actual model identity distinct from producer.

Each result stores source commit/tree hash, command/argv, tool version, UTC start/end, exit code, isolated artifact/service roots, stdout/stderr paths, acceptance IDs and negative-control result. 0 means PASS, 1 means FAIL, 2 means BLOCKED; a skip, unavailable local Codex provider/model, protected-test approval, missing distinct reviewer or busy Cargo never becomes PASS. Fix actual failures in coherent batches and rerun the smallest applicable final gate; any changed artifact invalidates its earlier receipt.

## Integration, certification and deployment are separate

A scratch candidate Git commit may anchor protected integrity before publishing; record its tree and prove the eventual published source matches. Source implementation alone cannot complete change 12. Only publish reviewable PRs after required local evidence passes; the user merges. Do not manufacture a successful boundary receipt, silently clear historical blockers or weaken acceptance because a prerequisite is unavailable.

Deployment is separately owner-approved and follows the versioned delivery-cadence procedure. It may read installed identity/health under that approval but all load/recall tests still use scratch services. Never modify the protected deployment worktree. If approvals/merges are pending, retain pending publication/deployment tasks and report the phase incomplete.

## Expanded final closure

The coordinator also covers Companion source/publication, all team/model/cross-project examples, both Docusaurus production builds plus served-site navigation, complete content inventory, source-disposition coverage, every final pin remote resolution and fresh-clone installation, and historical migration/two-process/standalone certification. The full skill-system pin is owner-selected ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61; all other dependency identities still require final provenance and release decisions. No fixed v1.10.1 target is authorized by this document. All publication/merge/approval and maintenance-readiness results remain pending.
