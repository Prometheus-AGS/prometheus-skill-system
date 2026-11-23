# Value-led scenario workflow

Three small, instruction-only procedures turn a broad intent into a finite set of
useful behavior scenarios and an honest, scoped verdict. They reuse the existing
BDD adapters and the KBD adopted contract; they add no runner, ledger or phase
system. Method background: the independently published VLBV paper cited by
`value-scenario-discovery`.

| Invoke | Does | Never does |
|---|---|---|
| `/value-scenario-discovery <phase-or-contract>` | Proposes a finite, owner-calibrated frontier (SCN-001) | Adopts scope, authors executable tests, runs anything |
| `/bdd-cucumber-js author-only <contract> <IDs>` and `/bdd-cucumber-rs author-only <contract> <IDs>` | Writes features, real assertions, steps and config for adopted IDs with an assertion map (SCN-002) | Executes, imports test support code, installs, dry-runs, edits protected scenarios |
| `/value-scenario-verify <contract>` | Runs the adopted packet at the authorized complete value-phase boundary and reports scoped evidence (VER-001-003) | Replaces a runner, starts early checks, retries blindly, publishes, creates a successor |

## Order of use

1. **Define** during Assess/Spec/Plan with `value-scenario-discovery`. Review
   the proposal; only owner-adopted cases enter the contract's `scenarioMatrix`
   (stable ID, owner task, production entry, oracle, negative control,
   fixtures, evidence). Acceptance-bearing calibration documents are
   identity-bound through the contract's own adoption or amendment mechanism.
2. **Implement** every planned production change. No test, build, lint, probe
   or runner command runs against the unfinished code.
3. **Author** (only if host policy allows executable test authoring now; this
   repository allows it after the complete production implementation) with
   `author-only`. Result: files plus "authored only; NOT_EXERCISED".
4. **Verify** once the complete value phase is implemented and a separate
   authorization names the actual boundary, candidate and effects. Record that
   authorization before the first attempt:

   ```bash
   node shared/scripts/kbd-boundary-authorize.mjs --phase <phase-id> \
     --authorizer "<who authorized it, and how>"
   ```

   Phase closure compares runner output against that instant and lists anything
   older. See [boundary authorization](/docs/kbd/boundary-authorization).

A task, hour, commit, checkpoint, reviewer request or compiler problem is not
step 4's boundary.

When the phase's value is that agents on several harnesses follow the method,
the packet's scenarios are the [protocol evals](/docs/kbd/protocol-evals-and-capability-evidence).

## Calibration

Provide a few reviewed examples: an ordinary useful operation, an available but
irrelevant test (deferred with its reason), and a rare mandatory constraint such
as an integrity or authorization requirement. Likelihood stays qualitative or
`UNKNOWN`; no probabilities are invented. A material model or context change
sends unfamiliar candidates back through review instead of silently making them
blocking.

## Dispositions

| Situation | Disposition |
|---|---|
| Exercised production oracle is unmet | `FAIL` |
| Required assertion never observed (or disabled/always true) | `NOT_EXERCISED` |
| Stale candidate/config/oracle, missing runner/permission/fixture, failed cleanup, uncertain effect | `BLOCKED` |
| Correct scoped observation, effective negative control, limits and cleanup met | `PASS` within that scope |

A zero process exit never passes. A disconnected `return false` probe proves
nothing about oracle sensitivity; challenge the real assertion or comparison
with its agreed plausible false positive. Earlier failures, consumed budget and
unknown effects are kept. After a failure the authorized repair is a complete
implementation batch followed by the smallest confirming run, with lineage kept.

## Worked example — changing the default provider in Settings

In a disposable Settings pilot, the browser saved a new default and showed it after reload. The service also reported the new value as saved and effective. The next browser conversation returned HTTP 401 before either synthetic provider saw the request. These observations answer two different questions. The operator could see that the value persisted. With an admitted principal and verified synthetic models, the observer saw provider A serve a marked turn before the switch and provider B serve one afterward. Both replies rendered in the browser. After UAR rejected an invalid configuration, B still served a fresh turn. On the Settings page that was open across the rejection, the observer found no matching warning text or visible alert during a 5.16-second window. Routing and visible failure reporting therefore had different outcomes on the same build.

Suppose the owner has adopted separate scenarios for persistence and routing. Keep their existing IDs, fixtures, protected oracles and acceptance criteria. Record the following outcomes in the adopted packet when the corresponding conditions occur.

| Observation | What to record |
|---|---|
| The browser reload and the independent settings read agree on the new default. | Record the persistence observation for its scenario. Award a scoped PASS only when that scenario's agreed controls and evidence are complete. |
| A valid, authorized request goes to the old provider. | Record FAIL for routing. The provider observation contradicts the promised outcome even if the Settings UI looks correct. |
| The disposable principal cannot start a conversation, as in the 401 attempt. | Record BLOCKED for the attempt and identify the missing prerequisite. The provider assertion was NOT_EXERCISED. Keep the earlier persistence observation. |
| An invalid candidate is rejected. | Record the rejection. A claim that the last good provider still serves requires a fresh observed request. An empty observer log supplies no PASS. |

For a later authorized boundary, compare each receipt with the current candidate. If a budget-only amendment kept the same phase lineage, scenario, serving artifacts, inputs, fixture and oracle, an earlier admitted PASS for persistence can be carried. Record the unchanged `scenarioDigest` on the carried outcome and set its `carriedFrom.contractDigest` to the prior binding's `contractDigest`. Retain the original receipt and source identities. Check the previous binding and receipt in the phase record. A pair of digest fields alone cannot demonstrate that the operator observed the outcome.

If fixing the principal changes the configuration used by a scenario, repeat that scenario on the new configuration. Changed code or served assets, changed inputs, a changed oracle, and unknown impact also require fresh evidence for the affected claim. Retain the earlier attempt as history. To challenge the comparison, present a receipt for a different serving artifact to the same oracle and confirm it cannot earn PASS. The existing closure command remains responsible for admission.

This example uses the bounded closure, method selection, scenario calibration and evidence admission already delivered in [#184](https://github.com/Prometheus-AGS/prometheus-skill-system/issues/184), [#195](https://github.com/Prometheus-AGS/prometheus-skill-system/issues/195), [#196](https://github.com/Prometheus-AGS/prometheus-skill-system/issues/196) and [#219](https://github.com/Prometheus-AGS/prometheus-skill-system/issues/219).

## Closure

Evidence goes to the existing canonical writer through `boundary-results.json`
(consumed by `shared/scripts/kbd-phase-close-eval.sh`). The report keeps six
claims apart: authored implementation, measured contract acceptance,
demonstrated runtime value, unresolved gaps, release eligibility and release
authority. Unsupported closure dispositions stay refused.

## Limits

- The procedures are instructions. They cannot force a model to obey and do not
  intercept arbitrary commands; enforcement is exactly what the host and the
  KBD runtime provide. Qualify the host sessions and runtime paths you rely on.
- The default frontier is small by design. A full multi-hundred-scenario
  catalog with a completeness contract is not covered here.
- Execution is deferred to the authorized boundary; this guide claims no
  behavior has been run.
- `bdd-testing` is a deprecated redirect; use the adapters above.

## Traceability

| Requirement / scenario | Where it lives |
|---|---|
| SCN-001 bounded, calibrated discovery | `skills/testing/value-scenario-discovery/SKILL.md` |
| SCN-002 author-only JS/Rust, ID and oracle linkage | `skills/testing/bdd-cucumber-{js,rs}/references/author-only.md` and mode routes in each `SKILL.md` |
| VER-001 real operation and sensitivity challenge | `skills/testing/value-scenario-verify/SKILL.md` ("Run the smallest adequate real path") |
| VER-002 attempt lineage, FAIL/BLOCKED/NOT_EXERCISED | same skill, "Retain every material attempt" |
| VER-003 canonical closure and stop | same skill, "Use the actual canonical writer" |
| Replaced conflicting guidance | `skills/testing/bdd-lifecycle-loop/SKILL.md`, `skills/process/kbd-process-orchestrator/references/integrations/bdd-testing.md`, `references/architecture.md` |
| Policy authority from #195 | [Selected method recovery](kbd-selected-method-recovery.md) |
