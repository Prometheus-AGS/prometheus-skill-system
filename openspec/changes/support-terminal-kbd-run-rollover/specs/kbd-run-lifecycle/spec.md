## Purpose

Defines how a terminal KBD run yields to a fresh successor run without rewriting project identity, audit history, or causal authority.

## ADDED Requirements

### Requirement: Terminal runs can start one signed successor
The KBD control plane SHALL accept an operator-signed successor-run command only when the current run is terminal. The command MUST supply a non-empty, different run identifier and a reason, and MUST be rejected while the current lifecycle is non-terminal or an unresolved lifecycle conflict exists.

#### Scenario: Cancelled run starts a successor
- **WHEN** an operator starts a successor from a cancelled run with a new identifier and reason
- **THEN** the successor becomes the current run in `ready` state and the prior run remains in immutable audit history

#### Scenario: Running run rejects rollover
- **WHEN** an operator attempts to start a successor while the current run is ready, running, paused, or blocked
- **THEN** the command is rejected without changing the journal, projections, or PAUSE valve

### Requirement: Rollover separates run state from project authority
A successful successor start SHALL reset run-scoped workflow state while preserving project-scoped identity and authority. Run-scoped phases, active path, completion, checkpoint, decisions, blockers, and claims MUST NOT leak into the successor; project identity, device authority, submodule pins, causal frontier, command idempotency, and immutable events MUST remain intact.

#### Scenario: Successor begins with fresh workflow state
- **WHEN** a terminal run containing phases, completion counters, a checkpoint, and claims starts a successor
- **THEN** current status exposes none of those run-scoped records and retains the same project and signer authority

#### Scenario: Audit spans both runs
- **WHEN** audit is requested after rollover
- **THEN** it returns the complete ordered history including the original initialization, terminal transition, and successor initialization

### Requirement: Causal folding distinguishes rollover from divergence
Sequential successor initializations SHALL fold as ordered run boundaries. Concurrent successor attempts from the same frontier SHALL be treated as a lifecycle conflict and MUST NOT synthesize a valid merged run identity.

#### Scenario: Sequential run IDs replay deterministically
- **WHEN** a journal contains causally ordered terminal and successor events with different run IDs
- **THEN** every replay selects the latest successor run without reporting a merge conflict

#### Scenario: Concurrent successor attempts conflict
- **WHEN** two replicas start different successors from the same terminal frontier
- **THEN** deterministic conflict handling selects one authority candidate and records the competing candidate for adjudication

### Requirement: CLI and phase skill preserve durable ordering
The CLI SHALL expose `prometheus kbd run start` with explicit run ID and reason inputs. `/kbd-new-phase` SHALL invoke it exactly once when status reports a terminal lifecycle. The local PAUSE valve MUST be released only after the signed successor event and compatibility projections commit.

#### Scenario: New phase follows a cancelled run
- **WHEN** `/kbd-new-phase example` runs against a cancelled canonical run
- **THEN** it starts one successor run, creates and activates `example`, and projects `/kbd-assess example` as the exact next work

#### Scenario: Durable rollover fails
- **WHEN** the successor command or projection write fails
- **THEN** the PAUSE valve remains active and the former terminal run remains authoritative
