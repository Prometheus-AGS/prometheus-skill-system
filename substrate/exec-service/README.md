# `prometheus-exec-service`

The shared durable run facade and sidecar API. It owns request replay/conflict behavior, spawn boundaries, ordered events, response-loss reconciliation, receipt publication, CAS ownership, health/readiness, and the same-user Unix-socket REST surface used by the CLI daemon.

Complete the production phase before validation. Acceptance must exercise a
production entry point with real collaborating components across its filesystem,
process or protocol boundary. Run the applicable local integration gate and record
source identity, commands, results and limitations; a module-only or unit suite
is not release evidence.


Canonical documentation: [local API, CLI, and MCP](../../site/docs/execution/local-api-cli-and-mcp.md) and [installation, doctor, and recovery](../../site/docs/execution/installation-doctor-and-recovery.md).
