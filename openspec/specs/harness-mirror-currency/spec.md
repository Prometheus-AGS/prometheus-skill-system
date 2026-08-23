# harness-mirror-currency Specification

## Purpose
TBD - created by archiving change change-drift-400-openspec-110-upgrade. Update Purpose after archive.

## Requirements

### Requirement: Vendored harness mirrors are adopted deliberately
An upgrade written into the vendored harness trees by an external generator SHALL be
committed as its own change, with the generator version and the shipped behaviour named,
and SHALL NOT be mixed with unrelated decisions.

#### Scenario: An external generator rewrites the harness trees
- **WHEN** `openspec update` modifies vendored skill or command files
- **THEN** the change adopting them SHALL state the generator version transition and the
  behavioural additions, so a large mechanical-looking diff is legible to a later reader

#### Scenario: An unrelated concern is dirty at the same time
- **WHEN** the working tree also carries a decision such as a harness removal or a
  submodule pointer move
- **THEN** that concern SHALL be committed separately, so no decision is buried inside a
  generator upgrade
