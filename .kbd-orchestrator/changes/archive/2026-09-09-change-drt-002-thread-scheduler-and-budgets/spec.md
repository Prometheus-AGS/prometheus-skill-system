# change-drt-002-thread-scheduler-and-budgets

**Title:** Bounded thread scheduler with a code-enforced concurrency cap and budgets at every level
**Repository:** `prometheus-skill-pack`
**Phase:** deep-research-onyx-parity
**Goal:** G1, G4
**Depends on:** `change-drt-001-dispatch-smoke-and-thread-contracts`
**Backend:** native-kbd

## Why

Analyze **D-01**: the crate already depends on `tokio` with `features = ["full"]`
(`Cargo.toml:25`), which supplies `sync::Semaphore`, `task::JoinSet`, and
`time::timeout`, none of them yet used. The scheduler needs **no new
dependency**, which is why the report's proposed workspace split is deferred
(D-04).

Analyze **D-02** is the reason this change matters beyond parity: Onyx states its
"never more than 3 in parallel" rule **four times in prompt prose**
(`orchestration_layer.py:88,116,181,197`) with **no code cap** — parallelism runs
through `run_functions_tuples_in_parallel` with no arity limit. A prompt cannot
enforce a bound. Holding the cap in the scheduler makes the pack strictly better
than the system it is learning from, at no extra cost.

Today `grep -nE "timeout|budget|max_cycles" run-research.sh` returns nothing: a
stalled stage has no bounded exit.

## Which runtime the scheduler drives (resolves adversarial round-1 finding 3)

**The Rust scheduler spawns harness CLI processes, one per thread.** It does not
and cannot cap in-session Agent-tool dispatches: those run inside the harness
session, in a different process from the scheduler, where a `tokio::Semaphore`
has no reach. The two dispatch strategies from analysis D-03 are therefore
**two distinct runtimes**, not two implementations of one:

| Strategy | Runtime | Who bounds concurrency |
|---|---|---|
| **B — process workers** (this change) | `threads.rs` spawns one harness CLI per thread via the existing `spawn_harness` path | The scheduler's `Semaphore`, in code, structurally |
| **A — in-session subagents** (drt-003, provisional) | The director dispatches subagents inside its own harness session | The director's own prompt, plus the merge gate in drt-004 — **not** this scheduler |

This change's "cap enforced in code" requirement therefore applies to **strategy
B only**, and says so in its scenario. Strategy A's bound is advisory at dispatch
time and enforced after the fact by the merge, exactly as drt-003 states for the
no-search rule. Claiming otherwise would be claiming an enforcement the
architecture cannot deliver.

drt-001's smoke test remains meaningful under both readings: it establishes
whether in-session parallel dispatch is available at all (D-03a), which decides
whether strategy A is on the table beside strategy B.

## What Changes

- `substrate/prometheus-research/src/threads.rs`: `Semaphore`-bounded scheduler
  over `JoinSet` with per-task `time::timeout` and kill. The cap is a constructor
  argument, never a prompt instruction.
- `prometheus-research threads run --package <dir> --max-parallel N` subcommand;
  returns after workers finish, leaving `threads/` populated.
- `budgets` object in `checkpoint.json`, with force-complete paths at job,
  director, and thread level. Starting values adopted from Onyx and verified at
  source (D-07): thread cycles 8, thread force-report 12 min, thread hard timeout
  30 min, job force-report 30 min; director cycles by depth (shallow 2, deep 4,
  exhaustive 8).
- Every dispatch gets a row in `threads/index.json` — including `partial`,
  `failed`, and `timeout` — so the ledger has no silent holes.
- `tests/thread-scheduler.sh`: fixture workers covering complete, partial,
  failed, and timeout, plus a **cap test** that dispatches more tasks than the cap
  and asserts concurrency never exceeds it.

## Scope

- `substrate/prometheus-research/src/threads.rs`
- `substrate/prometheus-research/src/lib.rs`
- `substrate/prometheus-research/src/main.rs`
- `substrate/prometheus-research/src/job/checkpoint.rs`
- `skills/research/deep-research/tests/thread-scheduler.sh`
- `skills/research/deep-research/tests/fixtures/thread-workers/`
- `skills/research/deep-research/references/stage-contracts.md`

## Capabilities

- `research-pipeline-execution (budgets and bounded concurrency added)`

## ADDED Requirements

### Requirement: The concurrency cap is enforced in code (process workers)
WHEN more thread tasks are dispatched **through the scheduler** than the configured cap, THEN the number of harness processes running concurrently SHALL never exceed the cap, regardless of what any prompt or model requests. This binds strategy B. Strategy A's bound is advisory at dispatch and enforced post hoc by drt-004's merge.

#### Scenario: Over-subscription
- **WHEN** six tasks are dispatched with `--max-parallel 3`
- **THEN** the observed peak concurrency is 3, all six complete, and the ledger records six rows

### Requirement: Every dispatch has a ledger row
WHEN a thread completes, is truncated by budget, fails, or times out, THEN `threads/index.json` SHALL carry a row naming its terminal state and reason.

#### Scenario: Failure is recorded, not lost
- **WHEN** a fixture worker exits non-zero and another exceeds its timeout
- **THEN** both appear in the ledger with `failed` and `timeout` respectively, and the run continues rather than aborting

### Requirement: Budgets bound every level
WHEN a thread or the job exceeds its budget, THEN it SHALL force-complete with whatever it has and record `budget_hit`, never hang.

#### Scenario: Force-complete
- **WHEN** a fixture worker sleeps past its thread budget
- **THEN** it is killed, its partial output is preserved, and `budget_hit` is true in the checkpoint

## Constraints

- Implementation-first, integration-only evidence (CLAUDE.md highest-precedence policy): finish the coherent edit batch, then run the smallest full-integration gate named in `verification.md`. No unit tests, mocks, or snapshots count as delivery evidence.
- One Cargo build machine-wide at a time. Check `pgrep -x cargo` before any `cargo` command; if another build is active, wait or record BLOCKED, never start a competing build.
- Local-only validation: no GitHub Actions run is evidence.
- Verification labels are `verified | unverified | blocked | inferred` on claims and `PASS | PASS WITH NOTES | BLOCKED` on provenance. A gate that could not run is recorded BLOCKED with the reason.
- Scripts that launchd may invoke stay bash 3.2 compatible (constraint C-05): no `mapfile`, no `declare -A`; test under `/bin/bash`.
- Every script touched keeps `set -euo pipefail` semantics and non-zero exit on failure; no silent `|| true` on a gate.
- The pack never depends on the Companion or any extension (integration contract rule 1).
- **Stage numbers 02-04 and 09 do not change.** This phase refactors how those stages are produced, never the contract. The driver validators, `check-research-package.sh`, the daemon mirroring, and every existing fixture must keep passing unchanged.
- **Onyx is licensed NOASSERTION** (analysis D-08). Mechanisms may be reimplemented freely; Onyx prompt strings must NOT be copied verbatim without a licence check.
- Constraint C-01 (generated artifacts): any change that edits a `SKILL.md`, a plist, or a systemd unit regenerates the affected surface and passes its drift validator in the same change.

## Open Questions

- Per-depth director cycle defaults are proposed (2/4/8) but not cost-validated; the first real run should report actual token spend before they are fixed.
- Dossier token cap: Onyx uses 10000 (`research_agent.py:93`) with an inline note that ~5000 performs better. Default: start at 6000 and record the choice.
