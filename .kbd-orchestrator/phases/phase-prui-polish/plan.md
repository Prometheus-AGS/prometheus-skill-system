# Plan — phase-prui-polish

_Generated: 2026-07-09_

## Change Backend
OpenSpec (`openspec/` directory present at project root)

## Overview

4 changes, all independently deliverable, ordered to front-load the binary
asset (HLS.js) so the static install step (change-002) can copy the real file.
Changes 003 and 004 are pure HTML edits with no dependency on 001/002.

## Changes

| # | Change ID | Goal | Description | Tasks |
|---|-----------|------|-------------|-------|
| 1 | `change-polish-001-vendor-hlsjs` | G-01 | Download real HLS.js and replace stub | 3 |
| 2 | `change-polish-002-static-install-step` | G-02 | git-untrack symlink, commit real files, add cp step to install-binaries.sh | 6 |
| 3 | `change-polish-003-cancel-error-toast` | G-03 | Replace best-effort catch with showToast error call | 4 |
| 4 | `change-polish-004-sse-reconnect-overlay` | G-04 | Add sseReconnecting state, overlay HTML, CSS, and motion gate | 8 |

**Total tasks: 21**

## Execution Order Rationale

- **001 before 002**: change-002 copies `src/static/` to `docs/deep-research/static/`; if 001 has already replaced the stub, the copy will include the real HLS.js.
- **003 and 004 are independent**: pure HTML/JS edits; can run in any order relative to 001/002.
- Recommended: 001 → 002 → 003 → 004 (preserves clean commit sequence).

## Apply Commands

```
/kbd-apply change-polish-001-vendor-hlsjs
/kbd-apply change-polish-002-static-install-step
/kbd-apply change-polish-003-cancel-error-toast
/kbd-apply change-polish-004-sse-reconnect-overlay
```

## First Change

```
/kbd-apply change-polish-001-vendor-hlsjs
```
