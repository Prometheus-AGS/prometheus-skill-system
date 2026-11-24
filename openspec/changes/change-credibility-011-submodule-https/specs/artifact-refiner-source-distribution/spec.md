## ADDED Requirements

### Requirement: Allow HTTPS checkout of artifact-refiner

GitHub submodules SHALL use HTTPS, including artifact-refiner and sycophancy-correction, so recursive checkout works without an SSH key.

#### Scenario: Initialize with no SSH identity

- **WHEN** a fresh clone initializes the artifact-refiner submodule
- **THEN** the registered HTTPS source resolves without an SSH key
