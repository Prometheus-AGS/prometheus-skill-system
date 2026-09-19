# change-rah-010-source-scoring-and-graph

**Title:** Score sources on the documented rubric with weight renormalisation and sensitivity, and build a graph with content-addressed claims and contradicts edges
**Repository:** `prometheus-skill-pack`
**Phase:** research-agent-hardening
**Goal:** G5
**Depends on:** `change-rah-005-claim-labels-and-provenance`
**Backend:** native-kbd

## Why

`verify-sources.sh` implements one of five rubric dimensions by domain string match and does not sort; `build-graph.sh` emits `cites` only in a shape that is not the spec's; `detect-contradictions.sh` is numeric-regex only (assessment G5, survey). Analysis D-11, cand-003, cand-010.

## What Changes

- Add `scripts/score-sources.py`: scores the five documented dimensions where evidence is present, drops absent signals from the weight denominator, records `appliedWeights` per source, sorts, and writes `sensitivity.json` classifying each rank `stable | sensitive | volatile` under four alternate weight vectors; `verify-sources.sh` delegates to it; stage 05 and the package spec document `sensitivity.json`.
- Rewrite `build-graph.sh` to emit the spec's `{topics, claims, relations}` shape with `claim:sha256(scope:normalised_text)[:16]` ids, label per claim, `cites` and `contradicts` relations, and status escalation on merge.
- Extend `detect-contradictions.sh`: keep the numeric path, add a semantic path through `kbd_complete` with the critic role, label results `inferred` when the judge decided and `blocked` when no gateway was reachable.
- Fixture tests `tests/scoring-graph.sh`: a registry with missing metadata scores without zeros, sensitivity classifies a known-volatile item, duplicate claim text across two artifacts collapses to one id, a contradicting pair yields a `contradicts` relation, and the semantic path without a gateway yields `blocked`.

## Scope

Files this change may create, edit, or delete (tasks.json `files` is the per-task view):

- `skills/research/deep-research/scripts/score-sources.py`
- `skills/research/deep-research/scripts/verify-sources.sh`
- `skills/research/deep-research/scripts/build-graph.sh`
- `skills/research/deep-research/scripts/detect-contradictions.sh`
- `skills/research/deep-research/skills/stage-05-verify/SKILL.md`
- `skills/research/deep-research/skills/stage-06-resolve/SKILL.md`
- `skills/research/deep-research/skills/stage-07-graph/SKILL.md`
- `skills/research/deep-research/references/research-package-spec.md`
- `skills/research/deep-research/references/schemas/research-manifest.schema.json`
- `skills/research/deep-research/tests/scoring-graph.sh`
- `skills/research/deep-research/tests/fixtures/registry-sparse.json`
- `skills/research/deep-research/tests/fixtures/claims-duplicate.json`

Scope amendment (recorded during apply): the drift-check acceptance criterion
needs a package that carries `sensitivity.json`, so
`tests/fixtures/package-labelled/sensitivity.json` (produced by the scorer
from that fixture's registry) was added and its `manifest.json` gained
`files.sensitivity`. Both belong to the rah-005 fixture; no other fixture
file changed.

Field-name note: the requirement below names `appliedWeights`, Feynman CLI's
TypeScript spelling. Every other field in the package contract is
`snake_case` (`credibility_score`, `verified_sources`, `stages_completed`), so
the scorer writes `applied_weights` and the package spec, stage 05, and the
test say the same. The camelCase form in the requirement is read as the
concept, not the key.

## Capabilities

- `research-pipeline-execution (scoring and graph requirements added)`

## ADDED Requirements

### Requirement: Missing evidence is not a zero
WHEN a source lacks evidence for a dimension, THEN that dimension is excluded from its denominator and `appliedWeights` records the renormalised vector.

#### Scenario: Sparse registry
- **WHEN** a source has no author and no date
- **THEN** its score is computed from the remaining dimensions and `appliedWeights` sums to 1 over those

### Requirement: Claims are content-addressed
Two claims with the same normalised text in the same scope SHALL share one id and the higher status.

#### Scenario: Duplicate
- **WHEN** the same sentence appears in two source chunks with statuses unverified and verified
- **THEN** one claim node exists with status verified

### Requirement: Contradictions are edges
A resolved contradiction SHALL appear as a `contradicts` relation between two claim ids.

#### Scenario: Pair
- **WHEN** the fixture has a contradicting pair
- **THEN** `graph.json` contains one `contradicts` relation between their ids

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

- Which four alternate weight vectors to ship (default: balanced, authority-heavy, recency-heavy, methodology-heavy, matching Feynman CLI's profile idea).

## Unresolved review findings

Adversarial review (k3 judge via gateway, producer `claude-fable-5-1`, `--mode diff` on `files.txt`), two rounds, receipts in `.kbd-orchestrator/phases/research-agent-hardening/review/change-rah-010-source-scoring-and-graph/` (`round1/` and the round-2 `findings.json`).

### Round 1 — BLOCK (2 CRITICAL, 5 WARNING, 4 SUGGESTION)

| # | Finding | Disposition |
|---|---|---|
| C1 | `parse_date` sliced by format length, so year-only and year-month dates silently lost the recency signal | **Accepted, fixed.** Explicit rendered widths per format; a `published: "2024"` source added to the sparse fixture and the suite asserts its recency signal is available and resolves to 2024-01-01. |
| C2 | `research-graph.schema.json` was not updated for `claim-<16 hex>` ids and the required claim fields | **Rejected.** That schema (rah-005, outside this change's `files.txt`) already accepts both id forms (`^claim-[0-9a-f]{16}$\|^claim-[0-9]{3,}$`) and requires `label`, `critical`, `confidence`, `sources`, `contradicts`; the suite validates the new `graph.json` against it with python `jsonschema` and passes. |
| W1 | `files.sensitivity` is stated but `export-package.sh` is not in the change | **Rejected.** `export-package.sh` (rah-002) already discovers `sensitivity.json` and writes `files.sensitivity`; the package scenario asserts the emitted manifest carries it. |
| W2 | The labelled fixture appears to contain only `manifest.json` and `sensitivity.json` | **Rejected.** The fixture's other files came from rah-005 and are outside this diff; the drift check on that package passes. |
| W3 | A judge reply that cannot be parsed was silently treated as "no contradiction" | **Accepted, fixed.** Such a pair is recorded `blocked` with the parse failure named and `detection.semantic.status` is `blocked`; a garbage-judge scenario asserts it. |
| W4 | A `verified` label without evidence kept its label with a synthetic evidence string | **Accepted, fixed.** Downgraded to `unverified` (confidence capped at 0.5) with the reason recorded; the duplicate scenario asserts it. |
| W5 | The manifest schema accepted `format_version` 1.x although the prose says 1.x does not validate | **Accepted, fixed.** Pattern tightened to `^2\.`; both static fixtures and the contract-mode check still pass. |
| S1 | Stage 05 example arithmetic (72 vs 71) | Fixed to 71. |
| S2 | "beside credibility.json" wording vs the package-root location | Fixed. |
| S3 | `jq` at the end of an `&&` chain aborts under `set -e` on a corrupt checkpoint | Fixed in both scripts with a guarded `if`. |
| S4 | `credibility: null` for registry-only claims vs the spec example | **Documented, not defaulted.** A source that was never scored has no score; fabricating 50 would be a false signal. The spec's `contradictions.json` section now says the field is nullable. |

### Round 2 — BLOCK (3 CRITICAL); cap reached, fixed and not re-vetted

| # | Finding | Disposition |
|---|---|---|
| C1 | The scorer writes `applied_weights`; the requirement text says `appliedWeights` | **Deliberate deviation, recorded in Scope.** The package contract is `snake_case` throughout; the camelCase spelling in the requirement is Feynman CLI's and names the concept. The scorer, the spec, stage 05, and the test agree on `applied_weights`. |
| C2 | A contradictions entry whose explicit claim id is not in the graph dereferences a missing key | **Accepted, fixed.** An entry's id is honoured only when the graph has it; otherwise the claim is readdressed by content and the relation uses the content-addressed id. Scenario: legacy `claim-001` and a foreign hash id produce one `contradicts` relation between the content-addressed ids and no synthetic duplicate claim. |
| C3 | `package-labelled/manifest.json` and `sensitivity.json` are outside the declared scope | **Accepted, recorded.** Scope amendment added: the drift-check acceptance criterion needs a package carrying `sensitivity.json`. |

Round-2 fixes are covered by the suite (45 assertions on bash 5 and 3.2) but were not seen by the judge.
