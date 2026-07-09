# Proposal — change-polish-001-vendor-hlsjs

## Phase
phase-prui-polish

## Goal
G-01: Replace the `hls.min.js` 125-byte stub with the real HLS.js library so
the song media card can play HLS streams.

## Summary
Download `hls.min.js` (latest stable v1.x) from the jsdelivr CDN and write it
to `substrate/prometheus-research/src/static/hls.min.js`. The UI already gates
behind `Hls.isSupported()` — no HTML/JS changes needed.

## Acceptance Criteria
- [ ] `substrate/prometheus-research/src/static/hls.min.js` is ≥ 100 KB (real library)
- [ ] File begins with a HLS.js copyright/banner comment (not the stub comment)
- [ ] `Hls.isSupported()` returns `true` when evaluated against the file in a browser JS context (i.e., the export is real)
- [ ] File is committed to git (not gitignored)

## Files Changed
- `substrate/prometheus-research/src/static/hls.min.js` — overwritten with real library

## Implementation Notes
Fetch from: `https://cdn.jsdelivr.net/npm/hls.js/dist/hls.min.js`
If CDN fetch fails, fallback: `npm pack hls.js && tar -xf hls.js-*.tgz && cp package/dist/hls.min.js ...`
Do NOT use `hls.js` (unminified) — the static path serves the minified variant.
