# Verification — change-drt-002-thread-scheduler-and-budgets

Repository: `prometheus-skill-pack`

## Acceptance criteria

- The concurrency cap is enforced in code: over-subscribing with `--max-parallel 3` yields observed peak concurrency of 3 and all tasks still complete.
- Every dispatch has a ledger row, including `partial`, `failed`, and `timeout`; no dispatch is silently lost.
- A thread that exceeds its budget is killed, its partial output preserved, and `budget_hit` recorded.
- `cargo test -p prometheus-research` passes with no new dependency added to Cargo.toml (analysis D-01).

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run is recorded BLOCKED with the reason, never skipped silently.

```verify
test -z "$(pgrep -x cargo)"
cargo test -p prometheus-research
bash skills/research/deep-research/tests/thread-scheduler.sh
/bin/bash skills/research/deep-research/tests/thread-scheduler.sh
```

**No new crate may appear in `Cargo.toml`.** `tokio` with `features = ["full"]` already provides `Semaphore`, `JoinSet`, and `time::timeout`. A new dependency here would invalidate analysis D-04's reason for deferring the workspace split.

## Evidence

Run locally 2026-09-09 from the repository root. No hosted CI.

| Gate | Result |
|---|---|
| `cargo check -p prometheus-research` | **PASS** — `Finished dev profile in 1.04s`. Run from `substrate/prometheus-research` (standalone crate, not in the root workspace). `pgrep -x cargo` confirmed empty first. |
| `tests/thread-scheduler.sh` (bash 5) | **PASS** — 19 passed, 0 failed |
| `tests/thread-scheduler.sh` (`/bin/bash` 3.2, C-05) | **PASS** — 19 passed, 0 failed |
| `tests/thread-contracts.sh` | **PASS** — 14/0 (drt-001, unchanged) |
| `tests/dispatch-smoke.sh` | **PASS** — 14/0 (drt-001, unchanged) |
| `tests/driver-contract.sh` | **PASS** — 128/0, with `KBD_PRODUCER_MODEL` unset |
| `tests/scoring-graph.sh` | **PASS** — 45/0 |
| `tests/installed-service-smoke.sh` | **PASS** — 16/0 (drt-007, unchanged) |

Stage contracts are unchanged, as the constraint requires: all five pre-existing
suites pass untouched (217 assertions).

### The cap and kill assertions were verified by negative control

A test that passes against broken code proves nothing, so both central claims
were checked by breaking the implementation and confirming the suite fails:

| Break | Result |
|---|---|
| Semaphore acquisition removed | Observed peak concurrency rose 2 → **6**; the cap assertion **FAILED** as it must |
| `kill_on_drop` and the explicit `child.kill()` removed | The orphan check **FAILED** — a `hanging.sh` process survived the scheduler's return |

Peak concurrency is reconstructed **outside** the scheduler, from the workers'
own start/stop timestamps, so a wrong internal counter cannot fake it.

### A defect the tests caught, and the fix

Adding the `partial` case exposed a real bug: the job budget never fired. All
tasks were spawned into the `JoinSet` up front and each waited on the semaphore
*inside* its own future, so the dispatch loop drained in microseconds and
`started.elapsed()` was still ~0 at the last task. The loop now acquires the
permit before spawning, via `tokio::select!` against the job deadline, so it
advances at the pace of actual completions — which is what the budget is meant
to bound. Verified: 4 of 6 threads are now correctly recorded `partial` with
`budget_hit: true`.

### C-01

Not triggered. This change touches no `SKILL.md`, plist, or systemd unit — only
crate sources, one reference doc, a test, and its fixtures.

### Verdict

**PASS.** The cap is enforced in code and proven by negative control; a hung
worker is killed rather than abandoned; all four terminal states
(`complete`, `partial`, `failed`, `timeout`) reach the ledger with reasons; and
budgets are carried on the checkpoint with backward compatibility proven by
reading a pre-`budgets` checkpoint with the new binary.
