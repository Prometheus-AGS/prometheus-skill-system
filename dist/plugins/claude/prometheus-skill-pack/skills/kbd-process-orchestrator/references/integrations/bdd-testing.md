# KBD Integration: behavioral scenarios (bdd-testing → value-scenario skills)

`bdd-testing` is a deprecated redirect. KBD now uses three small procedures
instead of a per-change BDD gate:

| Procedure | Skill | When |
|---|---|---|
| Define scenarios | `value-scenario-discovery` | Assess, Spec and Plan, before generation |
| Author features/steps | `bdd-cucumber-js` / `bdd-cucumber-rs` `author-only` | Only when host policy allows executable test authoring (this repository: after the complete production implementation) |
| Execute and close | `value-scenario-verify` | Only at the separately authorized complete value-phase boundary |

Existing method, contract and operator policy always govern. The selected
method reference (see `docs/guide/kbd-selected-method-recovery.md`) and the
adopted contract's `scenarioMatrix` / `boundaryPacket` own scenario identity,
oracles, budgets and the single post-implementation packet. This integration
adds no competing ledger, runner or acceptance record.

---

## Placement

```
Assess/Spec/Plan  → /value-scenario-discovery  (proposal; owner reviews; adopted cases enter scenarioMatrix)
Execute           → implement ALL planned production; no test/build/lint/probe commands
(host allows)     → bdd-cucumber-* author-only (features, real assertions, steps; NOT_EXERCISED)
Phase end         → /value-scenario-verify at the authorized boundary → canonical closure writer
Reflect           → plan-versus-outcome differences; PASS/FAIL/BLOCKED/NOT_EXERCISED stay distinct
```

A task, hour, commit, checkpoint, reviewer request or compiler problem is not
the verification boundary. Per-change QA runs are not part of this flow.

## Feature file convention

Tag each feature with its layer and the stable adopted scenario ID (not a
change number), so the assertion map traces to the contract:

```gherkin
@api @SCN-EXAMPLE-001
Feature: Preference survives the defined restart
```

## What KBD reads back

KBD does not read an exit code as acceptance. The authorized boundary produces
`boundary-results.json` (scenario id/status/attempted/observed/evidence plus
budget consumption) consumed by `shared/scripts/kbd-phase-close-eval.sh`.
Original runner output stays beside it. A zero exit without the required
assertion is `NOT_EXERCISED`, stale or wrong candidate identity is `BLOCKED`,
and a genuinely exercised oracle mismatch is `FAIL`. Report separately:
authored implementation, measured contract acceptance, demonstrated runtime
value, unresolved gaps, release eligibility and release authority.

## Browser recordings

Recordings are optional evidence for `@ui` scenarios and are produced only during
authorized execution. They never substitute for the assertion or oracle.

## When not to define a scenario

- Infrastructure-only work with no observable user or operator behavior.
- A candidate the owner rejected or deferred during calibration; keep the
  reason and do not let it silently become a blocking requirement.
