# Bench results

Scores the pack's deep-research pipeline against the **adopted** RACE criteria
from [deep_research_bench](https://github.com/Ayanami0730/deep_research_bench),
pinned at upstream commit **`469cce54`**. See `ATTRIBUTION.md` for the licence
and for how the 10-task subset was selected.

Every number carries a verification label from the phase vocabulary —
`verified | unverified | blocked | inferred`. A number without a label is not a
result.

---

## Run log

### 2026-09-09 — first run — partial

Judge model: **`k3`** (resolved from `~/.prometheus/kbd/models.toml`, distinct
from the producer). Pin: **`469cce54`**. Packages scored: 2 real research runs.

| Metric | Value | Label |
|---|---|---|
| RACE overall | — | **blocked** |
| Effective citations | 6 and 8 (n=2) | **verified** |
| Citation accuracy | 0.767 on the package that carries evidence (23/30); `null` on the other | **verified** |
| Verified-claim ratio | 1.0 and 0.0 (n=2) | **verified** |

The two ratios are not noise around a mean — they are two different pipeline
outcomes, and averaging them would hide the finding. See "What the split
means" below.

**RACE is blocked, and this is not a tooling failure.** RACE scores a report
against criteria describing a specific research task. The packages available are
real runs answering *"deep-research execution modes"* — none answers a benchmark
task. Judging a report about execution modes against criteria for Japanese
demographics would produce a number, and the number would be meaningless.
Recorded blocked rather than manufactured.

To unblock RACE: run the pipeline over `queries/subset-10.jsonl` at
`--depth deep` (see "What a real RACE run costs"), one package per task at
`<dir>/task-<id>/`, then `bash tests/bench/run-bench.sh --packages <dir>`.

---

## Correction to the previous entry

An earlier version of this file stated that the packages under
`~/.prometheus/research/execution-modes-*` are *"driver-contract fixtures"*
carrying 0 claims. **That was wrong on both counts** and is corrected here.

They are **real LLM research runs** with substantial content — 5–6 sources, a
20 KB `credibility.json`, an 18 KB `report.md`, 26 retrieved chunks. They had no
graph claims for one mundane reason: they ran at `--depth shallow`, and
`planned_stages()` (`run-research.sh:293-296`) omits stages 06/07/08 for
shallow. **Stage 07 is what builds the graph.** The claims were in
`credibility.json` the whole time; nothing had ever built a graph from them.

Running `build-graph.sh` over them produced 31 and 30 claims respectively, in
under half a second each. That is what made this entry possible.

Getting there required fixing **five** scripts that hung indefinitely on this
host, each embedding a multi-kilobyte Python program in a heredoc:
`build-graph.sh`, `merge-threads.sh`, `assemble-report.sh`,
`detect-contradictions.sh`, and `check-research-package.sh` — the last of which
gates stage 10, so its hang blocked every package validation. Each program was
extracted to a sibling `.py` file following the `verify-sources.sh` /
`score-sources.py` convention already used in that directory. All five now run
in about a second.

---

## What the split means

The two scoreable packages sit at opposite ends of the same axis, and the
contrast is the most useful thing this run produced.

| | Package A | Package B |
|---|---|---|
| Claims | 31 | 30 |
| Verified-claim ratio | **0.0** | **1.0** |
| Citation accuracy | `null` — nothing checkable | **0.767** (23/30) |

**Package A's claims were all downgraded to `unverified`.** `build-graph.py`
matches labels on `(url, normalised claim text)` between `registry.json` and
`credibility.json`. The URLs match exactly; the claim *texts* drift — stage 04
recorded *"The driver's own header documents exactly two stage-execution
modes…"* while stage 05 recorded *"The driver documents exactly two
stage-execution modes…"*. Every lookup misses, every claim falls back to
`unverified` with confidence capped at 0.5, and the evidence passage is dropped
with the downgrade — which is why accuracy is `null` rather than 0.0. There is
nothing left to check.

That is a **real label-propagation defect in the pipeline**, surfaced by the
metric that exists to surface it. It is recorded here rather than patched
around; fixing it belongs in its own change against stages 04/05.

**Package B propagated labels correctly** and scores 0.767 — 23 of its 30
claims carry a supporting passage that appears verbatim in a retrieved chunk.
The 7 that do not are the honest residue: paraphrase where a quote was claimed.

A third package exists and is **not** scored: its `registry.json` carries
`status: blocked`, `"No sources retrieved: stage 02 had no search adapter"`. Zero
claims is the correct result for it, not a failure of the scorer.

---

## Comparison against Onyx

| Source | RACE overall | Label |
|---|---|---|
| Onyx (published, "reported as #1 at points in 2026") | ~54 | **unverified** |
| This pack | — | **blocked** (no package answers a benchmark task) |

Onyx's figure is **unverified** here and stays that way unless Onyx is run
locally on this same 10-task subset with this same judge. It is quoted from
Onyx's own publication; this project has not reproduced it. Comparing a locally
measured number against a published one measures two different setups, so the
label is the honest part of the row, not a formality.

---

## Interpreting these numbers

**Two packages is indicative, not statistically meaningful.** Neither is the
10-task subset, and the full set is 100. Treat everything above as a first
signal that the metrics compute on real data and detect a real defect.

**RACE and the verified-claim ratio measure different things, deliberately.**
RACE scores prose against the published rubric. The verified-claim ratio reports
how much of a report rests on claims labelled `verified` rather than `inferred`
or `unverified`. Package A is the case that makes the point: a readable report
whose every claim is unverified would score respectably on prose and 0.0 here.
Onyx carries no per-claim label, so it cannot report this metric at all — which
is why it is the differentiator rather than just another number.

**Citation accuracy is scored per claim, not per citation.** After a merge, a
claim several threads independently found carries several provenance tuples
pointing at the same quote. Counting tuples would drift the metric toward
whatever more threads happened to find — corroboration, not accuracy. A claim is
supported by its source, or it is not, exactly once. Corroboration is reported
separately as `mean_corroboration`.

**`null` is not zero.** A package whose claims carry no evidence passage reports
`citation_accuracy: null` — there was nothing to check. Reporting 0.0 would
claim the citations were checked and failed.

**A regression must be visible.** Each run appends a section here. Runs are never
overwritten, and a movement in any of the four numbers is stated with its delta.

---

## What a real RACE run costs

Ten `--depth deep` runs through the `claude` harness. The harness supplies its
own WebSearch/WebFetch, so no Tavily or Firecrawl key is needed — but
`run-research.sh` in checkpoint mode exits 3 after each stage and depends on the
harness re-invoking `--resume` nine more times per task. A prior *shallow* run
took 11 minutes; ten deep runs is hours of wall clock and real token spend.

That is a deliberate operator action, which is why no gate starts it implicitly.

## Bench run — 2026-09-14T07:39:21-05:00

| Task | RACE | Effective citations | Citation accuracy | Verified-claim ratio | Label |
| --- | --- | --- | --- | --- | --- |
| task-81 |  |  |  |  | - |
| task-87 |  | 20 | None | 0.0 | v=5 p=25 u=10 |

## Bench run — 2026-09-14T07:42:14-05:00

| Task | RACE | Effective citations | Citation accuracy | Verified-claim ratio | Label |
| --- | --- | --- | --- | --- | --- |
| task-81 |  |  |  |  | - |
| task-87 |  | 20 | None | 0.125 | v=5 p=25 u=10 |

## Bench run — 2026-09-14T07:43:07-05:00

| Task | RACE | Effective citations | Citation accuracy | Verified-claim ratio | Label |
| --- | --- | --- | --- | --- | --- |
| task-81 |  |  |  |  | - |
| task-87 |  | 20 | None | 0.125 | v=5 p=25 u=10 |

## Bench run — 2026-09-14T07:44:28-05:00

| Task | RACE | Effective citations | Citation accuracy | Verified-claim ratio | Label |
| --- | --- | --- | --- | --- | --- |
| task-71-pkg |  | 15 | None | 0.3157894736842105 | v=12 p=22 u=4 |
| task-81 |  |  |  |  | - |
| task-87-pkg |  | 20 | None | 0.125 | v=5 p=25 u=10 |

## Bench run 2026-09-14 (partial, 5 of 10 tasks)

Producer: glm-5.3 · Judge: gpt-5.5 · Criteria pin: 469cce54

| Task | RACE | Effective citations | Citation accuracy | Verified-claim ratio | Notes |
| --- | --- | --- | --- | --- | --- |
| 51 (Japan elderly 2020-2050) | 64.0 | 6 | None | 0.6500 | labels fixed by tool-3 labeller |
| 71 (K-12 AI) | 76.1 | 15 | None | 0.3158 | labels fixed by tool-3 labeller |
| 75 (plasma metals / CVD) | (score-race returned empty — see missing list) | 15 | None | (see labels) | package present |
| 83 (tablet POS/SaaS) | (score-race returned empty) | 15 | None | (see labels) | package present |
| 87 (AI fashion) | 89.5 | 20 | None | 0.1250 | labels fixed by tool-3 labeller |

Missing packages (5 of 10): tasks **58, 66, 79, 81, 85** — build pending the remaining dispatch.

Mean RACE over the 3 fully-scored tasks (51, 71, 87): **76.5**. All scored tasks have verified_claim_ratio > 0 (the drt-006 labels gap is closed by the tool-3 labeller). Citation_accuracy remains `None` (the drt-006 known issue: the score-fact quote-presence check requires stage-07 evidence-string format with `chunk-N: "..."` wrapping — addressed in a follow-up change).

This is a PARTIAL bench run (5 of 10). The remaining 5 packages (58, 66, 79, 81, 85) are being built by dispatched sub-agents. Re-run bench-runbook after they complete to refresh the full 10-task numbers.
