## ADDED Requirements

### Requirement: Report toolchain readiness

Cowork SHALL report tools and MCP health from detect-toolchain.sh, make toolchain check exit 1 on missing or unhealthy entries, and print install instructions for known tools.

#### Scenario: Missing managed tool

- **GIVEN** toolchain discovery reports a missing managed binary
- **WHEN** the operator runs toolchain status, check, and install for that tool
- **THEN** status identifies the missing binary, check exits 1, and install gives the specific installation instructions
