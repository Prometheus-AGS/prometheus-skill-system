# change-rah-004-daemon-job-execution

**Title:** Give prometheus-research a real --daemon-job that spawns a headless harness, mirrors its checkpoint, and exports a validated package
**Repository:** `prometheus-skill-pack`
**Phase:** research-agent-hardening
**Goal:** G1
**Depends on:** `change-rah-001-compile-baseline-and-timestamps`, `change-rah-003-stage-contract-driver`
**Backend:** native-kbd

## Why

`spawn.rs:48` passes `--daemon-job`, which the clap `Cli` does not define; the child dies on parse and the parent records `running`. `research_export` returns a placeholder note. No daemon job has ever executed (assessment, verified). Analysis D-01 option C, cand-004, cand-006.

## What Changes

- Add `--daemon-job <job_id>` to `Cli`; its handler reads the checkpoint, resolves a harness binary from `PATH` (`claude` preferred, `codex` fallback, per `references/headless-execution.md`), spawns it headless with a prompt that invokes `/deep-research` with the job's query, depth, and output root, and records the child pid; when neither binary exists the job is marked `blocked` with the reason and the handler exits 0.
- The parent tails the package's `checkpoint.json` written by the driver, mirrors stage and progress into the daemon checkpoint, and emits `agent.status` and `agent.error` SSE events; child exit codes map to `complete`, `blocked`, or `failed`.
- Implement `research_export`: validate `manifest.json` with the `jsonschema` crate against `research-manifest.schema.json`, return the package path and verification verdict, refuse with the validation error when invalid.
- Author `references/headless-execution.md`: harness resolution order, the prompt template, the `codex exec` hook-trust flag, and the hook policy, which resolves the analyze open question: the daemon itself fires no hooks; the headless child runs the real driver from change-rah-003, which fires the four deep-research hook scripts exactly as in the foreground; the child never fires KBD lifecycle hooks because the spawn environment sets `KBD_HOOKS_DISABLED=1` and the prompt states it.
- Integration test `tests/job_execution.rs`: puts a fake `claude` script on `PATH` that runs the real driver with the fixture stage runner from change-rah-003, starts a job through the production `start` path, waits for `complete`, and asserts the SSE stream carried stage events, the four deep-research hook markers exist under `RESEARCH_HOOK_LOG`, no KBD hook marker exists, and `research_export` returns a validated package; a second case with an empty `PATH` asserts `blocked`.

## Scope

Files this change may create, edit, or delete (tasks.json `files` is the per-task view):

- `substrate/prometheus-research/Cargo.toml`
- `substrate/prometheus-research/src/main.rs`
- `substrate/prometheus-research/src/job/spawn.rs`
- `substrate/prometheus-research/src/job/mod.rs`
- `substrate/prometheus-research/src/job/checkpoint.rs`
- `substrate/prometheus-research/src/mcp_server/mod.rs`
- `substrate/prometheus-research/src/http_server/sse.rs`
- `substrate/prometheus-research/tests/job_execution.rs`
- `substrate/prometheus-research/tests/fixtures/fake-claude.sh`
- `skills/research/deep-research/references/headless-execution.md`

Scope amendment (recorded during apply): files outside the list above that
the daemon required, edited minimally and listed so the packet is honest:

- `substrate/prometheus-research/src/job/daemon.rs` (new): the `--daemon-job`
  handler proper; `job/mod.rs` re-exports it. The spec named `spawn.rs` and
  `mod.rs` for the handler; a supervisor of a few hundred lines belongs in its
  own module.
- `substrate/prometheus-research/src/http_server/mod.rs`: one route,
  `POST /api/v1/jobs/{id}/events`, and `RESEARCH_EVENT_SINK` set for the jobs
  the server starts. The daemon runs in its own process and cannot reach the
  server's broadcast channel; this is how its events reach the SSE stream.
- `substrate/prometheus-research/tests/job_lifecycle.rs`,
  `tests/mcp_tools.rs`: `..Default::default()` on four `JobCheckpoint`
  literals, because the checkpoint gained six optional fields. No assertion
  changed.
- `substrate/prometheus-research/Cargo.lock`: `jsonschema 0.50` and its tree.

## Capabilities

- `research-pipeline-execution (daemon requirements added)`

## ADDED Requirements

### Requirement: The daemon runs a job
WHEN `research_start` is called and a harness binary is on `PATH`, THEN a child process executes the driver and the job reaches `complete` with a package on disk.

#### Scenario: Fake harness
- **WHEN** a fake `claude` on PATH runs the real driver with the fixture runner
- **THEN** the job status becomes `complete`, the package validates, and SSE carried at least one `agent.status` per executed stage

### Requirement: Daemon jobs fire research hooks only
A daemon-spawned job SHALL fire the four deep-research hook scripts through the driver and SHALL NOT fire any KBD lifecycle hook.

#### Scenario: Hook markers
- **WHEN** the fake-harness job completes with `RESEARCH_HOOK_LOG` set
- **THEN** the four deep-research markers exist and no KBD hook marker exists

### Requirement: Absence is blocked, not a crash
WHEN no harness binary is on `PATH`, THEN the job is `blocked` with the reason recorded and the daemon keeps serving.

#### Scenario: Empty PATH
- **WHEN** the job is started with no harness available
- **THEN** status is `blocked`, `/health` still answers, and no panic is logged

### Requirement: Export validates
`research_export` SHALL validate the manifest against the schema and refuse an invalid package.

#### Scenario: Invalid manifest
- **WHEN** a required field is removed from a completed package's manifest
- **THEN** `research_export` returns an error naming the field

## Constraints

- Implementation-first, integration-only evidence (CLAUDE.md highest-precedence policy): finish the coherent edit batch, then run the smallest full-integration gate named in `verification.md`. No unit tests, mocks, or snapshots count as delivery evidence.
- One Cargo build machine-wide at a time. Check `pgrep -x cargo` before any `cargo` command; if another build is active, wait or record BLOCKED, never start a competing build. `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Local-only validation: no GitHub Actions run is evidence.
- Verification labels are `verified | unverified | blocked | inferred` on claims and `PASS | PASS WITH NOTES | BLOCKED` on provenance. A gate that could not run is recorded BLOCKED with the reason, never described as passed.
- Scripts that launchd may invoke stay bash 3.2 compatible (constraint C-05): no `mapfile`, no `declare -A`.
- Every script touched keeps `set -euo pipefail` semantics and non-zero exit on failure; no silent `|| true` on a gate.
- The pack never depends on the Companion or any extension (integration contract rule 1); capability is discovered, never assumed (rule 2).
- Constraint C-01 (generated artifacts): `SKILL.md` and `skill.toml` files edited in this phase are inputs only to `generate:skills-index`; `skill-system.json`, the harness adapters, and the service manifest are not edited by any change in this phase. `change-rah-011-integration-evidence-and-docs` is the named reconciliation change: it regenerates the skills index and runs `check:distribution`, `check-harness-adapters.js`, and `check:services-manifest` at certification. No earlier change claims distribution certification.

## Open Questions

- Whether the daemon should also accept a `harness` field on `research_start` to pin the binary per job (default: no; PATH resolution only).

## Unresolved review findings

Adversarial review (k3 judge via gateway, producer `claude-fable-5-1`, `--mode diff` on `files.txt`), two rounds, receipts in `.kbd-orchestrator/phases/research-agent-hardening/review/change-rah-004-daemon-job-execution/` (`round1/` and the round-2 `findings.json`).

### Round 1 — BLOCK (3 CRITICAL, 2 WARNING)

| # | Finding | Disposition |
|---|---|---|
| C1 | `depth` and `citation_style` from REST/MCP were interpolated unquoted into the driver line the harness executes with permissions bypassed; unknown values passed through | **Accepted, fixed.** `run_params` validates both against the driver's vocabulary before any prompt exists and blocks the job with a reason naming the field; every value on the driver line is shell-quoted, canonical ones included. Test: a depth of `deep; touch /tmp/rah004-injected` ends `blocked`, no prompt is written, the harness is never invoked, the file does not exist. |
| C2 | `spawn_job` rewrote the checkpoint to `running` after spawning, racing the daemon and able to overwrite a `blocked` it had already reached | **Accepted, fixed.** `spawn_job` writes nothing after a successful spawn (only a failed spawn records `failed`); the daemon records its own pid and every later status. Test asserts `pid` and `harness_pid` on the completed job. |
| C3 | Cancel was check-then-act: `mirror` could overwrite a `cancelled` written between the poll check and its own write | **Accepted, fixed.** Every daemon write goes through `save`, which re-reads the file immediately before writing and never overwrites `cancelled`; on cancel it kills the harness and returns. The surviving window is one read-write pair. |
| W1 | The ingest endpoint accepted events from any local process | **Accepted, fixed.** `run_server` generates `RESEARCH_EVENT_TOKEN`; only the daemons it spawns inherit it; ingest without it is 401, with a mismatched job id 400. Both asserted. |
| W2 | `find_package_dir` would adopt a stale package carrying the same job id | **Accepted, fixed.** A candidate must have `created_at` no earlier than the job's `started_at`; older or unparseable candidates are logged and skipped. |

### Round 2 — PASS (0 CRITICAL, 3 WARNING, 4 SUGGESTION); cap reached, fixed and not re-vetted

| # | Finding | Disposition |
|---|---|---|
| W1 | `std::env::set_var` inside the running multi-thread runtime is unsound | **Accepted, fixed.** `main` is now a plain function: it parses the CLI, sets `RESEARCH_EVENT_SINK` and `RESEARCH_EVENT_TOKEN` for server mode, and only then builds the runtime and enters `run(cli)`. `run_server` mutates nothing and warns when the token is unset. |
| W2 | A failure opening `daemon.log` after the checkpoint was written left the job `pending` forever, unlike the spawn-failure branch | **Accepted, fixed.** Log open and spawn are one closure; any error in it records `failed` with the reason. |
| W3 | On an operator cancel, `save` rewrote the whole file from the daemon's snapshot | **Accepted, fixed.** The cancelled path writes the operator's record back with only the daemon-owned fields merged (pid, harness, harness pid, package, exit code). |
| S1 | `files.report` from the manifest could escape the package with an absolute or `..` path | Fixed: absolute, prefixed, or parent components are refused. |
| S2 | A package whose checkpoint became unreadable was reported as "no package" | Fixed: distinct message naming the package directory. |
| S3 | The test's broadcast collector stopped on `Lagged` | Fixed: `Lagged` continues, `Closed` breaks. |
| S4 | Token compared with `!=` and re-read from the environment per request | Fixed: read once per process, compared in constant time. Moving it into `AppState` would change the struct every test constructs; deferred. |

Round-2 fixes are covered by the compile check, the four test targets, and the installed-service probes recorded in verification.md, but were not seen by the judge.
