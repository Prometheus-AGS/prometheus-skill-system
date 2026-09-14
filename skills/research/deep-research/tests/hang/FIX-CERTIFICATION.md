# FIX-CERTIFICATION — stage-10 hang

Change: change-drt-009-stage10-hang-fix-and-certification · Date: 2026-09-13
Campaign: skills/research/deep-research/tests/hang/captures-cert/campaign-20260913T170136Z (rate.txt and summary.tsv staged with this change; full table inlined below)

branch: none — no fix warranted by the reviewed diagnosis (DIAGNOSIS.md, accepted with unresolved findings at the 2-round cap; no confirmed hang site ⇒ extraction void; no diagnosis naming any file ⇒ elsewhere/escalated void). Instrumentation delivered instead: `--xtrace` diagnostic mode on tests/hang/run-hang-capture.sh (assertion counts unreliable under xtrace — fixture captures child stderr into assertion files; recorded in its usage note).

## N derivation

- Evidence base: 0 hangs in 38 runs (change-drt-008: 12 + 24 + 1 probe, plus 1 post-review smoke).
- p: **inferred** (bound-based, not a measured rate) — exact Clopper-Pearson 95% upper bound p_ub = 1 − 0.05^(1/38) = 0.0758.
- N = ceil( ln(0.01) / ln(1−p_ub) ) = ceil(4.60517 / 0.078835) = 59 → **N = 60** for margin (α = 0.01 per the phase's D1 convention).
- Total post-decision trials: 38 + 60 = **98**, zero hangs.

## Certification campaign — N = 60 consecutive clean full-run passes

Every run preceded by the harness's scoped pre-run cleanup (recorded-PID tree teardown + fresh logs); recorded 1–15-min load averages 211–493 across the campaign (the streak held under load an order of magnitude above normal — load-correlation as a hang cause is now severely weakened).

| run | duration | exit | assertions | result |
|---|---|---|---|---|
| run 01 | 21s | 0 | 12 | clean |
| run 02 | 18s | 0 | 12 | clean |
| run 03 | 13s | 0 | 12 | clean |
| run 04 | 20s | 0 | 12 | clean |
| run 05 | 12s | 0 | 12 | clean |
| run 06 | 12s | 0 | 12 | clean |
| run 07 | 20s | 0 | 12 | clean |
| run 08 | 16s | 0 | 12 | clean |
| run 09 | 25s | 0 | 12 | clean |
| run 10 | 19s | 0 | 12 | clean |
| run 11 | 13s | 0 | 12 | clean |
| run 12 | 29s | 0 | 12 | clean |
| run 13 | 16s | 0 | 12 | clean |
| run 14 | 27s | 0 | 12 | clean |
| run 15 | 20s | 0 | 12 | clean |
| run 16 | 16s | 0 | 12 | clean |
| run 17 | 20s | 0 | 12 | clean |
| run 18 | 15s | 0 | 12 | clean |
| run 19 | 13s | 0 | 12 | clean |
| run 20 | 17s | 0 | 12 | clean |
| run 21 | 32s | 0 | 12 | clean |
| run 22 | 26s | 0 | 12 | clean |
| run 23 | 30s | 0 | 12 | clean |
| run 24 | 30s | 0 | 12 | clean |
| run 25 | 39s | 0 | 12 | clean |
| run 26 | 89s | 0 | 12 | clean |
| run 27 | 40s | 0 | 12 | clean |
| run 28 | 39s | 0 | 12 | clean |
| run 29 | 73s | 0 | 12 | clean |
| run 30 | 74s | 0 | 12 | clean |
| run 31 | 61s | 0 | 12 | clean |
| run 32 | 35s | 0 | 12 | clean |
| run 33 | 33s | 0 | 12 | clean |
| run 34 | 29s | 0 | 12 | clean |
| run 35 | 26s | 0 | 12 | clean |
| run 36 | 28s | 0 | 12 | clean |
| run 37 | 30s | 0 | 12 | clean |
| run 38 | 27s | 0 | 12 | clean |
| run 39 | 33s | 0 | 12 | clean |
| run 40 | 47s | 0 | 12 | clean |
| run 41 | 41s | 0 | 12 | clean |
| run 42 | 37s | 0 | 12 | clean |
| run 43 | 30s | 0 | 12 | clean |
| run 44 | 22s | 0 | 12 | clean |
| run 45 | 23s | 0 | 12 | clean |
| run 46 | 24s | 0 | 12 | clean |
| run 47 | 23s | 0 | 12 | clean |
| run 48 | 11s | 0 | 12 | clean |
| run 49 | 12s | 0 | 12 | clean |
| run 50 | 10s | 0 | 12 | clean |
| run 51 | 14s | 0 | 12 | clean |
| run 52 | 10s | 0 | 12 | clean |
| run 53 | 9s | 0 | 12 | clean |
| run 54 | 10s | 0 | 12 | clean |
| run 55 | 8s | 0 | 12 | clean |
| run 56 | 12s | 0 | 12 | clean |
| run 57 | 11s | 0 | 12 | clean |
| run 58 | 12s | 0 | 12 | clean |
| run 59 | 21s | 0 | 12 | clean |
| run 60 | 23s | 0 | 12 | clean |

**Streak: 60/60 clean. Zero hangs, zero failures, zero perturbed runs (no hang ever occurred).**

## Full driver-contract suite (final gate)

**128 passed, 0 failed** — exit 0, 2026-09-13, run locally after the campaign (machine load ~400 during the suite).

## Fidelity diffs

Not applicable — branch `none`: no extraction performed, no product file edited (`bash -n build-review-packet.sh` PASS confirms the file parses untouched; git shows no modification by this change).

## BLOCKED gates

None. Every gate ran: campaign (60 runs), suite (128/128), diagnosis review (2 rounds, distinct judge, accepted with unresolved findings recorded in DIAGNOSIS.md — the acceptance itself is the spec's terminal path, not a laundered BLOCK).

## Falsifier remains armed

Any hang recurrence by any means — captured (harness/`--xtrace`) or merely reported — voids this certification and reopens the investigation (DIAGNOSIS.md, Falsifier).
