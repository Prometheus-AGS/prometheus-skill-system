# Verification — change-rah-011-integration-evidence-and-docs

Repository: `prometheus-skill-pack`
Depends on: `change-rah-004-daemon-job-execution`, `change-rah-006-agent-duties-and-report-review`, `change-rah-009-learner-model-write-paths-and-fsrs`, `change-rah-010-source-scoring-and-graph`

## Acceptance criteria

- The evidence document records a real driver run and a real daemon run with package paths, sidecar verdicts, and check results.
- All fixture suites, `validate:strict`, `check:skills-index`, and `openspec validate` pass locally; Cargo suites run only when no other build is active.
- The C-01 reconciliation checks `check:distribution`, `check-harness-adapters.js`, and `check:services-manifest` pass, proving no generated surface other than the skills index changed.
- No goal is marked MET in the evidence document without a command output supporting it.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run (for example, another Cargo build is active) is recorded BLOCKED with the reason, never skipped silently.

```verify
bash skills/research/deep-research/tests/driver-contract.sh
bash skills/research/deep-research/tests/scoring-graph.sh
bash skills/learn/feynman-loop/tests/learn-coherence.sh
bash skills/process/adversarial-review/tests/run-fixture-suite.sh
test -z "$(pgrep -x cargo)" && (cd substrate/prometheus-research && cargo test -p prometheus-research --test job_execution --test job_lifecycle --test mcp_tools --test sse_stream) && (cd substrate/learner-model && cargo test -p learner-model --test rpc_roundtrip)
npm run validate:strict && npm run check:skills-index && openspec validate --specs
npm run validate:codex
npm run check:distribution && node scripts/check-harness-adapters.js && npm run check:services-manifest
```

## Evidence

See `docs/research-agent-hardening-evidence.md` — sections 1 (driver run), 2 (daemon run), and 3 (final local certification), all recorded 2026-09-08 on this machine with no hosted CI.

## Verdict

**PASS WITH NOTES.**

- Note 1: two deployment defects were found by the evidence run and are recorded as open carry-forwards (D-A launchd `PATH`, D-B stale installed driver generation). Both are install-surface, outside this change's file scope, and neither is a defect in the daemon's own logic — in both cases it refused to report a success it had not achieved.
- Note 2: the adversarial review's round-1 findings included two real gaps in this change's own evidence (unrun Cargo suites, an unwritten site page). Both were closed before this verdict; the dispositions are in spec.md.
- Note 3: the eval re-baseline inherited from rah-008 remains BLOCKED on rah-007's operator review and is not claimed here.
