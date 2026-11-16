# Production integration acceptance

Use this contract for every behavior under specification, in native-kbd
`verification.md` and OpenSpec scenario/verification artifacts. Fill each field
with project-specific requirements; placeholders are not acceptance evidence.
Keep backend task IDs unchanged and link each behavior to its owning task and
the parent phase's final integration gate.

## Behavior: <concrete user operation>

| Field | Required declaration |
|---|---|
| Canonical identity | <phase / change / stable backend task ID> |
| Source identity | <repository, revision or candidate tree, generated package identity> |
| Production entry point | <shipped CLI/installer/server/UI path and exact invocation or user operation> |
| Real collaborators | <actual services, subprocesses, stores and installed artifacts used by that entry point> |
| Boundary exercised | <process, filesystem, network, database, protocol or UI boundary; name endpoints/paths> |
| Observable result | <externally observable state/output, including persistence or restart when relevant> |
| Negative control | <real invalid/absent/denied/conflicting input and expected externally observable rejection; no replacement mock> |
| Isolation | <scratch HOME, CODEX_HOME, CORTEX_DATA_DIR, plugin/queue/database roots, service ports and cleanup owner> |
| Prerequisites | <real tools/services, protected approvals and required source/packaging completion> |
| Final local gate | <exact command, argv, cwd and environment references; gate runs only at the completed-production boundary> |
| Evidence | <local receipt/log path, source identity, exit code and actual observed result; pending until run> |
| Limitations | <unsupported platforms/configurations and any behavior this gate cannot establish> |

### Execution boundary

Complete the entire parent phase's coherent production implementation before
authoring, modifying or executing tests. Specify acceptance requirements now;
write executable scenarios and run the smallest full integration batch only at
the final boundary. Fix failures in coherent batches and rerun the applicable
confirming final gate before commit/push/release. One machine-wide Cargo/rustc
build at a time; use existing repository build rules and isolated targets.

Exercise the installed/generated deliverable when packaging or installation is
the behavior under specification. Calling an internal helper directly does not
establish installer dispatch, executable modes or installed-generation behavior.
Use actual collaborating processes and stores; unit tests, structural assertions,
mock-only probes and snapshots are supplemental and cannot satisfy acceptance.

Do not touch real harness/memory roots or live service ports during tests. Declare
every test resource and cleanup boundary. Missing services, tools, approvals or
usable isolation are BLOCKED, not skipped success. Exit 2 is always BLOCKED;
exit 0 counts only when the declared observable and negative control were actually
exercised. Record source implementation, integration acceptance, publication and
installed operation separately. Never fabricate receipts or substitute hosted CI.

## Scenario contract fields (adopted-contract.v2.schema.json)

<!-- generated:contract-fields:start — scripts/generate-contract-templates.mjs; do not edit by hand -->
Scenario ID pattern: `^[A-Z][A-Z0-9]*(-[A-Z0-9]+)*-[0-9]+$` (for example `SCN-AUTH-001`).

Contract (v2) required top-level fields: `contractVersion`, `kind`, `identity`, `digestNormalization`, `adoptionSource`, `decisionOwner`, `amendmentPolicy`, `releaseClaim`, `statusVocabulary`, `scenarioMatrix`, `functionalScope`, `resilienceScope`, `exclusions`, `requiredSafetyChecks`, `budgets`, `evidenceInvalidation`, `workerRoutes`, `boundaryPacket`, `status`, `calibration`, `method`, `closure`.

| Scenario field | Required | Shape | Meaning |
|---|---|---|---|
| `id` | yes | string |  |
| `name` | yes | string |  |
| `ownerTask` | yes | string | Owning task ID(s) from tasks.json, e.g. "T4+T5+T7". |
| `productionEntry` | yes | string | Shipped script/CLI path and exact invocation. |
| `realCollaborators` | yes | array |  |
| `oracle` | yes | object (observation, expectedTruthSource, comparator) |  |
| `negativeControl` | yes | object (input, expectedRejection, commandRef) |  |
| `fixtures` | yes | string | Fixture identity: scratch roots, content digests, isolation guarantees. |
| `evidence` | yes | string | Required recorded evidence: argv, exit codes, stdout/stderr excerpts, digests. |
| `classification` | no | string | Optional: marks delivery/packaging-only validation so its failure is never cascaded into product-failure claims. |
| `actor` | yes | string | Who needs the behavior (VLBV 01). |
| `usefulOutcome` | yes | string | The observable value delivered (VLBV 01). |
| `sourceRequirement` | yes | string | OpenSpec requirement reference (openspec:<capability>#<requirement>), another backend reference, or an explicit 'assumption: <text>' (VLBV 01). |
| `reasonNow` | yes | string | Why this scenario matters in this phase (VLBV 01). |
| `likelihood` | yes | string | Sourced estimate, a qualitative judgment, or the literal UNKNOWN. Never an invented probability. |
| `mandatory` | yes | boolean | A mandatory gate, as opposed to ordinary priority (VLBV 03). |
| `scopeRefs` | no | array | functionalScope items (exact text) this scenario covers. |
| `substitutedBoundaries` | yes | array | Boundaries replaced by a stub or double; their evidence can never be promoted to a real claim (VLBV 08). |
| `limitations` | yes | array | What this scenario does not establish (VLBV 16). |
| `invalidationKeys` | yes | array | Source/fixture/config/oracle identities whose change makes only this scenario's evidence stale (VLBV 15). |
| `restoration` | yes | string | Restoration and cleanup this scenario requires before closure (VLBV 14). |
| `nondeterministic` | no | boolean |  |
| `stochastic` | no | object (samples, grader, stoppingRule) | Declared before any result exists (VLBV 21). Required when nondeterministic is true. |

Status vocabulary: `PASS` / `FAIL` / `BLOCKED` / `NOT_EXERCISED`.
Boundary packet command kinds: `build`, `run`, `scenario`, `negativeControl`.
<!-- generated:contract-fields:end -->

Fixture/permission readiness, owner task and per-scenario invalidation keys are
required fields above; a scenario whose prerequisite is unavailable is
`NOT_EXERCISED`, never silently dropped.

Status vocabulary (exactly four): PASS / FAIL (product: real oracle unmet) /
BLOCKED (verification: validator/env/fixture unavailable) / NOT_EXERCISED.
Record attempted/observed/evidence per run. Unknown is not success; unknown is
not observed failure. Delivery/packaging-only checks (e.g. strict skill
validation) must be marked as such; their failure never cascades into
product-failure claims for product scenarios.
