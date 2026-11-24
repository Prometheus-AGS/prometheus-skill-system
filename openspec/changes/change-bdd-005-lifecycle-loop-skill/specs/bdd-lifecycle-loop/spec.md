## ADDED Requirements

### Requirement: Protected BDD scenario lifecycle

The BDD lifecycle skill SHALL teach outside-in authoring, execution, triage, and maintenance, including a bounded tagged retry policy and the approval records required to update protected scenarios.

#### Scenario: Changed protected scenarios and a flaky case

- **GIVEN** a runner encounters a tagged flaky case and a proposed change edits protected scenario files
- **WHEN** the author applies the lifecycle guidance
- **THEN** the retry helper reads the configured flake budget, while the diff guard rejects edits without the recorded approval
