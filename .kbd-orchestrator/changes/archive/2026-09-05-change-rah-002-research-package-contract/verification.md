# Verification — change-rah-002-research-package-contract

Repository: `prometheus-skill-pack`
Depends on: `change-rah-001-compile-baseline-and-timestamps`

## Acceptance criteria

- `check-research-package.sh` exits 0 against the SKILL.md example and a fresh export from a fixture package, and exits non-zero when one required field is removed from the example (negative control, recorded).
- No hook or daemon source resolves `~/.research-jobs/` except the legacy notice.
- The A2UI component names in SKILL.md equal the names registered in `a2ui/registry.rs`.
- `openspec validate` passes with the new capability present.
- `npm run validate:strict skills/research/deep-research` passes.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run (for example, another Cargo build is active) is recorded BLOCKED with the reason, never skipped silently.

```verify
bash skills/research/deep-research/scripts/check-research-package.sh
! grep -rn 'research-jobs' skills/research/deep-research/hooks
bash skills/research/deep-research/scripts/check-research-package.sh  # includes A2UI table parity with registry.rs
openspec validate
npm run validate:strict skills/research/deep-research
```

## Evidence

See the Evidence table below. Record the exact commands, their outputs, the commit hash, and the date. Each gate's outcome is recorded as the command's real result (exit 0 or the failure text); a failing gate must be fixed before the change completes. The change's verification verdict uses the provenance enum only: PASS, PASS WITH NOTES, or BLOCKED (a gate that could not run, with the reason). No hosted CI.

## Evidence

Run locally 2026-09-05 in `prometheus-skill-pack`, branch `feat/cpc-001-002-integration-contract`. No hosted CI.

| Gate | Result |
|---|---|
| `check-research-package.sh` (contract mode, python jsonschema) | PASS: SKILL.md example, spec example, fresh export all validate; examples byte-identical; A2UI table equals registry (8) |
| `check-research-package.sh` forced jq path | PASS WITH NOTES on valid inputs; FAIL naming missing key, bad enum, wrong type on the negative fixture |
| `check-research-package.sh --package` negative (field removed) | exit 1, field named |
| `export-package.sh` on a full fixture | manifest validates; counts equal the artifacts; `--ingest-palace` writes the marker |
| `export-package.sh` on an empty directory | manifest validates; every defaulted field named on stderr |
| post-export hook exit 7 | export exits 7 |
| pre-research package id | valid id creates the directory; six-word slug exits 1 |
| schema slug limit | five-word id accepted, six-word rejected |
| No `.research-jobs` in hooks; daemon hits all carry `legacy` | PASS |
| `cargo check -p prometheus-research` after the root move | PASS (idle machine); `--test job_lifecycle` 4 passed |
| `prometheus-research status <job>` with legacy root present | prints the legacy notice (29 entries) |
| `openspec validate` with `research-pipeline-execution` | 30 passed, 0 failed |
| `npm run validate:strict skills/research/deep-research` | PASS (inside driver verify) |
| bash 3.2 (`/bin/bash`) on the check script | PASS |
| Driver `verify` | PASS |

QA: constraint checklist C-01 (SKILL.md/skill.toml feed only the skills index; reconciliation rah-011; the deleted sovereign-sync plist and unit in the working tree belong to the companion phase, not this change), C-02 none, C-03 n/a, C-05 no `mapfile`/`declare -A`; scope-vs-touched clean once rah-001's uncommitted files were excluded from `files.txt` (checkpoint.rs carries both changes' edits because neither is committed). Adversarial review: two rounds, disposition in spec.md.

Verdict: PASS WITH NOTES (round-2 findings fixed and probed, not re-vetted).
