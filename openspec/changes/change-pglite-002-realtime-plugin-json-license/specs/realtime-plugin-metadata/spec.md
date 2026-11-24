## ADDED Requirements

### Requirement: Declare the realtime sub-plugin license

The entity-graph-realtime Claude plugin manifest SHALL declare license MIT, matching the parent plugin declaration.

#### Scenario: Read the standalone realtime plugin manifest

- **WHEN** the marketplace parses the sub-plugin JSON
- **THEN** the document remains valid and exposes MIT without changing the other metadata
