---
license: MIT
name: htmx-alpine-lit
version: '1.0.0'
description: >
  HTMX 2.0.8 + Alpine.js + Lit component system for Prometheus AGS server-driven UI.
  Covers HTMX request/response patterns, Alpine.js x-data controllers, Lit web
  components, server-side fragment rendering, and the HTMX-in-React embedding pattern
  for interactive content islands inside React 19 host apps. Use when building
  server-driven dashboards, document viewers, interactive artifact displays, or
  HTMX islands embedded in React applications.
language: typescript
metadata:
  tags: [htmx, alpine, frontend]
---

# HTMX 2.0.8 + Alpine.js + Lit

## Stack Versions

| Package | Version | Role |
|---|---|---|
| HTMX | 2.0.8 | Server-driven partial HTML swaps |
| Alpine.js | 3.x | Declarative JS behavior without build step |
| Lit | 3.x | Web components for reusable complex UI |
| Tailwind CSS | 4 | Utility CSS |

## Core Design Principle

HTMX applications are **server-driven**. The server returns HTML fragments, not JSON.
The browser swaps them into the DOM. JavaScript exists only to:
1. Declare behavior that HTML attributes cannot express (Alpine.js)
2. Encapsulate complex interactive elements (Lit web components)
3. Embed HTMX islands inside a React host (React integration pattern)

**Never** fetch JSON with HTMX and process it in Alpine. If you need JSON, you need
a different architecture.

## HTMX Request/Response Pattern

### Triggering Requests

```html
<!-- GET on click, swap inner content of #result -->
<button
  hx-get="/api/search"
  hx-target="#result"
  hx-swap="innerHTML"
  hx-include="[name='query']"
>
  Search
</button>
<div id="result"></div>

<!-- POST on form submit with loading indicator -->
<form
  hx-post="/api/posts"
  hx-target="#posts-list"
  hx-swap="afterbegin"
  hx-indicator="#spinner"
>
  <input name="title" type="text" />
  <button type="submit">Create</button>
</form>
<div id="spinner" class="htmx-indicator">Loading…</div>
```

### Server Response (returning HTML fragments)

```rust
// In an Axum handler — return an HTML fragment, not JSON
use axum::response::Html;

async fn search_handler(Query(params): Query<SearchParams>) -> Html<String> {
    let results = db.search(&params.query).await?;
    let html = results
        .iter()
        .map(|r| format!("<li class='result-item'>{}</li>", r.title))
        .collect::<Vec<_>>()
        .join("\n");
    Html(format!("<ul>{}</ul>", html))
}
```

### Swap Strategies

| `hx-swap` | Behavior |
|---|---|
| `innerHTML` | Replace inner content (default) |
| `outerHTML` | Replace the element itself |
| `afterbegin` | Prepend before first child |
| `beforeend` | Append after last child |
| `none` | Execute response side-effects, no DOM change |

### Out-of-Band Updates

Update multiple page regions from one response:

```html
<!-- Server returns this fragment -->
<div id="search-results">
  <!-- Main content (replaces hx-target) -->
  <ul>...</ul>
</div>

<!-- Out-of-band: update a different element -->
<div id="result-count" hx-swap-oob="true">
  12 results
</div>
```

## Alpine.js Controllers

Use Alpine for declarative local state that doesn't need server communication.
Keep `x-data` scopes small — one controller per UI component, not per page.

```html
<!-- Dropdown controller -->
<div x-data="{ open: false }" class="relative">
  <button @click="open = !open">Options</button>
  <div x-show="open" x-transition @click.outside="open = false">
    <a href="/edit" hx-get="/modal/edit" hx-target="#modal">Edit</a>
    <a href="/delete" hx-delete="/api/item/1" hx-confirm="Are you sure?">Delete</a>
  </div>
</div>

<!-- Form with validation state -->
<form
  x-data="{ submitting: false, error: null }"
  hx-post="/api/items"
  hx-on::before-request="submitting = true"
  hx-on::after-request="submitting = false"
  hx-on::response-error="error = 'Failed to save'"
>
  <input name="title" :disabled="submitting" />
  <p x-show="error" x-text="error" class="text-red-500"></p>
  <button type="submit" :disabled="submitting">
    <span x-show="!submitting">Save</span>
    <span x-show="submitting">Saving…</span>
  </button>
</form>
```

## Lit Web Components

Use Lit for complex interactive elements that need encapsulated DOM and reactive
properties. Lit components work anywhere — in HTMX pages and in React apps via
`<web-component>` tags.

```typescript
// src/components/forge-viewer.ts
import { LitElement, html, css } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

@customElement('forge-viewer')
export class ForgeViewer extends LitElement {
  static styles = css`
    :host { display: block; }
    .content { font-family: var(--font-mono, monospace); }
  `

  @property({ type: String }) taskId = ''
  @state() private content = ''
  @state() private loading = true

  async connectedCallback() {
    super.connectedCallback()
    await this.loadContext()
  }

  private async loadContext() {
    this.loading = true
    const res = await fetch(`/api/forge/enriched/${this.taskId}`)
    this.content = await res.text()
    this.loading = false
  }

  render() {
    if (this.loading) return html`<div>Loading context…</div>`
    return html`
      <div class="content">
        <pre>${this.content}</pre>
      </div>
    `
  }
}
```

Use the component in HTMX pages:
```html
<forge-viewer task-id="CHANGE-042"></forge-viewer>
```

## HTMX-in-React Embedding Pattern

For React 19 apps that need HTMX-powered interactive islands (e.g., document viewers,
skill dashboards, artifact displays). The pattern uses a `useRef` mount with HTMX
process initialization.

```tsx
// src/components/HtmxIsland.tsx
import { useEffect, useRef } from 'react'

declare global {
  interface Window { htmx: typeof import('htmx.org') }
}

interface HtmxIslandProps {
  /** Initial HTML to render inside the island */
  initialHtml: string
  /** Additional CSS classes for the container */
  className?: string
}

export function HtmxIsland({ initialHtml, className }: HtmxIslandProps) {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current || !window.htmx) return

    // Set initial content
    ref.current.innerHTML = initialHtml

    // Tell HTMX to process the new content
    window.htmx.process(ref.current)

    // Cleanup: remove HTMX event listeners when island unmounts
    return () => {
      if (ref.current) window.htmx.remove(ref.current)
    }
  }, [initialHtml])

  return <div ref={ref} className={className} />
}
```

Load HTMX in the React app's HTML:
```html
<!-- index.html — load HTMX globally before React mounts -->
<script src="https://unpkg.com/htmx.org@2.0.8" defer></script>
```

Usage in React components:
```tsx
// src/features/artifacts/ArtifactViewer.tsx
import { HtmxIsland } from '@/components/HtmxIsland'
import { useEntityView } from '@prometheus-ags/prometheus-entity-management'

export function ArtifactViewer({ artifactId }: { artifactId: string }) {
  // Get the server-rendered HTMX fragment URL from the entity graph
  const { data } = useEntity<Artifact>({
    type: 'Artifact',
    id: artifactId,
    fetch: (id) => api.artifacts.get(id),
    normalize: (raw) => raw,
  })

  if (!data) return null

  // Initial HTML with HTMX attributes — server will handle subsequent interactions
  const initialHtml = `
    <div hx-get="/api/artifacts/${artifactId}/view"
         hx-trigger="load"
         hx-swap="outerHTML">
      <div>Loading artifact…</div>
    </div>
  `

  return (
    <div className="artifact-viewer">
      <h2>{data.title}</h2>
      <HtmxIsland initialHtml={initialHtml} className="artifact-content" />
    </div>
  )
}
```

## Page Layout Template

Full HTMX page with Alpine.js sidebar and Lit component slot:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <title>{{ page_title }}</title>
  <script src="https://unpkg.com/htmx.org@2.0.8" defer></script>
  <script src="https://unpkg.com/alpinejs@3.x.x/dist/cdn.min.js" defer></script>
  <script type="module" src="/components/forge-viewer.js"></script>
  <link rel="stylesheet" href="/styles.css" />
</head>
<body x-data="{ navOpen: false }">
  <nav>
    <button @click="navOpen = !navOpen">☰</button>
    <ul x-show="navOpen">
      <li><a href="/" hx-boost="true">Home</a></li>
      <li><a href="/skills" hx-boost="true">Skills</a></li>
    </ul>
  </nav>

  <main
    hx-get="/api/{{ resource }}"
    hx-trigger="load"
    hx-target="#content"
  >
    <div id="content" class="htmx-indicator">
      Loading…
    </div>
  </main>
</body>
</html>
```

## Forbidden Patterns

- Using HTMX to fetch JSON and parsing it in Alpine — return HTML fragments from the server
- Global Alpine `x-data` on `<body>` — scope controllers to the smallest necessary element
- Lit components that manage server state — Lit handles presentation, HTMX manages server sync
- `hx-boost="true"` on links that open modals — use `hx-target` and `hx-get` explicitly
- Inline `<script>` tags inside HTMX-swapped fragments — HTMX does not re-execute scripts
