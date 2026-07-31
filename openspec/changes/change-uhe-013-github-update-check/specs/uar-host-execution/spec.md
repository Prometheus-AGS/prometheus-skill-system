## ADDED Requirements

### Requirement: An unreachable network never reports up-to-date

UAR SHALL report up-to-date, behind-by-N, or unknown when comparing the loaded manifest against the GitHub repository, and SHALL report unknown on network failure. A desktop or server update SHALL be initiable. Tests SHALL use a fixture manifest, not live GitHub.

#### Scenario: Network failure is unknown, not current

- **GIVEN** the GitHub check cannot reach the network
- **WHEN** the update status is reported
- **THEN** it is unknown, never up-to-date
