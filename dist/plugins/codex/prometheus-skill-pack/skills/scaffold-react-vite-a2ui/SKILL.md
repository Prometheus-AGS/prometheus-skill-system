---
name: scaffold-react-vite-a2ui
description: >
  Layer an A2UI rendering host onto a scaffolded Vite/React project. Adds a
  zustand store that reduces the A2UI message stream into surface state, a hook
  and component that render it via the upstream `@a2ui/react` renderer, and a
  mock message source so the app runs without an agent. The generated app
  renders agent-described UI under `pnpm dev` with no second process. The mock
  source is development-only; production needs a real message transport.
---

# Scaffold React + Vite + A2UI Host

Wraps an existing `scaffold-react-vite` target with an
[A2UI protocol](https://github.com/a2ui-project/a2ui) rendering host.

## Usage

```bash
bash scripts/scaffold-react-vite-a2ui.sh \
  --target <path-to-react-project> \
  [--feature <feature-name>]
```

`--target` must already be a `scaffold-react-vite.sh` project — this is a thin
wrapper in the same shape as `scaffold-react-vite-tauri.sh` and
`scaffold-react-vite-agui.sh`, not a standalone generator.

## What it emits

| Path | Role |
|---|---|
| `src/features/<feature>/stores/surface-store.ts` | Reduces the message stream into surface state; owns all I/O |
| `src/features/<feature>/hooks/use-surface.ts` | Reads the store |
| `src/features/<feature>/components/a2ui-surface.tsx` | Renders via `@a2ui/react` |
| `src/dev/mock-surface-source.ts` | Canned message sequence for development |

It also pins `@a2ui/react@0.10.2` and `@a2ui/web_core@0.10.6` exactly, and
appends a README section.

**Both packages are required.** `a2ui-surface.tsx` imports `MessageProcessor`
from `@a2ui/web_core/v0_9`, and `@a2ui/react` does not re-export it — under
pnpm's strict isolation, installing only `@a2ui/react` leaves that import
unresolvable.

## Layering

Placement follows `references/scaffolds/state-architecture.md`:

- **stores/** own all I/O and the single `initRealtime()` subscription (line 185)
- **hooks/** read stores — never `fetch`, never `EventSource` (line 210)
- **components/** read hooks

The store imports no React; the architectural ESLint config enforces it.

## Rendering is adopted, not written

The surface renders through upstream `@a2ui/react` rather than a hand-written
component switch. That is deliberate: this repository previously shipped a
~30-line `switch` handling three lowercase component names that could never
match its own schema, and it went a full phase without rendering a single
conformant spec. Upstream ships and maintains the component catalog.

## Message semantics

- `createSurface` replaces the surface wholesale.
- `updateComponents` **merges by id** — updating one component leaves its
  siblings intact.
- `updateDataModel` merges, so `{"path": "/x"}` leaves re-resolve without
  components being resent.

## Protocol version

Messages are A2UI **v1.0**, validated by
`references/schemas/a2ui-component.schema.json`. The pinned renderer implements
**v0.9** and supports `createSurface`, `updateComponents`, and
`updateDataModel`; the component translates between the two at a single
boundary.

`deleteSurface`, `callFunction`, and `actionResponse` are reduced into store
state but not rendered — the pinned renderer does not implement them. The
surface reports any it receives rather than ignoring them silently.

## Development only

The mock source replays a canned sequence. `useSurface({ source })` accepts any
`AsyncIterable` of messages, so a real transport — WebSocket, SSE, fetch —
substitutes directly.

## Related

- `scaffold-react-vite` — the base scaffolder this wraps
- `scaffold-react-vite-agui` — sibling host for the AG-UI protocol
- `references/domain/a2ui.md` — A2UI spec refinement (`direct:a2ui`), a
  different concern: validating authored messages, not rendering them
