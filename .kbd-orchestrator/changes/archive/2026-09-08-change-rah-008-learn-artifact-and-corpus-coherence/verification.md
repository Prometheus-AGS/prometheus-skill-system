# Verification — change-rah-008-learn-artifact-and-corpus-coherence

Repository: `prometheus-skill-pack`
Depends on: `change-rah-007-eval-ground-truth-review`, `change-rah-005-claim-labels-and-provenance`

## Acceptance criteria

- `learn-coherence.sh` passes: corpus shape, wrapper byte-parity, artifact round trip across the three skills' documented paths.
- `openspec validate` passes with the new `learn-model-coherence` capability.
- The eval regression test passes against the reviewed truth with the new corpus shape, and any metric change is recorded rather than tuned away.
- `npm run validate:strict` passes for the five learn skills touched.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run (for example, another Cargo build is active) is recorded BLOCKED with the reason, never skipped silently.

```verify
bash skills/learn/tests/learn-coherence.sh
openspec validate --specs
bash skills/learn/learn-grade/references/eval-dataset/grader-regression-test.sh
for s in feynman-loop learn-retain learn-certify learn-grade learn-goal learn-kb; do npm run validate:strict skills/learn/$s || exit 1; done
```

## Evidence

Run locally on 2026-09-07 (UTC) on branch `feat/cpc-001-002-integration-contract` at base `cfbc262` (working tree, uncommitted). No hosted CI. No cargo command was needed.

| Gate | Command | Result |
|---|---|---|
| Coherence integration gate | `/bin/bash skills/learn/tests/learn-coherence.sh` and `bash …` | `learn-coherence: 58 passed, 0 failed` under both bash 3.2 and bash 5 (final run after round-2 fixes; 35, 43, 53, and 58 assertions across the four revisions). Covers: corpus shape from the real shared script against the fixture KB; `--normalize` on the real eval corpus (12 sources, 5 misconceptions, 7 authored `key_points` kept, file untouched); wrapper byte-parity for learn-goal and learn-kb, under `/bin/bash` too; no-`lib/` slug parity; artifact round trip through `write-artifact.sh` and both documented read paths; goal_id/concept_id/artifact_id traversal refused; documented paths agree |
| OpenSpec | `openspec validate --specs` | 31 passed, 0 failed (`learn-model-coherence` now carries artifact path, corpus shape, one grounding script, provenance, learner-model write paths, and eval-baseline requirements) |
| Eval regression | `bash skills/learn/learn-grade/references/eval-dataset/grader-regression-test.sh` | `OK — 24 items match baseline, no regressions`. Deterministic compare of unchanged `results/` against the unchanged pre-change snapshot; it does not exercise the new corpus shape |
| Eval live re-run against reviewed truth | not run | **BLOCKED**: `index.json` is `draft: 24, reviewed: 0`; rah-007 task 2 (operator review) has not landed and D-15 forbids a baseline against draft labels. Recorded in EVAL-RESULTS.md "Post-change baseline — change-rah-008" and `metrics-summary.json.post_change_baseline` with the steps to finish it |
| Skill validation | `npm run validate:strict skills/learn/{feynman-loop,learn-retain,learn-certify,learn-grade,learn-goal,learn-kb}` | rc=0 each (final run after round-2 edits) |
| Normalize preserves per-source fields | probe corpus with per-source `concept_id`, `tags`, `custom` through `--normalize` | all three survive; rebuilt `key_points`/`misconceptions` win; input order preserved (3 gate assertions) |
| Judge transport | round-2 dispatch, default timeout | first attempt exit 3 ("unavailable") on a 137 KB packet while the gateway answered HTTP 200; re-dispatched with `ADV_JUDGE_TIMEOUT=900` and completed. Transport limit, not a review outcome |
| Task verify strings | `kbd-apply` per-task `verify` (1–5) | All OK; task 2's `< 20 lines` wrapper bound met (16 lines) |
| learn-practice write path (finding R1-8) | `grep -c add_session skills/learn/learn-practice/SKILL.md` | present (landed by change-rah-009, archived 2026-09-07) |
| Adversarial review | k3 judge, 2 rounds | Round 1 BLOCK (3C/5W/1S): 7 accepted and fixed, 2 rejected with probes. Round 2 BLOCK (1C/3W): 3 accepted and fixed after the cap (HARNESS.md corpus claim, normalize dropping per-source fields, learn-home override in learn-certify/learn-retain), 1 CRITICAL restates the already-recorded blocked criterion. Sycophancy gate PASS both rounds (0.018, 0.0). See spec.md "Unresolved review findings" |

## Verdict

**BLOCKED** on one acceptance criterion, everything else passing.

- The criterion "the eval regression test passes against the reviewed truth with the new corpus shape" cannot be met until rah-007 task 2 lands; running it against draft truth would violate D-15. The harness (`--normalize`), the corpus shape, the artifact path, the wrappers, the spec, and the integration gate are complete and verified.
- Carry-forward (owed to rah-007 task 3 and rah-011 certification): regenerate the pre-change `baseline-snapshot.json` from reviewed truth, re-run the 24 items with normalized corpora, run `compute-eval-metrics.py`, record movement in EVAL-RESULTS.md.
- Out of scope, noted: `skills/learn/learn-goal/scripts/content-grounding.sh` (the public-web variant) still emits neither `key_points[]` nor `misconceptions[]`; learn-grade Step 1's `--normalize` route covers its output until it is aligned.
