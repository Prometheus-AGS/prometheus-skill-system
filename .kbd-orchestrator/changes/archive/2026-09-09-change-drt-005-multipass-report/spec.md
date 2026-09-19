# change-drt-005-multipass-report

**Title:** Split stage 09 into outline, parallel sections, assembly, and edit, bound by a claim-set invariant
**Repository:** `prometheus-skill-pack`
**Phase:** deep-research-onyx-parity
**Goal:** G3
**Depends on:** `change-drt-001-dispatch-smoke-and-thread-contracts`, `change-drt-004-deterministic-merge`
**Backend:** native-kbd

## Why

Analyze **D-11**: this is a genuine build, evaluated rather than defaulted. Onyx
offers nothing to adopt — its final report is one call capped at
`MAX_FINAL_REPORT_TOKENS = 20000` (`dr_loop.py:80`) over a history that
`construct_message_history` silently truncates, so early findings can drop out of
the synthesis. That is precisely the failure this change prevents. The pack has
no report-assembly skill to reuse either.

Long reports degrade because one context must hold all evidence *and* generate
all prose. Splitting the two removes the ceiling. The invariant that keeps
accuracy from drifting with length is specific to this pack: section writers may
cite only the claim ids their section spec assigns, and the assembler diffs the
cited set against the assigned set.

## What Changes

- `agents/outline-architect.md` (Read, Write): reads `plan.md`, the graph claim
  index, and dossier headers; emits `report/outline.json` mapping sections to
  sub-questions to claim ids, with a word budget per section. **Citation numbers
  are not assigned here.** drt-004's merge is the single numbering authority and
  writes `citation-map.json`; the outline carries claim ids only, and the
  assembler resolves each to its global number at assembly time. Two numbering
  authorities would drift (adversarial round-1 finding 7).
- `agents/section-writer.md` (Read, Write to its own file only): receives its
  section spec plus **only** the claims and dossier excerpts it references.
- `scripts/assemble-report.sh`: concatenates sections into `report/draft.md` and
  fails on an orphan citation marker, a missing reference, or a claim cited
  outside its label.
- `agents/coherence-editor.md` (Read, Edit — no new claims): executive summary,
  transitions, cross-section dedupe, consistent terminology.
- The assembler re-runs after the editor and requires the cited-claim set to be
  unchanged or strictly smaller, with any removed id logged in the `plan.md`
  decision log.
- `report-synthesizer.md` is retained as the `direct`-scale fallback.
- `tests/report-assembly.sh`: orphan marker, claim-set drift, label misuse.

## Scope

- `skills/research/deep-research/agents/outline-architect.md`
- `skills/research/deep-research/agents/section-writer.md`
- `skills/research/deep-research/agents/coherence-editor.md`
- `skills/research/deep-research/agents/report-synthesizer.md`
- `skills/research/deep-research/scripts/assemble-report.sh`
- `skills/research/deep-research/scripts/run-research.sh`
- `skills/research/deep-research/references/schemas/report-outline.schema.json`
- `skills/research/deep-research/templates/report-outline.json`
- `skills/research/deep-research/tests/report-assembly.sh`
- `skills/research/deep-research/tests/fixtures/report-passes/`
- `skills/research/deep-research/skills/stage-09-report/SKILL.md`
- `SKILLS.md` (regenerated skills index, C-01)

## Capabilities

- `research-pipeline-execution (multi-pass report added)`

## ADDED Requirements

### Requirement: A section may cite only what it was assigned
The set of claim ids cited by a section SHALL be a subset of the ids its outline entry assigns.

#### Scenario: Drift is caught
- **WHEN** a fixture section cites a claim id outside its assignment
- **THEN** `assemble-report.sh` exits non-zero naming the section and the id

### Requirement: The editor may not introduce claims
WHEN the coherence editor has run, THEN the cited-claim set SHALL be unchanged or strictly smaller, and any removal SHALL be logged.

#### Scenario: Editing is bounded
- **WHEN** the editor rewrites prose and drops one section
- **THEN** the re-run assembler passes and the removed claim ids appear in the `plan.md` decision log

### Requirement: Labels survive sectioning
Every citation marker SHALL resolve through `citations.json` to a graph claim and its label, so the rule that `verified` describes only verified claims is enforced **per section**.

#### Scenario: Label misuse
- **WHEN** a fixture section describes an `inferred` claim as verified
- **THEN** the assembler fails naming the claim and its actual label

### Requirement: No single call is asked for the whole report
The word budget SHALL be distributed by the outline so that no section exceeds roughly 3000 words.

#### Scenario: Budget distribution
- **WHEN** an exhaustive-depth outline is produced
- **THEN** every section's target is at or below the cap and the totals match the depth budget

## Constraints

- Implementation-first, integration-only evidence: no unit tests, mocks, or snapshots count as delivery evidence.
- One Cargo build machine-wide at a time.
- Local-only validation: no GitHub Actions run is evidence.
- Verification labels are `verified | unverified | blocked | inferred`; provenance is `PASS | PASS WITH NOTES | BLOCKED`. A gate that could not run is recorded BLOCKED with the reason.
- Scripts that launchd may invoke stay bash 3.2 compatible (C-05); test under `/bin/bash`.
- Every script keeps `set -euo pipefail` and non-zero exit on failure; no silent `|| true` on a gate.
- The pack never depends on the Companion or any extension.
- **Stage number 09 does not change**; this splits its internals only.
- **Onyx is NOASSERTION licensed**: reimplement mechanisms, never copy prompt strings verbatim.
- C-01: editing a `SKILL.md` requires regenerating the skills index and passing `npm run check:skills-index` in the same change.
- **C-01, `run-research.sh` is a distributed artifact.** This change edits it, which desyncs `dist/plugins/claude/prometheus-skill-pack/skills/deep-research/scripts/run-research.sh`. Regenerate the distribution and pass `npm run check:distribution` and `npm run validate:codex` **in this change** — there is no end-of-phase reconciliation, because a desynced driver is defect D-B itself.
- **C-05 binds here.** `run-research.sh` is the script the launchd daemon drives: no `mapfile`, no `declare -A`, and the driver suite runs under `/bin/bash` 3.2 as well as bash 5.

## Open Questions

- Whether one section per sub-question is the right default, or whether the outline should be free to merge and split (default: one per sub-question first, since it makes claim assignment mechanical).
