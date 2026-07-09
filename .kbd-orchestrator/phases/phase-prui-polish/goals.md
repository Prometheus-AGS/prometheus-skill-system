# Goals — phase-prui-polish

## Context

`phase-prometheus-research-ui` shipped the real SSE integration and Anthropic design
system for `deep-research-ui.html`. Four carry-forward items were identified in the
reflection that don't justify a full new capability phase but need to be closed before
the UI is considered production-ready.

## Goals

- [ ] **G-01: Vendor HLS.js** — replace the `hls.min.js` stub with the real
  `hls.js` library (vendored to `substrate/prometheus-research/src/static/hls.min.js`)
  so the song media card can play HLS streams when a URL is found; gate behind
  `Hls.isSupported()` check already present in the UI

- [ ] **G-02: Static install step** — add a `cp -r src/static docs/deep-research/static`
  step to `scripts/install-binaries.sh` (or equivalent) so the `docs/deep-research/static`
  symlink is replaced with a real copy on fresh installs; remove the symlink from git
  and add the real files instead

- [ ] **G-03: Cancel error feedback** — surface `DELETE /api/v1/jobs/{id}` failures in
  the UI via `showToast('Cancel failed — job may have already completed.', 'error')` rather
  than swallowing them in the `/* best-effort */` catch block

- [ ] **G-04: SSE reconnect indicator** — when `EventSource.onerror` fires and the SSE
  dot goes amber, show a "Reconnecting…" overlay on the progress ring instead of leaving
  the user with a silently-stale UI; clear it on `onopen`
