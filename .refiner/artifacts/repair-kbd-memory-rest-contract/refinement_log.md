# Refinement log — repair-kbd-memory-rest-contract

Artifact-refiner deterministic QA for the completed KBD memory REST contract repair.

## Specification

- Artifact type: code and owning content references
- Content type: `direct:code`
- Target: KBD shell memory discovery, lifecycle writes, recall, and documentation agree with the installed surreal-memory 1.7.0 REST contract without blocking orchestration.
- Blocking criteria: canonical routes and payloads; explicit-first discovery; deterministic recall ordering; bounded fail-open behavior; Bash 3.2 compatibility; local full-integration evidence; no stale owning documentation.

## Changed surfaces

- `shared/lib/memory.sh` — explicit/configured/default discovery, origin normalization, bounded `/health` probe, and process cache.
- `shared/lib/memory-log.sh` — canonical `/api/v1/entities` write with one compact string observation and bounded fail-open diagnostics.
- `skills/kbd-memory-recall/kbd-memory-recall.sh` — canonical entity search plus deterministic same-project/token/recency ranking and atomic digest outcomes.
- `shared/lib/tests/test-memory.sh` — nine production-entrypoint integration scenarios across real loopback HTTP and filesystem boundaries.
- Recall, retention, integration, orchestrator, and OpenSpec execution documentation.

## Constraint check (`.kbd-orchestrator/constraints.md`)

| Constraint | Status | Evidence |
|---|---|---|
| C-01 generated artifacts in sync | IN-PHASE BATCH / OPEN | The parent change `reconcile-kbd-control-plane-projections` is the named owner for deterministic double generation, tracked-hash identity, `validate:codex`, and installed-surface parity before parent-phase certification or commit/push. This archived source change does not claim distribution certification. |
| C-02 no committed secrets | PASS | Touched code, tests, and documentation contain only loopback fixture endpoints and synthetic identifiers; no key, token, password, or private-key material was introduced. |
| C-03 docs updated with surface changes | PASS | The recall skill, retention contract, integration reference, and owning orchestrator summary all describe the canonical routes, schema, precedence, and failure semantics. Focused stale-contract search returned no matches. |
| C-04 generators stay idempotent | N/A | No generator behavior changed. Parent reconciliation retains the required two-run distribution hash proof. |
| C-05 Bash 3.2 compatibility | PASS | `/bin/bash` 3.2.57 parsed all four touched shell files, and executed the complete integration suite successfully. |

## Deterministic evidence

- `/bin/bash -n` passed for `memory.sh`, `memory-log.sh`, `kbd-memory-recall.sh`, and `test-memory.sh`.
- `/bin/bash .../test-memory.sh` passed 9/9 full-integration scenarios.
- Strict validation passed for the orchestrator and its 22 subskills with no errors.
- `git diff --check` passed after the final evidence update.
- The phase-wide canonical integration gate was intentionally deferred: it correctly refused a change-level receipt while three parent changes remain incomplete and created no unresolved blocker.

## Reflection

- Correctness: PASS — live route/payload assumptions are sourced from the server contract and exercised across HTTP.
- Failure recovery: PASS — unreachable, HTTP write failure, reachable empty, and invalid response remain distinct and fail open.
- Regression check: PASS — legacy `mcpEndpoint` and object-observation reads remain supported while obsolete writes/routes were removed.
- Documentation: PASS — implementation and owning references agree.
- Blocking violations: 0.

## Verdict

PASS — the change converged in one QA iteration. Proceed to OpenSpec verification and archive; defer generated/install parity and the canonical phase-wide certification receipt to `reconcile-kbd-control-plane-projections` as planned.
