## ADDED Requirements

### Requirement: Cross-repo work requires explicit authorisation, and silence blocks

This change SHALL request authorisation before editing any file outside this repository. Absent an explicit grant, it SHALL be archived BLOCKED and no external file SHALL be modified. Changes 005 and 006 SHALL NOT be reported as end-to-end parity when this change is blocked.

#### Scenario: Silence blocks rather than proceeds

- **GIVEN** no authorisation has been granted
- **WHEN** the change runs
- **THEN** it is archived BLOCKED and no external file is modified
