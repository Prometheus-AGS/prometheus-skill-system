## ADDED Requirements

### Requirement: Classify reclaimable data by ecosystem

DSG SHALL provide detectors for Rust, Node, Python, Go, Docker, Xcode, and Homebrew, use them to choose deep-scan roots, and retain activity, age, and exclusion checks during cleanup.

#### Scenario: Filter a mixed workspace to Rust

- **WHEN** a Rust target directory and unrelated Node caches are detected
- **THEN** the Rust ecosystem scan returns Rust items only, and a dry-run cleanup previews candidates after safety checks
