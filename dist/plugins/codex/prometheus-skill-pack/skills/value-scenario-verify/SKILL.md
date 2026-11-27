---
name: value-scenario-verify
license: MIT
description: >
  Verify an adopted finite scenario set at the authorized complete value-phase
  boundary using existing real-path runners and canonical evidence/closure.
  Preserves oracle sensitivity, attempts, identity, budgets and cleanup; does
  not replace a runner or authorize early checks, repair loops or publication.
metadata:
  version: '1.3.0'
  author: prometheus-skill-pack
  category: testing
  tags: "testing, verification, vlbv, evidence, kbd"
---

# Value scenario verification

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


Invoke the `value-scenario-verify` skill with a contract reference. This
instruction-only procedure is steps 1–4 of the B4 sequence (admit, freeze, run the
boundary packet once, record) and the targeted rerun of an authorized B5 repair,
as defined once in `kbd-process-orchestrator/references/boundaries.md`. It routes
an adopted boundary packet through existing tools. It neither
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
test. Record the owner's boundary authorization before the first attempt with
`shared/scripts/kbd-boundary-authorize.mjs` (B4, or B5 for an authorized repair);
closure lists runner output older than that event. Require separate current authority for this actual boundary and its
effects. If the frozen contract needs material amendment (including its
implementationComplete field), use the existing signed amendment path; never
edit a bound snapshot or forge approval. Reuse approval already granted in scope.

## Run the smallest adequate real path

Resolve the installed runner capability and exact commands from the boundary
packet. Use existing JS/Rust BDD adapters or the adequate existing integration
entry point. No new universal runner, forced browser/video stack, substitute
mock boundary or broad unit suite is needed. Missing tool, permission, fixture,
identity or usable isolation is BLOCKED; do not silently skip an adopted case.

Invoke each packet command exactly as given, once: no appended commands, pipes,
`echo` wrappers or retries. The runner's receipt records its own exit status and
observations. Read receipts, results and evidence files with file-reading tools;
do not run additional commands to inspect, re-run or summarize them. Any command
outside the packet is outside this boundary's authority.

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
constraints. An authorized repair (B5) must finish its complete agreed implementation
before targeted reverification of only the affected scenarios and their negative
controls, consuming `budgets.repairAttempts`; preserve prior consumption and affected-evidence
invalidation. Do not install tools or start automatic repair loops on failure.

## Use the actual canonical writer

For KBD retain the current `boundary-results.json` contract consumed by
`shared/scripts/kbd-phase-close-eval.sh`: phase/projectId/runId/contractDigest,
sourceDigests, scenarios with id/status/attempted/observed/evidence,
`executedBy` (`harness`, `version`, `model` of the session that ran the packet), and
budgetConsumption with its source and measured/estimate/unknown classes. Keep
original assertions and negative-control receipts linked; do not fabricate fields
from exit status. Use only supported native writer/evaluator commands and
dispositions. An unsupported cancelled/deferred closure stays refused.

Optionally record evidence references, so a reader can tell which candidate a
result belongs to. They are shown in the closure packet and change no verdict:

| Field | Content |
|---|---|
| `candidate.baseHead` | `git rev-parse HEAD` of the candidate that was run |
| `candidate.treeDigest` | `git rev-parse HEAD^{tree}` of that candidate |
| `venue` | Where it ran: machine, OS, isolation |
| `scenarios[].scopeRef` | The contract scope reference the result covers |
| `scenarios[].fixtureRef` | The fixture used: path, digest or id |
| `scenarios[].oracleRef` | The source of truth the result was compared with |

A result recorded for a candidate that is no longer the current commit is
labelled stale in the packet: it is NOT_EXERCISED for the current candidate,
never PASS and never FAIL. Do not copy an earlier result forward.

Report separately: authored implementation, measured contract acceptance,
demonstrated runtime value, unresolved obligations, release eligibility and
release authority. Required independent final review/certification remains
separate. No automatic new task/phase, publication, release or global install.
