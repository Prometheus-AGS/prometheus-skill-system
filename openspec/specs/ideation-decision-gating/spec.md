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

