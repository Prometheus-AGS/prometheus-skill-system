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

