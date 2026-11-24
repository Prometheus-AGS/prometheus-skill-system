## ADDED Requirements

### Requirement: Reject legacy hex keys on future writes

The video-coverage validator SHALL reject 32-character lowercase hexadecimal manifest keys and accept valid slug keys.

#### Scenario: Future write contains a legacy key

- **GIVEN** a manifest update contains a legacy hex key
- **WHEN** the coverage validator examines the update
- **THEN** it rejects the offending key instead of counting it as coverage
