## ADDED Requirements

### Requirement: Define the deep-research skill layout

The skill SHALL provide a TOML manifest naming deep-research and its run and export scripts, plus directories for ten stages, references, templates, hooks, agents, and scripts.

#### Scenario: Locate a pipeline entry point from the skill manifest

- **WHEN** the harness inspects the distributed research skill
- **THEN** its manifest parses and identifies the executable scripts while the ten stage directories resolve
