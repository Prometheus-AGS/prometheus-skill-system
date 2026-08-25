## Purpose

Defines how a registered KBD project crosses from identity-only registry state into its first signed canonical run without requiring unsafe manual projection edits or unrelated migration commands.

## ADDED Requirements

### Requirement: Registration remains separate from run initialization
Project registration SHALL preserve immutable project and replica identity without inventing a run, phase, lifecycle, or signed event. Read-only status inspection MUST NOT initialize an empty registered runtime.

#### Scenario: Empty project is registered
- **WHEN** a project with a valid identity manifest is registered before any KBD run exists
- **THEN** registration records the replica without creating a runtime initialization event

#### Scenario: Status inspects an empty runtime
- **WHEN** status is requested for a registered runtime with no signed events
- **THEN** status reports that canonical runtime initialization is pending without mutating the journal

### Requirement: First typed mutation initializes canonical state
The first typed mutation against a writable registered runtime with no signed events SHALL create one operator-signed initialization boundary before applying the requested mutation. Initialization MUST derive compatible lifecycle, active phase, plan revision, and exact-next-work from existing legacy projections when present, and subsequent mutations MUST reuse the initialized run rather than creating another boundary.

#### Scenario: First mutation follows legacy projections
- **WHEN** a registered empty runtime has a legacy waypoint and receives a typed mutation
- **THEN** one signed initialization event preserves compatible waypoint state and the requested mutation commits after it

#### Scenario: First mutation has no legacy phase
- **WHEN** a registered empty runtime without a populated legacy waypoint receives a typed mutation
- **THEN** one signed ready-state run is created and the requested mutation commits against it

#### Scenario: Later mutation reuses the run
- **WHEN** a second typed mutation follows successful automatic initialization
- **THEN** no additional initialization event is created

### Requirement: Initialization failures are actionable
If automatic initialization cannot complete, the CLI SHALL exit non-zero and identify the canonical runtime path in the error chain. It MUST NOT advise `migrate --apply` unless migration inventory itself requires migration.

#### Scenario: Runtime cannot be initialized
- **WHEN** a typed mutation cannot initialize the registered runtime
- **THEN** the command exits non-zero and names the affected canonical runtime path

#### Scenario: Status reports pending initialization
- **WHEN** status observes an empty registered runtime
- **THEN** human-readable and machine-readable output explain that the first typed mutation initializes automatically and do not recommend migration

### Requirement: Typed command rejection is a process failure
A typed command rejected by canonical validation SHALL produce a non-zero CLI process status while preserving the already committed journal and projections.

#### Scenario: Invalid mutation follows initialization
- **WHEN** canonical validation rejects a typed mutation after the runtime is initialized
- **THEN** the CLI exits non-zero and does not report the rejected mutation as committed
