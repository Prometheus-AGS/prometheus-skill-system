## MODIFIED Requirements

### Requirement: A declared harness has a corresponding tree
Every declared harness SHALL state its source-tree lifecycle as required or install-only. Required trees SHALL exist and contain skills; install-only trees MAY be absent before installation. Removing a harness SHALL remove its declaration and regenerate every derived artifact.
#### Scenario: A declared harness loses its tree
- **WHEN** a required harness has no populated skills directory
- **THEN** validation fails and names that harness
#### Scenario: Install-only source tree is absent
- **WHEN** a harness declares install-only and has no source tree before installation
- **THEN** validation accepts its declared lifecycle without inventing an empty tree
#### Scenario: A harness declaration is removed
- **WHEN** a harness entry is deleted
- **THEN** every generator consuming the manifest is run and its output committed
