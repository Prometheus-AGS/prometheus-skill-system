# A2A 1.0 agent integration reference

Use this reference when generating, reviewing, or migrating code for `@prometheus-ags/entity-graph-a2a`.

The machine-readable runtime surface is [`a2a-library-exports.json`](a2a-library-exports.json). The full release evidence and migration rationale are in [`release/a2a-conformance-agent.md`](../../../release/a2a-conformance-agent.md).

## Non-negotiable rules

1. Use A2A protocol `1.0` and the official package root. Do not recreate protocol Task, Message, Part, Artifact, or status types.
2. Advertise JSON-RPC only. Do not infer REST, gRPC, push, signing, or extended-card support from SDK exports.
3. Serve `/.well-known/agent-card.json`; the server already owns discovery.
4. Use `SendMessage`, `SendStreamingMessage`, `GetTask`, `ListTasks`, `CancelTask`, and `SubscribeToTask`. Slash methods are migration-only.
5. Protocol validity never grants application authority. Authenticate, scope task visibility, authorize entity/action/field access, and obtain destructive approval before graph writes.
6. Authorize every member of a mutation batch before opening its single graph transaction.
7. Carry graph requests in official structured-data Parts with `PROMETHEUS_GRAPH_EXTENSION_URI`.
8. `PROMETHEUS_A2UI_EXTENSION_URI` is the official A2UI A2A extension `https://a2ui.org/a2a-extension/a2ui/v1.0`: DataParts use `metadata.mimeType` `application/a2ui+json` and their `data` is the A2UI 1.0 message list. Send A2UI only when the turn's `message.metadata.a2uiRendererCapabilities` offers it, and route inbound client actions to `a2uiInbound` (deny by default, after `authorize`).
9. Default CI must remain deterministic and keyless. External agent endpoints are explicit opt-in dependencies.
   They must use HTTPS outside loopback development, keep discovery and
   transport on the same injectable fetch boundary, and remap remote lifecycle
   IDs before publishing into the local task boundary.
10. Import pre-v3 compatibility only from `@prometheus-ags/entity-graph-a2a/legacy`; never expose it from the stable root.

## Minimal server pattern

```ts
import {
  buildAgentCard,
  createA2AServer,
  createEntityGraphA2APolicy,
} from "@prometheus-ags/entity-graph-a2a";

const policy = createEntityGraphA2APolicy({
  entities: {
    Task: {
      actions: ["upsert", "replace", "remove", "query"],
      fields: ["id", "title", "status", "projectId"],
    },
  },
  authorize: ({ caller }) => caller.isAuthenticated,
  requestApproval: ({ operation }) => ({
    allowed: operation !== "replace" && operation !== "remove",
  }),
});

export const a2aServer = createA2AServer({
  card: buildAgentCard({ url: "https://api.example.com/a2a" }),
  policy,
});
```

The sample approval function denies destructive work; real approval must come from a trusted host-owned workflow.

## Evidence required for claims

| Claim | Required receipt |
| --- | --- |
| Official task lifecycle and security | `pnpm run test:a2a-conformance` |
| Packed ESM/CJS and declarations | `pnpm run verify:a2a-conformance` |
| Optional external executor | focused tests plus the packed external-executor consumer in `pnpm run verify:a2a-conformance` |
| Upstream selected-binding conformance | `pnpm run test:a2a-tck` and the pinned receipt |
| BDD traceability | `pnpm run bdd:a2a-conformance` |
| Root/legacy export synchronization | package build plus `pnpm --filter @prometheus-ags/entity-graph-a2a run verify:skills` |
| Rendered A2UI application | downstream agentic showcase browser/accessibility/visual receipts; not this package gate |

Never turn a JSON-RPC TCK pass into a broader protocol or platform claim. Every selected-binding skip needs an explicit applicability rationale.

## Alpha migration

Replace old discovery and slash methods with official v1 equivalents. If an incremental migration is unavoidable, wrap the existing `A2AServer` with `createLegacyA2AAdapter` from `./legacy`. The adapter does not support legacy streaming. New generators must not scaffold the legacy subpath.
