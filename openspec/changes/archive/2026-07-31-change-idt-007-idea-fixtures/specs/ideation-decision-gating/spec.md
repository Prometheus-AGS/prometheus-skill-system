## ADDED Requirements

### Requirement: The idea gate is proven to discriminate

Committed fixtures SHALL prove a weak idea is blocked and a sound idea passes, both
cross-model verified, and an inversion SHALL fail the suite. A suite that asserts only
that a review completed proves nothing.

#### Scenario: Fixtures sort correctly

- **GIVEN** committed weak-idea and sound-idea fixtures sharing a domain and stated intent
- **WHEN** each is submitted to the gate
- **THEN** the weak idea is BLOCKed and the sound idea PASSes
- **AND** both record `cross_model_check: verified-distinct`

#### Scenario: An inversion fails the suite

- **GIVEN** a run where a weak fixture passes or a sound fixture is blocked
- **WHEN** the suite evaluates
- **THEN** it exits non-zero

#### Scenario: Zero assertions is a failure

- **GIVEN** a run in which no assertion executed
- **WHEN** the suite reports
- **THEN** it exits non-zero rather than reporting success
