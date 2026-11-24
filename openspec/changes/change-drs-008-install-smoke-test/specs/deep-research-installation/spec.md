## ADDED Requirements

### Requirement: Verify runnable deep-research installation

An installed user-scope research skill SHALL be readable under ~/.claude/skills/research/deep-research with all ten stage descriptors and five executable scripts.

#### Scenario: Inspect a fresh Claude Code user-scope installation

- **WHEN** the local smoke check inspects the installed skill after installation
- **THEN** the parent and all ten stages resolve and each of the five script paths is executable
