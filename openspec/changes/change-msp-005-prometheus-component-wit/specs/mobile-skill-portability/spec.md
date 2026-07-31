## ADDED Requirements

### Requirement: One WIT family supersedes the divergent worlds

The prometheus:component@0.1.0 family SHALL parse under wasm-tools, its skill world SHALL be a superset of UAR's run contract, and a mapping SHALL record how each existing target relates to it, including any that cannot be expressed. The change SHALL abort if UAR's discovery path no longer reads the submodule skills dir.

#### Scenario: A changed discovery path aborts the change

- **GIVEN** UAR no longer discovers components in the submodule skills dir
- **WHEN** the precondition check runs
- **THEN** the change aborts rather than proceeding
