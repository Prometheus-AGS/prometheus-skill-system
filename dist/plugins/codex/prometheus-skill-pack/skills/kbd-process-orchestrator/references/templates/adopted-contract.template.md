# Adopted completion contract — authoring template

Author this during Spec/Plan, freeze it before production implementation, and
adopt it through the existing canonical writer. The machine-checked artifact is
`adopted-contract.json` in the phase directory, validated against
`references/schemas/adopted-contract.schema.json`. This template explains each
section; placeholders are not acceptance evidence.

## Adoption (canonical writer only)

Run from the consumer project. For repository development or a custom payload,
set `KBD_PACK_ROOT` to the absolute package root; do not change into that root.
All package and contract paths must remain quoted.

```bash
KBD_PACKAGE_ROOT="${KBD_PACK_ROOT:-${CLAUDE_PLUGIN_ROOT:-${PLUGIN_ROOT:-${PROMETHEUS_PLUGIN_ROOT:-$HOME/.prometheus/plugins/prometheus-skill-pack}/current}}}"
bash "$KBD_PACKAGE_ROOT/shared/scripts/kbd-contract-adopt.sh" ".kbd-orchestrator/phases/<phase>/adopted-contract.json"
```

The generation installer also exposes both helpers under
`"${PROMETHEUS_PLUGIN_ROOT:-$HOME/.prometheus/plugins/prometheus-skill-pack}/stable"`.
A missing helper is an installation error; an available helper reporting missing
or invalid contract state requires fixing the consumer contract/adoption.

The writer validates the contract against the schema, computes the digest with
the frozen normalization, stamps `identity.planRevision` (from
`prometheus kbd status --json`) and `identity.contractDigest`, and records an
immutable `prometheus kbd decision record` bound to the current plan revision.
It never hand-writes journals, waypoints, or signatures. Re-running with
identical content is idempotent (no duplicate decision).

## Digest normalization (frozen per contract)

```
sha256( jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)' adopted-contract.json )
```

`identity.contractDigest` is excluded to avoid self-referential recursion.
`identity.planRevision` is excluded because the canonical Decision binds
`plan_revision` at record time. Writer and checker MUST use the identical
normalization command recorded in `digestNormalization.command`.

## Amendment / restart

An amendment is a content change (e.g. `status.implementationComplete` false→true)
adopted with `--supersedes <prior-decision-id>`: new digest, new canonical
decision, history preserved. The readiness checker accepts only the latest
non-superseded adoption digest for the contract path; any older digest is
REJECT-STALE. Consumed budgets and history are never reset by amendment.

## Section checklist

| Section | Requirement |
|---|---|
| `identity` | projectId / runId / phaseId from canonical sources; planRevision + contractDigest stamped at adoption |
| `adoptionSource` | explicit operator instruction; never inferred |
| `releaseClaim` | real user operation, supported environments, honest current capabilities, explicit UNKNOWNs — never task counts |
| `scenarioMatrix` | finite, frozen stable IDs; each entry: production entry, real collaborators, oracle, negative control, fixture identity (scratch roots + digests), evidence requirements, owner task |
| `functionalScope` / `resilienceScope` | separate lists; adopted resilience never defers via budget overflow; only never-adopted optional items in `deferredOptional` |
| `exclusions` | frozen; scope change only via operator-adopted amendment |
| `requiredSafetyChecks` | non-waivable; never downgraded by repair-budget exhaustion |
| `budgets` | time/tool/repair/no-progress ceilings; token = measured or `UNKNOWN` + fallback; exhaustion => checkpoint + BLOCKED |
| `evidenceInvalidation` | digest/revision/fixture-keyed; only affected evidence invalidated; history preserved |
| `workerRoutes` | agent + model route, billing confirmation, subagent policy, review-round cap |
| `boundaryPacket` | `postImplementationOnly: true`; exact commands, expected effects, approvals, receipts, cleanup, stop conditions |

## Status vocabulary (exactly four verdicts)

`PASS` / `FAIL` (product: real oracle unmet) / `BLOCKED` (verification:
validator/env/fixture unavailable) / `NOT_EXERCISED`. Every scenario run
records attempted / observed / evidence. Unknown is neither success nor
observed failure. Delivery/packaging-only checks (e.g. strict skill validation)
are marked with `classification` and their failures never cascade into
product-failure claims.
