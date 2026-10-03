## Purpose
Make existing project teams discoverable and reusable by supported coding harnesses without replacing native permissions or ownership.

## ADDED Requirements

### Requirement: Explicit project installation
Export SHALL remain proposal-only. A separate install-project operation SHALL offer dry-run and check, persist an active-team record, and update both instruction entrypoints. Normal authorized team creation SHALL finish with project installation.

#### Scenario: Sole existing team
- **WHEN** exactly one valid project team exists and no explicit selection is recorded
- **THEN** it becomes the default for relevant code work.

#### Scenario: Ambiguous teams
- **WHEN** multiple teams exist without an explicit valid selection
- **THEN** installation requests a selection rather than choosing arbitrarily.

### Requirement: Preserve native authority
Installation SHALL preserve role IDs, ownership, model policy, concurrency and native permissions. Presence SHALL reference actual manifests and native definitions. Zed SHALL use the effective instruction file, with external ACP agents retaining their own native configuration.

#### Scenario: Delegation unavailable
- **WHEN** the active tool lacks a native delegation API
- **THEN** it uses relevant role instructions sequentially and reports the limitation.

### Requirement: Role skill binding
Design and UI implementation roles SHALL bind to the shared UI contract through roles[].skills, while reviewers SHALL receive review guidance without taste instructions.

#### Scenario: Backend task on a mixed team
- **WHEN** backend-only code work is assigned to its owning role
- **THEN** the team remains active and no UI skills are loaded.

