## ADDED Requirements

### Requirement: Verify a complete research HTTP session

The local smoke entry point SHALL start the server, bound the health wait, create and read a job, observe SSE data, cancel the job, and clean up the server. It SHALL fail on missing observations and use the crate entry point if the binary is absent.

#### Scenario: Exercise the server through the smoke entry point

- **WHEN** the research service is started in a disposable local instance
- **THEN** health responds within five seconds, the job appears in status, an SSE data line arrives within ten seconds, cancellation succeeds, and the server exits
