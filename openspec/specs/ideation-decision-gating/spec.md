# ideation-decision-gating Specification

## Purpose
TBD - created by archiving change change-idt-001-decision-review-packet. Update Purpose after archive.
## Requirements
### Requirement: Decision artifacts are cross-model verified

A decision artifact SHALL carry `cross_model_check`, and a validator SHALL reject any
decision artifact whose value is absent, `same-model-collision`, or
`unverified-producer-unknown`. Demonstrating that one artifact can carry the stamp is
not the same property as enforcing that every artifact does.

#### Scenario: A decision packet is judged cross-model

- **GIVEN** a decision packet built with `--mode decision`
- **WHEN** it is dispatched to the judge
- **THEN** the findings artifact records `cross_model_check`
- **AND** the mandate for `decision` mode resolves

#### Scenario: An unverified decision artifact is rejected

- **GIVEN** a decision artifact whose `cross_model_check` is absent or not `verified-distinct`
- **WHEN** the validator runs
- **THEN** it exits non-zero
- **AND** the artifact is not treated as a completed decision

### Requirement: Ideation diversity is enforced structurally

Candidate generation SHALL produce at least three candidate sets in separate dispatches,
none of which receives another set as input. Diversity SHALL NOT be assumed from persona
prompting: multi-agent LLM ideation synchronises despite architectural attempts to
diversify.

#### Scenario: Independent generation precedes pooling

- **GIVEN** an ideation run
- **WHEN** generation completes
- **THEN** at least three candidate sets exist
- **AND** no dispatch received another candidate set as input

#### Scenario: The critic does not grade its own output

- **GIVEN** candidate sets awaiting scoring
- **WHEN** the critic scores them
- **THEN** the scoring dispatch is distinct from the generating dispatch

### Requirement: Analysis is withheld until the user commits

The flow SHALL refuse to emit analysis until a user judgement is recorded, and output
SHALL carry a machine-checkable confidence field, a non-empty `what_would_change_this`
field, and at least one disconfirming item. Confidence claims that cannot be checked are
not acceptable substitutes.

#### Scenario: Analysis is refused without a prior judgement

- **GIVEN** a decision flow with no recorded user judgement
- **WHEN** analysis is requested
- **THEN** the flow refuses
- **AND** no analysis is emitted

#### Scenario: Output carries checkable calibration fields

- **GIVEN** a completed decision analysis
- **WHEN** its artifact is validated
- **THEN** `confidence` is present
- **AND** `what_would_change_this` is non-empty
- **AND** at least one disconfirming item is present

### Requirement: Decisions persist with their outcomes

A decision SHALL be written with `outcome_status: pending`, an outcome-update flow SHALL
record what actually happened, and a revisit query SHALL return both the decision and its
outcome. Persisting decisions without outcomes is the half the surveyed market already
has and does not close the loop.

#### Scenario: A decision is written pending an outcome

- **GIVEN** a completed decision
- **WHEN** it is persisted
- **THEN** one wiki entry records the decision, assumptions, and falsifier
- **AND** `outcome_status` is `pending`

#### Scenario: A revisit returns decision and outcome

- **GIVEN** a decision whose outcome was later recorded
- **WHEN** the revisit query runs for that topic
- **THEN** both the original decision and the recorded outcome are returned

#### Scenario: Re-running a decision does not duplicate

- **GIVEN** a decision already persisted
- **WHEN** the same decision is run again
- **THEN** no second entry is created

### Requirement: Coach and reflector are separate roles

A coach agent SHALL advance the plan and SHALL NOT evaluate its own output; the existing
reflector SubagentStop hook SHALL perform that evaluation unmodified.

#### Scenario: The coach does not grade itself

- **GIVEN** coach output
- **WHEN** it is evaluated
- **THEN** the evaluation is performed by the reflector
- **AND** the coach performs no evaluation of its own output

### Requirement: Delivery is verified on a non-Claude harness

The ideation flow SHALL emit a `UiIntent` rather than rendering directly, and Tier 1
delivery SHALL be verified by running it on one named non-Claude harness. Tier 0 text is a
floor, not evidence of harness delivery.

#### Scenario: Tier 1 round trip completes on a non-Claude harness

- **GIVEN** the flow running on a named non-Claude harness
- **WHEN** it emits a UiIntent
- **THEN** `__ui_intent__.json` is written
- **AND** a `__ui_response__.json` placed there is consumed within the timeout
- **AND** the flow continues using that response

#### Scenario: Tier 0 remains a working floor

- **GIVEN** a harness resolving to `tier0_text`
- **WHEN** the flow runs
- **THEN** it completes in text

### Requirement: storage-provider can target wasm32

`iroh-docs` SHALL be behind a feature flag rather than an unconditional dependency, and
the `iroh` floor SHALL be at least 1.0.2. An unconditional non-wasm dependency silently
forecloses every browser target.

#### Scenario: Native behaviour is unchanged

- **GIVEN** the default feature set
- **WHEN** `cargo build` runs
- **THEN** it succeeds
- **AND** IrohDocsAdapter remains available

#### Scenario: wasm32 progresses past iroh-docs

- **GIVEN** the feature disabled
- **WHEN** `cargo check --target wasm32-unknown-unknown` runs
- **THEN** compilation does not fail on `iroh-docs`

### Requirement: Fabric decisions are recorded and checkable in-repo

Each fabric decision SHALL be recorded with its alternatives and rationale, and any edit
to a file outside this repository SHALL be evidenced by a `knowme_sync` block naming the
path, the external repository commit, and the resulting file hash — so a reviewer scoped to
this repository can verify it. No code SHALL land in the consuming repositories.

#### Scenario: Decisions are recorded with alternatives

- **GIVEN** the fabric decision set
- **WHEN** the records are read
- **THEN** each names the selected option and the rejected alternatives with rationale

#### Scenario: External edits are evidenced in-repo

- **GIVEN** an edit to the KnowMe integration guide
- **WHEN** the decision record is written
- **THEN** it contains a `knowme_sync` block with the external path, that repository's commit, and the guide hash

#### Scenario: No consuming repository receives code

- **GIVEN** this change complete
- **WHEN** the consuming repositories are inspected
- **THEN** no source change was made to flint-realtime-fabric, universal-agent-runtime, or know-me-system

### Requirement: The idea gate is proven to discriminate

Committed fixtures SHALL prove a weak idea is blocked and a sound idea passes, both
cross-model verified, and an inversion SHALL fail the suite. A suite that asserts only
that a review completed proves nothing.

#### Scenario: Fixtures sort correctly

- **GIVEN** committed weak-idea and sound-idea fixtures sharing a domain and stated intent
- **WHEN** each is submitted to the gate
- **THEN** the weak idea is BLOCKed and the sound idea PASSes
- **AND** both record `cross_model_check: verified-distinct`

#### Scenario: An inversion fails the suite

- **GIVEN** a run where a weak fixture passes or a sound fixture is blocked
- **WHEN** the suite evaluates
- **THEN** it exits non-zero

#### Scenario: Zero assertions is a failure

- **GIVEN** a run in which no assertion executed
- **WHEN** the suite reports
- **THEN** it exits non-zero rather than reporting success

