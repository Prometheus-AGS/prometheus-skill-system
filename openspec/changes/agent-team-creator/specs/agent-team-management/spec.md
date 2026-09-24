## ADDED Requirements

### Requirement: Guided selection
The skill SHALL recommend minimal roles with reasons, a single-agent alternative, editable responsibilities, budget and review questions.
#### Scenario: Simple task
- **WHEN** a user supplies a small isolated task
- **THEN** guidance recommends one implementer and explains optional independent review

### Requirement: Native fidelity
The runtime SHALL export eight target harnesses, preserve supplied native configuration and disclose unsupported semantics.
#### Scenario: Native limitation
- **WHEN** exporting MiniMax or DeepSeek
- **THEN** the output documents actual discovery and composition without invented flags

### Requirement: Durable ownership
The runtime SHALL support revision-checked atomic task state and explicit ownership transfer with evidence, Git identity and memory references.
#### Scenario: Stale mutation
- **WHEN** a stale revision attempts a task mutation
- **THEN** the mutation fails without losing current state
#### Scenario: Cross-harness transfer
- **WHEN** a destination accepts a handoff
- **THEN** ownership transfers once using a versioned receipt

### Requirement: Model and memory portability
The runtime SHALL use declared model capabilities and price constraints and SHALL operate without services.
#### Scenario: Unknown price
- **WHEN** a price-limited selection encounters unknown cost
- **THEN** that model is not claimed to satisfy the limit
#### Scenario: Offline memory
- **WHEN** publication fails
- **THEN** work remains usable and the memory record stays queued

### Requirement: Shared distribution
Both packages SHALL ship identical Node.js artifacts authored in TypeScript 7, compliant skills, supported plugins and updated documentation.
#### Scenario: Packaged execution
- **WHEN** copied into a temporary project
- **THEN** the compiled CLI runs without repository-root imports or runtime dependencies
