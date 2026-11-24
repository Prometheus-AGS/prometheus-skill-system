## ADDED Requirements

### Requirement: Connect the research UI to real jobs by default

The research UI SHALL load local HTMX and Alpine assets, create and cancel real jobs over HTTP, and use SSE to update progress, logs, errors, and out-of-band components. Simulated jobs SHALL require `?demo=1`.

#### Scenario: Open the normal research UI

- **WHEN** a user starts a query with demo mode disabled
- **THEN** the UI uses the returned job ID to open its stream and renders backend events, and its cancel control targets the same ID

### Requirement: Keep live states accessible

Job controls SHALL retain visible keyboard focus and accessible contrast. Status, progress, and completion feedback SHALL use the shared design tokens and respect reduced motion.

#### Scenario: Work in reduced-motion mode

- **WHEN** a keyboard user enables reduced motion and starts a live job
- **THEN** focus remains visible and stage, progress, and completion information remain available with animations disabled
