# kbd-phase-consistency Specification

## Purpose
TBD - created by archiving change kbd-consistency-before-agent-teams. Update Purpose after archive.

## Requirements

### Requirement: Phase creation completes shared postconditions
Runtime-authoritative and legacy phase creation SHALL update project activePhase without losing unknown fields and SHALL dispatch phase:before exactly once before success.
#### Scenario: Runtime phase creation
- **WHEN** a new phase is created through the production helper with canonical runtime authority
- **THEN** canonical active phase and project activePhase agree, unrelated project fields survive, and a real configured hook observes the new phase exactly once
#### Scenario: Canonical registration rejects creation
- **WHEN** canonical phase creation rejects the requested identity
- **THEN** the helper emits no success, writes no local goals artifact for that rejected attempt and does not update project activePhase or invoke the phase hook
#### Scenario: Invalid project configuration
- **WHEN** existing project JSON is malformed
- **THEN** creation fails before creating a phase or mutating state
#### Scenario: Hook unavailable
- **WHEN** metadata and canonical activation succeed but the hook fails
- **THEN** the helper reports the failure without claiming the hook succeeded and preserves the created phase

### Requirement: Phase progress does not inherit run completion
Phase progress SHALL report phase-local implementation and SHALL NOT attribute run-wide evidence, certification or publication to the phase.
#### Scenario: New phase after certified earlier work
- **WHEN** the run has complete evidence and a new empty phase is projected
- **THEN** the new phase has zero local changes, its unsupported phase-specific dimensions are NOT_TRACKED, and old run evidence remains unchanged in explicitly scoped runCompletion
#### Scenario: Repeated replay
- **WHEN** the same journal revision is replayed and projected twice
- **THEN** output is deterministic and the journal contains no compensating edits to erase run history

### Requirement: Position distinguishes stage and lifecycle
The waypoint and reminder SHALL expose the actual active stage separately from run lifecycle and SHALL identify the immediate parent or null.
#### Scenario: Assessment in a ready run
- **WHEN** activePath.stageId is assess and lifecycle is ready
- **THEN** the reminder shows assess as stage and ready as lifecycle
#### Scenario: Top-level and nested phases
- **WHEN** a top-level or nested phase is active
- **THEN** parentPhase is null for the top level and the immediate parent for the nested phase

### Requirement: Current instructions share one authority contract
Current KBD instructions SHALL route mutations through typed commands, execution through kbd-apply boundaries, and required predecessor handoffs through the implemented gate.
#### Scenario: Missing predecessor
- **WHEN** a required stage handoff is missing
- **THEN** instructions describe the actual failure and remediation, not an automatic legacy exemption
#### Scenario: Execution dispatch is not completion
- **WHEN** execution.md and the dispatch contract are prepared while phase work remains pending
- **THEN** Execute remains active and its completion handoff is deferred until every phase change finishes implementation, QA, independent review and verification/archive
#### Scenario: Completion report
- **WHEN** a tool reports phase completion
- **THEN** it uses phase-local state, separates implementation from evidence, and does not edit generated projections

### Requirement: Verification and authorization are explicit
Current instructions SHALL require completed production before integration evidence, SHALL preserve current QA/adversarial gates, and SHALL distinguish explicit operator authorization from generic defaults.
#### Scenario: This phase scope
- **WHEN** this phase edits and later pushes the two requested code repositories
- **THEN** its authorization is recorded without granting permission to push personal/team knowledge logs or change other products
#### Scenario: Documentation-only change
- **WHEN** a full-pack change contains only documentation
- **THEN** it is not exempt from the current QA/adversarial-review requirements

### Requirement: Apply boundaries require real backend progress
The apply driver SHALL refuse task boundary mutation when backend progress cannot be read. It SHALL NOT substitute synthetic completed or remaining counts.
#### Scenario: Backend launcher fails
- **WHEN** the configured OpenSpec launcher fails before begin-task or end-task
- **THEN** the driver exits nonzero before marking task state or firing completion hooks, preserving pending work
#### Scenario: Backend fails after task mutation
- **WHEN** task mutation succeeds but refreshed progress cannot be read
- **THEN** the driver reports the read failure and does not fabricate final change completion
