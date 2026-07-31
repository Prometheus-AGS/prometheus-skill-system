## ADDED Requirements

### Requirement: Bindings are built for a real mobile target, or the goal is PARTIAL

The bindings SHALL build for at least one real mobile target and a round-trip test SHALL assert on a value returned across the boundary. If the toolchain is unavailable, the change SHALL be recorded BLOCKED naming the prerequisite and goal 3 SHALL be reported PARTIAL, never MET on the decision alone.

#### Scenario: A decided pattern alone is not MET

- **GIVEN** the pattern is decided but no bindings are built
- **WHEN** goal 3 is reported
- **THEN** it is PARTIAL, not MET
