# wasm-component-execution Specification

## Purpose
TBD - created by archiving change change-exec-003-tier-w-mobile. Update Purpose after archive.
## Requirements
### Requirement: Typed component execution
Tier W SHALL execute only components valid for `prometheus:component@0.1.0` through the typed `capabilities.wit` host boundary and SHALL never interpret a component as an ambient WASI command.

#### Scenario: Reference component
- **WHEN** the authorized reference component receives canonical declared inputs
- **THEN** it executes through the typed interface and produces content-addressed streams and outputs

### Requirement: Hard capability boundary
Tier W SHALL expose only granted typed host capabilities. `host:exec` and `host:memory` SHALL remain unavailable, and an unsupported import SHALL fail before component code runs.

#### Scenario: Native execution import
- **WHEN** a component imports `host:exec`
- **THEN** validation rejects the component without instantiation or Tier P fallback

### Requirement: Resource-bounded execution
Every Tier W store SHALL enforce fuel, epoch interruption, memory/table/instance limits, stream limits, and artifact-output limits. Exhaustion SHALL produce a terminal failed receipt identifying the enforced fence.

#### Scenario: Fuel exhaustion
- **WHEN** a component consumes its configured fuel budget
- **THEN** execution traps, no further host call runs, and the signed receipt records the bounded failure

### Requirement: Cross-backend deterministic verification
For identical authorized component bytes, canonical inputs, capabilities, and granted nondeterministic values, Pulley and Cranelift SHALL produce the same canonical deterministic receipt projection and output hashes.

#### Scenario: Pulley to Cranelift replay
- **WHEN** a Pulley-produced verified receipt is replayed with the same material under Cranelift
- **THEN** offline verification succeeds only when every deterministic field and referenced output matches bit-for-bit

### Requirement: Honest backend availability
Tier W SHALL select only a backend permitted by the target platform. iOS SHALL use Pulley, desktop SHALL support Cranelift, and unavailable backends SHALL return `tier_unavailable` without unsandboxed fallback.

#### Scenario: JIT unavailable
- **WHEN** executable memory is unavailable on a mobile target
- **THEN** Tier W selects the certified Pulley profile or reports unavailable before execution
