---
paths: ['**/*.ts', '**/*.tsx', '**/package.json', '**/tsconfig.json']
---

# TypeScript

Loaded when a TypeScript file is read. Not resident.

| Tier | Commands |
|---|---|
| T0 every edit | `tsc --noEmit` on the touched project; Biome or ESLint |
| T1 unit complete | targeted `vitest run <file>` or `bun test <file>` |
| T2 phase complete | full `vitest run`; `vite build` or `next build` |
| T3 milestone only | Playwright e2e; visual regression; bundle-size gate |

## Hard rules

- Bun and esbuild strip types without checking them. `tsc --noEmit` is the real
  type gate — a green Bun run proves nothing about types.
- Cache `.tsbuildinfo`. Incremental typecheck drops substantially with it.
- Watch mode is the inner loop, not a gate. A gate is a command that exits.
- Keep e2e to the flows where failure costs money, not to everything reachable.

## Structure

Organize by capability under `features/<domain>/`, not by technical layer.
Layer order is UI, then hooks, then stores, then services, then external. A
component does not call a service or mutate a store directly.

Components render and submit intent. No business rule exists only in a
component. No browser storage in artifacts.

<!-- Replace the commands above with this project's real ones if they differ. -->
