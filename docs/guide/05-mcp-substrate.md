# 05 · MCP connectivity and services

MCP configuration tells a harness how to contact a tool. It does not install a service, prove authentication or execute inference. `scripts/mcp-port-table.json` is the full pack's declared connectivity source; `configure-mcp-all-tools.sh` merges the selected tool configuration.

| Component | Declared transport | Shipped default |
|---|---|---|
| surreal-memory | SSE/HTTP MCP | `http://localhost:23001/mcp/sse` |
| prometheus-knowledge | HTTP MCP | `http://localhost:8942/mcp` |
| Forge | HTTP JSON-RPC | `http://localhost:8943/mcp` |
| liter-llm | stdio MCP | `liter-llm mcp --transport stdio --config <explicit-config>` |
| sycophancy-correction | On-demand stdio | Installed binary from the pinned skill package |
| sequential-thinking | On-demand stdio | Configured npm MCP package |
| Tavily / Firecrawl | On-demand stdio adapters | Configured package and environment credentials |

These defaults are configuration observations, not a promise that all adapters exist in a session. The liter-llm MCP process needs an explicit config path; a configured gateway alias or successful model listing is not demonstrated inference. Optional hosted web adapters require their own credentials and policies.

## Operations and scope

Follow [Services, ownership and recovery](26-service-operations.md) for source owners, platform templates, database identity, install/start/stop and backups. Native memory defaults to namespace/database `memory/mcp`; mini Compose uses `memory/main_local_384`. A matching port does not merge those stores.

Local KBD uses its signed runtime without a sync daemon. Connected control and replication belong to optional Companion through the [integration contract](/docs/kbd/integration-contract); no Sovereign service is declared in the current MCP port table.

Memory recall, hook enqueueing, worker reconciliation and knowledge snapshots have separate paths. See [Memory and Learning](06-memory-and-learning.md), [team memory](24-agent-teams.md#keep-lessons-scoped-to-their-audience) and [hook lifecycle](15-hooks-and-lifecycle.md). Missing remote services can leave durable work pending; inspect the receipt rather than interpreting exit zero as delivery.

Previous: [The Four-Layer Pipeline](04-four-layer-pipeline.md) · Next: [Memory and Learning](06-memory-and-learning.md).
