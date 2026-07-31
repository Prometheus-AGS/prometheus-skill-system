## ADDED Requirements

### Requirement: Harness delivery claims rest on executed round trips

A harness SHALL NOT be claimed as Tier 1 without an executed file-pair round trip; a failure SHALL be recorded rather than the claim narrowed.

#### Scenario: An unrun harness is not claimed

- **GIVEN** opencode or kimi has no executed round trip
- **WHEN** delivery is reported
- **THEN** that harness is not listed as verified Tier 1
