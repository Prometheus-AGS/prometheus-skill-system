---
name: value-scenario-discovery
version: '1.0.0'
license: MIT
description: >
  Propose a finite, owner-calibrated set of useful behavioral scenarios before
  generation. Use independently or from KBD Assess, Spec and Plan; preserves
  mandatory constraints, rejected alternatives and adoption authority. Does not
  author executable tests or run generated-code checks.
metadata:
  author: prometheus-skill-pack
  category: testing
  tags: [testing, scenarios, discovery, vlbv, kbd]
---

# Value scenario discovery

Invoke `/value-scenario-discovery <phase-or-contract-reference>` to propose
scenarios, not to run them. This is instruction-only integration of the
independently usable [VLBV method](https://ipfs.prometheusags.ai/ipfs/bafybeihp44qigr3g5zoalkt3p6xvovaglcuzbsp3l4os5vba26glfz6zya/index.html).

Starting: emit `Starting value-scenario-discovery — <phase>`.
Completion: emit `Completed value-scenario-discovery — <phase> (proposal)`.
Take phase identity and totals from actual context; never invent a counter.

## Resolve context

Read the current governing repository/host policy, phase method-context reference,
canonical selected method/contract and existing owner calibration. Use the latest
authorized supersession, not a stale next-command or remembered workflow. A
missing critical identity or policy conflict is explicit; do not silently pick
another method. Tools being available is not permission to execute them.

Reuse the owner's useful operation, baseline including UNKNOWNs, environment,
constraints, exclusions and decision owner. Reuse calibration already reviewed in
this phase; do not ask for the same approval again. If examples are absent, draft
a few useful-positive, rejected/deferred and rare-critical examples for owner
review before dependent generation. Examples remain proposals until reviewed.

## Produce the finite frontier

Start with the ordinary operation the component exists to deliver. Include
mandatory integrity, safety, permission and irreversible-effect constraints even
when rare. An available test is not automatically relevant. For each candidate:

| Field | Required content |
|---|---|
| Candidate identity | Stable local candidate ID; an adopted scenario ID only when it already exists |
| Actor and useful outcome | Who needs the behavior and the observable value |
| Basis | Source requirement/reference or explicitly labelled assumption |
| Reason now | Why this case matters to the current phase |
| Likelihood | Credible sourced likelihood, qualitative estimate or UNKNOWN |
| Mandatory constraints | Separate from ordinary usefulness/priority |
| Proposed observation | Real production entry/boundary, oracle source and plausible false positive |
| Alternatives | Selected, rejected or deferred, with reason |
| Next decision | What this observation would let the owner decide |

Use the current contract's finite budget/frontier where defined. Otherwise
propose a small bounded frontier and its stopping criterion before expanding it.
Do not manufacture probabilities or weighted scores. Stop when the frontier
supports the current decision; an imagined alternative does not become a new
blocking requirement. A meaningful model/context change calls for reviewing
unfamiliar candidates against the saved examples, not resetting reviewed scope.

## Save and hand off

Write the proposal to the requested phase artifact, normally
`scenario-candidates.md`, with links to reviewed examples and unchanged
mandatory constraints. Report assumptions, rejected alternatives, UNKNOWNs and
the specific pending owner decision. Do not label unreviewed output adopted.

Assess uses the proposal to refine usefulness. Spec/Plan map only adopted cases
into the existing `adopted-contract.json` scenarioMatrix: stable ID, owner task,
production entry, oracle, negativeControl, fixtures and evidence. Acceptance-bearing
calibration/supporting documents must be identity-bound through that contract's
current adoption/amendment mechanism; never create a competing acceptance ledger.

Discovery defines scenarios before generation. It does not author executable
features/steps, import generated code, install dependencies, run test dry-runs,
builds, lint, typechecks, probes or runners. Later executable authoring follows
host policy through `bdd-cucumber-js author-only` or `bdd-cucumber-rs author-only`.
Execution belongs to [value-scenario-verify](../value-scenario-verify/SKILL.md) at
the separately authorized complete value-phase boundary. No automatic task,
phase, contract adoption, publication or release follows this proposal.
