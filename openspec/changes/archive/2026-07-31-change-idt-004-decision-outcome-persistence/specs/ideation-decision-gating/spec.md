## ADDED Requirements

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
