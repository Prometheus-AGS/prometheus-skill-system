## ADDED Requirements

### Requirement: Publish target-specific Cowork archives

Tagged Cowork releases SHALL supply archives for Linux x86_64 musl, macOS arm64 and x86_64, and Windows x86_64 with version and target in their names.

#### Scenario: Select an archive for a supported target

- **GIVEN** a Cowork version has been tagged for release packaging
- **WHEN** a user selects the release asset for their system
- **THEN** the release names each archive as `cowork-{version}-{target}`, using tar.gz on Linux and macOS and zip on Windows
