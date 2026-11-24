## ADDED Requirements

### Requirement: Distribute runnable research scripts and package templates

The skill SHALL include five executable research scripts and five templates. The package manifest SHALL represent job identity, stage completion, sources, graph, contradictions, confidence, optional Feynman grade, and exported files.

#### Scenario: Inspect an exported research package

- **WHEN** the export entry point assembles a completed job
- **THEN** manifest.json parses and maps the job outcome and artifact file inventory to the package
