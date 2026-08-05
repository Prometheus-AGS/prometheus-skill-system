# execution-mcp-surface Specification

## Purpose
TBD - created by archiving change change-exec-004-remote-mcp-docs. Update Purpose after archive.
## Requirements
### Requirement: Shared-service MCP parity
`prometheus-exec --mode mcp` SHALL expose run submission, run status, ordered events, terminal receipt, artifact retrieval, and offline verification through MCP tools backed by the same service interfaces and durable state as REST and embedded callers.

#### Scenario: Response-loss replay through MCP
- **WHEN** an MCP client resubmits a same-ID/same-hash request after losing the original response
- **THEN** it receives the original run identity and receipt without a second execution

#### Scenario: Cross-surface event cursor
- **WHEN** a run created over REST is queried through MCP with an event cursor
- **THEN** MCP returns only later events in the same strictly increasing sequence order

### Requirement: Envelope-free stdio trust boundary
MCP stdio requests SHALL be authenticated by the spawning process boundary, SHALL NOT accept private signing keys in tool arguments, and SHALL still return receipts signed by the configured host identity for portable verification.

#### Scenario: Private key argument rejected
- **WHEN** a caller supplies a private-key field to any execution MCP tool
- **THEN** the request is rejected as invalid input before durable state is created

### Requirement: Bounded deterministic MCP results
MCP tool schemas, success payloads, error envelopes, event pages, and artifact responses SHALL be deterministic, size-bounded, and generated or drift-checked against the canonical execution contracts.

#### Scenario: Oversized artifact retrieval
- **WHEN** an MCP caller requests an artifact larger than the configured inline response ceiling
- **THEN** the tool returns bounded metadata and explicit retrieval guidance without truncating bytes into a false success
