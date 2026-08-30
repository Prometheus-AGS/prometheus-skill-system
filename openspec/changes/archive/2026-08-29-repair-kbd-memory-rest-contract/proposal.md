## Why

KBD memory hooks report the installed surreal-memory service as unreachable and call REST routes that were removed, so active phases silently receive stub recall digests and lifecycle events are never mirrored. The service is healthy; the KBD client contract must be brought back into alignment now.

## What Changes

- Discover the canonical local surreal-memory service by default while preserving environment, tool-list, and project-config overrides.
- Normalize configured MCP URLs to the server origin before using REST routes.
- Write lifecycle entities through the current entity contract and retrieve them through supported search.
- Rank relevant same-project events locally and produce a real digest when the service is reachable.
- Expand smoke tests and update all owning memory references.

## Capabilities

### New Capabilities

- `kbd-memory-integration`: Reliable discovery, lifecycle-event mirroring, and prior-context recall through the supported surreal-memory REST API.

### Modified Capabilities

None.

## Impact

Affects the KBD orchestrator memory helper, memory-log hook, memory-recall skill, their shell tests, and memory contract documentation. It does not add a legacy compatibility route to surreal-memory-server.
