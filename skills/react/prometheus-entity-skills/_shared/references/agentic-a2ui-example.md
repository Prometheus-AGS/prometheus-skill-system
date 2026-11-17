# Agentic A2A and A2UI application reference

Load this reference when generating, reviewing, or documenting a complete
application that streams an A2A task into an A2UI surface and lets the user act
on normalized entities.

Read the two protocol references with it:

- [`a2a-conformance-agent.md`](a2a-conformance-agent.md) owns A2A v1 transport,
  task visibility, lifecycle, and TCK limits.
- [`a2ui-protocol-bridge.md`](a2ui-protocol-bridge.md) owns official A2UI
  1.0 (and v0.9.1) processing, rendering, catalog, and action-policy limits.

The runnable repository example is
[`examples/agentic-a2ui-app`](../../../examples/agentic-a2ui-app/README.md), and
its engineering boundary is
[`release/agentic-a2ui-example.md`](../../../release/agentic-a2ui-example.md).

## Required composition

```text
Component -> hook -> session store -> A2A adapter
Official A2UI renderer -> action policy -> command store -> canonical graph
```

- Keep components free of direct graph and store access.
- Keep the default reference agent deterministic and free of model credentials.
- Make an external A2A endpoint an explicit opt-in; require HTTPS outside
  loopback development, reject URL-embedded credentials, and keep authentication
  in a server-side gateway.
- Validate the complete streamed message batch before mutating the live A2UI
  surface; a later rejection must not partially commit an earlier update.
- Allowlist exact A2UI components, functions, and action names.
- Validate every action context before authorization.
- Apply application-owned tenant and scope authorization after protocol
  validation.
- Require trusted human approval for destructive or archival actions.
- Admit only one pending destructive approval and fail overlapping requests
  closed without replacing the active decision.
- Deny and resolve any pending destructive approval before starting another
  scenario or resetting its state.
- Clear prior official surfaces when a new scenario starts or resets so a failed
  run cannot display stale agent UI.
- Route approved mutations through a command store into one canonical graph.
- Keep list membership as IDs and project entity data at read time.
- Record decisions without exposing credentials or private authorization data.

## Example action contract

The repository example allows `task.update`, requires approval for
`task.archive`, denies `task.delete` for its deterministic caller, and rejects
unknown actions such as `system.run`. Do not broaden these rules when copying
the example. Define the target application's own exact schemas and authority.

## Required evidence

```bash
pnpm run typecheck:agentic-a2ui
pnpm run test:agentic-a2ui:unit
pnpm run verify:agentic-a2ui
```

Unit tests prove the source policy and lifecycle. A focused canonical package
regression proves message-batch atomicity for an existing surface. The
implemented clean verifier proves the rendered happy, denied, malformed, approval, cancellation,
accessibility, screenshot, and trace claims for the deterministic
source-workspace example. Consult `examples/coverage.json` for the exact paths
and limits; it does not certify packed installation or an external agent.

The example changes no public package export. Do not regenerate
`a2a-library-exports.json` or `a2ui-library-exports.json` unless the built
package entry points actually change.
