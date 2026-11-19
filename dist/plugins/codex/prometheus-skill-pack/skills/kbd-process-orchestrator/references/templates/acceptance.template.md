# Production integration acceptance

## Outcome-based exit contract

Done requires the actual agreed user goal and value to be achieved, supported by relevant end-of-phase VLBV evidence against accepted business rules and all existing mandatory exit criteria. Deliver the necessary user guidance and access, verify the observable outcome, disclose limits and remaining risks, and record reflection. Review-ready means a coherent change is ready for maintainer assessment, not that the user outcome is already delivered. Preserve existing approval, release and verification boundaries; tests alone are not completion.

## Mandatory value and business-rule trace

Always think in terms of user goals and value—not in tests or assumptions.

Before selecting work or defining acceptance, state the real actor's intended
outcome and its value. In the existing accepted specification or behavior
contract, write plain-language business rules: conditions, expected outcome,
exact representative examples/evidence, exceptions, and unresolved assumptions.
Do not proceed on an unresolved assumption that changes the intended outcome.
Every adopted VLBV scenario must identify the rule it verifies through the
existing `sourceRequirement`, explain value in `usefulOutcome` and `reasonNow`,
and bind its oracle, controls and evidence to that outcome. A labelled assumption
is not a confirmed business rule; use existing owner calibration/adoption to
resolve it before claiming acceptance.

Include only reachable real user journeys, plausible supported failures,
material risks or required safeguards. Omit impossible/artificial cases and
outcomes nobody values; do not invent cases to satisfy counts. Positive and
negative controls remain required where the adopted contract requires them and
must materially distinguish the intended outcome from a false positive.

When a case fails, classify its impact on a reachable, material accepted outcome
before a repair loop. An out-of-scope test failure does not justify changing
proven infrastructure or blocking review. Record and route genuine scope/check
conflicts through existing governance; never silently delete, mask, or weaken a
required check or signed scope. Preserve current phase-end BDD/VLBV, build,
release and approval processes. Tests verify business value, not define it.

### Business rules to complete before solution/query/test design

| Rule/reference | Actor and conditions | Expected outcome/value | Exact evidence/examples | Exceptions | Unresolved assumptions | Scenario IDs |
|---|---|---|---|---|---|---|
| REQUIRED: existing accepted specification reference | REQUIRED | REQUIRED | REQUIRED | Explicit, or none | Explicit, or none | Trace every adopted case |


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
| `usefulOutcome` | yes | string | The reachable observable user outcome and business value, not a test pass (VLBV 01). |
| `sourceRequirement` | yes | string | Reference the plain-language business rule in the accepted specification (conditions, outcome, examples/evidence, exceptions and unknowns), using openspec:<capability>#<requirement> or another backend reference. An explicit 'assumption: <text>' remains unresolved until owner calibration/adoption; it is not a confirmed rule (VLBV 01). |
| `reasonNow` | yes | string | Why this reachable journey, plausible supported failure, material risk or required safeguard matters in this phase; no artificial cases for counts (VLBV 01). |
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
