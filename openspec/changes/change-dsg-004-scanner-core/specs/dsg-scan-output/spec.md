## ADDED Requirements

### Requirement: Expose typed scans and disk status

DSG scan SHALL collect file, directory, and symlink entries with size, modification time, and detected ecosystem, and apply depth, ecosystem, age, and size filters. Scan and status SHALL provide human and JSON output.

#### Scenario: Compare human and JSON scan output

- **WHEN** a scan root contains entries above and below the configured size floor
- **THEN** both formats describe the qualifying entries, and status reports disk usage in the requested format
