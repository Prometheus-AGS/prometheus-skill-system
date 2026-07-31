## ADDED Requirements

### Requirement: A skill component returns its own output

The Wasm runtime SHALL instantiate and invoke a component so the reference skill returns its own output rather than the placeholder string. Nothing SHALL be described as end-to-end parity until this passes.

#### Scenario: Placeholder output is not execution

- **GIVEN** the runtime returns the placeholder string
- **WHEN** the result is evaluated
- **THEN** the change is not complete and parity is not claimed
