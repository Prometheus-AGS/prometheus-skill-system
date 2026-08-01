## ADDED Requirements

### Requirement: Every REST verb R4 names has a passing test

Skill installation and query endpoints SHALL each have a passing request/response test covering install, list, get, search, and toggle. The existence of a mounted route SHALL NOT by itself satisfy this requirement.

#### Scenario: Existence is not acceptance

- **GIVEN** an endpoint is mounted but has no passing test
- **WHEN** R4 coverage is reported
- **THEN** that verb is recorded as a gap rather than counted as covered
