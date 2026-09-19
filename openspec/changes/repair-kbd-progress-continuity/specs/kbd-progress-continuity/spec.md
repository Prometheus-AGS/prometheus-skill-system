## Purpose

Defines how KBD preserves an unambiguous execution position across task
completion, nested phase movement, process restart, context loss, and memory
service outages.

## ADDED Requirements

### Requirement: Canonical task state selects next work

KBD SHALL derive the next change and task from non-terminal canonical work,
ordered by declared sequence and stable ID. In-progress work MUST take
precedence over blocked work, and blocked work MUST take precedence over
pending work. Stored operator intent MUST NOT select the next change or task.

#### Scenario: A completed task advances the cursor
- **WHEN** a task transition makes its change terminal while a later change remains open
- **THEN** canonical state and every generated position view identify the first task of the later change

#### Scenario: Operator intent names old work
- **WHEN** exact-next operator text names a change that canonical task state has completed
- **THEN** the text remains visible as historical operator intent while routing uses the derived open change and task

### Requirement: Boundary identities are unambiguous

Task completion and bottleneck evaluation SHALL carry a qualified change and
task identity. An unqualified task ID or title MAY resolve only when it matches
one canonical task.

#### Scenario: Two changes use the same task ordinal and title
- **WHEN** the driver completes `change-a/1` and `change-b/1` also exists
- **THEN** the completion, guard receipt, and cursor update apply only to `change-a/1`

#### Scenario: An unqualified subject is ambiguous
- **WHEN** a boundary subject matches tasks in more than one change
- **THEN** evaluation fails closed without guessing a task

### Requirement: Position projections agree after navigation and restart

Phase activation SHALL reconstruct the complete ancestor path and derived work
cursor. Top-level activation SHALL clear child, change, and task residue. JSON,
Markdown, reminder, status, and resume views MUST be generated from the same
canonical state.

#### Scenario: A child phase exits to its parent
- **WHEN** the child completes and the parent is activated
- **THEN** every position view contains only the parent path and its derived open work

#### Scenario: A child phase resumes after process restart
- **WHEN** a new CLI process replays state and activates the child with its ancestors
- **THEN** every position view reproduces the child path and the same derived open task

### Requirement: Progress boundaries are canonical and idempotent

The progress recorder SHALL accept a versioned, bounded event with a stable ID,
boundary identity, task class, elapsed hours, touched files, verification
results, commit hash, blocker, and exact next work. It MUST validate the event
against canonical KBD status before writing. It SHALL append at most one
chronological session record and MUST NOT mutate generated KBD projections.

#### Scenario: A successful task boundary is replayed
- **WHEN** the same stable completion event is delivered after process restart
- **THEN** the existing receipt is returned without a second session-log append or memory submission

#### Scenario: An unrelated canonical revision advances
- **WHEN** the same completed boundary is replayed after exact-next, HEAD, touched-file, or unrelated canonical state changes
- **THEN** its stable event ID and semantic receipt identity remain unchanged

#### Scenario: A successor run reuses work IDs
- **WHEN** a new canonical run completes the same phase/change/task identity as an earlier run
- **THEN** the run-scoped event ID creates a distinct progress record

#### Scenario: A stable ID carries altered progress data
- **WHEN** a caller reuses an existing event ID with a different substantive payload
- **THEN** the recorder fails closed without appending or submitting the altered event

#### Scenario: Event identity disagrees with canonical status
- **WHEN** an event reports a completed task that canonical KBD state reports as pending
- **THEN** the recorder exits non-zero, reports the disagreement, and writes no progress record

### Requirement: Memory outage does not lose local progress

The progress recorder SHALL submit the bounded record through project-scoped
`pk` ingestion. If `pk` or its service is unavailable, it SHALL enqueue the
same record in the existing durable memory outbox, report queued or degraded
state, and allow subsequent work to continue.

#### Scenario: Project memory is unavailable
- **WHEN** canonical progress is recorded while `pk` returns a failure
- **THEN** the session log contains one record and the durable outbox contains one idempotent pending operation

#### Scenario: The unavailable event is replayed
- **WHEN** the same outage event is delivered again after restart
- **THEN** no duplicate session record or outbox operation is created

#### Scenario: Project memory does not return
- **WHEN** `pk` exceeds the bounded submission timeout
- **THEN** the recorder terminates the attempt, queues the event durably, and allows work to continue

#### Scenario: The recorder stops before memory submission
- **WHEN** the session record and pending receipt are durable but the process exits before calling the memory transport
- **THEN** replay resumes the original durable delivery intent and produces one accepted or queued operation

### Requirement: Completion memory follows successful transitions

Task, change, and phase boundary drivers SHALL invoke progress recording only
after the canonical terminal transition and any required postcommit guard
succeed. Failed or partial transitions MUST NOT produce completion records.

#### Scenario: A postcommit guard rejects task completion
- **WHEN** the canonical task transition or its required signed boundary receipt fails
- **THEN** the task completion recorder is not invoked

#### Scenario: A final task completes its change
- **WHEN** the last open task passes its task and change precommit and postcommit guards
- **THEN** the task completion record is emitted first and exactly one change completion record follows

#### Scenario: A phase transition succeeds
- **WHEN** a phase driver commits terminal status and passes its postcommit guard
- **THEN** the phase completion hook records the terminal canonical phase identity before the cursor moves

#### Scenario: Reflection already completed the phase
- **WHEN** next-phase activation observes canonical phase status `complete`
- **THEN** it creates and activates the successor without repeating the terminal transition or phase completion record

### Requirement: Progress records reject likely secrets

The progress recorder MUST reject events containing recognized credential
patterns anywhere in scalar fields, touched paths, verification commands, or
verification summaries.

#### Scenario: A progress field contains a provider token
- **WHEN** a boundary event contains a GitHub, Slack, AWS, or OpenAI credential pattern
- **THEN** the recorder exits non-zero before writing the session log, receipt, outbox, or memory record
