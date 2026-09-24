## ADDED Requirements

### Requirement: Guided team selection
The skill SHALL ask about desired outcome, scope, deliverables, budget and independent review, recommend the smallest useful team with reasons, offer a single-agent alternative, and allow editing roles before export.
#### Scenario: Simple task
- **WHEN** a user supplies a small isolated task
- **THEN** guidance recommends one implementer and explains optional independent review without requiring knowledge of orchestration terminology
#### Scenario: Complex work
- **WHEN** work spans design, implementation and verification
- **THEN** guidance proposes bounded roles with inputs, outputs, skills, dependencies and file ownership
#### Scenario: Expert configuration
- **WHEN** the user already knows the desired team
- **THEN** the user can provide a manifest directly without completing novice questions

### Requirement: Native harness fidelity
The runtime SHALL cover UAR, Codex, Claude Code, Copilot, Kimi Code, MiniMax CLI, OpenCode and DeepSeek Harness with source-linked, version-aware native exports and configuration preservation.
#### Scenario: Arbitrary native configuration
- **WHEN** a manifest supplies native options or files beyond the common fields
- **THEN** their values are preserved in staged artifacts, collisions fail explicitly, and the report distinguishes preservation from native validation
#### Scenario: Native limitations
- **WHEN** MiniMax custom roles or DeepSeek teams are requested
- **THEN** output uses actual native discovery/composition and states selector and experimental limitations without inventing flags
#### Scenario: Existing project configuration
- **WHEN** a generated artifact would replace an existing native file
- **THEN** the tool stages the proposal and refuses silent overwrite
#### Scenario: UAR or BossFang contract
- **WHEN** exporting a registration artifact
- **THEN** it identifies the exact deployed route/schema/version and preserves execution-loop ownership; discovery-only records are not labeled executable teams

### Requirement: Model policy and discovery
The runtime SHALL discover available model identifiers through configured native/gateway interfaces, adapt liter-llm catalog metadata, and resolve explicit policy in team, role, skill and task order.
#### Scenario: Constrained model selection
- **WHEN** a task requires a strength tier, capabilities and price ceiling
- **THEN** selection considers only declared compatible models, reports unknown metadata and explains its choice
#### Scenario: Unknown cost
- **WHEN** a model has no known price
- **THEN** it is not asserted to satisfy a price ceiling
#### Scenario: Unsupported native override
- **WHEN** the destination cannot express a requested per-role model override
- **THEN** export reports the unsupported behavior rather than silently ignoring it

### Requirement: Durable task management
Team state SHALL support explicit task assignment, dependencies, status, cancellation and reassignment with revision checks and atomic persistence.
#### Scenario: Stale writer
- **WHEN** a mutation uses an old revision
- **THEN** it fails without losing current state
#### Scenario: KBD-linked task
- **WHEN** a team task references a canonical KBD work item
- **THEN** the skill records that identity and uses KBD commands for KBD completion instead of creating competing canonical status

### Requirement: Cross-harness handoff
A handoff SHALL carry schema version, source/destination harness, task context, evidence, Git identity, remaining work and memory references, and SHALL transfer ownership only after explicit acceptance.
#### Scenario: Fresh destination context
- **WHEN** work transfers between different harnesses
- **THEN** the destination receives a fresh prompt packet without treating the source session identifier or permissions as portable authority
#### Scenario: Duplicate or conflicting acceptance
- **WHEN** a handoff is accepted twice or against a changed task revision
- **THEN** the operation is idempotent for the same accepted receipt or fails the conflicting transfer
#### Scenario: Unfinished work
- **WHEN** a handoff includes blockers or dirty work
- **THEN** those facts remain explicit rather than being reported as completed or clean

### Requirement: Optional shared memory and Karpathy integration
The skills SHALL preserve local work when memory is unavailable, use actual discovered/configured memory contracts, and maintain provenance and scope.
#### Scenario: Offline publication
- **WHEN** memory publication fails
- **THEN** a durable local receipt/outbox remains available for retry
#### Scenario: KBD boundary
- **WHEN** recording Karpathy progress
- **THEN** canonical identity is validated and only a real successful boundary is emitted; pk remains the sole writer of its knowledge bundle
#### Scenario: Additional provider
- **WHEN** another shared-memory provider is configured
- **THEN** its discovered tool/schema mapping is explicit and it is optional, with no extra required resident service

### Requirement: Portable skills and distribution
Both packages SHALL ship identical runtime source and compiled artifacts authored in TypeScript 7, four AgentSkills-compliant procedures, supported native agent/plugin/marketplace packaging, and updated repository and Docusaurus documentation.
#### Scenario: Packaged execution
- **WHEN** a packaged skill is copied to a temporary project
- **THEN** its compiled Node entry point works without root imports, runtime package installation, shell scripts, symlinks or executable bits
#### Scenario: Native plugin differences
- **WHEN** a harness lacks a verified plugin/marketplace or plugin-agent mapping
- **THEN** documentation and export state the limitation and provide the supported native skill/role path
#### Scenario: Final certification
- **WHEN** publication is attempted
- **THEN** both inventories, generated outputs, local integration and documentation builds have recorded results and unresolved limitations are disclosed

### Requirement: KBD discipline
Execution SHALL follow assessed, analyzed, specified and reviewed plans through kbd-apply task boundaries, then deterministic QA, independent review, verification, archive and reflection.
#### Scenario: Missing stage handoff
- **WHEN** a required predecessor is incomplete
- **THEN** team implementation does not start

#### Scenario: Supported marketplaces
- **WHEN** a harness supports marketplace packaging
- **THEN** its native marketplace entries are generated and validated with the distribution; unsupported marketplace mechanisms are reported explicitly
