---
name: legacy-app-embed
description: Embed a frozen or independently deployed legacy application inside a governed shell through a versioned bridge. Use for iframe integration, postMessage protocols, origin validation, correlation IDs, compatibility negotiation, or frozen-substrate constraints.
---

# Legacy Application Embed

Treat the embedded application as an untrusted external system.

## Bridge contract

- Pin allowed origins and bridge versions.
- Use a discriminated, schema-validated message envelope.
- Require request, correlation, and session IDs.
- Validate origin, source window, version, message type, and payload before use.
- Permit only explicitly declared capabilities.
- Time out requests and make mutating requests idempotent.
- Keep authentication tokens, tenant IDs, and policy decisions out of messages.
- Record redacted bridge outcomes for audit.

The host may translate authorized intent into its own BFF or UAR calls. The
embedded app never receives raw runtime, database, or tool-governance access.

Test hostile origins, replay, malformed payloads, unknown versions, navigation
away, reload during an in-flight request, and duplicate responses.
