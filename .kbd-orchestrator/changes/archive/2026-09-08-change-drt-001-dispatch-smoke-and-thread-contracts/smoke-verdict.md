# Dispatch smoke verdict — change-drt-001 task 2

**Date:** 2026-09-08 · **Run locally**, no hosted CI.

## Result: concurrent dispatch is AVAILABLE via the process path (strategy B)

| Run | Command | Result |
|---|---|---|
| Fixture workers, bash 5 | `bash tests/dispatch-smoke.sh` | **14 passed, 0 failed** |
| Fixture + real harness children, bash 3.2 | `/bin/bash tests/dispatch-smoke.sh --with-harness` | **20 passed, 0 failed** |

(Counts rose from 9 and 11 after the adversarial review: scratch-state isolation
with a negative control, and strict harness exit-code assertions replacing
suppressed failures. See the disposition in `spec.md`.)

What was proven:

- Two workers dispatched together **overlap in wall-clock time**. Total elapsed
  was 2s against a 4s serial floor, so the executions genuinely interleaved
  rather than queueing.
- Each worker received **only its own brief**: neither saw the other's task text,
  and no sibling context leaked through the environment.
- **Two real `claude` children** ran concurrently, spawned with the same flags
  `daemon.rs:249-262` uses. This is the binary the daemon actually spawns, not a
  stand-in.
- The suite passes under `/bin/bash` 3.2 as well as bash 5 (C-05).

## What this does NOT prove — and why the distinction matters

**Strategy A (in-session subagents) remains unproven.** A bash script cannot
dispatch a harness subagent; only the harness itself can. This script therefore
covers the process path only, and says so in its own output rather than implying
coverage it lacks.

The spec-stage review already forced this distinction once: a `tokio::Semaphore`
in the scheduler process cannot bound subagent dispatches inside a harness
session, because they are different processes. That is why drt-002 scopes its
code-enforced cap to strategy B. This run confirms strategy B is real; it says
nothing about A.

## Effect on the plan

**D-03a's fallback does not fire.** The condition for it was "concurrent dispatch
cannot be demonstrated", and it has been demonstrated on the path the scheduler
will actually drive. The planned round order stands:

- `harness-subagents` stays **adopt-provisional** rather than dropping to reject.
  It is not confirmed either — it is simply not what this change tested, and
  drt-003 does not depend on it, since the director can dispatch process workers.
- drt-007 stays **parallel in R1**, not blocking. But note the real dependency it
  carries: under the *installed launchd service* the daemon cannot resolve any
  harness (defect D-A), so process workers work from a shell today and will not
  work as a service until drt-007 lands. This run used a shell PATH, not launchd.

## Isolation semantics, stated once

Isolation here means **context isolation, not filesystem isolation**. All threads
share a package directory by design — the merge reads all of them — so "can a
worker open a sibling's file" is the wrong question; it always can. The question
that matters is whether a worker is *given* anything beyond its own brief, and
the assertions test exactly that.
