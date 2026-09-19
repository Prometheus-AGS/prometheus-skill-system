# change-rah-005-claim-labels-and-provenance

**Title:** Put the four-label verification vocabulary on every claim in the research package and on feynman-loop artifacts
**Repository:** `prometheus-skill-pack`
**Phase:** research-agent-hardening
**Goal:** G3
**Depends on:** `change-rah-003-stage-contract-driver`
**Backend:** native-kbd

## Why

The pack has three package-level labels and no claim-level label; grep of the references finds no `inferred` or `blocked` (assessment G3, verified). Feynman CLI carries `verified | unverified | blocked | inferred` per claim and derives the package verdict from them (analysis D-05).

## What Changes

- Define the label enum in `okf-research-format.md` and add `label` to every claim in the `graph.json` schema and every row of the report evidence table; define the derivation rule for the package label (all verified gives `verified`; any blocked or inferred on a critical claim gives `partial`; stage 05 skipped gives `unverified`).
- Update stage 05, 06, 07, 08, 09 skills and the three research agents to emit and consume the label; `templates/report-template.md` gains the label column; the verifier agent gains Feynman CLI's meaning-not-topic rule and the no-orphan-citation rule, attributed.
- Extend `check-research-package.sh` to validate `graph.json` claim labels against the enum and the derivation rule against `report.md` frontmatter.
- feynman-loop: `write-artifact.sh` requires `verification: {label, evidence}` per transfer score and a `provenance` block naming the grade file and corpus; SKILL.md documents the fields; learn-grade gaps carry `label`.

## Scope

Files this change may create, edit, or delete (tasks.json `files` is the per-task view):

- `skills/research/deep-research/references/okf-research-format.md`
- `skills/research/deep-research/references/research-package-spec.md`
- `skills/research/deep-research/references/schemas/research-manifest.schema.json`
- `skills/research/deep-research/references/schemas/research-graph.schema.json`
- `skills/research/deep-research/templates/report-template.md`
- `skills/research/deep-research/skills/stage-05-verify/SKILL.md`
- `skills/research/deep-research/skills/stage-06-resolve/SKILL.md`
- `skills/research/deep-research/skills/stage-07-graph/SKILL.md`
- `skills/research/deep-research/skills/stage-08-cite/SKILL.md`
- `skills/research/deep-research/skills/stage-09-report/SKILL.md`
- `skills/research/deep-research/agents/source-verifier.md`
- `skills/research/deep-research/agents/contradiction-resolver.md`
- `skills/research/deep-research/agents/report-synthesizer.md`
- `skills/research/deep-research/scripts/check-research-package.sh`
- `skills/learn/feynman-loop/SKILL.md`
- `skills/learn/feynman-loop/scripts/write-artifact.sh`

Scope amendment (recorded during apply, round-1 review finding): the labelled
fixtures this change adds are exercised end to end by the rah-003 driver
suite, which required three files owned by earlier changes to learn the label
vocabulary. They are edited here, minimally, and listed so the packet is
honest about what moved:

- `skills/research/deep-research/scripts/export-package.sh` (rah-002): reads
  `package_id` from `checkpoint.json` so a fixture directory need not match the
  `<slug>-<yyyymmdd>-<4hex>` pattern; explicit `jq`/`python3` prerequisite check.
- `skills/research/deep-research/tests/fixtures/stage-runner.sh` (rah-003):
  stage 05 emits labelled claims, stage 07 emits `critical`/`evidence`, stage 06
  labels its unresolved entry, stage 09 reports `verification_status: partial`.
- `skills/research/deep-research/tests/fixtures/package-labelled/`,
  `.../package-partial/`, `.../package-unlabelled/` (new): static packages for
  the verified, blocked-critical (`partial`), and missing-label drift-check
  scenarios.
- `skills/learn/learn-grade/SKILL.md` (planned, listed in `tasks.json`).

## Capabilities

- `research-pipeline-execution (label requirements added)`
- `learn-model-coherence (artifact provenance requirement, authored in change-rah-008)`

## ADDED Requirements

### Requirement: Every claim carries a label
Every claim in `graph.json` and every evidence row in `report.md` SHALL carry one of `verified`, `unverified`, `blocked`, `inferred`.

#### Scenario: Missing label
- **WHEN** a fixture package has one claim without a label
- **THEN** `check-research-package.sh` exits non-zero naming the claim id

### Requirement: The package label is derived
The `verification_status` in the manifest and report frontmatter SHALL equal the value derived from the claim labels by the documented rule.

#### Scenario: Derivation
- **WHEN** a fixture has one `blocked` critical claim
- **THEN** the derived and declared package label are both `partial`

### Requirement: Learn artifacts carry provenance
A feynman-loop artifact SHALL be refused by `write-artifact.sh` unless each transfer score has a label with evidence and the provenance block names the grade file.

#### Scenario: Refusal
- **WHEN** an artifact JSON without `verification` is passed to the writer
- **THEN** it exits non-zero and writes nothing

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

- Whether `inferred` claims may appear in the executive summary at all (default: yes, marked inline as inference, matching Feynman CLI's rule).

## Unresolved review findings

Adversarial review (k3 judge via gateway, producer `claude-fable-5-1`, `--mode diff` on `files.txt`), two rounds, receipts in `.kbd-orchestrator/phases/research-agent-hardening/review/change-rah-005-claim-labels-and-provenance/` (`round1/` and the round-2 `findings.json`).

### Round 1 — BLOCK (5 CRITICAL, 2 WARNING, 3 SUGGESTION)

| # | Finding | Disposition |
|---|---|---|
| C1 | `research-manifest.schema.json` never added; first acceptance command exits 1 | **Rejected.** The schema exists and is intent-added (rah-002); it was outside this change's `files.txt`, so the judge did not see it. The labelled fixture command exits 0 (verification.md). |
| C2 | `export-package.sh` and `stage-runner.sh` edited outside the declared Scope | **Accepted.** Scope amendment recorded above with the reason for each file. |
| C3 | jq fallback `index([$v])` can never match a scalar enum | **Rejected as stated, tightened anyway.** jq's array `index` with an array argument is a subsequence search: `["verified","partial"] \| index(["partial"])` returns 1, and the forced-fallback probe passed before the change. Rewritten to `any($d.enum[]; . == $v)` so the intent is unambiguous. |
| C4 | The no-python3 path is unreachable (block extraction, files loop, export step, both heredocs all call python3) | **Accepted, fixed.** Block extraction is awk; the files loop is jq; the export step and the report/provenance agreement are NOTEs without python3; the claim-label rules and the derivation have a jq form (`check_labels_jq`) that applies the documented rule verbatim. Verified with `CHECK_RESEARCH_PACKAGE_NO_PYTHON=1` in both modes and under `/bin/bash` 3.2. |
| C5 | Negative scenario says "naming the claim id" but output named only `claims/1` | **Accepted, fixed.** Both paths print `claims[1] (claim-002)`. |
| W1 | Prose requires `misconceptions_absent == 1.0`, rule and derivation ignore it | **Accepted, fixed.** Added to the jq rule, the python derivation, the jq fallback, and the report/manifest agreement keys; gate values read from `checkpoint.json` first, `report.md` frontmatter second. Negative probe fails closed in both paths. |
| W2 | `export-package.sh` gains an unguarded python3 dependency | **Accepted, fixed.** `jq` and `python3` checked up front, documented in the header. |
| S1 | Dead `for k in ("integrations",): pass` | Fixed. |
| S2 | Unlabelled fixture still says "labelled" in job id, query, titles | Fixed. |
| S3 | A bare-string `verification` entry crashes jq before the structured refusal | Fixed with an object-type guard; probe shows the documented `{"ok":false,...}`. |

### Round 2 — BLOCK (1 CRITICAL, 5 WARNING); cap reached, fixed and not re-vetted

| # | Finding | Disposition |
|---|---|---|
| C1 | python `load()` prints FAIL for corrupt JSON but never sets `rc`, so a corrupt `graph.json` exits 0 | **Accepted, fixed.** `load()` calls `fail()`. Probe: corrupt `graph.json` exits 1. |
| W1 | NOTEs printed inside the python blocks never set the shell `NOTES` flag, so PASS could hide a downgraded check | **Accepted, fixed.** Python blocks exit 3 for pass-with-notes; the shell maps 3 to `NOTES=1`, anything else non-zero to `FAIL=1`. Probe: legacy-graph package reports PASS WITH NOTES. |
| W2 | Prose says "no claim carries a label" is `unverified`; the rule derived `partial` | **Accepted, fixed.** `labels` counts only present labels in the doc rule, the python derivation, and the jq fallback. |
| W3 | Exporter fabricates empty `graph.json`/`citations.json`/`contradictions.json` without naming them in `defaulted:` | **Accepted, fixed.** Each fabricated file is appended to the `defaulted:` line as `<file>(empty)`. |
| W4 | `report-synthesizer.md` reads `<package_id>/...` while stage 05–09 skills still write `<job_id>/...` | **Accepted in scope, remainder deferred.** The seven in-scope mentions (stage 05, 06, 07, 09, source-verifier, contradiction-resolver) now say `<package_id>/`. `stage-contracts.md` already documents `<job_id>/` as the legacy alias for the package directory; the remaining mentions (stages 01–04, 10, three references, SKILL.md line 448) are documentation reconciliation owned by `change-rah-011-integration-evidence-and-docs`. |
| W5 | No fixture exercises "one blocked critical claim derives `partial`" | **Accepted, fixed.** `tests/fixtures/package-partial/` (claim-002 `blocked`, critical, with evidence; declared `partial`) added to `files.txt` and the verify block; passes in both paths. |

Rejected findings are recorded with the probe that refutes them. Round-2 fixes are covered by the probes in verification.md but were not seen by the judge.
