# Verification — change-rah-003-stage-contract-driver

Repository: `prometheus-skill-pack`
Depends on: `change-rah-002-research-package-contract`

## Acceptance criteria

- `tests/driver-contract.sh` passes all six scenarios against the real driver with the fixture runner, and each scenario asserts on files left on disk, not on driver stdout alone.
- The hooks-fired scenario shows one marker per hook firing point in order; a run with no unresolved contradiction shows no on-contradiction marker (negative control, recorded).
- `run-research.sh --check-tools` exits 0 and reports each capability as present or absent.
- Running the driver with bash 3.2 (`/bin/bash`) on the full-run scenario succeeds (C-05).
- `openspec validate` passes.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run (for example, another Cargo build is active) is recorded BLOCKED with the reason, never skipped silently.

```verify
bash skills/research/deep-research/tests/driver-contract.sh
bash skills/research/deep-research/tests/driver-contract.sh --scenario hooks-fired
/bin/bash skills/research/deep-research/tests/driver-contract.sh --scenario full-run
bash skills/research/deep-research/scripts/run-research.sh --check-tools
openspec validate --specs
```

## Evidence

See the Evidence table below. Record the exact commands, their outputs, the commit hash, and the date. Each gate's outcome is recorded as the command's real result (exit 0 or the failure text); a failing gate must be fixed before the change completes. The change's verification verdict uses the provenance enum only: PASS, PASS WITH NOTES, or BLOCKED (a gate that could not run, with the reason). No hosted CI.

## Evidence

Run locally 2026-09-05 in `prometheus-skill-pack`, branch `feat/cpc-001-002-integration-contract`. No hosted CI.

| Gate | Result |
|---|---|
| `tests/driver-contract.sh` (bash 5) | PASS, 61 assertions, 0 failed: full-run, shallow-depth, resume-from-04, failing-05, invalid-05-blocks-06, direct-scale, hooks-fired (+ negative control), hook-failure (stage 03 and stage 10 with recovery), checkpoint-mode |
| `tests/driver-contract.sh` (`/bin/bash` 3.2) | PASS, 61 assertions |
| `--scenario hooks-fired` | PASS: markers `pre-research, post-stage 01..06, on-contradiction 06, post-stage 07..10, post-export` in order; no on-contradiction marker without an unresolved contradiction |
| `run-research.sh --check-tools` | exit 0; reports search NONE (no keys on this machine), python3 and jq present, gateway `http://localhost:8181/v1`, surreal-memory absent, runner none, output root |
| `openspec validate --specs` | 30 passed, 0 failed (bare `openspec validate` is interactive and was replaced in every remaining change's verify block) |
| Slug library under `/bin/bash` | five-word cap, filler words dropped, `package_id_is_valid` accepts five words and rejects six |
| Grounding script copies | three byte-identical copies, library branch and inline fallback both produce `cell-respiration` |
| Thin export probe | `completed_at` named among defaulted fields |
| Driver `verify` (per-task checks plus the block above) | PASS |

QA: C-01 no generator input touched (SKILL.md feeds only the skills index; reconciliation rah-011); C-02 none; C-05 no `mapfile`/`declare -A` in any touched script, suite run under `/bin/bash`; scope extended to the three hooks, the exporter, SKILL.md, and the two learn-script copies that the round-1 fixes touched. Adversarial review: two rounds, disposition in spec.md.

Verdict: PASS WITH NOTES (round-2 findings fixed and covered by scenarios, not re-vetted).
