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
   authorization names the actual boundary, candidate and effects.

A task, hour, commit, checkpoint, reviewer request or compiler problem is not
step 4's boundary.

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
