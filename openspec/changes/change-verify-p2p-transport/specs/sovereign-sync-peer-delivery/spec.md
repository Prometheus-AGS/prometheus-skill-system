## ADDED Requirements

### Requirement: Verify skill-index propagation between paired hosts

Operator qualification of real peer transport SHALL use separate daemon processes on two paired machines and record peer discovery and received skill-index content. A failed exchange SHALL preserve logs for investigation.

#### Scenario: Observe received content on the second host

- **WHEN** both daemons use the paired identity and bootstrap and host A pushes the skill-index domain
- **THEN** host B reports the peer through sync status and its local skill-index search exposes the content pushed by A
