# Proposal — change-polish-004-sse-reconnect-overlay

## Phase
phase-prui-polish

## Goal
G-04: When `EventSource.onerror` fires, show a "Reconnecting…" overlay on the
progress ring so the user knows the connection is retrying.

## Summary
Add a `sseReconnecting` boolean to the Alpine job model, set it on `onerror`,
clear it on `onopen`, and render a visually distinct overlay on the progress
ring area gated by `x-show="activeJob?.sseReconnecting"`.

## Acceptance Criteria
- [ ] Alpine job object initialised with `sseReconnecting: false`
- [ ] `openSseStream()` `onerror` sets `job.sseReconnecting = true` (in addition to existing `job.sseConnected = false`)
- [ ] `openSseStream()` `onopen` sets `job.sseReconnecting = false`
- [ ] A "Reconnecting…" overlay element exists in the DOM, hidden by default, visible when `sseReconnecting` is true
- [ ] The overlay uses compositor-safe CSS (opacity/transform transitions, not layout properties)
- [ ] The overlay is cleared when the job reaches a terminal state (`done`, `error`, `cancelled`)
- [ ] `prefers-reduced-motion` is respected (no animation when motion is reduced)

## Files Changed
- `docs/deep-research/deep-research-ui.html` — `openSseStream()` onerror/onopen + new overlay element + CSS

## Implementation Notes

### State change (in `openSseStream()`):
```js
es.onerror = () => {
  job.sseConnected = false;
  job.sseReconnecting = true;   // NEW
};
es.onopen = () => {
  job.sseConnected = true;
  job.sseReconnecting = false;  // NEW
};
```

### Job model init (in `newJob()` or wherever job objects are created):
```js
sseReconnecting: false,
```

### Clear on terminal state (in wherever status is set to done/error/cancelled):
```js
job.sseReconnecting = false;
```

### Overlay HTML (inside the progress ring container, sibling to the ring SVG):
```html
<div class="reconnect-overlay"
     x-show="activeJob?.sseReconnecting"
     x-transition:enter="transition-opacity duration-200"
     x-transition:leave="transition-opacity duration-200">
  <span class="reconnect-label">Reconnecting&hellip;</span>
</div>
```

### CSS (add to the existing `<style>` block):
```css
.reconnect-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: oklch(0.14 0.01 70 / 0.8);
  border-radius: inherit;
  backdrop-filter: blur(2px);
  z-index: 10;
}
.reconnect-label {
  color: var(--accent);
  font-size: 0.75rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  @media (prefers-reduced-motion: no-preference) {
    animation: sse-glow-pulse 1.2s ease-in-out infinite;
  }
}
```
