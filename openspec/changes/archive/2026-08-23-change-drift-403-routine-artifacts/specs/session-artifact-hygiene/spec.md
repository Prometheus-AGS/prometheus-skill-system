## ADDED Requirements

### Requirement: Session logs are scanned before they are committed
Session-log and knowledge artifacts SHALL be scanned for secret-shaped content in the
change that commits them.

#### Scenario: Session logs are staged
- **WHEN** .prometheus knowledge or session-log files are added to a change
- **THEN** the change SHALL run a secret scan over their diff and record the result, and
  SHALL NOT rely on an authorization granted in a different repository

#### Scenario: A new tool artifact appears untracked
- **WHEN** a tool writes a new top-level directory or dotfile into the tree
- **THEN** the change SHALL record an explicit decision to track or ignore it, rather than
  committing it by default
