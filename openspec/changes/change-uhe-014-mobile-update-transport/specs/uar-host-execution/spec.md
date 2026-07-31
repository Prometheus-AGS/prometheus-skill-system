## ADDED Requirements

### Requirement: Mobile updates without git, or PARTIAL

The transport SHALL be chosen in a reviewed decision record, then a mobile-reachable path SHALL fetch a versioned bundle and the provenance surface SHALL reflect the new version. If the transport cannot be exercised, the change SHALL be archived BLOCKED naming the prerequisite and R5 SHALL be reported PARTIAL, never MET on the decision alone.

#### Scenario: A decision alone is not MET

- **GIVEN** the transport is decided but never exercised
- **WHEN** R5 is reported
- **THEN** it is PARTIAL, not MET
