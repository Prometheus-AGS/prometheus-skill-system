# Assessment — phase-prui-polish

_Generated: 2026-07-09_

## Summary

Four carry-forward items from `phase-prometheus-research-ui` are confirmed open.
All gaps are narrowly scoped, independently deliverable, and require no new
architectural decisions. This phase closes them.

---

## Goal Gap Analysis

### G-01: Vendor HLS.js — GAP CONFIRMED

**Evidence:** `substrate/prometheus-research/src/static/hls.min.js` is a 125-byte
stub:
```js
/* hls.js stub — media playback not available in this build */
window.Hls = { isSupported: function() { return false; } };
```

The song media card in `deep-research-ui.html` calls `Hls.isSupported()` and
renders a `<video>` element when true, but the stub always returns `false`. HLS
streams will never play until the real library is vendored.

**Change required:** Download `hls.min.js` (latest stable, e.g. v1.5.x) from
`cdn.jsdelivr.net/npm/hls.js/dist/hls.min.js` and write it to
`substrate/prometheus-research/src/static/hls.min.js`. No UI changes needed —
the `Hls.isSupported()` gate is already in place.

---

### G-02: Static install step — GAP CONFIRMED

**Evidence 1:** `git ls-files --stage docs/deep-research/static` returns mode
`120000` (symlink), pointing to `substrate/prometheus-research/src/static/`.

**Evidence 2:** `scripts/install-binaries.sh` lines 301-322 build and install
the `prometheus-research` binary + launchd plist but contain **no** `cp` or
`rsync` step for `src/static/`. On a fresh clone without the symlink being
resolvable (e.g. if the binary moves), the HTML will fail to load HTMX/Alpine.

**Change required:**
1. Remove `docs/deep-research/static` symlink from git (`git rm docs/deep-research/static`).
2. Add the real files: copy `substrate/prometheus-research/src/static/` to
   `docs/deep-research/static/` and commit them.
3. Add a `cp -r` step in `scripts/install-binaries.sh` after the binary install
   block so the static dir is refreshed on every `install-binaries.sh` run.

---

### G-03: Cancel error feedback — GAP CONFIRMED

**Evidence:** `cancelJob()` in `deep-research-ui.html`:
```js
async cancelJob(jobId) {
  // ...
  if (job.serverJobId) {
    try { await fetch('.../' + job.serverJobId, { method: 'DELETE' }); }
    catch { /* best-effort */ }
  }
  // ...
}
```

The `catch` block swallows all network errors silently. A user who tries to
cancel a job that's already completed (404) or hits a transient network error
gets no feedback.

**Change required:** Replace `catch { /* best-effort */ }` with:
```js
catch (err) {
  this.showToast('Cancel failed — job may have already completed.', 'error');
}
```

---

### G-04: SSE reconnect indicator — GAP CONFIRMED

**Evidence:** `openSseStream()` `onerror` handler in `deep-research-ui.html`:
```js
es.onerror = () => {
  job.sseConnected = false;
  // removes .sse-dot.connected — dot goes amber
  // NO overlay, NO countdown
};
```

When the SSE connection drops, the `.sse-dot` goes amber but the progress ring
and visible UI give no indication of reconnection. The user can't tell if the
job is still running silently or has genuinely failed.

**Change required:**
1. On `onerror`: set `job.sseReconnecting = true` in Alpine state.
2. Render a "Reconnecting…" overlay on the progress ring gated by
   `x-show="activeJob?.sseReconnecting"`.
3. On `onopen`: clear `job.sseReconnecting = false`.

---

## Open Questions

None — all changes are precisely specified. No external dependencies, no
contested decisions, no blocked-on items.

---

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| hls.js fetch fails (no internet) | LOW | Provide sha256 checksum; script can skip if binary already present |
| Symlink removal breaks existing local installs | NEGLIGIBLE | Git untrack + real files committed; local setups resolvable with `git pull` |
| Cancel toast fires on intentional 404 (completed job) | LOW | Message text covers this: "may have already completed" |
| SSE reconnect overlay never clears | LOW | Clear on `onopen` + on terminal states (done/error) |

---

## Recommended Change Order

| # | Change ID | Description | Goal |
|---|-----------|-------------|------|
| 1 | change-polish-001 | Vendor real HLS.js | G-01 |
| 2 | change-polish-002 | Static install step + git-untrack symlink | G-02 |
| 3 | change-polish-003 | Cancel error toast | G-03 |
| 4 | change-polish-004 | SSE reconnect overlay | G-04 |

Changes 1–4 are independent and can be implemented in any order. Recommended
ordering front-loads the binary asset (HLS.js) before the install step so
change-polish-002 can verify the static dir is populated correctly.
