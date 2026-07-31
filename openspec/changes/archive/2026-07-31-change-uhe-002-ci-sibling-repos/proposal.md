# Give CI the sibling repos

**Change:** `change-uhe-002-ci-sibling-repos`
**Phase:** uar-host-execution
**Goal:** S4

## Why

See `.kbd-orchestrator/phases/uar-host-execution/plan.md` for full rationale,
acceptance criteria, and the two-round adversarial review record.

## Outcome: 2 of 4 verified — the stated-limit branch, not the 4-of-4 branch

**Task 2 as written ("4 of 4 verified, 0 SKIP") was NOT achieved, and is
recorded as not achieved.** Marking it complete would have been false. The
acceptance criteria anticipated this: task 3 is the branch taken.

### Why 2, measured rather than assumed

| Repo | Visibility | Reachable from CI |
|---|---|---|
| `Prometheus-AGS/flint-realtime-fabric` | **PUBLIC** | yes — checked out |
| `Prometheus-AGS/universal-agent-runtime` | **PUBLIC** | yes — checked out |
| `Know-Me-Tools/know-me-system` | **PRIVATE, different org** | **no** |

Reaching the third would mean putting a cross-org PAT in a public workflow to
compare two version strings. That is a poor trade, so **two of four is the
honest ceiling** and it is nowhere described as full coverage.

| Invariant | CI status |
|---|---|
| `loro-minor-aligned` | **PASS** |
| `iroh-floor-1.0.2` | **PASS** |
| `wasmtime-major-aligned` | **SKIP** — needs know-me-system |
| `wit-world-version-pinned` | **SKIP** — needs know-me-system |

### A real defect found while measuring

Simulating the CI environment surfaced a **pass earned by absence**: with
`know-me-system` missing, `wit-world-version-pinned` reported **PASS** — because
the repository holding the violation simply was not there to be seen. The
`knowme:plugin` split still existed; the checker just could not see it.

That is precisely the failure this checker exists to prevent, and it would have
been invisible: a green CI reporting an invariant "verified" that nothing had
checked. Fixed — an unreadable repo now yields **SKIP**, never PASS.

### Coverage is pinned, not just documented

`scripts/assert-ci-coverage.sh` fails if the split drifts in **either**
direction. SKIP never fails a build, so an invariant that quietly stops being
verified would otherwise go unnoticed. Mutation-tested: dropping `FRF_ROOT`
turns `loro-minor-aligned` into SKIP and the assertion exits 2 naming it.

A SKIP becoming PASS also fails — which is the prompt to update the expectation
rather than let coverage drift upward unrecorded.
