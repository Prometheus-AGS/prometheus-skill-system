# change-sync-011: MCP client pool (rmcp, mcp-servers.json)

**Phase:** phase-learn-sovereign-sync
**Tier:** 2 (parallelize with 008, 009, 010 after Tier 1)
**Status:** pending
**Library:** cand-004 (rmcp 1.8.0 client mode)
**Gap:** G-NEW-1

## Summary

Implement an MCP client pool that reads `~/.config/sovereign-sync/mcp-servers.json`
(same format as Claude Desktop config) and spawns one rmcp client per declared
server. Privacy gate strips KB-derived content from MCP tool call arguments
before forwarding to external servers.

## Files to change

- `substrate/sovereign-sync/src/mcp_client_pool.rs` — new file

## Config format (mcp-servers.json)

```json
{
  "mcpServers": {
    "surreal-memory": {
      "command": "npx",
      "args": ["-y", "@anthropic/surreal-memory-mcp"],
      "transport": "stdio"
    },
    "external-api": {
      "url": "http://localhost:3001/mcp",
      "transport": "http"
    }
  }
}
```

## Privacy gate

Before forwarding tool arguments to external MCP servers, strip any fields
that contain palace/KB-derived content (checked via a content-pattern list).
Log stripped fields to local audit log.

## Tasks

- [ ] Implement McpClientPool struct
- [ ] Load mcp-servers.json config
- [ ] Spawn rmcp clients per server (stdio via StdioClientTransport, HTTP via SseClientTransport)
- [ ] Implement aggregated tool registry
- [ ] Implement privacy gate for tool call forwarding
- [ ] Test: pool connects to a local test MCP server and calls a tool
