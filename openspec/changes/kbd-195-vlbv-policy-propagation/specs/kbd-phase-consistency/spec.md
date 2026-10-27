## ADDED Requirements

### Requirement: Selected method and governing authority are recovered consistently
Lifecycle discovery and recovery MUST resolve the current method, contract identity, included value, exclusions and permitted stage operations from authoritative references. Stored commands, memory and tool inventory MUST NOT grant new execution authority.

#### Scenario: POL-001 fresh and resumed phase
- **WHEN** Init/Assess, a fresh session, child handoff or compaction/resume recovers the same phase
- **THEN** each identifies the same current method and contract revision, value scope, exclusions and stage authority, or reports a missing/stale reference without guessing

### Requirement: Selected VLBV boundaries survive checkpoints
For selected VLBV, generated-code checks MUST wait for the complete agreed value-phase implementation and separately authorized boundary. A governing-policy conflict MUST be surfaced before dependent action without silently modifying operator-owned policy.

#### Scenario: POL-002 premature trigger
- **WHEN** only a task, elapsed hour, commit, partial repair, reviewer request or stale next-command suggests a check
- **THEN** the recovered instructions retain the complete phase boundary and do not authorize generated-code tests, builds, imports, lint, typecheck or probes

#### Scenario: POL-002 host compiler exception conflicts
- **WHEN** host policy permits a narrow compiler exception but selected VLBV forbids it during generation
- **THEN** the conflict is explicit, no generated-code check is inferred from the exception and dependent action waits for a compatible authorized course

### Requirement: Recovery preserves supersession and outstanding effects
Recovery MUST preserve latest authorized supersession, original evidence, candidate freshness, consumed budgets and unresolved effects. It MUST reconcile uncertain prior effects before retry and MUST NOT restore a retired operating method from old instructions.

#### Scenario: POL-003 stale method and uncertain effect
- **WHEN** old handoff text names a superseded method and an attempted action has an uncertain effect
- **THEN** recovery selects the latest authorized method, retains prior evidence and consumption, and identifies reconciliation needed before retry

### Requirement: Control claims match the supported boundary
Guidance MUST distinguish instruction-only behavior, observed behavior, enforcement at a named trusted boundary and unsupported behavior. Completion and publication cadence MUST NOT imply release authority or create an automatic successor.

#### Scenario: POL-001 instruction-only host
- **WHEN** a host can carry instructions but cannot intercept arbitrary generated-code execution
- **THEN** its export reports instruction-only control and makes no universal enforcement claim
