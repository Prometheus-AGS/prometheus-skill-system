## ADDED Requirements

### Requirement: Builtin registration holds on all three persistence providers

After startup, the count of builtin skills in the database SHALL equal the loader's discovered count, on postgres, surreal, and memory. The memory provider is the embedded case and SHALL NOT be skipped. A provider that cannot be exercised SHALL be recorded BLOCKED and R1 reported PARTIAL.

#### Scenario: One provider is not enough

- **GIVEN** only one persistence provider has been verified
- **WHEN** R1 is reported
- **THEN** it is PARTIAL, not MET
