## ADDED Requirements

### Requirement: Coach and reflector are separate roles

A coach agent SHALL advance the plan and SHALL NOT evaluate its own output; the existing
reflector SubagentStop hook SHALL perform that evaluation unmodified.

#### Scenario: The coach does not grade itself

- **GIVEN** coach output
- **WHEN** it is evaluated
- **THEN** the evaluation is performed by the reflector
- **AND** the coach performs no evaluation of its own output
