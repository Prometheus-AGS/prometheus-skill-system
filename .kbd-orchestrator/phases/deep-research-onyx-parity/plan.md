# Plan — deep-research-onyx-parity

**Phase:** `deep-research-onyx-parity`
**Date:** 2026-09-08
**Backend:** native-kbd (OpenSpec is present and its capability specs are authored *inside* changes, per the `research-agent-hardening` precedent; changes themselves live in `.kbd-orchestrator/changes/`)
**Changes:** 7 · **Tasks:** 38
**Evolver bridge:** none — this phase is not part of an evolution cycle

---

## Ordering constraints that actually bind

Three real constraints shape the order. None is stylistic.

| Constraint | Which changes | Consequence |
|---|---|---|
| **One Cargo build machine-wide** (CLAUDE.md) | drt-002 and drt-007 both edit `substrate/prometheus-research` and both run `cargo` in their verify blocks | These two **must never be applied concurrently**. They are placed in different rounds |
| **Three changes edit `run-research.sh`** | drt-003, drt-004, drt-005 | Sequential application, or the later edit silently reverts the earlier. drt-004 lands first because it owns the merge that drt-003 wires and drt-005 resolves against |
| **drt-001 is a decision point, not setup** | drt-001 | Its outcome can re-plan the phase (see the branch below). Nothing that assumes in-session parallelism may precede it |

## Round order

| Round | Changes | Why together |
|---|---|---|
| **R1** | `drt-001`, `drt-007` | drt-001 must run first because its result may re-plan everything after it. drt-007 is fully independent (no shared file, no shared dependency) and unblocks process workers, so it runs alongside rather than idling. Both are Cargo-safe together: drt-001 touches no Rust, drt-007 is the only Cargo user in this round |
| **R2** | `drt-002` | Cargo-gated and alone in its round, so it can never contend with drt-007's build. Depends on drt-001 |
| **R3** | `drt-004` | Depends on drt-001's frozen thread artifact contracts (it consumes exactly those shapes). First of the three `run-research.sh` editors: owns the merge that drt-003 wires and drt-005's citation numbering resolves against |
| **R4** | `drt-003` | Wires stage 02 to *both* scheduler and merge, so it needs drt-002 and drt-004 complete |
| **R5** | `drt-005` | Last `run-research.sh` editor; consumes drt-004's `citation-map.json` |
| **R6** | `drt-006` | Benchmarks the finished threaded pipeline. Depends on 002, 003, 004, 005 — scoring anything earlier would measure the pipeline this phase replaces |

Rounds are sequential; only R1 has two changes, and they are provably independent.

## The changes

### R1 · change-drt-001-dispatch-smoke-and-thread-contracts — **5 tasks**
**Goals:** G1, G2 · **Library:** `harness-subagents` (adopt-provisional) · **Agent:** general-purpose

Proves concurrent in-session dispatch, then freezes the thread artifact contracts.
**This change can re-plan the phase** — see the branch below.

### R1 · change-drt-007-install-surface-repair — **5 tasks**
**Goals:** G1 only · **Library:** `process-workers` (adopt) · **Agent:** rust-build-resolver

Repairs the two defects the predecessor's evidence run found: the launchd plist
that sets no `PATH` (which blocks process workers outright) and the installed
generation shipping a stub driver. Independent of every other change.

### R2 · change-drt-002-thread-scheduler-and-budgets — **5 tasks**
**Goals:** G1, **and all of G4** · **Libraries:** `tokio-sync` (adopt), `onyx-budgets` (adopt-values-only), `onyx-parallel-cap` (adopt-and-improve) · **Agent:** rust-reviewer

The scheduler, using primitives already compiled into the binary. **No new
dependency may appear in `Cargo.toml`** — that is the whole reason the workspace
split is deferred. Enforces the concurrency cap in code for process workers,
where Onyx enforces it only in prompt prose.

**This change carries G4 by itself**, both halves: the budgets with
force-complete paths at job, director, and thread level (task 3), and the rule
that every dispatch gets a ledger row in `threads/index.json` — including
`partial`, `failed`, and `timeout` — so no dispatch is silently lost (task 4).
An earlier draft of this plan also credited drt-007 with G4; that was wrong, and
the adversarial review caught it. drt-007 repairs an install surface and contains
no budget, no force-complete path, and no ledger.

### R3 · change-drt-004-deterministic-merge — **6 tasks**
**Goals:** G1, G2 · **Agent:** general-purpose

Folds threads back into the *existing* stage 02/03/04 artifact shapes. This is
what makes threading a refactor rather than a rewrite: if the driver's validators
need changing to accept merged output, the merge is wrong, not the validators.
Also the enforcement point for the director's no-search rule.

### R4 · change-drt-003-director-worker-agents — **6 tasks**
**Goals:** G1, G2 · **Agent:** general-purpose

A director with no search tools and workers that see only their brief. Wires
stage 02 to scheduler **and** merge together — never one without the other.

### R5 · change-drt-005-multipass-report — **7 tasks**
**Goals:** G3 · **Agent:** general-purpose

Splits stage 09 into outline, parallel sections, assembly, and edit, bound by a
claim-set invariant the assembler checks mechanically.

### R6 · change-drt-006-bench-and-metrics — **4 tasks**
**Goals:** G5 · **Library:** `deep-research-bench` (adopt, Apache-2.0, pinned `469cce54`) · **Agent:** general-purpose

Makes parity falsifiable. Adopts the published RACE criteria so the number is
comparable to something; builds the FACT metrics and the verified-claim ratio,
which is the metric Onyx structurally cannot report.

## The branch: what happens if drt-001 fails

drt-001 task 2 is a genuine decision point, and the plan must survive either
outcome rather than assuming success.

**If concurrent in-session dispatch is demonstrated** (expected): the order above
stands unchanged.

**If it is not** (analysis D-03a's fallback):
- Strategy A is struck. `harness-subagents` drops from adopt-provisional to
  **reject**, recorded in `library-candidates.json` with the smoke output as
  evidence.
- drt-007 becomes **blocking rather than parallel**: without the `PATH` fix no
  process worker can spawn under the installed service, so it must complete
  before drt-002 is useful.
- drt-003's director becomes a process-spawning role rather than an in-session
  one; its spec's "Which runtime" table already anticipates this, so the change
  is re-scoped, not rewritten.
- Round order becomes R1 `drt-001` → R2 `drt-007` → R3 `drt-002` → unchanged
  thereafter.

This branch is recorded now so taking it is a planned move, not an improvisation.

## Goal coverage

| Goal | Changes | Verdict path |
|---|---|---|
| G1 Thread execution | drt-001, drt-002, drt-003, drt-004, drt-007 | adopt (tokio, process-workers) + build (merge, agents) |
| G2 Lossless handoff | drt-001, drt-003, drt-004 | build, on the pack's own claim ids |
| G3 Multi-pass report | drt-005 | build — no adoptable alternative exists |
| G4 Budgets and failure semantics | drt-002 **alone** | adopt-values-only (Onyx constants, source-verified) + build (force-complete, ledger) |
| G5 Measurement | drt-006 | adopt (RACE criteria) + build (FACT, verified-claim ratio) |

Every goal has at least one change, and every change serves at least one goal.

## What must not drift during execution

- **Stage numbers 02–04 and 09 never change.** Every change refactors how a stage
  is produced, never the contract. `driver-contract.sh` passing its full count is
  the standing proof.
- **No new crate in `Cargo.toml`** (drt-002). A new dependency invalidates the
  reason the workspace split was deferred.
- **Onyx is NOASSERTION licensed.** Mechanisms are reimplemented; prompt strings
  are not copied verbatim.
- **A bench run costs real gateway tokens.** The 10-task subset is this phase's
  ceiling and its results are indicative, not leaderboard-grade.
### C-01: `run-research.sh` is a **distributed artifact**, and four changes edit it

This is the constraint the plan review caught, and it is not confined to drt-007.
`dist/plugins/claude/prometheus-skill-pack/skills/deep-research/scripts/run-research.sh`
is generated output. drt-007 re-synchronises the installed generation — and then
**drt-003, drt-004, and drt-005 each edit the source again**, desyncing it every
time.

**Rule for this phase:** every change that edits `run-research.sh` or any skill
payload regenerates the distribution and passes `npm run check:distribution` and
`npm run validate:codex` **in its own change**. There is no end-of-phase
reconciliation change, because a desynced driver is exactly defect D-B, which
this phase exists partly to repair. Leaving it desynced across four changes would
recreate the bug while fixing it.

- drt-007 additionally proves the generator **byte-identical across two runs**,
  since re-synchronising the installed generation is its deliverable (rah-011's
  discipline).
- The *plist* half of D-A is **not** a services-manifest event; that claim was
  corrected as analysis D-11c.

### C-05: bash 3.2 applies to every `run-research.sh` editor

`run-research.sh` is the script the launchd daemon drives, so drt-003, drt-004,
and drt-005 are bound by C-05 exactly as drt-007 is: no `mapfile`, no
`declare -A`, and every suite that exercises the driver runs under `/bin/bash`
as well as bash 5. Their verify blocks already carry the `/bin/bash` run; this
states why it is mandatory rather than stylistic. `install-binaries.sh` is
additionally bound by the rule that a submodule build must never abort the
installer.

### C-03: does drt-007 trigger the Codex docs constraint?

**No, and this was determined rather than assumed** (analysis D-12). C-03 governs
the Codex *plugin surface* — `docs/codex-plugin.md` and the CLAUDE.md Codex
section — and binds when a plugin manifest, MCP declaration, hook registration,
or installer *contract* changes. drt-007 changes an installer *implementation*
(a `PATH` substitution) and republishes existing content; the plugin surface is
unchanged. **If the repair widens** to a version bump or a manifest edit, C-03
binds and both documents must be updated in the same change.

## First command

```
/kbd-apply change-drt-001-dispatch-smoke-and-thread-contracts
```

---

## Unresolved review findings

Adversarial review of the plan: k3 judge, producer `claude-opus-5`, artifact
mode, two rounds (cap). Receipts under `review/plan/`. Round 1 PASS, round 2
BLOCK; both passed the findings sycophancy gate at 0.0 strict. **Six findings,
all accepted, none rejected.**

### Round 1 (PASS: 2 WARNING, 1 SUGGESTION)

| # | Finding | Disposition |
|---|---|---|
| 1 | drt-007 was credited with G4, but contains no budget, force-complete path, or dispatch ledger | **Accepted — the mapping was wrong.** Verified: `grep -ciE "budget|force-complete|ledger"` on drt-007's spec returns 0, while drt-002 returns 11. Corrected in **both** the plan and drt-007's own spec; G4 is now carried by drt-002 alone, with both halves named |
| 2 | The drift section omitted C-01 and C-05 for drt-007, which both handoffs had carried forward | **Accepted.** Added, then extended further in round 2 |
| 3 | The R3 row cited only file-ordering, not drt-004's dependency on drt-001's frozen contracts | **Accepted.** The round table now renders the full dependency edge |

### Round 2 (BLOCK: 1 CRITICAL, 2 WARNING)

| # | Finding | Disposition |
|---|---|---|
| 1 | **C-01 was scoped to drt-007 alone, but `run-research.sh` is a distributed artifact and drt-003/004/005 each edit it after drt-007 re-syncs the generation** | **Accepted — the best finding of the stage.** Confirmed the driver exists under `dist/plugins/claude/...`. Leaving it desynced across four changes would recreate defect D-B while ostensibly fixing it. Every editor now regenerates in its own change: the rule is in the plan, in all three specs, in their verify blocks, and as a new task each (35 → 38 tasks). No end-of-phase reconciliation change, deliberately |
| 2 | C-05 was scoped to drt-007 although drt-003/004/005 edit the launchd-driven driver | **Accepted.** Extended to all three, with the reason stated rather than left as a style rule |
| 3 | The C-03 question deferred at assess was answered at analyze but never carried into the plan | **Accepted.** The determination is now in the plan: C-03 does not bind, because drt-007 changes an installer *implementation* and republishes existing content rather than altering the plugin *surface* — and it binds if the repair widens to a version bump or manifest edit |

Round 1 corrected a goal mapping; round 2 corrected a constraint that would have
let three changes silently undo the repair a fourth had just made. No finding was
rejected.
