## ADDED Requirements

### Requirement: Pin the sycophancy checker revision

The pack SHALL record a reproducible sycophancy-correction gitlink and document the gate required before advancing the pin.

#### Scenario: Reproduce the selected checker revision

- **WHEN** a fresh checkout initializes the submodule
- **THEN** the content resolves to the version recorded by the parent gitlink
