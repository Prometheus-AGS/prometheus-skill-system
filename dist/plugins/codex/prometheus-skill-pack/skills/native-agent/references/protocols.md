# Protocol Reference: A2A + AG-UI + A2UI

## A2A (Agent-to-Agent) Protocol

Standardized by Google and the Linux Foundation. Enables agents to discover each
other and exchange tasks.

### Agent Card

Every native agent serves its card at `GET /.well-known/agent.json`:

```json
{
  "name": "research-agent",
  "description": "Research and summarization agent",
  "url": "http://localhost:8081",
  "version": "1.0.0",
  "capabilities": {
    "streaming":  true,
    "tools":      true,
    "multi_turn": true,
    "skills":     ["rust/axum-patterns", "research"]
  },
  "input_modes":  ["text"],
  "output_modes": ["text", "stream"]
}
```

### Task Endpoint

`POST /a2a/tasks` — another agent sends a task here:

```json
{
  "task_id": "task-abc123",
  "message": "Summarize the latest developments in Rust async runtimes",
  "context": { "depth": "technical", "format": "bullet_points" }
}
```

Response:
```json
{
  "task_id": "task-abc123",
  "status":  "complete",
  "result":  "The key developments are..."
}
```

### Wiring Two Generated Agents Together

Agent A wants to use Agent B as a tool. In Agent A's `agent.toml`:

```toml
[[mcp_servers]]
name      = "research-agent"
url       = "http://localhost:8081/a2a/tasks"
transport = "sse"
enabled   = true
```

Agent B's A2A task endpoint then appears as a tool in Agent A's tool list.
The LLM can call it like any other MCP tool.

---

## AG-UI Protocol (CopilotKit)

SSE-based event stream protocol. The generated agent fully supports it.

### Run Initiation

`POST /agui/run`:
```json
{
  "model":    "claude-sonnet-4-6",
  "provider": "anthropic",
  "messages": [{"role": "user", "content": "Hello"}]
}
```
Returns: `{ "run_id": "run-abc123" }`

### Event Stream

`GET /agui/events/:run_id` — SSE stream of `agui.*` events:

```
event: agui.run.started
data: {"type":"agui.run.started","run_id":"run-abc123"}

event: agui.text.delta
data: {"type":"agui.text.delta","run_id":"run-abc123","delta":"Hello"}

event: agui.text.delta
data: {"type":"agui.text.delta","run_id":"run-abc123","delta":" there!"}

event: agui.run.complete
data: {"type":"agui.run.complete","run_id":"run-abc123"}
```

### Tool Calls in AG-UI

```
event: agui.tool.call.started
data: {"type":"agui.tool.call.started","tool_name":"forge_enrich","tool_call_id":"tc-1"}

event: agui.tool.call.result
data: {"type":"agui.tool.call.result","tool_call_id":"tc-1","result":{...}}
```

---

## A2UI Protocol (Prometheus)

The Prometheus-specific combined protocol: A2A task semantics + AG-UI streaming.

`POST /a2ui/session`:
```json
{
  "message":    "Enrich this OpenSpec task",
  "session_id": "optional-for-continuity",
  "stream":     true
}
```

Returns a combined response with A2A task tracking + AG-UI event stream reference.

---

## Frontend Integration (assistant-ui)

The generated `Chat.tsx` uses `@assistant-ui/react` with a custom adapter that:
1. POSTs to `/agui/run` to start a streaming run
2. Opens `GET /agui/events/:run_id` as an EventSource
3. Streams `agui.text.delta` events into the `Thread` component in real time
4. Closes the EventSource on `agui.run.complete` or `agui.run.error`

This means the chat interface works with any AG-UI-compatible backend — the agent
can be replaced with any Prometheus agent that speaks the AG-UI protocol.
