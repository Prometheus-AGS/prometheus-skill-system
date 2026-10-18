# Prometheus 1.13.0 local release evidence

This directory records local evidence for the 1.13.0 release. GitHub workflow
output is not certification evidence.

## What ships

The KBD phase readiness contract: adopted exit contracts, the Plan→Execute
readiness gate, and the terminal evidence-bound closure evaluator with the
budget predicate (missing accounting is `BLOCKED`). See
[CHANGELOG.md](../../CHANGELOG.md) and
[site/docs/kbd/phase-readiness-contract.md](../../site/docs/kbd/phase-readiness-contract.md).

## Candidate qualification (preserved)

The feature was qualified for controlled use as commits `ebc8f687` + `9a7c4bbf`
before this release. The full qualification record — capability claim,
checklist C1–C8, payload digests, demonstrated installed-lifecycle scope, and
known limitations — is preserved intact at
[`docs/releases/1.12.2-candidate-qualification.md`](../1.12.2-candidate-qualification.md).
Nothing in this release replaces or rewrites that history.

Evidence carried forward as still applicable to the final release content:

- Contract lineage v1 `3f471d2e` ← v2 `248c84d3` ← v3 `982855ed` ← v4 `4b3c3321`
  and closures v3 `2b18d0f8` / v4 `b0ed035e` (canonical decisions).
- Boundary suite 37/37, gate `a3191be3`; `validate:strict` exit 0.
- Distribution `--check` exit 0 with the `budget-consumption-missing` marker in
  both payloads; scratch-installed lifecycle demonstration (READY → gate pass →
  CLOSED → missing-accounting BLOCKED → bound replay ALREADY-CLOSED) with
  disclosed `--status-file` / `--decision-sink` / `KBD_READINESS_STATUS_FILE`
  seams.

## Known limitations (unchanged from qualification)

- Shell-layer enforcement only; the compiled CLI does not invoke the gate or
  evaluator; direct state manipulation bypasses them.
- Budget consumption is self-reported, not independent telemetry.
- #184 (successor corpus) and #185 (host-side enforcement) remain open.
