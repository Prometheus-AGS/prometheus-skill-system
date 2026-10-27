---
name: value-scenario-verify
version: '1.0.0'
license: MIT
description: >
  Verify an adopted finite scenario set at the authorized complete value-phase
  boundary using existing real-path runners and canonical evidence/closure.
  Preserves oracle sensitivity, attempts, identity, budgets and cleanup; does
  not replace a runner or authorize early checks, repair loops or publication.
metadata:
  author: prometheus-skill-pack
  category: testing
  tags: [testing, verification, vlbv, evidence, kbd]
---

# Value scenario verification

Invoke `/value-scenario-verify <contract-reference>`. This instruction-only
procedure routes an adopted boundary packet through existing tools. It neither
intercepts arbitrary commands nor grants permissions the host does not provide.

Emit `Starting value-scenario-verify — <phase>` before work. Emit completion with
the actual scoped verdict only after observations/receipts have been recorded.
BLOCKED or NOT_EXERCISED is never a completed acceptance claim.

## Admit the boundary

Resolve current canonical phase/run, latest non-superseded contract binding and
selected method. Read its complete included value, exclusions, scenario IDs,
current candidate, assertions/oracle, evidence invalidation keys, budgets,
permissions, owned resources and cleanup. Reconcile uncertain previous effects
before any retry. Do not rebuild a retired method from old handoff or memory.

Every agreed production obligation must be complete in canonical state. Evidence
tasks do not reopen implementation, but unfinished production prevents running
this packet. A task/hour/commit/checkpoint/reviewer request, compiler problem or
renamed partial slice is not this boundary. Host/selected-method conflicts must
be resolved before dependent execution; never silently modify operator rules.

For KBD use its package-owned phase-readiness checker and current contract
adoption/readiness machinery. Readiness for implementation is not permission to
test. Require separate current authority for this actual boundary and its
effects. If the frozen contract needs material amendment (including its
implementationComplete field), use the existing signed amendment path; never
edit a bound snapshot or forge approval. Reuse approval already granted in scope.

## Run the smallest adequate real path

Resolve the installed runner capability and exact commands from the boundary
packet. Use existing JS/Rust BDD adapters or the adequate existing integration
entry point. No new universal runner, forced browser/video stack, substitute
mock boundary or broad unit suite is needed. Missing tool, permission, fixture,
identity or usable isolation is BLOCKED; do not silently skip an adopted case.

Freeze source/build/configuration/fixture/reference identities and observe the
actual process/network/filesystem/database/UI boundary. Assert the contract's
externally meaningful observation, with the expected truth's independent source.
Challenge each material oracle family safely with its agreed plausible false
positive/negative control. Disabling the actual assertion or observing the wrong
candidate must not yield PASS. A disconnected `return false` probe proves no
oracle sensitivity. Process exit zero or formatter success alone is insufficient.

## Retain every material attempt

Save original runner outputs and observations before deriving a verdict. Record:

- Contract/run/phase/scenario/assertion identity and source of expected truth.
- Source/build/configuration/fixture/reference digests and actual entry argv.
- What was attempted and actually observed, original evidence paths and time.
- Real versus substituted/unexercised boundaries and their limitations.
- Material failed and successful attempts, consumed-budget high-water marks,
  repair authorization/lineage, uncertain effects, restoration and cleanup.

| Situation | Disposition |
|---|---|
| Real exercised product oracle is unmet | FAIL, with the actual observation |
| Required assertion/obligation never exercised | NOT_EXERCISED; cannot pass |
| Wrong/stale candidate, configuration, oracle or evidence | BLOCKED; retain stale attempt |
| Runner, prerequisite, permission or isolation unavailable | BLOCKED, not observed product failure |
| Cleanup/restoration failed or effects uncertain | BLOCKED; reconcile before retry |
| Correct scoped observation and effective negative control, limits/cleanup met | PASS within that scope |

Exit 2 never passes. Do not overwrite an earlier failure with a later green
receipt. Budget exhaustion checkpoints BLOCKED without dropping mandatory
constraints. Authorized repair must finish its complete agreed implementation
before targeted reverification; preserve prior consumption and affected-evidence
invalidation. Do not install tools or start automatic repair loops on failure.

## Use the actual canonical writer

For KBD retain the current `boundary-results.json` contract consumed by
`shared/scripts/kbd-phase-close-eval.sh`: phase/projectId/runId/contractDigest,
sourceDigests, scenarios with id/status/attempted/observed/evidence, and
budgetConsumption with its source and measured/estimate/unknown classes. Keep
original assertions and negative-control receipts linked; do not fabricate fields
from exit status. Use only supported native writer/evaluator commands and
dispositions. An unsupported cancelled/deferred closure stays refused.

Report separately: authored implementation, measured contract acceptance,
demonstrated runtime value, unresolved obligations, release eligibility and
release authority. Required independent final review/certification remains
separate. No automatic new task/phase, publication, release or global install.
