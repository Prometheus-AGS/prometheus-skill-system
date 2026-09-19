# change-rah-007-eval-ground-truth-review

**Title:** Human-review the 24 learn-grade eval ground-truth items and record the pre-change baseline
**Repository:** `prometheus-skill-pack`
**Phase:** research-agent-hardening
**Goal:** G2
**Depends on:** none
**Backend:** native-kbd

## Why

The learn-grade eval dataset's `metrics-summary.json` records `ground_truth_review_status: {draft: 24, reviewed: 0}` and labels its F1 0.96 provisional. Change-rah-008 alters the corpus schema the eval depends on; a re-baseline against unreviewed truth would measure the grader against unchecked labels (analysis D-15, adversarial review W1).

## What Changes

- Operator reviews each of the 24 ground-truth items in `references/eval-dataset/` and records `review_status: reviewed` with reviewer and date per item in `index.json`; disagreements are corrected in place with a note.
- Re-run `compute-eval-metrics.py` and `grader-regression-test.sh` against the reviewed truth and write `baseline-snapshot.json` as the pre-change baseline; `metrics-summary.json` reports `reviewed: 24`.
- This change contains an operator task; the executor prepares the review sheet and waits. It is not complete until the operator has recorded every item.

## Scope

Files this change may create, edit, or delete (tasks.json `files` is the per-task view):

- `skills/learn/learn-grade/references/eval-dataset/index.json`
- `skills/learn/learn-grade/references/eval-dataset/metrics-summary.json`
- `skills/learn/learn-grade/references/eval-dataset/baseline-snapshot.json`
- `skills/learn/learn-grade/references/eval-dataset/EVAL-RESULTS.md`
- `skills/learn/learn-grade/references/eval-dataset/REVIEW-SHEET.md`

## Capabilities

- `learn-model-coherence (eval baseline requirement, authored in change-rah-008)`

## ADDED Requirements

### Requirement: Ground truth is reviewed before it is used
No eval baseline SHALL be recorded as reviewed unless every item carries `review_status: reviewed` with a reviewer and date.

#### Scenario: Count
- **WHEN** `index.json` is read
- **THEN** 24 of 24 items are `reviewed` and `metrics-summary.json` reports `reviewed: 24, draft: 0`

### Requirement: The pre-change baseline is captured
`baseline-snapshot.json` SHALL be regenerated from the reviewed truth before change-rah-008 lands.

#### Scenario: Snapshot
- **WHEN** the metrics script runs against the reviewed set
- **THEN** the snapshot's timestamp is later than every review date and EVAL-RESULTS.md cites it

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

- Who reviews: the operator alone, or the operator plus one independent reviewer per corpus (default: operator; independent review is a later phase).
