## ADDED Requirements

### Requirement: Provide a third domain independent of the pack

The grader evaluation SHALL add a third non-software corpus compatible with learn-grade, containing 10 to 15 sources and three to five explicitly flagged misconceptions. Correct sources SHALL include key points for transfer problems.

#### Scenario: Load the added domain

- **WHEN** the grader harness resolves the third corpus alongside the two existing domains
- **THEN** the source records match the expected schema and the declared misconception count and keys are present
