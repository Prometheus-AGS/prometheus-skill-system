## ADDED Requirements

### Requirement: Publish documentation with scoped permissions

The Pages workflow SHALL publish only the intended main-branch documentation artifact, restrict deploy permissions to its deployment job, and pin third-party actions by revision. The deployment workflow SHALL be scoped to site sources, docs sources, and its own definition.

#### Scenario: Deploy an approved main-branch documentation artifact

- **WHEN** the Pages workflow receives a deployable documentation artifact
- **THEN** the deployment job alone has Pages and identity-token write permissions, and it publishes the artifact to the configured Pages environment
