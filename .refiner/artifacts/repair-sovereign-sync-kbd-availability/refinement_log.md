# Refinement log — repair-sovereign-sync-kbd-availability

Artifact-refiner deterministic validation for the completed sovereign-sync/KBD availability repair.

## Changed surfaces

- `substrate/kbd-runtime/src/lib.rs` — canonical managed signer discovery with custom-root isolation.
- `substrate/sovereign-sync/src/{rest_api.rs,kbd_control.rs}` — partial project-authority availability and concrete failure diagnostics.
- `tools/prometheus-cli/` — Unix-socket control transport, KBD/doctor integration, explicit embedded-runtime diagnostics, dependencies, and lockfile.
- `skills/process/kbd-process-orchestrator/{skills/kbd-new-child/kbd-new-child.sh,shared/lib/stage-gate.sh}` — runtime child-label and fully-qualified child-ID compatibility fixes.
- `dist/plugins/{claude,codex}/prometheus-skill-pack/` — generator-owned copies refreshed from source.
- OpenSpec and KBD child artifacts — behavioral contract, execution evidence, and lifecycle handoffs.

## Constraint check (`.kbd-orchestrator/constraints.md`)

| Constraint | Status | Evidence |
|---|---|---|
| C-01 generated artifacts in sync | PASS | `npm run build:codex` refreshed both distributions and `npm run validate:codex` passed. |
| C-02 no committed secrets | PASS | No key material is written or logged; signer code reads the existing user-local mode-0600 key path. The staged-diff secret scan is repeated immediately before commit. |
| C-03 docs updated with surface changes | N/A | No Codex manifest, marketplace schema, MCP server, hook registration, or installer contract changed. |
| C-04 generators stay idempotent | PASS | Consecutive generator runs produced the identical combined SHA-256 `0b4ab45771b497d3724a1a3da758080581800eb76ab2b8b9e1035eec9d8b214e`. |
| C-05 bash 3.2 compatibility | PASS / N/A | Both edited shell scripts pass `/bin/bash -n`; launchd directly invokes the Rust binary, so neither script is launchd-reachable. |

## Local evidence

- Prometheus CLI: formatting and warnings-denied Clippy passed; 20 unit, 10 doctor, and 3 KBD integration tests passed, including embedded-runtime hosting and reachable-HTTP-failure diagnostic regressions; release build passed.
- kbd-runtime: formatting and warnings-denied Clippy passed; 74 tests passed and 6 explicit operator proofs were ignored; release build passed.
- sovereign-sync: formatting and warnings-denied Clippy passed; 47 unit, 5 domain-sync, and 22 integration tests passed, including the all-authorities-failed startup gate; release build passed.
- Generated-distribution validation, protected-test integrity, Bash syntax checks, and `git diff --check` passed locally.
- Installed release binaries survived two forced launchd restarts; Unix health returned 200 and signed canonical mutations remained accepted while three stale registrations were isolated.

## Refine-validate report

```text
Schema:       N/A — this is a KBD code-change artifact, not a PMPO dist package
Files:        PASS — implementation, OpenSpec, child-phase, and handoff artifacts exist and are non-empty
Constraints:  PASS — C-01 through C-05 have explicit dispositions with no blocker
Consistency:  PASS — the generated distribution, canonical KBD ledger, OpenSpec checklist, and installed runtime evidence agree
Overall:      PASS — proceed to independent diff-mode adversarial review
```

## Verdict

PASS — deterministic QA found no unresolved constraint violation. The final distinct-model review passed with zero critical findings; its transport-race, reachable-HTTP-classification, and non-Unix import warnings were corrected. Its managed-key warning was verified as already covered by the shared `load_device_key` symlink and mode validator.
