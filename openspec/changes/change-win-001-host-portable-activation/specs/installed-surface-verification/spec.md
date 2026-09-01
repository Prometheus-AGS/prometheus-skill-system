## ADDED Requirements

### Requirement: An activation fix is verified on every supported host
A change to the activation substrate SHALL be verified on every supported host, not only the authoring host. Verification SHALL include a host whose filesystem lacks the capability the substrate previously assumed.

#### Scenario: Windows without elevated symlink permission
- **WHEN** activation is verified on a Windows host with Developer Mode disabled
- **THEN** activation completes through the fallback link primitive, and a passing result cannot come from an elevated configuration

#### Scenario: Cross-host identity equality
- **WHEN** the same payload is activated on every supported host in the verification matrix
- **THEN** all hosts report the same bundle identity, and any divergence fails the change

### Requirement: Capability degradation is asserted, not assumed
Verification SHALL assert that a host materializing an entry by a weaker primitive records the degradation and still reports the unchanged identity.

#### Scenario: Copy substituted for a link
- **WHEN** a verification host lacks every link primitive
- **THEN** the materialization record names the substitution and the bundle identity matches the link-capable hosts
