# Verification — change-rah-010-source-scoring-and-graph

Repository: `prometheus-skill-pack`
Depends on: `change-rah-005-claim-labels-and-provenance`

## Acceptance criteria

- `tests/scoring-graph.sh` passes all five scenarios against the real scripts.
- `check-research-package.sh` validates a package containing `sensitivity.json` and the new `graph.json` shape.
- The semantic contradiction path returns `blocked` with the gateway unreachable and `inferred` with it reachable (the latter recorded manually if the gateway is down at certification).

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run (for example, another Cargo build is active) is recorded BLOCKED with the reason, never skipped silently.

```verify
bash skills/research/deep-research/tests/scoring-graph.sh
bash skills/research/deep-research/scripts/check-research-package.sh --package skills/research/deep-research/tests/fixtures/package-labelled
```

## Evidence

Run 2026-09-06 on the uncommitted working tree (branch `feat/cpc-001-002-integration-contract`, base `cfbc262`); no hosted CI; no cargo involved.

| Gate | Command | Result |
|---|---|---|
| Scoring and graph suite | `tests/scoring-graph.sh` under bash 5 and `/bin/bash` 3.2 | `45 passed, 0 failed` both (41 before review; added: year-only date yields a recency signal, an evidence-free `verified` label is downgraded to `unverified`, an unparseable judge reply leaves the pair `blocked`, legacy or foreign claim ids in contradictions.json are readdressed by content). sparse: the blog with no author, date, or references scores above zero, its `applied_weights` lack the missing dimensions and sum to 1, the arxiv source ranks first, sorted descending, missing evidence flagged; sensitivity: four profiles, the recent low-authority medium source is not stable (recency-heavy lifts it), arxiv stable at rank 1, drivers cite the missing evidence; duplicate: one claim for the sentence shared by two sources (case, period, whitespace normalised), label `verified` with the verified evidence, sources union, id equals `claim-` + sha256(`scope:normalised`)[:16] computed independently, `cites` to both, schema-valid; contradicts: the 40 ms vs 400 ms pair yields one numeric entry and one `contradicts` relation whose ids match the detector's, both claims list each other, the topic groups both; semantic-blocked: with `LITER_LLM_BASE_URL` at a closed port every candidate pair is `blocked`/unresolved with the reason and the numeric entry stays `inferred`; semantic-inferred: with the fixture judge the pair is `inferred` with its reason and confidence; package: the labelled fixture with `sensitivity.json` and a package assembled from the new scripts both pass `check-research-package.sh --package` (derived `partial`, since the scorer labels claims `unverified`) |
| Semantic path, real gateway | `detect-contradictions.sh --semantic --max-pairs 4` with the words-only fixture, gateway `http://localhost:4000/v1`, critic `MiniMax-M3` | 7 s; `detection.semantic.status inferred`, 2 pairs checked; the build-time pair recorded `inferred`, topic `Index build time`, confidence 0.99, audit trail quotes the model's reason |
| Drift check | `check-research-package.sh --package tests/fixtures/package-labelled` | PASS (manifest lists `sensitivity.json`) |
| Regressions | `tests/driver-contract.sh`; adversarial-review `test-research-target.sh` | `128 passed`; `17 passed` |
| Strict validation | `npm run validate:strict skills/research/deep-research` | PASS |
| Constraints | C-02 secrets grep over the diff; C-05 grep over the four scripts and the test | 0 hits |

Scope note: `tests/fixtures/package-labelled/sensitivity.json` (new, produced by the scorer) and its `manifest.json` (`files.sensitivity`) were added so the drift-check gate exercises a package that carries the new artifact.

QA: C-01 no generator input touched beyond SKILL.md files (skills-index reconciliation is rah-011). Adversarial review: disposition in spec.md.

Verdict: PASS WITH NOTES (two review rounds disposed in spec.md, round-2 fixes covered by the suite but not re-vetted; the scorer labels every claim `unverified` by design, so a package built by the scripts alone derives `partial` until the verifier agent relabels).

_Pending execution._ Record the exact commands, their outputs, the commit hash, and the date. Each gate's outcome is recorded as the command's real result (exit 0 or the failure text); a failing gate must be fixed before the change completes. The change's verification verdict uses the provenance enum only: PASS, PASS WITH NOTES, or BLOCKED (a gate that could not run, with the reason). No hosted CI.
