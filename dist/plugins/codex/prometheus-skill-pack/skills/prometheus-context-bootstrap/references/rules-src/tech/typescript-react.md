---
paths: ['**/*.ts', '**/*.tsx', '**/*.mts', '**/package.json', '**/tsconfig*.json', '**/vite.config.*', '**/next.config.*', '**/eslint.config.*']
---

# TypeScript and React

Loaded when a TypeScript file is read. Not resident.

| Tier | Commands |
|---|---|
| T0 every edit | `tsc --noEmit` (Bun and esbuild strip types without checking them — `tsc` is the gate); ESLint or Biome on the touched file |
| T1 unit complete | `vitest run <file>` (or `bun test <file>`) — watch mode is the inner loop, not a gate |
| T2 phase complete | full `vitest run`; `vite build` or `next build`; `scripts/check-file-lines.sh`; `scripts/check-architecture.sh` |
| T3 milestone only | Playwright e2e; visual regression; bundle-size gate |

Cache `.tsbuildinfo`. Keep e2e to the flows where failure costs money.

## Layering (B-3, B-4) — `Component → Hook → Store → Service → External`

| Layer | Owns | May import | Never |
|---|---|---|---|
| `components/`, `pages/` | render, interaction, layout, a11y | own-feature hooks, `shared/ui`, `components/ui` | stores, services, PEM, `fetch`, business rules |
| `hooks/` | UI state coordination, selectors | own-feature stores | services, `@prometheus-ags/*`, `fetch`, `EventSource` |
| `stores/` | application state, the single source of truth; PEM's imperative graph API | own-feature services, `@prometheus-ags/entity-graph-core` | React, JSX, render logic |
| `services/` | every external call: Forge client, server API, AG-UI client, storage | `fetch`, `EventSource`, `@ag-ui/client`, transports | React, stores, UI |

Reverse flow only through reactive state. No manual refresh, no imperative UI sync. A rule that exists only
in a component is a bug (B-5). ESLint enforces the edges; never disable a rule to get green.

## Feature folders (B-2)

`src/features/<feature>/{components,hooks,stores,services,entities,schemas,types,pages,tests}` — create only
the folders a feature needs. `src/app/` wires routes and providers; `src/shared/` holds what two or more
features use. Features never import each other. No `utils/`, `helpers/`, `common/` dumping grounds.

## File names — kebab-case, always

Every file and directory in a React package is kebab-case: `roster-store.ts`, `use-squad.ts`, `mark-card.tsx`,
`mark-card.test.tsx`, `forge-client.ts`. **Identifiers do not change**: `mark-card.tsx` exports `MarkCard`,
`use-squad.ts` exports `useSquad`, `roster-store.ts` exports `rosterStore`. Suffixes carry the role:
`-store`, `-service`, `use-` prefix for hooks, `.types.ts`, `.schema.ts`, `.test.ts(x)`. Enforced by
`eslint-plugin-check-file`. Rename algorithm and edge cases (acronyms, numerals, multi-part extensions):
the skill pack's `artifact-refiner/references/scaffolds/kebab-case-rename.md`.

## File size

No file over 500 lines (`max-lines` in ESLint; `scripts/check-file-lines.sh`). Before it gets there, turn
`roster-store.ts` into `roster-store/{index.ts,state.ts,actions.ts,selectors.ts}`; a component into
`mark-card/{index.tsx,mark-card-header.tsx,…}`. Split by responsibility, keep the import path stable via
`index.ts`, never split by line count alone. Vendored registry components are the only exemption and must
be listed in `rules/line-limit-allowlist.txt`.

Strong types (B-7): no `any`, no stringly-typed domain values, schema-derived types where a schema exists.
