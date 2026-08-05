# execution-release-distribution Specification

## Purpose
TBD - created by archiving change change-exec-004-remote-mcp-docs. Update Purpose after archive.
## Requirements
### Requirement: Strict signed binary installation
The strict installer SHALL build or select the certified `prometheus-exec` binary, verify its expected version and hash, install it atomically, apply the platform signature, and verify the installed identity before reporting success. Development best-effort mode SHALL report every skipped or failed step without claiming installation.

#### Scenario: Post-install version mismatch
- **WHEN** the installed binary does not report the release version or does not match the certified hash
- **THEN** installation fails and does not print a success result

### Requirement: Non-mutating execution diagnosis
Root and component doctor surfaces SHALL verify the execution binary, signature/hash, service definition and loaded state when requested, socket permissions, readiness, receipt-log/CAS reconciliation, active component trust, and remote queue configuration without contacting excluded KBD or Sovereign services or mutating state.

#### Scenario: Excluded services stay unevaluated
- **WHEN** doctor runs with KBD and `service:sovereign-sync` excluded
- **THEN** no excluded check is constructed, contacted, installed, restarted, or rewritten while execution diagnostics still run

### Requirement: Transactional component distribution
The reference execution component, its exact hash, capability metadata, and search/mobile parity metadata SHALL be included in one Ed25519-signed immutable plugin generation. Activation and rollback SHALL switch payload and index through one pointer after signature verification and SHALL emit signed receipts for all 14 supported targets.

#### Scenario: Component tampering before activation
- **WHEN** the generated component bytes differ from the signed manifest hash
- **THEN** activation is rejected and the prior generation remains active

#### Scenario: Target receipt mismatch
- **WHEN** any supported target receives different component or index bytes
- **THEN** publication fails and no universal distribution success is claimed

### Requirement: Unified release metadata
The product, binary, plugin, OpenAPI, Docusaurus, generated reference, and installation manifest SHALL agree on the release version and execution capability status.

#### Scenario: Version drift
- **WHEN** any managed execution release surface reports a different version
- **THEN** the local release contract fails before installation or publication
