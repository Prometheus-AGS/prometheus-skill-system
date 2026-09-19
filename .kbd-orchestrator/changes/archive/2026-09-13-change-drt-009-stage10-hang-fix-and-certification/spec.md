# change-drt-009-stage10-hang-fix-and-certification

**Title:** Review-gated fix and N-run certification for the intermittent stage-10 hang
**Repository:** `prometheus-skill-pack`
**Phase:** deep-research-onyx-parity › stage-10-hang-investigation
**Goal:** G3 (fix so a full deep run completes, proven), G4 (adversarial review of the diagnosis before any fix)
**Depends on:** `change-drt-008-hang-capture-harness` — no task below may start before drt-008's HANG-CAPTURE.md exists
**Backend:** native-kbd

## Why

Goal 4 is a hard ordering gate: **no fix is written before the diagnosis
survives adversarial review.** And the phase's original proof standard —
"driver-contract passes its full count" — certifies nothing for an
intermittent defect: the suite is green 128/128 while the hang persists
(assessment, Goal 3).

Analyze decisions carried here:

- **D1 (ADOPT)** — the proof gate becomes **N consecutive clean
  `--scenario full-run` passes**, with N sized from the failure rate that
  drt-008 actually measures: `N = ceil(ln(0.01) / ln(1-p))`, floor 10 (at the
  observed ~1/3 rate, N=12 keeps P(false clean streak) under 1% **under an
  independence assumption** that drt-008's per-run cleanup exists to justify).
- **D3 (CONDITIONAL)** — the structural suspect is the 919-line
  `skills/process/adversarial-review/scripts/build-review-packet.sh` (the
  stage-10 chain's packet builder, invoked from `run-research.sh`
  `review_report`), which embeds **9 Python heredoc programs** (verified
  anchors :141 :259 :298 :363 :406 :459 :537 :655 :720) plus one non-Python
  heredoc (:568). Extraction — the repo's proven remedy — applies **only if**
  drt-008 isolates the hang there. Verifiable precedent, not host folklore:
  **nine `.sh`/`.py` companion pairs exist in
  `skills/research/deep-research/scripts/`** (analysis.md Appendix A; count
  re-verified this stage), and change-drt-006's verification records **five
  hanging scripts fixed** by that extraction. Two honesty constraints from
  this phase's own record: (1) "large heredocs hang on this host" was
  **disproved** in change-drt-006's verification (`export-package.sh` embeds a
  larger program and runs fine), so heredoc-ness alone is not the diagnosis;
  (2) the live mechanism hypothesis is a ~64 KB OS pipe-buffer deadlock, which
  drt-008's output-volume observation must confirm or refute.

## What Changes

- **Task 1 — diagnosis review (Goal 4), with machine evidence.** A diagnosis
  brief (`tests/hang/DIAGNOSIS.md`) distilled from HANG-CAPTURE.md goes
  through the adversarial-review pipeline: `build-review-packet.sh --mode
  decision --target <brief>`, findings written to
  `.kbd-orchestrator/phases/deep-research-onyx-parity/children/stage-10-hang-investigation/review/diagnosis/findings.json`,
  sycophancy screen with **explicit `--findings`** (counter-key
  `adv-review-…-diagnosis`). The gate is satisfied by **machine evidence, not
  prose**: findings.json must record a judge distinct from the producer
  (`cross_model_check == "verified-distinct"`, or `judge_model !=
  producer_model`), and the screen's PASS record must exist. If no distinct
  judge is reachable — the failure mode this phase has already recorded —
  task 1 records **BLOCKED with the reason and no fix is attempted**; the
  same-model harness-native fallback does **not** satisfy Goal 4. Max 2
  revise rounds, then accept with an "Unresolved review findings" section.
- **Task 2 — the fix the reviewed diagnosis warrants (conditional branch).**
  - **Extraction branch:** if the hang site is inside
    `build-review-packet.sh`'s embedded Python, extract the implicated
    programs to sibling `.py` companions following the repo's proven pattern
    (byte-identical bodies, `HERE`-relative resolution, explicit existence
    check, `exec` — or explicit exit-code forwarding where a shell-function
    callback must survive, the `detect-contradictions.sh` precedent). The
    `:568` non-Python heredoc moves only if implicated.
  - **Elsewhere branch:** implement the fix the reviewed diagnosis names
    **within this change's Scope**. If the diagnosis names a file outside
    Scope (e.g. `driver-contract.sh` itself), the fix is **escalated to a new
    spec'd change** at the plan stage rather than applied here — this change
    does not widen its own scope.
  - The branch taken is recorded in task 2 notes and on a `branch:
    extraction|elsewhere|escalated|none` line in FIX-CERTIFICATION.md.
- **Task 3 — certification (D1).** `tests/hang/FIX-CERTIFICATION.md` records:
  the `branch:` line; N with its derivation from drt-008's measured rate p;
  the consecutive-clean-run table **with one row per run, each row beginning
  `| run`** (machine-countable); each run preceded by the same scoped cleanup
  drt-008 uses; and one final full driver-contract suite run at its full
  count. A gate that cannot run (e.g. the harness never hung and p is only
  bounded) is recorded BLOCKED with the reason and N's basis labelled — never
  silently downgraded.

## Scope

- `skills/research/deep-research/tests/hang/DIAGNOSIS.md` (new)
- `skills/research/deep-research/tests/hang/FIX-CERTIFICATION.md` (new)
- `skills/process/adversarial-review/scripts/build-review-packet.sh` — edited
  **only** on the extraction branch; companion `.py` files in the same
  directory on that branch
- `skills/research/deep-research/tests/hang/run-hang-capture.sh` —
  **SCOPE AMENDMENT (recorded 2026-09-13, diff-review round 2)**:
  instrumentation edit delivering the `--xtrace` diagnostic mode the
  reviewed diagnosis Decision ADOPTED (buffering-defeat tripwire), plus the
  RC=124 hang-row recording fix from diff-review round 1 (exit column now
  numeric; result=HANG carries semantics). No behavior change to
  standard-mode clean-run paths (all 60 certification runs used standard
  mode). Recorded in the stage decision log (2026-09-13 entry).
- `skills/research/deep-research/scripts/run-research.sh` — only if the
  reviewed diagnosis names it (elsewhere branch, in-Scope)

`driver-contract.sh` is not edited by this change, on any branch: if the
diagnosis names it, that is the escalation path above. The dist copies under
`dist/plugins/` are **not** regenerated here — see Constraints.

## Capabilities

- `research-pipeline-execution (stage-10 stability)`

## ADDED Requirements

### Requirement: No fix precedes a reviewed diagnosis, proven by machine evidence
The fix task SHALL NOT begin until task 1 records an accepted adversarial-review verdict whose findings file shows a judge distinct from the producer and a sycophancy-screen PASS record; a same-model fallback or an unreachable judge leaves the gate BLOCKED, and no fix is attempted.

#### Scenario: Ordering held, machine-checked
- **WHEN** task 2 is marked done
- **THEN** `review/diagnosis/findings.json` shows `cross_model_check: verified-distinct` (or `judge_model != producer_model`), the screen's PASS record exists, and DIAGNOSIS.md cites both — all dated before the fix edit

#### Scenario: No distinct judge reachable
- **WHEN** dispatch cannot produce a distinct-judge review
- **THEN** task 1 is recorded BLOCKED with the reason, and tasks 2–3 do not start

### Requirement: An extraction is byte-faithful, if it happens
If the extraction branch runs, each extracted `.py` SHALL be diffed byte-identical to the heredoc body it replaced, and the wrapper SHALL forward exit codes (exec, or explicit forwarding where a callback must survive).

#### Scenario: Fidelity checked
- **WHEN** a program is extracted
- **THEN** a diff against the replaced heredoc body is recorded in FIX-CERTIFICATION.md and is empty

### Requirement: Certification is statistical, not a single green run
FIX-CERTIFICATION.md SHALL state N, the measured (or bounded) rate p it was sized from, a consecutive-clean-run table with one `| run` row per pass, and the final full-suite count.

#### Scenario: N-run gate satisfied, machine-countable
- **WHEN** the change is certified
- **THEN** the table carries at least N ≥ 10 `| run` rows, each run clean, followed by one full driver-contract suite pass

### Requirement: BLOCKED is recorded, never laundered
Any gate that could not run is recorded BLOCKED with its reason in FIX-CERTIFICATION.md; a verify string passing for the wrong reason is not evidence (the change-drt-006 task-4 precedent).

#### Scenario: Honest failure
- **WHEN** the campaign cannot complete (e.g. elevated machine load)
- **THEN** the affected gate reads BLOCKED with the reason rather than PASS

## Constraints

- Implementation-first, integration-only evidence; local-only validation; no hosted CI.
- Goal-4 ordering is a hard gate; a fix edit made before the machine-checked verdict is a defect of this change, not a shortcut within it.
- The judge for the diagnosis review SHALL be a model distinct from the producer; the harness-native same-model fallback does not satisfy Goal 4.
- bash 3.2 compatible; `set -euo pipefail`; verification labels `verified | unverified | blocked | inferred`.
- Consecutive certification runs use the same scoped pre-run cleanup as drt-008 (independence assumption behind the N-bound).
- No pre-emptive extraction "while we are in there": the branch is chosen by the reviewed diagnosis, nothing else.
- **C-01 (generated artifacts in sync):** this change (and drt-008) touch `skills/**` that `dist/plugins/` mirrors; the reconciliation owner is the **phase-close distribution-regeneration change to be specced at the kbd-plan stage of this child** (regenerate twice, hash-identical, `npm run check:distribution` PASS) — named here so the deferral is owned, per C-01's identification requirement.

## Open Questions

- The analyze-stage warning on `library-candidates.json` `build_required` semantics (entries duplicating adopted candidates with `capability_gap_id: null`, vs the schema's "gaps with no adoptable candidate" description) is **dispositioned here as intentional**: the duplication documents the build need behind each adopted method candidate; the schema permits the shape (null is valid), and no consumer reads `build_required` as adopt-gap-only. Recorded rather than edited, so the handoff chain stays auditable.
- The pre-existing `scoring-graph.sh` stall (closed-port `semantic-blocked` scenario) will block any "all suites green" claim; whether it joins this child's scope is a plan-stage decision, default no.
- `build-review-packet.sh` cannot assemble a `--target spec` packet for a **child** phase (CHANGES_ROOT derivation drops the `/children/<child>` component); discovered during this stage's own vet and bridged in-scope via a phase-local symlinked changes dir. A proper fix belongs in its own reviewed change — plan stage to slot it.
- If drt-008 bounds rather than measures p (no hang in K runs), N is sized from the bound and labelled `inferred` rather than `verified`.
