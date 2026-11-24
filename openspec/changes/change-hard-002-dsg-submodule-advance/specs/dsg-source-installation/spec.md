## ADDED Requirements

### Requirement: Pin DSG to the release with corrected Intel packaging

The pack SHALL point tools/disk-space-guardian to the v0.1.4 revision abe2e1c, aligning the source choice with the release that restored the Intel macOS artifact.

#### Scenario: Resolve DSG after updating the submodule

- **WHEN** the operator initializes the pin recorded by the pack
- **THEN** the submodule resolves to abe2e1c, corresponding to v0.1.4
