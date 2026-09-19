# Verification — change-rah-005-claim-labels-and-provenance

Repository: `prometheus-skill-pack`
Depends on: `change-rah-003-stage-contract-driver`

## Acceptance criteria

- `check-research-package.sh --package <labelled fixture>` exits 0; the same command against the negative fixture (one unlabelled claim) exits non-zero naming the claim.
- `write-artifact.sh` refuses an artifact without `verification` and accepts the documented shape.
- `npm run validate:strict` passes for deep-research, feynman-loop, and learn-grade.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run (for example, another Cargo build is active) is recorded BLOCKED with the reason, never skipped silently.

```verify
bash skills/research/deep-research/scripts/check-research-package.sh --package skills/research/deep-research/tests/fixtures/package-labelled
bash skills/research/deep-research/scripts/check-research-package.sh --package skills/research/deep-research/tests/fixtures/package-partial
! bash skills/research/deep-research/scripts/check-research-package.sh --package skills/research/deep-research/tests/fixtures/package-unlabelled
CHECK_RESEARCH_PACKAGE_NO_PYTHON=1 bash skills/research/deep-research/scripts/check-research-package.sh --package skills/research/deep-research/tests/fixtures/package-partial
! CHECK_RESEARCH_PACKAGE_NO_PYTHON=1 bash skills/research/deep-research/scripts/check-research-package.sh --package skills/research/deep-research/tests/fixtures/package-unlabelled
! bash skills/learn/feynman-loop/scripts/write-artifact.sh --goal-id fixture --artifact-json '{"artifact_id":"x","concept_id":"c"}'
npm run validate:strict skills/research/deep-research && npm run validate:strict skills/learn/feynman-loop && npm run validate:strict skills/learn/learn-grade
```

## Evidence

Run 2026-09-06 on the uncommitted working tree (branch `feat/cpc-001-002-integration-contract`, base `cfbc262`); no hosted CI.

| Gate | Command | Result |
|---|---|---|
| Labelled fixture | `check-research-package.sh --package tests/fixtures/package-labelled` | exit 0, `RESULT: PASS` (graph validates via jsonschema, 2 claims; citations/contradictions labels valid; `verification_status verified agrees with the derivation rule`) |
| Partial fixture (blocked critical claim) | same, `--package tests/fixtures/package-partial` | exit 0, `RESULT: PASS`; `verification_status partial agrees with the derivation rule` (python and jq paths) |
| Unlabelled fixture | same, `--package tests/fixtures/package-unlabelled` | exit 1, `FAIL graph.json claims[1] (claim-002): 'label' is a required property` |
| Corrupt graph.json | labelled copy with `{not json` as graph.json | exit 1, `FAIL graph.json is not valid JSON: ...` (round-2 C1) |
| NOTE propagation | labelled copy with legacy `{nodes, edges}` graph | `RESULT: PASS WITH NOTES` (derivation falls back to credibility.json claims); same copy without credibility.json derives `unverified` and fails against declared `verified` (round-2 W1, W2) |
| Exporter fabrication | thin package with no graph/citations/contradictions | `defaulted: graph.json(empty), citations.json(empty), contradictions.json(empty), ...` (round-2 W3) |
| No-python path | both fixtures with `CHECK_RESEARCH_PACKAGE_NO_PYTHON=1`, also under `/bin/bash` 3.2 | labelled `PASS WITH NOTES` (jq manifest structure, jq label rules, jq derivation; report/provenance agreement and fresh export reported as NOTEs); unlabelled exit 1 `claims[1] (claim-002) missing label` |
| Contract mode | `check-research-package.sh` with and without python3 | `RESULT: PASS`; `PASS WITH NOTES` (awk block extraction, export step NOTEd) |
| Misconceptions rule | labelled copy with `misconceptions_absent: 0.0` in checkpoint, report, manifest | exit 1 in both python and jq paths: `declared 'verified' but derived 'partial'` |
| Artifact writer | refuse without `verification`/`provenance`; accept documented shape; refuse length mismatch; refuse bad label; refuse a bare-string entry | exit 1 / `{"ok":true,...}` / exit 1 / exit 1 / exit 1 with `{"ok":false,"error":"every verification entry must be an object {label, evidence}"}` |
| Exporter prerequisite | `export-package.sh` with python3 off `PATH` | exit 1, `{"error": "python3 is required"}` |
| Driver suite (rah-003) | `tests/driver-contract.sh` under bash 5 and `/bin/bash` 3.2 | `61 passed, 0 failed` both |
| Strict validation | `npm run validate:strict` deep-research, feynman-loop, learn-grade | PASS (pre-existing exclusion-clause warnings only) |
| Constraints | C-02 secrets grep, C-05 `mapfile`/`declare -A` grep over touched scripts | none |

QA: C-01 no generator input touched beyond SKILL.md files (skills-index reconciliation is rah-011); scope amendment recorded in spec.md for the three earlier-change files the fixtures required. Adversarial review: two rounds, disposition in spec.md.

Verdict: PASS WITH NOTES (round-2 findings fixed and covered by the probes above, not re-vetted by the judge; the no-python path checks less than the python path and says so in its NOTEs; the `<job_id>/` → `<package_id>/` rename outside this change's scope is owed to rah-011).
