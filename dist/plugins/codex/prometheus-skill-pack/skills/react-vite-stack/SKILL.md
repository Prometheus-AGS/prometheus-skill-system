---
license: MIT
name: react-vite-stack
version: '1.0.0'
description: >
  Canonical React 19 + Vite 8 stack for Prometheus AGS frontend projects. Covers
  TanStack Router (file-based routing), TanStack Query (server state), TanStack
  React-Table (data grids), Zustand 5 (client state), shadcn/ui + Tailwind 4
  (component system), and @prometheus-ags/prometheus-entity-management (normalized
  entity graph). Always use .tsx for React components. Use when scaffolding or
  extending any Prometheus AGS frontend application.
language: react
metadata:
  tags: [react, typescript, entity-management]
---

# React 19 + Vite 8 Stack

## Stack Versions

| Package | Version | Role |
|---|---|---|
| React | 19 | UI runtime |
| Vite | 8 | Bundler + dev server |
| TypeScript | 6 | Type system |
| `@tanstack/react-router` | latest | File-based routing |
| `@tanstack/react-query` | latest | Server state + caching |
| `@tanstack/react-table` | latest | Headless data grid |
| `zustand` | 5 | Client state (ephemeral UI) |
| `shadcn/ui` | latest | Component system |
| Tailwind CSS | 4 | Utility CSS |
| `@prometheus-ags/prometheus-entity-management` | latest | Normalized entity graph |

## Project Structure

```
src/
├── main.tsx                     ← app entry, router + query provider setup
├── router.tsx                   ← TanStack Router config
├── routes/
│   ├── __root.tsx               ← root layout (nav, auth guard)
│   ├── index.tsx                ← /
│   └── entities/
│       ├── index.tsx            ← /entities list
│       └── $entityId.tsx        ← /entities/:entityId detail
├── features/                    ← feature-scoped modules
│   └── posts/
│       ├── PostsPage.tsx        ← page component (composes hooks + UI)
│       ├── usePostList.ts       ← TanStack Query or entity hook
│       └── PostsTable.tsx       ← TanStack Table grid component
├── components/                  ← shared UI (shadcn wrappers, layout)
├── lib/
│   ├── query.ts                 ← QueryClient config
│   ├── entity.ts                ← entity graph config (configureEngine)
│   └── utils.ts                 ← cn() and other utilities
└── stores/                      ← Zustand stores (UI-only state)
```

## File Extension Rule

All React component files use `.tsx`. TypeScript-only files (hooks that return
no JSX, utilities, types) use `.ts`. Never use `.jsx` or `.js`.

## Providers Setup

```tsx
// src/main.tsx
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { RouterProvider } from '@tanstack/react-router'
import { QueryClientProvider } from '@tanstack/react-query'
import { queryClient } from './lib/query'
import { router } from './router'
import { configureEngine } from '@prometheus-ags/prometheus-entity-management'

// Configure entity graph engine once at startup
configureEngine({ staleTime: 30_000, gcTime: 300_000 })

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </StrictMode>
)
```

## TanStack Router: File-Based Routes

```tsx
// src/routes/entities/$entityId.tsx
import { createFileRoute } from '@tanstack/react-router'
import { useEntity } from '@prometheus-ags/prometheus-entity-management'

export const Route = createFileRoute('/entities/$entityId')({
  component: EntityDetailPage,
})

function EntityDetailPage() {
  const { entityId } = Route.useParams()
  const { data, isLoading } = useEntity<Post, Post>({
    type: 'Post',
    id: entityId,
    fetch: (id) => api.posts.get(id),
    normalize: (raw) => raw,
  })

  if (isLoading) return <div>Loading…</div>
  return <div>{data?.title}</div>
}
```

## TanStack Query for Non-Entity Server State

Use `useQuery` for data that doesn't need cross-view normalization (user prefs,
config, system status). Use `prometheus-entity-management` for anything with a
`type + id` identity that appears in multiple places.

```tsx
import { useQuery } from '@tanstack/react-query'

export function useSystemStatus() {
  return useQuery({
    queryKey: ['system', 'status'],
    queryFn: () => api.system.status(),
    staleTime: 10_000,
    refetchInterval: 30_000,
  })
}
```

## TanStack React-Table with Entity Graph

```tsx
import { useReactTable, getCoreRowModel, flexRender } from '@tanstack/react-table'
import { useEntityView } from '@prometheus-ags/prometheus-entity-management'
import { textColumn, actionsColumn } from '@prometheus-ags/prometheus-entity-management'

const columns = [
  textColumn<Post>({ id: 'title',  header: 'Title',  accessorKey: 'title' }),
  textColumn<Post>({ id: 'status', header: 'Status', accessorKey: 'status' }),
  actionsColumn<Post>({ onEdit, onDelete }),
]

export function PostsTable() {
  const { items } = useEntityView<Post>({
    type: 'Post',
    queryKey: ['posts'],
    fetch: (p) => api.posts.list(p),
    normalize: (row) => ({ id: row.id, data: row }),
  })

  const table = useReactTable({
    data: items,
    columns,
    getCoreRowModel: getCoreRowModel(),
  })

  return (
    <table>
      {table.getHeaderGroups().map(hg => (
        <tr key={hg.id}>
          {hg.headers.map(h => (
            <th key={h.id}>{flexRender(h.column.columnDef.header, h.getContext())}</th>
          ))}
        </tr>
      ))}
      {table.getRowModel().rows.map(row => (
        <tr key={row.id}>
          {row.getVisibleCells().map(cell => (
            <td key={cell.id}>{flexRender(cell.column.columnDef.cell, cell.getContext())}</td>
          ))}
        </tr>
      ))}
    </table>
  )
}
```

## Zustand 5 for Client State

Use Zustand only for ephemeral UI state that is not server-derived (sidebar open/close,
selected tab, unsaved form draft before entity mutation). Never duplicate server data
in Zustand — that's what the entity graph is for.

```tsx
import { create } from 'zustand'

interface UIStore {
  sidebarOpen: boolean
  setSidebarOpen: (open: boolean) => void
}

export const useUIStore = create<UIStore>((set) => ({
  sidebarOpen: false,
  setSidebarOpen: (open) => set({ sidebarOpen: open }),
}))
```

## shadcn/ui Patterns

shadcn/ui components live in `src/components/ui/` (generated by shadcn CLI).
Wrap them in feature-specific components — never use shadcn primitives directly
in page components.

```tsx
// src/components/ui/data-table.tsx — wrapper, not a raw shadcn import
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from './table'

// src/features/posts/PostsTable.tsx — uses the wrapper
import { DataTable } from '@/components/ui/data-table'
```

## Forbidden Patterns

- `.jsx` file extensions — always `.tsx` for React components
- `import React from 'react'` — not needed in React 19
- `// @ts-ignore` — fix the type error
- Direct Zustand for server data — use entity graph or TanStack Query
- `any` type — use `unknown` and narrow, or define explicit types
- Default exports from hook files — use named exports
