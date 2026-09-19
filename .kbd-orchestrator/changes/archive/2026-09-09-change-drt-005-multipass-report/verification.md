# Verification — change-drt-005-multipass-report

Repository: `prometheus-skill-pack`

## Acceptance criteria

- A section citing a claim id outside its assignment fails assembly, naming the section and the id.
- After the coherence editor runs, the cited-claim set is unchanged or strictly smaller, and every removal is logged in the plan decision log.
- A section describing an `inferred` claim as verified fails assembly, naming the claim and its actual label.
- Every section's word target is at or below the cap and the totals match the depth budget.
- Stage number 09 is unchanged; `driver-contract.sh` passes its full count.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run is recorded BLOCKED with the reason, never skipped silently.

```verify
bash skills/research/deep-research/tests/report-assembly.sh
/bin/bash skills/research/deep-research/tests/report-assembly.sh
bash skills/research/deep-research/tests/driver-contract.sh
/bin/bash skills/research/deep-research/tests/driver-contract.sh
npm run check:skills-index
npm run check:distribution
npm run validate:codex
```

**The invariant is the deliverable.** Splitting one call into four only helps if the assembler mechanically enforces the claim-set subset rule; prose review of the sections is not the gate.

## Evidence

Run locally 2026-09-09 from the repository root. No hosted CI.

| Gate | Result |
|---|---|
| `tests/report-assembly.sh` (bash 5) | **PASS** — 14 passed, 0 failed |
| `tests/report-assembly.sh` (`/bin/bash` 3.2, C-05) | **PASS** — 14 passed, 0 failed |
| `tests/driver-contract.sh` (`KBD_PRODUCER_MODEL` unset) | **PASS** — 128/0, **unchanged** after the stage 09 edit |
| `tests/merge-threads.sh` | **PASS** — 17/0 (drt-004, unchanged) |
| `npm run check:skills-index` | **PASS** |
| `npm run check:distribution` | **PASS** after regenerating |
| `npm run validate:codex` | **PASS** |

All four new artifacts are in the distribution: `assemble-report.sh`,
`outline-architect.md`, `section-writer.md`, `coherence-editor.md`.

### Negative controls

| Break | Result |
|---|---|
| Disable the per-section drift check | "claim-set drift is caught" **FAILED** (exit 0 instead of 2) |
| Disable the editor-add check | "the editor ADDING a claim is caught" **FAILED** |

### Two flaws the tests caught in my own work

**The editor-add test was initially vacuous.** My first probe added a claim id
that another section already cited, so the cited set did not grow and the check
correctly passed while testing nothing. The suite now adds a claim **no section
cited in pass 1**, and asserts the pass-1 count is 3 first, so the setup itself
is verified. Same class as the drt-004 and parking-lot vacuous passes.

**The label check over-reached, and the mirror test found it.** The first
implementation read a fixed 240-character window before a marker, so "Research
confirms X [verified]. The quota may be raised [inferred]." failed on the
*inferred* marker for a word belonging to the previous sentence. The check now
scopes to the marker's own sentence. The suite asserts **both** directions —
strong wording over an `inferred` claim fails, the same wording over a
`verified` claim passes — because a one-directional test would have been
satisfied by a check that simply banned a vocabulary.

### Single numbering authority

The outline carries claim ids only; `merge-threads.sh` remains the sole assigner
of citation numbers via `citation-map.json`, and the assembler resolves ids to
numbers at assembly time. Asserted directly: the suite reads the number
`citation-map.json` gave `src-aaa` and requires that same number in `draft.md`.

### Scope

`--scale direct` never enters the multi-pass branch, so `report-synthesizer.md`
is retained and unchanged as the fallback. Stage number 09 is unchanged; only
its internals are split. driver-contract at 128/0 with no validator edited is
the evidence.

### Verdict

**PASS.** Sections may cite only their assigned claims; a missing reference,
label misuse, or claim-set drift each fail the build naming the offender; the
coherence editor may remove but never introduce, and an unlogged removal blocks
the run; no section may exceed 3000 words; and all C-01 gates pass.
