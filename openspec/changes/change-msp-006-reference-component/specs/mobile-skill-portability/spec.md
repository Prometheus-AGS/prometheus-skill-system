## ADDED Requirements

### Requirement: A well-formed component is not a working one

A built component SHALL validate under wasm-tools with the component-model feature and sit where UAR discovery expects it. The change SHALL NOT claim the component executes; execution is proven only by change-msp-008.

#### Scenario: Well-formed is not reported as executing

- **GIVEN** a component validates but has never run
- **WHEN** the change reports its outcome
- **THEN** it states the artifact is well-formed and execution is unproven
