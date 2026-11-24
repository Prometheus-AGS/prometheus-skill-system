## ADDED Requirements

### Requirement: Equivalent Rust scenario examples

The Rust BDD skill SHALL provide reqwest and thirtyfour examples that exercise the same business behavior.

#### Scenario: Author chooses a Rust test surface

- **GIVEN** an author opens the examples
- **WHEN** they inspect the features and step definitions
- **THEN** both examples describe the same business outcome and the README explains when to choose HTTP or browser execution
