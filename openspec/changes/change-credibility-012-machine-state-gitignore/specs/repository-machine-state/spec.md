## ADDED Requirements

### Requirement: Exclude machine-local runtime state

The repository SHALL exclude .prometheus runtime data, KBD project.json files, SCRATCHPAD.md, .envrc, and *.local.env from version control.

#### Scenario: Use local runtime and environment files

- **WHEN** an operator creates matching files in their local checkout
- **THEN** status treats the files as ignored and the tracked index contains none of the named runtime state
