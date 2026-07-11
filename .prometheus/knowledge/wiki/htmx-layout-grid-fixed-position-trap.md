---
type: Reference
id: htmx-layout-grid-fixed-position-trap
title: "HTMX Layout: The CSS Grid + Fixed Position Sidebar Trap"
description: "When building a sidebar + main content layout with HTMX/Alpine.js, a common mistake is to use `display: grid` on a container that has a `position: fixed` sidebar inside it."
tags:
- htmx
- css-grid
- layout
- fixed-position
- gotcha
sources:
- manual-backfill
timestamp: 2026-07-10T19:49:34.767928+00:00
created_at: 2026-07-10T19:49:34.767928+00:00
updated_at: 2026-07-10T19:49:34.767928+00:00
revision: 0
---
# HTMX Layout: The CSS Grid + Fixed Position Sidebar Trap

## Problem

When building a sidebar + main content layout with HTMX/Alpine.js, a common mistake is to use `display: grid` on a container that has a `position: fixed` sidebar inside it.

### The Broken Pattern

```css
.app-shell {
  display: grid;
  grid-template-columns: 280px 1fr;  /* WRONG */
}

.sidebar {
  position: fixed;  /* removed from normal flow */
  left: 0;
  width: 280px;
}

.main {
  margin-left: 280px;  /* manually offset */
}
```

**Why it breaks:** A `position: fixed` element is removed from the document flow. The CSS Grid sees only one in-flow child (`.main`) and auto-places it into the **first** `280px` track instead of the second `1fr` track. The entire main content gets squeezed into a narrow sliver next to the sidebar.

### The Correct Pattern

```css
.app-shell {
  display: block;  /* NOT a grid — plain block flow */
  min-height: 100vh;
  min-height: 100dvh;
}

.sidebar {
  position: fixed;
  left: 0; top: 0; bottom: 0;
  width: 280px;
  z-index: 50;
}

.main {
  margin-left: 280px;  /* offset for the fixed sidebar */
}
```

**Why this works:** Plain block flow doesn't try to place children into tracks. The fixed sidebar sits at its `left: 0` position, and the main content starts at its `margin-left` offset. No auto-placement conflicts.

## Mobile Bottom Drawer Variant

For mobile, the sidebar becomes a bottom drawer that slides up from the bottom:

```css
@media (max-width: 639px) {
  .sidebar {
    width: 100%;
    left: 0; right: 0;
    top: auto; bottom: 0;
    height: 85vh;
    border-radius: var(--radius) var(--radius) 0 0;
    transform: translateY(110%);   /* hidden below viewport */
    box-shadow: 0 -8px 32px rgba(0,0,0,.2);
  }
  .sidebar.drawer-open {
    transform: translateY(0);      /* slides up */
  }
  .sidebar::before {
    /* draggable handle indicator */
    content: '';
    display: block;
    width: 36px; height: 4px;
    background: var(--border);
    border-radius: 2px;
    margin: 8px auto 0;
  }
}
```

## Key Rules

1. **Never mix CSS Grid with `position: fixed` children** — fixed elements are removed from flow, so the grid can't place them correctly. Use `display: block` with `margin-left` offset instead.

2. **Always use `min-height: 100dvh`** on the app shell — `dvh` (dynamic viewport height) handles mobile browser chrome properly, unlike `vh` which can cause content to be hidden behind the bottom bar.

3. **Always read the user's existing reference file first** — if they provide an HTML file, don't rebuild from scratch. Add the requested features (PWA, mobile, etc.) to the existing design, not on top of a new one.

4. **Use exact CDN versions** — `alpinejs@3.14.3/dist/cdn.min.js` not `alpinejs@3.x.x`. The wildcard 404s silently, breaking all `x-show`/`x-cloak` directives and leaving a blank page.

5. **Fixed sidebar + margin offset is the canonical pattern** for dashboard apps. This is what shadcn/ui, Tailwind UI, and most design systems do. Grid only works when ALL children are in-flow (no `position: fixed`).

## Related

- [CSS Grid Level 1 Spec — Out-of-flow items](https://www.w3.org/TR/css-grid-1/#grid-item-concept)
- [CSS Positioned Layout — Fixed positioning](https://www.w3.org/TR/css-position-3/#fixed-pos)
- [Deep Research UI Interface Design](/prometheus-deep-research-skill-master-spec.md)
