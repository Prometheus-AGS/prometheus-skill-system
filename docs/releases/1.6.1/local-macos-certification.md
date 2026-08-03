# Deterministic Memory and Learning 1.6.1 — Local macOS Certification

Date: 2026-08-03

Host class: macOS x86_64

Release loop: local only; GitHub Actions was not used for development or diagnosis.

## Certified source heads

| Repository | Recovery base | Certified implementation head |
| --- | --- | --- |
| `surreal-memory-server` | `83c9bc2` | `acd962d` |
| `prometheus-knowledge-rs` | `e5cb0dd` | `5143891` |
| `prometheus-skill-system` | `7e83779` | `2f42091` |

The server and knowledge heads merge their newly advanced `main` branches without rebasing or dropping the certified implementation commits (`7a0d5d2` and `b2f796c`). The root documentation/evidence commit follows the implementation head and does not alter the installed runtime. The isolated release worktrees preserved the dirty `main` worktree.

## Mandatory exclusions

Every Prometheus doctor, repair-plan, refresh-plan, service installer, and repository health command used these exclusions:

```text
--exclude control.kbd-runtime
--exclude state.kbd-orchestrator
--exclude control.kbd-rollout
--exclude service:sovereign-sync
```

The negative fixtures passed for diagnosis, `--fix`, `--refresh`, rendering, restart, and install selection. Excluded checks are filtered before construction, and excluded repair actions are absent from the plan.

## Local build and test evidence

All Cargo caches and targets were on the internal SSD under `$HOME/.cargo` and `<projects>/prometheus/.cargo-target/docs-release`.

| Surface | Commands | Result |
| --- | --- | --- |
| Memory server, native contract | `cargo fmt --all -- --check`; `cargo check --workspace --all-targets`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo build --release` | PASS |
| Memory server, installed Mac contract | Same check/clippy/test flow with `--features embedded,metal,local-embeddings`, then release build | PASS; 45 library tests, 36 integration tests, operation/recovery suites, server-mode suites, and doc tests green |
| Knowledge and worker | workspace format, check, clippy with warnings denied, tests, release build | PASS |
| Root CLI | workspace format, check, clippy with warnings denied, tests, release build | PASS; doctor exclusion and dependency fixtures green |
| Hooks | Karpathy dispatch, memory bridge, rotation, and `pk-health` fixtures | PASS; Stop performs one atomic mode-0600 enqueue and no synchronous network/model/memory work |
| Installer | learning-service rendering and service-exclusion fixtures | PASS |
| Plugin distribution | generation, manifest, 14-target parity, rollback, restore, collision, and uninstall fixtures | PASS |
| Skills | lenient, strict, Codex plugin, and generated index checks | PASS; 145 install payloads across 14 targets |
| OpenAPI and docs | public-doc safety, OpenAPI 3.1 validation, deterministic examples, semantic drift, links/sidebars, production build | PASS |
| Dependency security | root and Docusaurus `npm audit --audit-level=high` | PASS; zero vulnerabilities |
| Secret scanning | `gitleaks git` and `gitleaks dir` in all three repositories | PASS; one documented UUID redaction fixture is fingerprint-allowlisted |
| Root smoke | `scripts/smoke-test.sh` | PASS: 11, FAIL: 0, SKIP: 1 |

The smoke skip is explicit: the optional `librefang-wasm-skill` source is not present in this recovery checkout. The smoke script now distinguishes an absent optional source from a missing Rust target.

## Installation evidence

Previous binaries were copied to `$HOME/.prometheus/repair/1.6.1-preinstall-20260803` before replacement. The following ad-hoc-signed SHA-256 hashes were installed:

| Binary | Installed SHA-256 |
| --- | --- |
| `prometheus` | `a5d6cd539d4212a37dbee419efeb9f6afcfeba7bf443f296a6e520dc60960dbd` |
| `pk` | `e1c13f0fa6a2e8c2fdc89a533a3bfa359a02ca9268681540170503038209c436` |
| `pk-cherry` | `44987c7dd9f5c61e5833b5f59262d295dbd78fbe03ae7ddc3a5214181ef07e13` |
| `prometheus-learning-worker` | `5b40632d654c7e3b402b2f35715945303c9efcf2a6806967a612ed695d07066b` |
| `surreal-memory-server` | `ef95a1f19666008f68c2a5888e17ca8c6291b970049974c2e044149b8254e0ab` |

`pk`, `pk-cherry`, the worker, and the memory server were installed in every path used by the LaunchAgents. `codesign --verify --strict` passed for every managed binary.

Installed runtime state:

- Memory, knowledge, learning-worker, and hook-rotation LaunchAgents are loaded.
- The worker is event-driven and exits zero when the queue is settled.
- The rotation agent is scheduled daily, resolves `/usr/local/sbin/logrotate` plus `/usr/local/bin/flock`, and a manual scheduled run exited zero.
- Active plugin generation `a9ca01e7…fe01d` verifies its manifest, stable dispatchers, rollback pointer, receipts, and all 14 targets.
- A host rollback to `165b0707…1200` and a second rollback to the original generation both verified.
- The obsolete Claude plugin cache was moved to Trash as `prometheus-skill-pack-1.6.0-stale-cache-20260803`; it is recoverable. No active configured path resolves through `1.6.0`.

## Doctor and health matrix

| Command | Result |
| --- | --- |
| `prometheus doctor --json` with all four exclusions | PASS: 11, WARN: 1, FAIL: 0, SKIP: 3, exit 0 |
| `npm run doctor` | PASS with exclusions embedded in the package script |
| `pk doctor --json` | PASS: 6, WARN: 0, FAIL: 0 |
| `TERM=xterm-256color codex doctor --json` | `overallStatus: ok`, 18/18 checks ok |
| `cowork doctor` | Required checks pass; optional router warning |
| `cowork toolchain check` | PASS: all required tools present |
| `prometheus-services.sh doctor --exclude sovereign-sync` | PASS for allowed binaries, definitions, loaded state, health, and readiness |
| `check-mcp-health.sh --json --exclude sovereign-sync` | PASS for all allowed HTTP/MCP surfaces; protected Forge returns expected 401 |
| `prometheus learning status --json` | 2 completed jobs, 4 completed memory receipts, zero pending/processing/retry/rejected/dead-letter records |

Redacted machine-readable reports are in `prometheus-doctor.redacted.json`, `runtime-surfaces.redacted.json`, and `memory-certification.redacted.json`.

## Behavioral certification

The live Mac runtime certified:

- `/health` and `/ready`, including coordinator, ledger, storage, tokenizer, search index, and model executor readiness.
- Same-ID/same-hash byte-equivalent replay.
- Same-ID/different-hash HTTP 409 conflict.
- Response-loss reconciliation by operation ID.
- Ordered SSE history and `after` resume.
- A 36,705-token logical memory committed as one memory after persisted multipart execution.
- Restart recovery from executor generation 1 to 2 with the same operation ID and persisted plan.
- A real stable Stop dispatcher enqueue, supervised worker ingestion, accepted receipt reconciliation, and terminal committed memory receipt.
- Project/shared/global immutable snapshot pointers and settled queue state.
- Plugin activation, rollback, restoration, target receipts, and stale-path absence.
- Log-rotation configuration, dependencies, lock coordination, schedule, and exit zero.

## Warning dispositions

1. **Discovery-budget warning:** doctor reports 0/4 measured harness budgets. This is an instrumentation gap, not an installation failure. The installed payloads are independently covered by the 145-payload/14-target parity test, stable-dispatcher verification, and active-generation receipts.
2. **Optional doctor skips:** declarative MCP config reconciliation is reserved but unimplemented; Evolver is not initialized; no optional trace store exists. None is required by the deterministic memory/learning release.
3. **Codex non-interactive terminal:** the first run inherited `TERM=dumb` from the command runner and failed only that environment check. Re-running with `TERM=xterm-256color` produced `overallStatus: ok`. Codex 0.144.3 reports a newer standalone release, but the installed runtime is internally consistent and was not changed during this release.
4. **CoWork optional inventory:** `cowork-router` and `mmx` are optional and absent. CoWork's generic status also labels authenticated MCP routes as missing; the canonical exclusion-aware health checker proved knowledge HTTP 200 and Forge's expected protected 401. `cowork toolchain check` passed all required tools.
5. **Knowledge lint inventory:** `pk lint` completed with exit 0 and reported 364 pre-existing wiki quality findings, mainly missing optional descriptions, broken historical cross-links, and duplicate session records. These are content-quality backlog, not queue, snapshot, receipt, or runtime failures; `pk doctor` remained 6/6 green.
6. **CUDA all-features build:** an exploratory `--all-features` build requires NVIDIA `nvcc`, which is unavailable on this Mac. The installed and certified platform contract is `embedded,metal,local-embeddings`; its format, check, clippy, tests, and release build all pass.
7. **Excluded-service caveat:** the legacy `cowork toolchain status` command has no exclusion flag and performed one read-only health probe of the excluded service before its behavior was observed. It made no install, restart, rewrite, or credential change and is not used as certification evidence. All repository-owned diagnosis, repair, refresh, install, and health surfaces remained exclusion-aware.
8. **Initial PR workflow cancellation:** the first root PR validation run inherited legacy unconditional jobs. Before cancellation completed, the KBD packaging fixture started and failed; the Ubuntu and macOS control-plane jobs completed setup, but their KBD and sovereign test steps were skipped. The run was cancelled immediately. The workflow now honors `exclude:kbd` and `exclude:sovereign-sync` PR labels before scheduling those steps/jobs, and the release PR carries both labels.

## Publication boundary

GitHub is used only after these local gates for final PR parity and Pages deployment. Any follow-up failure must be reproduced and fixed locally before another push.
