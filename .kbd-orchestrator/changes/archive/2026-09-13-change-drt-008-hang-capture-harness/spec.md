# change-drt-008-hang-capture-harness

**Title:** Isolate the intermittent stage-10 hang with an instrumented repeat-run capture harness
**Repository:** `prometheus-skill-pack`
**Phase:** deep-research-onyx-parity › stage-10-hang-investigation
**Goal:** G1 (exact blocking command with a reproducible trace), G2 (pre-existing vs introduced, first appearance)
**Depends on:** none — extends the existing `tests/driver-contract.sh` suite from outside (no edit to the driver)
**Backend:** native-kbd

## Why

The stage-10 review chain hangs intermittently: on 2026-09-09 the full
driver-contract suite passed 128/128 four times, yet a single
`--scenario full-run` invocation hung once (timeout 90 → exit 124, zero
assertions printed) and passed on immediate retry — roughly one hang in three
observations, on one uncommitted tree. Serial one-shot hypothesis testing
disproved six theories without finding the cause. A stall with zero assertions
means the driver invocation itself never returned; which child holds it is
knowable only from a **live capture**, not from further reading (assessment,
open question 1).

Analyze decision **D2 (ADOPT)**: an instrumented repeat-run harness is the
only candidate that can (a) isolate the exact blocked command and its stack
and (b) measure the failure rate that change-drt-009 needs to size its
N-consecutive-clean-runs certification (D1).

## What Changes

- New `skills/research/deep-research/tests/hang/run-hang-capture.sh`:
  loops `tests/driver-contract.sh --scenario full-run` for all `--runs K`
  (default 12). Termination semantics (as resolved at the plan stage): the
  campaign runs **all K runs by default** — p needs the denominator — and on a
  hang it captures the evidence and **continues**, marking post-capture runs
  as potentially perturbed; `--stop-on-hang` is the opt-in that stops at the
  first captured hang. Per run:
  - **Pre-run cleanup** (independence for D1's `(1-p)^N` bound): kill orphaned
    process trees from prior runs **by recorded PID only** (never a broad
    `pkill` by name), reset scratch/output dirs.
  - **Per-run timeout** (default 90 s — the external `timeout(1)` setting under
    which the hang was observed as exit 124; the suite itself imposes no
    per-scenario timeout, verified by grep on `driver-contract.sh`),
    timestamped log per run, and a run record: duration, exit code, assertion
    count, and the machine **load average** (a timeout under load ~45 is not
    evidence about code — recorded in change-drt-006's verification).
  - **On timeout, capture before teardown**: full `ps` process tree rooted at
    the driver invocation; `sample` (macOS) stacks of the stalled child
    processes; output-volume observation at the hang site (tests the ~64 KB
    OS pipe-buffer deadlock hypothesis). Then tear the tree down by PID.
- New `skills/research/deep-research/tests/hang/HANG-CAPTURE.md`: the
  evidence pack — per-run table, measured failure rate (hangs/K), and on the
  first captured hang: the exact blocked command, its stack, and the
  output-volume observation. Plus the goal-2 dating analysis (below).

## Scope

- `skills/research/deep-research/tests/hang/run-hang-capture.sh` (new)
- `skills/research/deep-research/tests/hang/HANG-CAPTURE.md` (new)
- Capture artifacts land under a per-campaign output dir and are not committed.

No file outside `tests/hang/` is created or edited; `driver-contract.sh`
itself is **not** modified.

## Capabilities

- `research-pipeline-execution (diagnosis harness added)`

## ADDED Requirements

### Requirement: A hang yields its blocked command from live capture
The harness SHALL capture the process tree and child stacks of a hanging run before tearing it down, and HANG-CAPTURE.md SHALL name the exact blocked command from that capture, not from inference.

#### Scenario: First hang captured
- **WHEN** a run times out
- **THEN** the ps tree and `sample` stacks are captured before teardown, and HANG-CAPTURE.md names the blocked command with its stack

### Requirement: Runs are independent
The harness SHALL remove orphaned processes from prior runs (by recorded PID) and reset scratch state before each run.

#### Scenario: No cross-run leakage
- **WHEN** the next run starts
- **THEN** no process from a prior run is alive and scratch state is fresh

### Requirement: The failure rate is measured, not assumed
HANG-CAPTURE.md SHALL state K (runs attempted), hangs observed, and hangs/K, alongside per-run load averages.

#### Scenario: Rate recorded
- **WHEN** the campaign ends (all K runs, or earlier with `--stop-on-hang`)
- **THEN** the measured rate and K appear with the per-run table

### Requirement: Dating is honest about the uncommitted tree
Any first-appearance claim SHALL cite git evidence; where the implicated code exists only in this phase's uncommitted work, HANG-CAPTURE.md SHALL say so rather than dating from committed history that cannot contain it.

#### Scenario: Boundary stated
- **WHEN** the hang site is dated
- **THEN** the claim cites its evidence and states the uncommitted-tree boundary explicitly

## Constraints

- Implementation-first, integration-only evidence: the harness run against the real driver IS the evidence; no mocks or stubs substitute for it.
- Local-only validation; no GitHub Actions run is evidence.
- bash 3.2 compatible, `set -euo pipefail`, non-zero exit on failure.
- Process teardown by recorded PID only; a broad name-based kill is a defect, not a shortcut.
- Verification labels `verified | unverified | blocked | inferred`; a gate that could not run is recorded BLOCKED with the reason.
- Wall-clock ceiling: K=12 with 90 s timeouts bounds a clean campaign near 20 minutes; the harness must remain a foreground, interruptible process.
- C-01 (generated artifacts in sync): this change adds files under `skills/research/deep-research/tests/hang/`; the dist regeneration that reconciles them is owed by the phase-close distribution change named in change-drt-009's Constraints — this change defers to that named owner rather than leaving the debt unowned.

## Open Questions

- Run-to-run independence after cleanup is plausible but unproven; residual coupling (shared caches, lock files) would inflate a clean-run streak. Recorded as a risk for drt-009's N-bound, not solved here.
- If no hang occurs in K runs, the rate is bounded (0 hangs in K) rather than measured at ~1/3; drt-009 must size N from the bound and label it as such.
- Machine load distorted timeouts before (load 45, change-drt-006); if load is high during the campaign, affected runs are recorded, not silently discarded.
