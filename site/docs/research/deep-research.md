---
id: deep-research
title: Deep Research
sidebar_label: Deep Research
---

# Deep Research

`deep-research` is a ten-stage research pipeline: Planner → Search → Retrieve →
Collect → Verify → Resolve → Graph → Cite → Report → Export. It produces a
persistent, schema-validated **research package** rather than a disposable
report.

## The stage-contract driver

Stages are not a suggestion. `scripts/run-research.sh` owns the loop and
enforces `references/stage-contracts.md`: every stage declares required
artifacts, the driver validates them at the boundary, hooks fire **only after**
validation, and the checkpoint is written on every exit path — including
failure.

Two execution modes:

| Mode | Selected by | Behaviour at a boundary |
|---|---|---|
| **Checkpoint** (default) | `RESEARCH_STAGE_RUNNER` unset | Validate, stop at the first incomplete stage, print the stage skill to run plus one JSON line `{"next_stage","skill","package_dir"}`, exit 3. The caller runs that stage and re-invokes with `--resume` |
| **Runner** | `RESEARCH_STAGE_RUNNER=<command>` | The driver invokes `<command> <stage> <package_dir>` itself; a non-zero exit is a blocked stage, never a skipped one |

`--resume` skips only stages whose artifacts **still validate now**. A stage
whose artifact was deleted or corrupted is re-run. A scale gate reads the
planner's sub-question count and keeps narrow questions out of multi-agent mode.

## The package

One output root, `~/.prometheus/research` (override with
`RESEARCH_OUTPUT_DIR`). Package directories are named
`<slug>-<yyyymmdd>-<4hex>`; there is no `.research` suffix.

```
<package_id>/
  plan.md            # research plan AND task ledger, verification log, decision log
  sources/           # url-list.json, chunk-<n>.json, registry.json, credibility.json
  contradictions.json  graph.json  citations.json
  report.md          # front matter carries verification_status
  manifest.json      # JSON-Schema validated
  index.md
  <slug>.provenance.md   # the sidecar
  checkpoint.json
```

`plan.md` doubles as the run's ledger. Every stage completion, every artifact
validation, every hook firing, and every decision the driver made (such as the
scale choice) is appended to it, so the plan file records what actually
happened, not only what was intended.

Validate any package with:

```bash
bash scripts/check-research-package.sh --package <dir>
```

## Claim labels

Every claim carries one of four labels, the same vocabulary the learn skills
use:

| Label | Meaning |
|---|---|
| `verified` | Grounded in a source the verifier actually read |
| `inferred` | Judged without a direct source reference |
| `unverified` | Recorded without verification |
| `blocked` | Verification could not complete — dead link, missing tier, refused fetch |

The rule is "verify meaning, not topic overlap" and "read before you label". A
dead link is `blocked`, never quietly dropped.

`verification_status` on the report is **derived, never chosen**:
`check-research-package.sh --derive` recomputes it from the labels and refuses a
manifest that disagrees. A run with no usable sources reports `unverified`
rather than inventing a finding.

## The provenance sidecar

`<slug>.provenance.md` is written on **every** exit path, including a blocked or
failed run. It records the stages planned and completed, sources consulted,
accepted and rejected, the verification verdict (`PASS`, `PASS WITH NOTES`, or
`BLOCKED`), any blocked stage, and the adversarial review outcome.

## Agent duties

Four research agents carry `tools:` allowlists in their frontmatter. Two
ordering rules are enforced by the driver rather than by convention:

- The **verifier completes before the reviewer** runs; review is refused when
  `credibility.json` is missing.
- The final report is judged by `adversarial-review`, so **the producer never
  reviews itself**.

## Source scoring and contradictions

Credibility scoring renormalises its weights over the dimensions for which
evidence is actually present, and emits a rank-sensitivity artifact alongside
the scores. The scorer refuses an empty registry rather than synthesising
scores. Claim ids are content-addressed and shared by the graph builder and the
contradiction detector, which finds both numeric and semantic contradictions
and resolves them by authority, then recency, then consensus, escalating what it
cannot settle.

## Running it

```bash
# checkpoint mode, one stage at a time
bash scripts/run-research.sh --query "<question>" --depth shallow
bash scripts/run-research.sh --resume <package_dir>

# what tiers are available before you start
bash scripts/run-research.sh --check-tools
```

`--check-tools` reports the search tier, gateway, and output root up front. When
no search adapter is configured it says so, and the run that follows will label
its claims `blocked` rather than pretending to have searched.

For headless and daemon execution, see
[prometheus-research](../substrate/prometheus-research.md).

## Bench Run 2026-09-14

The 10-task G5 benchmark was executed on 2026-09-14, scoring 3 of 10 tasks:

| Task | RACE | Effective citations | Verified-claim ratio |
| --- | --- | --- |
| 51 (Japan elderly 2020–2050) | 64.0 | 6 | 0.6500 |
| 71 (K-12 AI) | 76.1 | 15 | 0.3158 |
| 87 (AI fashion) | 89.5 | 20 | 0.1250 |

`skills/research/deep-research/scripts/label-claims.py` closes the
verified-claim-ratio gap. Five tasks (58, 66, 79, 81, 85) remain to be built.
Skill package version 1.9.0; deep-research skill version 1.1.0.
