---
paths: ['**/uar/**', '**/agent/**', '**/agui/**', '**/a2ui/**', '**/a2a/**', '**/tools/**', '**/skills/**/SKILL.md']
---

# Agent runtime (G-3, A-12, A-14)

Loaded when agent, tool, protocol or skill files are read. Not resident.

## Auditability (G-3)

Agentic execution without an audit trail is not acceptable. Preserve, for every run: the request, each
decision, tool calls with inputs and outputs, files changed, external effects, errors and human approvals.
The trail is append-only, carries ids and timestamps, and is replayable.

## Human override (A-12)

Every automated decision stays inspectable, auditable, overridable and recoverable. Agents act
autonomously **within** a phase. Humans gate architecture, skill and rule promotion, escalations, phase
boundaries and knowledge-base promotion. An approval state is rendered to the user, never inferred.

## Artifacts and state (A-14)

Business state lives in explicit systems — databases, event streams, explicit stores, durable queues —
never in UI components, untracked globals, implicit caches or agent-only memory. Artifacts are typed,
versioned, inspectable, portable and replay-safe; use a formal schema where one exists. Unknown event or
block types are preserved and surfaced, never treated as success.

## Runtime ownership

- One runtime owns the agent loop, model routing and tool governance. Application code never writes a
  competing loop and never touches raw MCP transport.
- Tools call domain code. A model never performs a calculation the domain crate owns.
- Tool governance, in order: resolve a stable tool identity → validate input against a pinned schema →
  classify effects independently of any annotation → evaluate policy with the verified session → request
  approval when required → execute with an idempotency id, a deadline and cancellation → validate and
  redact output → append the audit record.
- Agent runs stream over SSE with event ids and `Last-Event-ID` replay. Resume is part of the contract.

## Skills

A skill is a procedure, not a rule: nothing from the constitution belongs in one. Writing or changing a
`SKILL.md`: keep the description specific enough to route by name, read any third-party skill before it is
installed, and never auto-update a skill from an agent's own evaluation of its own output (D-6).
