# harness-declaration-integrity Specification

## Purpose
TBD - created by archiving change change-drift-401-windsurf-harness. Update Purpose after archive.

## Requirements

### Requirement: A declared harness has a corresponding tree
The harness manifest SHALL NOT declare a harness whose skills tree does not exist. Removing
a harness SHALL remove its declaration and regenerate every artifact derived from it.

#### Scenario: A declared harness loses its tree
- **WHEN** a harness declared in the manifest has no skills directory on disk
- **THEN** either the tree SHALL be restored or the declaration SHALL be removed, and the
  discrepancy SHALL NOT be resolved by committing the deletion alone

#### Scenario: A harness declaration is removed
- **WHEN** a harness entry is deleted from the manifest
- **THEN** every generator that reads the manifest SHALL be re-run and its output committed,
  so the distribution does not retain a harness the manifest no longer declares
