## Purpose

Select a finite set of useful scenarios and preserve their identity through BDD authoring, authorized real-path observation and honest phase closure.

## ADDED Requirements

### Requirement: Scenario discovery is bounded and calibrated
Discovery MUST propose a finite frontier with actor, useful outcome, source or explicit assumption, reason-now, likelihood or UNKNOWN, mandatory constraints, excluded alternatives and next decision. Owner-reviewed calibration MUST distinguish relevance from mandatory constraints; proposals MUST NOT adopt scope or grant execution.

#### Scenario: SCN-001 ordinary rejected and rare-critical examples
- **WHEN** reviewed examples include an ordinary useful operation, an available irrelevant test and a rare mandatory integrity constraint
- **THEN** discovery selects the useful operation and mandatory constraint, defers the irrelevant test with a reason and preserves the reviewed examples without fabricating probability estimates

#### Scenario: SCN-001 unfamiliar candidate after context change
- **WHEN** a material model/context change introduces an unfamiliar candidate
- **THEN** its relevance is reviewed against the finite frontier before adoption and it does not silently become a blocking requirement

### Requirement: Author-only BDD preserves scenario and oracle identity
Author-only invocation MUST consume approved stable scenario IDs and produce declarative features, real assertions, step definitions and configuration linked to the existing contract. It MUST name the oracle's source of expected truth and excluded false positive, and obey governing authoring policy without execution, imports, installation or protected-scenario mutation.

#### Scenario: SCN-002 JS and Rust author-only invocation
- **WHEN** an authorized author-only request supplies adopted scenarios to either existing BDD adapter
- **THEN** it emits linked authoring artifacts without running checks, loading test-bearing modules, installing dependencies or changing protected scenarios

#### Scenario: SCN-002 unknown scenario or disallowed authoring stage
- **WHEN** the requested ID is unadopted or host policy defers executable authoring until production completes
- **THEN** the invocation identifies the missing adoption or stage authority and does not author executable tests or run a dry-run

### Requirement: Real-value verification requires a complete authorized phase
Verification MUST resolve the current contract, candidate, selected scenarios, runner capability, permissions, limits and resource ownership at the complete agreed value-phase boundary. It MUST use the smallest adequate existing real-path runner and retain the actual observation and assertion identity.

#### Scenario: VER-001 real operation and sensitivity challenge
- **WHEN** the complete authorized phase executes its selected useful operation and safe negative control
- **THEN** retained evidence demonstrates the intended result and oracle sensitivity across the actual production boundary

#### Scenario: VER-001 green process without assertion
- **WHEN** a process exits zero while the required observation or assertion is absent, disabled or always true
- **THEN** acceptance does not pass and evidence identifies the unexercised obligation

### Requirement: Attempts and repair effects remain attributable
Verification MUST retain contract/scenario/assertion identities, source/build/configuration/fixture provenance, material attempts, real/substituted boundaries, cleanup and limitations. Product failures MUST remain distinct from verification failures; uncertain effects MUST remain explicit and prevent blind retries.

#### Scenario: VER-002 invalid evidence or environment
- **WHEN** the candidate/configuration differs, oracle is stale, runner or permission is unavailable, an obligation is unexercised, or cleanup fails
- **THEN** the corresponding FAIL, BLOCKED or NOT_EXERCISED disposition names the observed cause without reporting unobserved product failure or acceptance

#### Scenario: VER-002 targeted repair lineage
- **WHEN** a failed attempt justifies an authorized repair
- **THEN** the agreed repair implementation finishes before targeted reverification and prior failures, consumption and unresolved effects remain linked

### Requirement: Closure reports distinct claims without creating work
The procedure MUST send supported evidence to the existing canonical writer and separately report authored implementation, measured contract acceptance, demonstrated runtime value, unresolved gaps, release eligibility and release authority. It MUST NOT fabricate unsupported closure dispositions, successors or publication.

#### Scenario: VER-003 canonical closure and stop
- **WHEN** bounded observations support a terminal decision under existing canonical authority
- **THEN** the closure reports the separate claims and outstanding obligations, leaves unsupported dispositions refused and creates no new phase, task or release automatically
