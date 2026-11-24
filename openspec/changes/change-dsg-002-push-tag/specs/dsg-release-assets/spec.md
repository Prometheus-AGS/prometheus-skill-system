## ADDED Requirements

### Requirement: Publish a traceable first DSG release

The DSG v0.1.0 release tag SHALL identify the published source with its versioned binary packaging definition for macOS arm64 and x86_64, Linux x86_64 musl, and Windows x86_64.

#### Scenario: Resolve the release source by tag

- **WHEN** an operator selects the public v0.1.0 tag
- **THEN** the selected revision includes the packaging definition for all four targets
