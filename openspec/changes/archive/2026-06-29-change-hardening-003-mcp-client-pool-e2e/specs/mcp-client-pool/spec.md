## ADDED Requirements

### Requirement: MCP client pool forwards stdio tool calls

`McpClientPool` SHALL forward a single MCP `tools/call` request to a configured stdio child-process server.

#### Scenario: Happy-path tool forwarding

- **GIVEN** an MCP server entry with a command, arguments, and no allow-list restrictions
- **WHEN** `McpClientPool::call_tool` is invoked for a tool exposed by that server
- **THEN** the pool starts the child process
- **AND** performs the MCP initialize handshake
- **AND** sends the initialized notification
- **AND** forwards a `tools/call` request with the requested tool name and arguments
- **AND** returns the upstream MCP result.

### Requirement: MCP client pool enforces allowed tools

`McpClientPool` SHALL reject calls to tools that are not included in a configured server's `allowed_tools` list.

#### Scenario: Disallowed tool is rejected locally

- **GIVEN** an MCP server entry whose `allowed_tools` list contains `echo`
- **WHEN** `McpClientPool::call_tool` is invoked for `blocked`
- **THEN** the pool returns an error before forwarding the call to the upstream server.

### Requirement: MCP client pool surfaces upstream failures

`McpClientPool` SHALL report upstream MCP errors and premature child-process exits to callers.

#### Scenario: Upstream returns JSON-RPC error

- **GIVEN** a configured MCP child-process server
- **WHEN** the upstream server returns a JSON-RPC error for `tools/call`
- **THEN** the pool returns an error that includes the upstream failure.

#### Scenario: Upstream exits before handshake

- **GIVEN** a configured MCP child-process server command that exits before sending an initialize response
- **WHEN** `McpClientPool::call_tool` is invoked
- **THEN** the pool returns an error indicating that the server exited before the expected response.
