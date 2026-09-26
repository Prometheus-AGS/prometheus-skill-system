# Tauri/React Feature Module Template

Use `scripts/new-feature.mjs <n> tauri` to scaffold this structure.

```
features/<feature-name>/
  api/<feature>Api.ts          ← Rust invoke() wrappers (stores call these)
  stores/<feature>Store.ts     ← Zustand: client-side state (UI, selection, streaming)
  entities/<feature>Entities.ts ← Prometheus Entity Management 3.x: normalized server/entity state
  hooks/use<Feature>.ts        ← Composed hook: what components import
  components/<Feature>*.tsx    ← React components (import only hooks)
  types.ts                     ← Feature-local TypeScript types
  index.ts                     ← Public API for the feature
```

## Layer contract

```
Component → Hook → Store → API (Rust invoke)
                 ↘
                  Prometheus Entity Management → registered transport/API (Rust invoke)
```

**Never:**
- `import ... from '../stores'` in a component
- `invoke()` in a component or hook
- `fetch()` in a component or hook
- Prometheus Entity Management hooks directly in a visual component (compose them in the feature hook)
