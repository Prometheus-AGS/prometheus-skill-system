## Purpose

Define how KBD plans recommend an evidenced concrete model and executable route for every task while preserving task identity and existing execution ownership.

## ADDED Requirements

### Requirement: Every task has a scoped model assignment

The planner SHALL assign every task a row keyed by full phase path, change ID and backend task ID, containing requirements, concrete provider/model, supported effort, dated rationale, harness/route, native alternative when different, availability/evidence status and unresolved prerequisites. Generated tasks SHALL reconcile with these assignments before handoff.

#### Scenario: Repeated task numbers

- **WHEN** two changes both contain task 1.1
- **THEN** their model rows remain distinct and task IDs, titles and checklist syntax remain unchanged.

### Requirement: Selection prioritizes evidenced task quality

The planner SHALL honor explicit model choices, project policy and budget constraints, then prioritize demonstrated task suitability over cost and latency. Unknown metadata SHALL remain explicit; provider branding alone SHALL NOT establish capability or quality.

#### Scenario: Cheapest model is a weaker fit

- **WHEN** two eligible models differ in evidenced task suitability and no cost-first policy overrides the default
- **THEN** the better-fit model is recommended with evidence even if the other is cheaper.

#### Scenario: Unknown capability or stale evidence

- **WHEN** required capability evidence is missing or model information is stale
- **THEN** the plan records the uncertainty and any verification prerequisite rather than claiming a verified match.

### Requirement: Routes respect harness capabilities

The planner SHALL inspect installed capabilities for Codex, Claude Code, OpenCode, DeepSeek Harness or Kimi Code as applicable. External model inference SHALL use supported liter-llm interfaces; tasks requiring workspace actions SHALL also identify a supported tool-enabled worker route. Planning SHALL NOT launch workers, configure providers or store credentials.

#### Scenario: External model requires workspace tools

- **WHEN** the preferred model is unavailable through the active harness's native selector
- **THEN** the assignment identifies liter-llm inference and a documented worker launcher, or marks the route unresolved if no such launcher exists.

#### Scenario: Gateway unavailable

- **WHEN** the selected gateway cannot be used
- **THEN** the recommendation remains visible with its unresolved prerequisite and native alternative, without silent substitution or blocking unrelated eligible tasks.

### Requirement: Execution preserves the selected assignment

Execution SHALL carry assignments into its dispatch contract and recheck them before dispatch. The KBD apply driver SHALL retain canonical task hooks and completion ownership. Task, harness or catalog changes SHALL trigger reselection; legacy plans SHALL receive explicit current selections.

#### Scenario: Task scope changes after planning

- **WHEN** a task requires capabilities absent from its recorded assignment
- **THEN** its assignment is revised before dispatch, while task identity remains intact.

#### Scenario: Legacy plan

- **WHEN** a plan lacks task assignments
- **THEN** execution records an explicit current selection without requiring runtime-state migration.

### Requirement: Latest stable OpenSpec lifecycle
The packs SHALL standardize on latest stable upstream OpenSpec, with release baseline1.14.0, and refresh generated instructions for existing KBD/OpenSpec projects at startup and phase entry through a portable managed runner. Authored specs, changes, task completion and KBD state SHALL be preserved. Subsequent OpenSpec commands SHALL use the selected CLI. Explicit overrides, offline freshness uncertainty, backup locations and incomplete migrations SHALL be visible in receipts; authored Markdown SHALL NOT be relabeled as semantically migrated.

#### Scenario: Existing project startup and phase entry
- **WHEN** startup or a phase/child creation or entry encounters an existing KBD/OpenSpec project
- **THEN** the runner checks latest stable, refreshes generated instructions with a backup, and records the version and result before OpenSpec-dependent work

#### Scenario: Registry or migration unavailable
- **WHEN** latest discovery is offline or generated-artifact refresh cannot complete
- **THEN** cached execution reports freshness uncertainty or the refresh remains pending, preserves authored work, and does not claim an upgrade succeeded

#### Scenario: Installed package and task ownership
- **WHEN** the runner is invoked from either generated pack without source node_modules or a global OpenSpec executable
- **THEN** its full module closure and selected CLI resolve, and KBD retains task identity and completion ownership
