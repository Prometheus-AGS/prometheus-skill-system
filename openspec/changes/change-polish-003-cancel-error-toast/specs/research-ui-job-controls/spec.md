## ADDED Requirements

### Requirement: Report a failed cancel request

The research UI SHALL display an error toast when cancel fails and preserve its successful cancel confirmation and dashboard navigation.

#### Scenario: Cancel while the server cannot respond

- **WHEN** the active job's cancel request fails at the network boundary
- **THEN** the UI displays an error explaining that cancel failed and the job may have completed
