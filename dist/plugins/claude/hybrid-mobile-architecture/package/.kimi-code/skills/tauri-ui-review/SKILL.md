---
name: tauri-ui-review
description: ALWAYS invoke after building or changing any React/Tauri UI surface, before calling it done — run the screenshot-driven review loop at 320/768/1024/1440 in both themes and check against the design-quality bar. Triggers on Tauri UI, desktop UI, React component, screen, page, layout, responsive, breakpoint, screenshot review, visual check, does it look right, UI review, web UI.
---
<!-- TJ-ARCH-MOB-001 compliant -->

> **Binding:** Prefer simple, surgical, strongly typed changes; preserve strict
> layering and verify dependency versions. When installed in a project, also
> obey that project's `AGENT_BASE_RULES.md`; this skill remains self-contained.

# Tauri / React UI Review Loop

A React/Tauri surface is not done until it has passed a screenshot-driven review at every
breakpoint in both themes. When reference screenshots/HTML/designs exist, invoke
`reference-ui-fidelity` first and treat those artifacts as the acceptance oracle. This
catches missing product surfaces—not only overflow, layout jumps, and contrast failures.

## The loop

1. **Run the app** and navigate to the surface (`pnpm tauri dev`, or the web dev server for
   the browser target).
2. Build the complete route/state coverage matrix from `reference-ui-fidelity`. A screenshot
   of one easy page cannot stand in for the product.
3. **Capture screenshots** at **320, 768, 1024, 1440** px wide, in **both light and dark**
   themes. Use BrowserClaw (`screenshot`) or Playwright. That is 8 shots for a themed
   surface.
4. **Inspect each** against the checklist below and its approved reference when one exists.
5. **Fix and re-capture** only the shots that regressed. Do not declare done from a single
   viewport.

## Checklist (per shot)

- [ ] **No horizontal overflow.** The page body never scrolls sideways; wide content
      (tables, code, diagrams) scrolls inside its own `overflow-x:auto` container.
- [ ] **No layout shift** from streaming content or async data (reserve space; explicit
      image `width`/`height`).
- [ ] **Complete product coverage.** Every specified destination and meaningful state is
      implemented; no placeholder/absent route is hidden by a polished shell.
- [ ] **Hierarchy through scale contrast**, not uniform emphasis.
- [ ] **Intentional rhythm** — spacing varies with meaning, not uniform padding everywhere.
- [ ] **Designed hover / focus / active states** on every interactive element.
- [ ] **Both themes look intentional** — dark isn't just inverted light (see
      [[hybrid-design-tokens]]).
- [ ] **Not a default template.** Fails if it reads as stock shadcn/Tailwind with no point
      of view (centered hero + gradient blob + generic CTA; uniform card grid; gray-on-white
      with one accent). See the design-quality bar below.
- [ ] **Flat 2.0:** no visible borders, divider lines, or layout shadows; adjacent
      regions are distinguished through background-color changes.
- [ ] **Shadcn/Assistant UI are mounted at the real boundary**, not merely installed.
- [ ] **Motion is compositor-friendly** (opacity/transform/clip-path only).
- [ ] **Touch targets ≥ 44px** at the 320/768 widths.

## Design-quality bar

Every meaningful surface should demonstrate **at least four**: clear hierarchy via scale
contrast; intentional spacing rhythm; depth/layering through flat surface color and motion;
typography with a real pairing; semantic (not decorative) color; designed interaction
states; editorial/bento composition where it fits; atmosphere/texture when apt; motion that
clarifies flow; data-viz treated as part of the design system.

## Layer-contract reminder (don't violate while fixing)

- Components import only hooks. No direct store imports, no `invoke()`/`listen()` in a
  component or hook — those live only in Zustand stores.
- Server/async/entity data via `@prometheus-ags/prometheus-entity-management` 3.x; client/UI state via Zustand; shareable state (tab,
  filter, sort) in the URL.

## Related skills

- `frontend-design`, `web-design-guidelines`, `ui-ux-pro-max` (external) — the design lift
- `shadcn` MCP — component sourcing; the shadcn/ui skill for correct APIs
- [[hybrid-design-tokens]] — the tokens every surface must use
- [[a11y-gate]] — the accessibility half of "done"
- [[content-block-ui]] — reviewing chat/ContentBlock surfaces
- [[reference-ui-fidelity]] — reference discovery, screen inventory, and comparison oracle
