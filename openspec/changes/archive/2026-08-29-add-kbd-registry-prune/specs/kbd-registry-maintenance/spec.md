## Purpose

Defines explicit and recoverable cleanup of KBD replica registrations whose filesystem paths no longer exist.

## ADDED Requirements

### Requirement: Missing registrations can be inventoried without mutation
The KBD project registry SHALL expose a dry-run inventory of registered replica paths that do not exist at evaluation time.

#### Scenario: Dry run finds missing paths
- **WHEN** an operator requests missing-registration pruning without the apply flag
- **THEN** the command reports the exact candidate paths and project IDs and leaves the registry byte-for-byte unchanged

#### Scenario: No missing paths exist
- **WHEN** every registered path exists
- **THEN** the command reports an empty candidate set and does not create a backup

### Requirement: Applied pruning is evidence-preserving
Applying missing-registration pruning MUST acquire the registry lock, back up the pre-change registry with integrity evidence, and remove only registrations whose paths still do not exist under that lock.

#### Scenario: Apply removes stale registrations
- **WHEN** the operator explicitly applies a prune and candidate paths remain absent
- **THEN** the command writes a timestamped backup, checksum, and receipt before atomically persisting the registry without those entries

#### Scenario: Candidate path reappears before apply
- **WHEN** a path exists when the locked apply evaluation runs
- **THEN** its registration is preserved even if an earlier dry run reported it missing

### Requirement: Runtime history and valid registrations are preserved
Registry pruning MUST NOT delete project runtime directories, retained journals, checkpoints, or registrations whose paths exist.

#### Scenario: Multiple replicas share a project ID
- **WHEN** one path is missing and another registered path for the same project exists
- **THEN** only the missing path entry is removed and the project remains registered through the existing path

#### Scenario: Prune is repeated
- **WHEN** the same applied prune command is run after all missing entries were removed
- **THEN** it reports no removals and makes no further registry mutation
