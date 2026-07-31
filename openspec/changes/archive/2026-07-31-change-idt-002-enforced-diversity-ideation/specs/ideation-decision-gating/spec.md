## ADDED Requirements

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
