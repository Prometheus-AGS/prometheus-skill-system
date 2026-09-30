## ADDED Requirements

### Requirement: Selective complete distribution
The full pack SHALL install Delivery Cadence with its scripts, references and schemas for Codex, Claude Code, Kimi Code, MiniMax, Zed and OpenCode using real copies and separately recorded pack adapter dependencies.

#### Scenario: Install requested clients
- **WHEN** the operator invokes the selective full-pack installer for all targets
- **THEN** all six supported discovery locations contain the complete owned Cadence payload
- **AND** unrelated skills and the shared plugin generation remain unchanged

### Requirement: Preserve user ownership
The installer SHALL refuse differing unmanaged destinations and modified managed files, and back up managed updates.

#### Scenario: Edited destination
- **WHEN** a destination contains a user modification
- **THEN** installation refuses before replacing that content

### Requirement: Shared payload parity
Full and mini source and plugin copies SHALL contain identical Cadence files with recorded source provenance. Mini SHALL retain its existing restriction against global installation alongside a full pack.

#### Scenario: Publish both packs
- **WHEN** the delivery is committed and pushed
- **THEN** the shared payload hashes agree in both packs and their Claude/Codex payloads
