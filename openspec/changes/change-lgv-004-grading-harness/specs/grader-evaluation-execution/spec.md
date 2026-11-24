## ADDED Requirements

### Requirement: Execute the published grading protocol per indexed item

The harness SHALL invoke the agent-executed learn-grade protocol for each indexed item and retain one grade result JSON per item under `results/<item_id>.json`.

#### Scenario: Reconcile after a grading batch

- **WHEN** all indexed items have been dispatched for grading
- **THEN** the recorded result identities match the index and each raw result uses the grade schema
