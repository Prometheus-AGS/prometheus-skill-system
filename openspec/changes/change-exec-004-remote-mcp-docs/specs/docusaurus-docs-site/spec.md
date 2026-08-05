## ADDED Requirements

### Requirement: Canonical execution documentation
The Docusaurus site SHALL document execution positioning, local and remote use cases, Tier P/W/R behavior, request and receipt contracts, MCP/REST/embedded surfaces, policy and grants, artifact retention, verification, certification semantics, installation, doctors, troubleshooting, and honest platform status from the finished code.

#### Scenario: Reader executes and verifies a use case
- **WHEN** a reader follows the canonical local example
- **THEN** the documented commands produce a run, ordered events, signed receipt, content-addressed artifact, and successful offline verification using checked deterministic fixtures

### Requirement: Execution architecture diagrams
The execution documentation SHALL include locally parser-checked Mermaid diagrams for local lifecycle, shared service surfaces, remote dispatch/receipt return, and plugin/component activation, with accessible textual explanations.

#### Scenario: Production diagram rendering
- **WHEN** the production site is built
- **THEN** every execution Mermaid fence parses and renders without a client-side error

### Requirement: Generated execution contract references
OpenAPI routes, MCP tool schemas, request/receipt schemas, CLI options, capability tables, platform status, plugin target counts, and release metadata SHALL be generated or deterministically drift-checked from canonical source inputs.

#### Scenario: Source contract changes
- **WHEN** an execution route, tool schema, or release field changes without refreshing managed documentation
- **THEN** the local documentation check fails with the stale generated artifact

### Requirement: Local-only execution documentation gate
The local `docs:check` entry point SHALL validate execution OpenAPI/examples, MCP schema parity, semantic drift, links/sidebars, Mermaid, public-doc safety, version agreement, and a production Docusaurus build. GitHub workflows SHALL NOT run product tests, lint, doctors, or certification.

#### Scenario: Hosted workflow attempts product validation
- **WHEN** a workflow adds an execution test, lint, doctor, or certification command
- **THEN** the local workflow-policy check rejects the workflow before publication
