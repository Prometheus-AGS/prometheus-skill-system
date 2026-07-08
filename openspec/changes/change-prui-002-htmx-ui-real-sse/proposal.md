# change-prui-002-htmx-ui-real-sse

## Summary

Transform `docs/deep-research/deep-research-ui.html` from a 4339-line simulation engine
into a production-quality AG-UI streaming research visualization UI — applying the
`htmx-alpine-lit`, `animate`, `delight`, and `polish` skills from the skill collection.

The current file calls `simulateProgress()` / `simulateSongResearch()` with no real HTTP
calls. This change wires it to `prometheus-research` on `:7891` via `EventSource` SSE
and applies a full design-system pass aligned to the Anthropic visual language.

## Goal

G-02: Ship polished `deep-research-ui.html` with real SSE

## Files Changed

- `docs/deep-research/deep-research-ui.html` — replace simulation + CDN scripts + apply design system

## Design System (Mandatory — htmx-alpine-lit + animate + delight + polish skills)

```
Visual direction:  Dark luxury / technical editorial
                   Anthropic warm-dark palette (OKLCH throughout)
Typography:        Space Grotesk (prose) + JetBrains Mono (IDs/code); NO Inter/system-ui
Motion:            ease-out-expo (cubic-bezier(0.16,1,0.3,1)) everywhere; prefers-reduced-motion
Delight moments:   start pulse, stage-done flash, job-done checkmark draw, SSE-connect dot
Forbidden:         glassmorphism, generic card grid, CDN HTMX/Alpine, hardcoded hex colors
```

CSS token mandate:
```css
:root {
  --bg: oklch(0.14 0.01 70);
  --surface: oklch(0.18 0.01 70);
  --fg: oklch(0.95 0.02 70);
  --accent: oklch(0.70 0.14 45);    /* terracotta/copper — Anthropic brand */
  --success: oklch(0.72 0.16 145);  /* sage green */
  --error: oklch(0.65 0.20 30);     /* warm red */
  --border: oklch(0.28 0.01 70);
  --radius: 6px;                    /* ≤6px in dense UI per Anthropic rule */
  --ease-expo: cubic-bezier(0.16, 1, 0.3, 1);
  --duration-fast: 150ms;
  --duration-normal: 300ms;
}
```

## Acceptance Criteria

### Connectivity (htmx-alpine-lit skill)
- [ ] HTMX loaded from `/static/htmx.min.js` (vendored binary), NOT `unpkg.com`
- [ ] Alpine.js loaded from `/static/alpine.min.js` (vendored binary), NOT CDN
- [ ] `startResearch()` POSTs to `http://127.0.0.1:7891/api/v1/jobs` and captures `job_id`
- [ ] `EventSource` opened at `http://127.0.0.1:7891/api/v1/jobs/{job_id}/events`
- [ ] `agent.status` events: update progress ring `stroke-dashoffset` + stage timeline
- [ ] `agent.message` events: append to log panel (prepend, not replace)
- [ ] `agent.error` events: trigger error card with warm-red fade
- [ ] `a2ui.component` events: swap HTMX fragment into target div via `hx-swap-oob`
- [ ] Cancel button → `DELETE http://127.0.0.1:7891/api/v1/jobs/{job_id}`
- [ ] Simulation (`simulateProgress` / `simulateSongResearch`) gated behind `?demo=1`

### Design system (animate + polish skills)
- [ ] All CSS colors use custom properties — zero hardcoded hex or hsl values
- [ ] Background `var(--bg)`, surface `var(--surface)`, foreground `var(--fg)`, accent `var(--accent)`
- [ ] Font stack: `'Space Grotesk', 'JetBrains Mono', system-ui`
- [ ] All state transitions use `--ease-expo`, durations `--duration-fast` or `--duration-normal`
- [ ] Progress ring uses `stroke-dashoffset` transition (compositor-friendly, no layout repaints)
- [ ] Stage timeline items animate: slide in from left as they activate
- [ ] Entrance choreography on page load: staggered fade + translate-y
- [ ] Every button/input has all 5 states: default, hover, focus, active, disabled
- [ ] Focus rings: 2px offset, `var(--accent)` color, never hidden
- [ ] Spacing: 4px base grid, multiples only
- [ ] No drop shadows (contrast-based depth)
- [ ] Border radius ≤ 6px in dense areas

### Delight moments (delight skill)
- [ ] Research start: 400ms pulse animation on progress ring border
- [ ] Stage completion: scale 1 → 1.02 → 1 on stage badge (100ms)
- [ ] Job complete: animated SVG checkmark draw in progress ring center
- [ ] Error state: shake animation (translateX) on error badge
- [ ] SSE connected: status dot fades amber → green with 300ms glow pulse

### Accessibility + motion
- [ ] `@media (prefers-reduced-motion: reduce)` disables all animations
- [ ] All text meets WCAG AA contrast against `var(--bg)` / `var(--surface)`

## Risk

Medium. Large existing file. Simulation must be preserved in demo mode.
Design constraints are clear and binding — executor must not deviate.
