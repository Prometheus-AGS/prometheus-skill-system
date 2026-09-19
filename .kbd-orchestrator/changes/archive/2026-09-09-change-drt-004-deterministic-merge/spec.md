# change-drt-004-deterministic-merge

**Title:** Fold threads back into the existing stage 02/03/04 artifacts, deterministically
**Repository:** `prometheus-skill-pack`
**Phase:** deep-research-onyx-parity
**Goal:** G1, G2
**Depends on:** `change-drt-001-dispatch-smoke-and-thread-contracts`
**Backend:** native-kbd

## Why

This is the change that makes threading a refactor rather than a rewrite. Onyx's
analogue, `citation_utils.py::collapse_citations`, is a **pure function**: it
renumbers each worker's markers into a global map keyed by `document_id`,
reusing an existing number when another worker already cited the same document.
No LLM is involved, which is why it cannot drift.

The pack already emits **content-addressed claim ids** shared by
`build-graph.sh` and `detect-contradictions.sh` (change-rah-010), so the merge
keys on canonical URL for sources and that existing hash for claims (analysis
D-05) rather than inventing a scheme.

The merge is also the **enforcement point** for the director's no-search rule on
harnesses that ignore `tools:` (drt-003).

## What Changes

- `scripts/merge-threads.sh`: deterministic, no LLM. Unions
  `threads/*/sources.json` by canonical URL into `sources/registry.json` and
  `sources/url-list.json` **in their existing stage 02/04 shapes**; copies chunks
  to `sources/chunk-<n>.json` in the existing stage 03 shape; unions claims by
  content-addressed id keeping every `(thread_id, source_id, quote)` provenance
  tuple; rewrites per-thread citation markers into global numbers and writes
  `citation-map.json`.
- Refuses, as a CRITICAL failure, any dossier source absent from that thread's
  `sources.json`.
- `tests/merge-threads.sh`: fixture threads with overlapping sources and
  duplicate claims; asserts the driver's existing stage 02/03/04 validators pass
  on merged output **unchanged**, that the same URL cited by three threads becomes
  one citation number, and that the no-search violation is caught.

## Scope

- `skills/research/deep-research/scripts/merge-threads.sh`
- `skills/research/deep-research/tests/merge-threads.sh`
- `skills/research/deep-research/tests/fixtures/threads-merge/`
- `skills/research/deep-research/references/stage-contracts.md`
- `shared/scripts/lib/` (canonical URL normalisation, extracted so stage 02 and the merge cannot drift)
- `skills/research/deep-research/scripts/run-research.sh`

## Capabilities

- `research-pipeline-execution (thread merge added)`

## ADDED Requirements

### Requirement: The merge is deterministic
Given the same thread artifacts, the merge SHALL produce byte-identical output across runs and involve no model call.

#### Scenario: Two runs
- **WHEN** `merge-threads.sh` runs twice on the same fixture threads
- **THEN** the sha256 of every emitted artifact is identical, and no gateway request is made

### Requirement: Existing stage validators pass unchanged
Merged output SHALL satisfy the stage 02, 03, and 04 validators in `run-research.sh` with no change to those validators.

#### Scenario: Contract preserved
- **WHEN** merged artifacts are validated by the unmodified driver
- **THEN** stages 02, 03, and 04 validate, and `tests/driver-contract.sh` still passes its full assertion count

### Requirement: One document, one citation number
WHEN several threads cite the same canonical URL, THEN the merged report SHALL carry one citation number for it, recorded in `citation-map.json`.

#### Scenario: Overlapping sources
- **WHEN** three fixture threads cite the same URL under different local markers
- **THEN** all three resolve to one global number and `citation-map.json` records the mapping

### Requirement: The merge enforces the no-search rule
WHEN a dossier cites a source absent from its own thread's `sources.json`, THEN the merge SHALL fail with a CRITICAL finding.

#### Scenario: Director leakage
- **WHEN** a fixture dossier cites an unlisted source
- **THEN** the merge exits non-zero naming the thread, the source, and the rule

## Constraints

- Implementation-first, integration-only evidence (CLAUDE.md highest-precedence policy): finish the coherent edit batch, then run the smallest full-integration gate named in `verification.md`. No unit tests, mocks, or snapshots count as delivery evidence.
- One Cargo build machine-wide at a time. Check `pgrep -x cargo` before any `cargo` command.
- Local-only validation: no GitHub Actions run is evidence.
- Verification labels are `verified | unverified | blocked | inferred` on claims and `PASS | PASS WITH NOTES | BLOCKED` on provenance. A gate that could not run is recorded BLOCKED with the reason.
- Scripts that launchd may invoke stay bash 3.2 compatible (constraint C-05): no `mapfile`, no `declare -A`; test under `/bin/bash`.
- Every script touched keeps `set -euo pipefail` semantics and non-zero exit on failure; no silent `|| true` on a gate.
- The pack never depends on the Companion or any extension (integration contract rule 1).
- **Stage numbers 02-04 and 09 do not change.** This phase refactors how those stages are produced, never the contract.
- **Onyx is licensed NOASSERTION** (analysis D-08). Mechanisms may be reimplemented freely; Onyx prompt strings must NOT be copied verbatim without a licence check.
- Constraint C-01 (generated artifacts): a change that edits a `SKILL.md` regenerates the skills index and passes `npm run check:skills-index` in the same change.
- **C-01, `run-research.sh` is a distributed artifact.** This change edits it, which desyncs `dist/plugins/claude/prometheus-skill-pack/skills/deep-research/scripts/run-research.sh`. Regenerate the distribution and pass `npm run check:distribution` and `npm run validate:codex` **in this change** — there is no end-of-phase reconciliation, because a desynced driver is defect D-B itself.
- **C-05 binds here.** `run-research.sh` is the script the launchd daemon drives: no `mapfile`, no `declare -A`, and the driver suite runs under `/bin/bash` 3.2 as well as bash 5.

## Open Questions

- Canonical URL normalisation currently lives in stage 02's dedupe step; whether to extract it into the shared lib so the merge and stage 02 cannot drift (default: extract, since two copies of a normalisation rule is exactly how they diverge).
