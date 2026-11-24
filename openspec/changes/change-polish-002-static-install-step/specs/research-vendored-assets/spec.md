## ADDED Requirements

### Requirement: Make vendored research assets available to standalone docs

The docs static directory SHALL contain real copies of the vendored HTMX, Alpine, and HLS assets. The managed installer SHALL copy available assets into the directory after installing the binary.

#### Scenario: Open a fresh checkout without source-path symlinks

- **WHEN** the operator previews the standalone deep research page or refreshes the assets through the managed install
- **THEN** its script paths resolve to regular files in the docs static directory
