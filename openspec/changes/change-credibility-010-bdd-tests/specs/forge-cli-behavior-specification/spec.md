## ADDED Requirements

### Requirement: Exercise Forge through its CLI

The Forge behavior suite SHALL exercise validate and enrich through the production executable, assert failure for an invalid task path, and be callable through the documented Cucumber script without a daemon.

#### Scenario: Reject an out-of-project task

- **WHEN** the Cucumber step invokes the real Forge CLI on a task outside the project
- **THEN** the recorded result is nonzero and the daemon-free scenario detects the rejection
