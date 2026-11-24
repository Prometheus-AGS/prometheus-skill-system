## ADDED Requirements

### Requirement: Delegate disk operations to DSG

Cowork disk status, scan, and clean SHALL forward supported options to dsg when available and return an actionable nonzero result when it is absent.

#### Scenario: Safe delegation or actionable failure

- **GIVEN** the operator requests a dry-run clean for the Rust ecosystem
- **WHEN** Cowork resolves dsg through PATH
- **THEN** it forwards the dry-run and ecosystem options if available, or identifies the DSG installation route and exits nonzero
