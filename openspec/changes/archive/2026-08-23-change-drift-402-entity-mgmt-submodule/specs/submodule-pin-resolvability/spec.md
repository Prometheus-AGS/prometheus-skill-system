## ADDED Requirements

### Requirement: A committed submodule pin resolves from a fresh clone
The parent SHALL NOT pin a submodule to a commit that exists on no remote branch of that
submodule.

#### Scenario: The submodule checkout has advanced past its pin
- **WHEN** the working checkout is ahead of the committed pin
- **THEN** the pointer SHALL be adopted only if the new commit is reachable from a remote
  branch, and otherwise the pin SHALL be restored

#### Scenario: The submodule worktree carries uncommitted content
- **WHEN** the submodule has modified tracked files of its own
- **THEN** the parent SHALL treat that as the submodule owner s decision, and SHALL record
  that the residual status entry blocks any tooling that requires a clean parent tree
