## ADDED Requirements

### Requirement: Index every explanation deterministically

The grader evaluation SHALL document its item schema and index every item with its ID, domain, corpus path, explanation, annotation, and draft or reviewed status.

#### Scenario: Enumerate the evaluation dataset

- **WHEN** the harness traverses index.json
- **THEN** every intended item is reachable once, its corpus path resolves, and its review status is explicit
