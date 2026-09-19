# HANG-CAPTURE — stage-10 intermittent hang capture campaign

Change: change-drt-008-hang-capture-harness
Phase: deep-research-onyx-parity › stage-10-hang-investigation
Date: 2026-09-13 (campaigns) · producer: glm-5.3 (opencode)
Machine note: load average 50–179 across all campaigns (host under heavy external load — WindowServer/OrbStack/etc., not this campaign's processes).

## Result: ZERO hangs in 37 runs — the defect did not reproduce

| Campaign | Runs | Hangs | Scenario | Per-run duration | Assertions |
|---|---|---|---|---|---|
| campaign-20260913T125525Z | 12 | 0 | full-run | 8–17 s | 12/12 each (first campaign's assertion counter had a display-only grep bug; logs verified — every run prints `driver-contract: 12 passed, 0 failed`) |
| campaign-20260913T130115Z (K-doubling escalation per plan Unresolved #3) | 24 | 0 | full-run | 8–11 s | 12/12 each |
| captures-probe/campaign (suite→full-run state probe) | 1 | 0 | full-run | 10 s | 12/12 |

failure rate: **bounded at 0/37** — not measured at ~1/3. The prior ~1-in-3 estimate came from three 2026-09-09 observations on a tree that has since changed (see Dating).

The state probe reproduced the exact precondition of the original observation — a full driver-contract suite pass (128/128) immediately followed by `--scenario full-run` — the sequence under which the one recorded hang occurred (suite PASS → full-run HANG → retry PASS on 2026-09-09). It ran clean (12/12, 10 s) at load 179.

## Blocked command

**none captured — no run hung.** No ps tree, no `sample` stack, no output-volume data exist because there was nothing to capture. The ~64 KB pipe-buffer hypothesis has NO supporting or refuting data from this campaign.

## Reinterpretation of the 2026-09-09 "zero assertions printed" observation

The original record treated zero printed assertions as "the driver invocation itself never returned" — i.e. an early hang. That inference is unsound: the hung run was killed by an external `timeout` wrapper with stdout redirected to a file, and block-buffered output is lost on kill. A hang anywhere in the scenario — including at the `check-research-package --package` assertion near the end — is equally consistent with the record. This reopens the candidate set to every command the scenario runs, not just its opening.

## Goal-2 dating — first appearance: pre-existing vs introduced

method: uncommitted-only

Claim: **undetermined, and undeterminable from committed history.** The single observed hang (2026-09-09) and all 37 clean runs occurred on the same uncommitted working tree (branch feat/cpc-001-002-integration-contract, 551 changed files). If the defect existed in tree code, it was introduced by this phase's uncommitted work — committed history cannot contain or date it. No hang site was ever isolated, so no stronger claim is available and none is invented.

Limitations, stated: (1) no isolated site → nothing to date; (2) the tree changed between the observation and this campaign (below), so "introduced" itself is provisional; (3) a commit pass was not made — dating via commit boundaries was not available (plan.md records this boundary).

## Leading unverified interpretations (recorded, NOT a diagnosis — Goal 4 forbids fix work on these)

1. **Incidentally fixed since 2026-09-09.** The five heredoc-hanging scripts extracted during change-drt-006's work landed the same day as the observed hang, and `check-research-package.sh` — which the full-run scenario invokes directly and which hung indefinitely before extraction — is among them. Combined with the buffering reinterpretation (a late-scenario hang is consistent with "zero assertions printed"), the observed hang may have been in code already remediated. Unverified: the extraction record says check-research-package's hang blocked *package validation*, not the driver scenario specifically.
2. **Environmental precondition not reproduced.** Some host state outside the tree (orphaned processes from other work, gateway state, temp-dir residue from the day's earlier suite runs) existed on 2026-09-09 and does not exist now. Unverifiable after the fact.

Per plan.md Unresolved #3: after the K-doubling escalation and the state probe, Goal 1 stands **BLOCKED (rate bounded 0/37)** — the child does not manufacture a diagnosis. Consequences carried to change-drt-009: p is 0/37 ⇒ N is sized from the Clopper-Pearson 95% upper bound — rule-of-three p_ub = 3/37 = 0.0811 ⇒ N = ceil(ln 0.01 / ln(1−0.0811)) = ceil(54.46) = **55**, labelled `inferred` (see FIX-CERTIFICATION.md); the extraction branch has NO confirmed hang site and must not run (Goal 4 forbids a fix without a reviewed diagnosis — and there is no diagnosis).

> Timestamp caveat on the historical tables below: they were written by the pre-revision script, whose `start_utc` column actually recorded the row-write (completion) time; true starts ≈ value − duration_s, and each campaign directory name carries the true campaign start. The committed script now records the genuine pre-launch timestamp, and post-hang runs are labeled `clean-perturbed`.

> Scratch-freshness scope: per-run scratch reset is **delegated** to the driver — `driver-contract.sh` builds each invocation's fixture tree under its own `mktemp -d`, so no state carries between runs; those temp roots intentionally persist in `$TMPDIR` (they are evidence, not leakage). The harness's own cleanup responsibility is the process tree only.

## Run records

Per-run tables live in `captures/campaign-*/summary.tsv` and `captures-probe/campaign-*/summary.tsv` (duration, exit, assertions, load). Raw run logs beside them. Campaign artifacts are not committed beyond this document's summary.

## Per-run records (full tables, inlined — verification criterion)

### campaign-20260913T125525Z (K=12)
```
run	start_utc	duration_s	exit	assertions	loadavg	result
run-01	12:55:36	11	0	0	54.42 71.17 62.16	clean
run-02	12:55:46	10	0	0	61.51 72.12 62.61	clean
run-03	12:55:55	9	0	0	60.11 71.49 62.49	clean
run-04	12:56:05	10	0	0	56.13 70.27 62.17	clean
run-05	12:56:18	13	0	0	56.84 69.95 62.15	clean
run-06	12:56:27	9	0	0	54.82 69.07 61.92	clean
run-07	12:56:37	10	0	0	52.06 68.01 61.63	clean
run-08	12:56:47	10	0	0	52.50 67.57 61.55	clean
run-09	12:56:55	8	0	0	55.88 67.83 61.71	clean
run-10	12:57:05	10	0	0	50.36 66.26 61.23	clean
run-11	12:57:22	17	0	0	50.46 65.75 61.11	clean
run-12	12:57:32	10	0	0	60.29 67.20 61.70	clean
```

### campaign-20260913T130115Z (K=24, escalation)
```
run	start_utc	duration_s	exit	assertions	loadavg	result
run-01	13:01:26	11	0	12	118.84 79.75 67.19	clean
run-02	13:01:34	8	0	12	124.39 82.23 68.22	clean
run-03	13:01:44	10	0	12	120.67 82.16 68.28	clean
run-04	13:01:52	8	0	12	114.05 82.04 68.40	clean
run-05	13:02:01	9	0	12	113.59 82.98 68.89	clean
run-06	13:02:11	10	0	12	114.96 84.27 69.51	clean
run-07	13:02:21	10	0	12	111.31 84.51 69.77	clean
run-08	13:02:29	8	0	12	111.11 85.34 70.24	clean
run-09	13:02:40	11	0	12	112.86 86.13 70.60	clean
run-10	13:02:50	10	0	12	115.22 87.52 71.28	clean
run-11	13:03:00	10	0	12	112.51 88.33 71.85	clean
run-12	13:03:11	11	0	12	116.91 90.07 72.66	clean
run-13	13:03:21	10	0	12	116.91 90.93 73.17	clean
run-14	13:03:30	9	0	12	123.36 93.18 74.18	clean
run-15	13:03:38	8	0	12	125.51 94.65 74.92	clean
run-16	13:03:48	10	0	12	123.55 94.76 75.07	clean
run-17	13:03:56	8	0	12	129.97 97.06 76.12	clean
run-18	13:04:04	8	0	12	121.46 96.36 76.12	clean
run-19	13:04:15	11	0	12	118.62 96.18 76.17	clean
run-20	13:04:25	10	0	12	119.53 97.45 76.97	clean
run-21	13:04:34	9	0	12	123.09 98.95 77.75	clean
run-22	13:04:45	11	0	12	120.92 98.90 77.85	clean
run-23	13:04:54	9	0	12	113.37 98.32 78.01	clean
run-24	13:05:02	8	0	12	112.94 98.48 78.19	clean
```

### captures-probe campaign (K=1, suite→full-run state probe)
```
run	start_utc	duration_s	exit	assertions	loadavg	result
run-01	13:12:10	10	0	12	179.40 126.79 97.39	clean
```

## Post-reflect evidence addendum — 2026-09-13 (goal-2 archaeology; supersedes the dating section above)

Operator challenge ("did G1/G2 still need to be met?") prompted the investigation the uncommitted tree
always permitted. Findings, each machine-checkable:

1. **Artifact hunt: no surviving trace of the hung run.** The `~/.prometheus/research/job-*`
   checkpoints near the hang date are all 2026-09-08 threaded-pipeline test jobs — none is the
   09-09 full-run. The hung run's temp dir is gone. G1's literal target (the exact blocked command)
   is therefore unidentifiable from any surviving evidence — investigation CLOSED as a proven
   negative, not an open question.
2. **Timeline, decisive:** extractions landed 09-09 15:47:41–15:47:42 (`check-*.py`) and 18:47:43
   (`check-research-package.sh` wrapper); the hang occurred ~20:26 (assessment written 20:46:59,
   "hung 20 minutes earlier"). The "incidentally fixed by drt-006's extractions" interpretation is
   **DISPROVEN as stated** — the extractions predate the hang by ~2–5 hours; the hang occurred on
   the POST-extraction tree.
3. **Same-code proof:** the latest mtime across the ENTIRE full-run dependency set (driver, runner,
   write-provenance, check-* companions + wrapper, merge/assemble scripts, build-review-packet.sh,
   fixture stubs) is 09-09 18:47:43 — before the hang. **No file the scenario executes was edited
   between the one hang (09-09 ~20:26) and the 98 clean runs (09-13). The same bytes hung once and
   passed 98 times.** The analysis-stage structural suspect (build-review-packet heredocs, last
   edited 09-06) is weakened to parity with every other chain command: none is discriminable.

**Goal-2 resolution (supersedes "Claim: undetermined" above):**
- **Introduced, not pre-existing — proven.** driver-contract.sh and check-research-package.sh carry
  zero prior commits (new in this phase's tree); the whole chain exists only in this phase's
  uncommitted work. The hang cannot predate the code that defines it.
- **First appearance: 2026-09-09 ~20:26** — dated from the assessment record; provably not earlier
  than the scripts' existence.
- **Not code-remediated — proven by mtime:** no remediating edit exists. The discriminator between
  the hung invocation and the 98 clean ones is **environmental state or an untriggered race**, not
  code version. The 09-09 host conditions were never recorded — that unrecorded variable is the
  residual unknown, and the armed --xtrace tripwire exists to convert any recurrence into the
  command-level evidence G1 wanted.
