## ADDED Requirements

### Requirement: Validate parent and stage skill metadata

Strict validation SHALL cover the parent and all stage skills, enforce usable frontmatter and relative paths, and flag non-executable scripts.

#### Scenario: Detect an invalid staged skill

- **WHEN** a stage has malformed frontmatter or a distributed script lacks its executable bit
- **THEN** strict validation identifies the offending file and rejects it
