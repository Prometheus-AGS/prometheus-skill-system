# The UI distinguishes builtin skills

**Change:** `change-uhe-012-ui-surfaces-builtin`
**Phase:** uar-host-execution
**Goal:** R3

## Why

See `.kbd-orchestrator/phases/uar-host-execution/plan.md` for full rationale,
acceptance criteria, and the two-round adversarial review record.

## Outcome: the UI already met R3 — the *test* did not

Measured in `frontend/src/admin/pages/skills-page.tsx`, not assumed:

| Criterion | Status | Evidence |
|---|---|---|
| Builtins visually distinguishable | ✅ already met | `isBuiltin` (line 248) renders an amber `built-in` badge with a `Shield` icon |
| Delete absent or disabled | ✅ already met | `disabled={isBuiltin}` + `title="System skill — cannot be removed"` |
| Toggle still works for builtins | ✅ already met | toggle is gated on `isBusy` only — **not** `isBuiltin` |
| Verified in `admin-skills.spec.ts` | ⚠️ **this was the gap** | see below |

### The real gap was in the test, not the product

`frontend/e2e/admin-skills.spec.ts` exists (1,866 bytes, confirmed at plan time)
— but every test in it mocks an **empty** skills list:

```ts
await route.fulfill({ json: { skills: [] } });
```

**No row is ever rendered**, so nothing about builtin handling was exercised. The
UI happened to be correct; the suite could not have told us if it were not.

This is the same shape as the `uhe-010` finding: a passing suite that asserts a
different thing than the one that matters. There the tests had no persistence;
here they have no rows.

### What was added

Three tests that mock **one builtin and one user skill**, which is the only way
to assert the rules:

1. the builtin carries the `built-in` badge and the user skill is also listed
2. `Delete <builtin>` is **disabled** while `Delete <user skill>` is **enabled**
   — the contrast matters, because a blanket-disabled Delete column would pass a
   one-sided check while breaking normal skill management
3. `Disable <builtin>` remains **enabled** — disabling is allowed, deleting is
   not, which is R2 as the operator experiences it

**A button that 409s is a worse experience than no button.** The product already
got that right; now a regression would be caught.

## Task 3 — the e2e tests are WRITTEN but NOT EXECUTED

Stated plainly rather than claimed as verified.

The three R3 tests parse and register:

```
admin-skills.spec.ts:93  › a builtin skill is visually marked and a user skill is not
admin-skills.spec.ts:102 › delete is DISABLED for a builtin, enabled for a user skill
admin-skills.spec.ts:114 › a builtin skill can still be toggled
```

They **cannot run**: Playwright's `webServer` fails to start.

```
Failed to resolve import "@prometheus-ags/prometheus-entity-management"
  (imported by frontend/src/lib/realtime/optimistic.ts)
Error: Timed out waiting 60000ms from config.webServer
```

**Pre-existing, not caused by this change** — verified by stashing the spec and
running an *original* test from the same file, which fails identically. The
frontend workspace dependency is unlinked, and port 8080 is also occupied.

### Why this is recorded rather than worked around

Fixing the frontend workspace is a different problem from R3, and standing up a
dev server by hacking around a missing package would prove nothing about the UI.
The honest position:

- The **product** meets R3 — verified by reading `skills-page.tsx`: badge at
  line 248, `disabled={isBuiltin}` on Delete, toggle gated only on `isBusy`.
- The **regression protection** is written and will run the moment the frontend
  workspace builds.

**Task 3 is therefore NOT complete**, and R3 is **PARTIAL** — the behaviour is
correct today but not yet guarded by an executed test.

Carry-forward: repair the frontend workspace (`@prometheus-ags/prometheus-entity-management`
unresolved) so the e2e suite can run at all. That blocks every frontend test in
this repo, not only these three.
