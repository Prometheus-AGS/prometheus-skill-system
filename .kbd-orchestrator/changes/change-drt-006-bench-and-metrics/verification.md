# Verification — change-drt-006-bench-and-metrics

Repository: `prometheus-skill-pack`

## Acceptance criteria

- `BENCH-RESULTS.md` names the upstream commit `469cce54`, the judge model, and the run date.
- A run where the judge model equals the producer is recorded BLOCKED, never scored.
- All four numbers (RACE overall, effective citations, citation accuracy, verified-claim ratio) are present, each labelled `verified` or `unverified`.
- Onyx's published figure is labelled `unverified` unless Onyx was run locally on the same subset, and the comparison says which.
- Apache-2.0 attribution is present for the adopted criteria, prompts, and queries.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run is recorded BLOCKED with the reason, never skipped silently.

```verify
bash -n skills/research/deep-research/tests/bench/run-bench.sh
python3 -c "import ast;ast.parse(open('skills/research/deep-research/tests/bench/score-fact.py').read())"
test -s skills/research/deep-research/tests/bench/BENCH-RESULTS.md
grep -q '469cce54' skills/research/deep-research/tests/bench/ATTRIBUTION.md
```

**A bench run costs real tokens.** The 10-task subset is this phase's ceiling, and its results are indicative rather than statistically strong — `BENCH-RESULTS.md` must say so rather than implying a leaderboard-grade number.

## Evidence

Run locally 2026-09-09 from the repository root. No hosted CI.

| Gate | Result |
|---|---|
| `bash -n run-bench.sh` / `score-race.sh` | **PASS** — bash 5 and `/bin/bash` 3.2 (C-05) |
| `python3 -c ast.parse(score-fact.py)` | **PASS** |
| `run-bench.sh --dry-run` | **PASS** — gateway reachable, judge resolves to `k3`, 10 tasks loaded |
| Judge-equals-producer refusal | **PASS** — exit 2 BLOCKED, verified by execution |
| `score-fact.py` on a real merged package | **PASS** — effective citations 2, citation accuracy 0.5, verified-claim ratio 0.5, mean corroboration 2.0 |
| `npm run check:distribution` | **PASS** after regenerating |
| `npm run validate:codex` | **PASS** |
| `npm run check:skills-index` | **PASS** |
| **The bench run itself (task 4)** | **BLOCKED** — see below |

### Task 4 is BLOCKED, and its verify string passing does not change that

Task 4's verify string checks that `BENCH-RESULTS.md` contains `unverified`, the
pin `469cce54`, a judge/model reference, and a date. All four are present, so the
string **passes** — but it checks the document's shape, not that a run happened.
The spec requires running the bench and recording four scored numbers.

**No research packages exist for the 10 benchmark tasks.** Scoring needs
`report.md` plus `graph.json` with labelled claims per task. The three packages
under `~/.prometheus/research/` are driver-contract fixtures: each carries **0
claims** and none matches a benchmark task id. Scoring them would produce a
number about the fixtures, not about the pipeline.

Task 4 is therefore marked `blocked` in `tasks.json` with that reason, and
`BENCH-RESULTS.md` records all four numbers as `blocked` rather than scored. A
gate that could not run is never reported as a pass — and a verify string that
passes for the wrong reason is not evidence that it did.

### A modelling flaw the hand-check caught

`score-fact.py` first scored citation accuracy **per provenance tuple**. On the
merged fixture that gave 4 of 6 = 0.667 — arithmetically right, and wrong as a
metric: `claim-aaaa1111` was found by three threads, so it carried three tuples
pointing at the same quote and contributed three of the six checks, while a
single-thread claim contributed one. The metric would drift toward whatever more
threads happened to find, which measures **corroboration, not accuracy**.

It is now scored **per claim** with quotes deduplicated: 1 of 2 = 0.5, which the
hand-check confirms (`aaaa1111` verbatim-present, `bbbb4444` absent).
Corroboration is reported separately as `mean_corroboration` (2.0) because it is
a real signal, just a different one.

This surfaced only because the number was verified by hand rather than accepted
on first plausible output.

### Correction: the heredoc diagnosis above was wrong, and so was the fixture claim

Two things recorded earlier in this change were false. Both are corrected here
rather than quietly overwritten.

**"Large heredocs hang on this host" — disproved.** `export-package.sh` embeds a
*larger* program (8730 bytes vs 6191) through the same mechanism and runs fine.
The argument form is not the discriminator either: a reproduction with the same
program hangs in both the positional and env-var-prefix forms. The verified
failure set is **five** scripts — `build-graph.sh`, `merge-threads.sh`,
`assemble-report.sh`, `detect-contradictions.sh` and `check-research-package.sh`
— each of which runs in about a second once its Python is extracted to a file.
Root cause is not fully isolated; the remedy does not depend on it.

`check-research-package.sh` matters most of the five: `validate_stage 10` calls
it, so its hang blocked every package validation in the pipeline, not just this
benchmark. It carried three separate embedded programs, now
`check-manifest-schema.py`, `check-manifest-files.py` and
`check-graph-claims.py`.

**"The three packages are driver-contract fixtures with 0 claims" — wrong on
both counts.** They are real LLM research runs with 5–6 sources, a 20 KB
`credibility.json` and an 18 KB `report.md`. They carried no graph claims
because they ran at `--depth shallow`, and `planned_stages()`
(`run-research.sh:293-296`) omits stages 06/07/08 for shallow — stage 07 is what
builds the graph. `BENCH-RESULTS.md` now carries the correction.

### What task 4 delivered

The three scripts were fixed by extracting their embedded Python to sibling
`.py` files, following the `verify-sources.sh` / `score-sources.py` convention
already in that directory (`HERE`-relative resolution, explicit existence check,
`exec` so exit codes pass through).

| Gate | Result |
|---|---|
| `tests/merge-threads.sh` | **PASS** — 17/17, bash 5 and 3.2 (was unrunnable) |
| `tests/report-assembly.sh` | **PASS** — 14/14, bash 5 and 3.2 (was unrunnable) |
| `tests/thread-contracts.sh` | **PASS** — 14/14 |
| `tests/dispatch-smoke.sh` | **PASS** — 14/14 |
| `build-graph.sh` on a real package | **PASS** — 31 claims in 0.4s (was hanging) |
| `check-research-package.sh` on a real package | **PASS** — RESULT: PASS, 30 claims validated, 1.2s (was hanging) |
| `tests/scoring-graph.sh` | **INCOMPLETE** — 33 PASS, 0 FAIL, then stalls; see below |

With graphs built, the FACT metrics ran on real research data:

| Metric | Result | Label |
|---|---|---|
| Effective citations | 6 and 8 (n=2) | **verified** |
| Citation accuracy | 0.767 (23/30) where evidence exists; `null` where it does not | **verified** |
| Verified-claim ratio | 1.0 and 0.0 (n=2) | **verified** |
| RACE overall | — | **blocked** |

RACE stays blocked because no available package answers a benchmark task;
scoring a report about execution modes against criteria for Japanese
demographics would yield a meaningless number.

### `scoring-graph.sh` is INCOMPLETE, and why that is not a regression here

The suite reaches **33 PASS, 0 FAIL** and then stalls on one scenario:
`semantic-blocked`, which deliberately points `LITER_LLM_BASE_URL` at a closed
port to prove the semantic path records every pair as blocked rather than
silently absent. Reproduced identically across separate runs.

Two things establish that this is not caused by this change:

1. **The pre-edit version hangs the same way.** The heredoc version of
   `detect-contradictions.sh` was reconstructed from its saved head plus its
   saved program body and run against the same fixture with the same
   environment: it also times out at 40s. The stall predates the extraction.
2. **The machine is saturated.** Load average measured 26.3, then 44.96 and
   still climbing, driven by WindowServer, OrbStack, ChatGPT/Codex and Open
   Design — none of them this session's processes. Under that load *every*
   scoring-graph scenario times out at 60s, including ones that had just
   produced 33 PASS in a full run minutes earlier. A timeout under load 45 is
   not evidence about code.

Recorded INCOMPLETE rather than PASS: the suite did not finish, so claiming it
green would be exactly the unverified pass this change's own gates refuse. It
should be re-run on an unloaded machine before the next change in this area.

An `exec`-related hazard was found and fixed while investigating, even though it
turned out not to be today's cause: `detect-contradictions.sh`'s Python calls
back into `semantic_judge`, which reaches it as an **exported shell function**
(`export -f`). `exec` replaces the shell that exported it, so the callback would
find nothing. That wrapper forwards the exit code explicitly instead of
`exec`-ing; the other four wrappers have no callback and keep `exec`.

### The metrics found a real pipeline defect

The 1.0/0.0 split is not noise. One package's claims were all downgraded to
`unverified` because `build-graph.py` keys labels on
`(url, normalised claim text)` and the texts drift between `registry.json` and
`credibility.json` — stage 04 wrote "The driver's own header documents…", stage
05 wrote "The driver documents…". Every lookup misses. The evidence passage is
dropped with the downgrade, which is why accuracy is `null` there rather than
0.0. That is a genuine label-propagation defect in stages 04/05, surfaced by the
metric built to surface it, and recorded rather than patched around.

### Two scorer defects the real data exposed

`score-fact.py` was written against the merge's claim shape and silently
mis-read stage 07's:

1. It read `source_id`/`provenance` only, while `build-graph.py` emits a
   `sources` array — reporting **0 effective citations for 31 cited claims**.
2. It read `quote` only, while stage 07 records the passage as `evidence`,
   often as prose *containing* the quote rather than being it. Checking the
   whole sentence verbatim reported **0.0 accuracy**; extracting the inner
   quoted fragment gives 0.767, corroborated by an independent hand-count
   (19/26 fragments verbatim-present).

Both are fixed. The merged-fixture regression still reads exactly 0.5, so the
fixes did not move the previously verified number.

### The extraction changed no logic

Each extracted program was diffed against the heredoc body it replaced. All
seven are byte-identical: `build-graph.py`, `merge-threads.py`,
`assemble-report.py`, `detect-contradictions.py`, `check-manifest-schema.py`,
`check-manifest-files.py`, `check-graph-claims.py`. Only the location of the
code changed.

(The first fidelity check reported the three `check-*` files as differing. That
was an off-by-one in the comparison — the docstring header is 9 lines, not 10 —
not a content change. Re-run correctly rather than accepted either way.)

### Verdict

**PASS WITH NOTES.** The benchmark is adopted, pinned and attributed under
Apache-2.0; the harness refuses to score when judge equals producer; five
hanging scripts are fixed, byte-faithfully, and four suites run at their prior
scores (17/17, 14/14, 14/14, 14/14); all three C-01 gates pass; the FACT metrics
and the verified-claim ratio are exercised on real research data, reproduce from
scratch, and found a real label-propagation defect in stages 04/05.

Two items are recorded honestly rather than resolved:

- **RACE overall is BLOCKED** — no available package answers a benchmark task.
- **`scoring-graph.sh` is INCOMPLETE** — 33 PASS, 0 FAIL, then stalls on a
  scenario proven to stall identically before this change, on a machine at load
  45. Re-run it unloaded before the next change in this area.
