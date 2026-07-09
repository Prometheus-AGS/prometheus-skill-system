# Proposal — change-polish-003-cancel-error-toast

## Phase
phase-prui-polish

## Goal
G-03: Surface `DELETE /api/v1/jobs/{id}` failures to the user via a toast
instead of swallowing them silently.

## Summary
Replace the `/* best-effort */` catch block in `cancelJob()` inside
`docs/deep-research/deep-research-ui.html` with a `showToast` call.

## Acceptance Criteria
- [ ] `cancelJob()` catch block calls `this.showToast('Cancel failed — job may have already completed.', 'error')`
- [ ] The `/* best-effort */` comment is removed
- [ ] Simulating a network error (e.g. by pointing the fetch at a bad URL in demo mode) causes a red toast to appear in the UI
- [ ] Successful cancels still show "Job cancelled." info toast and navigate to dashboard

## Files Changed
- `docs/deep-research/deep-research-ui.html` — `cancelJob()` catch block

## Implementation Notes
Change:
```js
catch { /* best-effort */ }
```
To:
```js
catch (err) {
  this.showToast('Cancel failed — job may have already completed.', 'error');
}
```
`showToast` is already defined in the Alpine component and accepts `(message, type)` where
`type` is one of `'success'`, `'info'`, `'error'`.
