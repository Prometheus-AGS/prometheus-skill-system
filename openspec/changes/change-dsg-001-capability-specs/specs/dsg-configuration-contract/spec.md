## ADDED Requirements

### Requirement: Set safe defaults and honor explicit settings

DSG SHALL default to dry-run, a one-day age floor, a ten-megabyte scan floor, and info logging. It SHALL read user exclusions and accept the documented environment overrides.

#### Scenario: Start with missing or malformed configuration

- **WHEN** an operator invokes DSG before creating a configuration file, then tries a malformed one
- **THEN** the first invocation applies defaults, while the second exits 1 with a configuration diagnostic
