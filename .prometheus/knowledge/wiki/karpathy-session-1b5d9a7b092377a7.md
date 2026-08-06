---
type: SessionRecord
id: karpathy-session-1b5d9a7b092377a7
title: Karpathy session 1b5d9a7b0923
tags:
- karpathy
- session-learning
sources:
- session:cebcfbb5-668f-49de-8c67-ea6fae9d262c
timestamp: 2026-08-06T12:08:19.807832+00:00
created_at: 2026-08-06T12:08:19.807832+00:00
updated_at: 2026-08-06T12:08:19.807832+00:00
revision: 0
---

## Delta

Both paths confirmed working: **19 projects, all `ready: true`**, and the local CLI reads fine. The `not_ready` was my parser hitting a different response shape.

You're unblocked. Here's the honest account.

## I was wrong twice, and you were right twice

**The stale locks were not the cause.** I moved 11 lock files aside on that theory, restarted, and got the identical `5 of 24`. I've **restored every one of those files** — the change is reverted.

**The real cause:** all 5 failing projects were **deleted git worktrees**. Registry entries pointing at directories that no longer exist:

```
/Users/gqadonis/Projects/sansaba/.worktrees/seller-tax-checklist-remediation
/Users/gqadonis/Projects/references/librefang/.worktrees/surrealdb-image-v3.2.3
…
```

**11 of 37 replica paths were dead.** Five deleted temp worktrees were blocking KBD across your entire machine.

Pruned them (registry backed up first, twice). Result: **19 projects, all `ready: true`**, control plane initialized, local CLI working.

## What the explorers found that I'd missed

Your "you made everything a slave to a passive process" was precisely correct, and the mechanism is one line:

`tools/prometheus-cli/.../kbd.rs:207-208` opens a **fully capable local `Runtime`** — then immediately builds an HTTP `ControlClient` and routes **all 10 mutation sites** through it. `Runtime::execute_command` is never called by the CLI at all.

The damning part: `status()` at line 611 **already has the right pattern** — try the daemon, fall back to `runtime.replay()`. Reads got a recovery door. Writes didn't. That asymmetry is exactly why `prometheus kbd status` kept working while Codex's mutation failed.

And `kbd-mobile` + `skill-ffi` already do in-process writes with no daemon. The desktop CLI is the *only* consumer that links the library and declines to use it.

## Three defects, not one

1. **CLI routes local writes over HTTP** — the root cause, `kbd.rs:208`
2. **Aggregate startup gate** — `rest_api.rs:354`, one bad project 503s all 24
3. **No prune command** — `prometheus kbd` has no way to deregister a dead worktree, so this recurs every time a worktree is deleted

Also worth flagging: the daemon logs a *count* and swallows the *reason*. Nothing anywhere said "path does not exist" — I had to resolve the registry by hand.

## Where things stand

Change 1 is done, but not as planned — the plan's lock-clearing premise was wrong, and the fix was registry pruning. Changes 2–4 are unstarted, and I'd revise the plan first: the CLI fallback (defect 1) matters more than the startup gate (defect 2), since it's what actually made KBD depend on sync.

Nothing is committed. Want me to update the plan with the corrected root cause before proceeding?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cebcfbb5-668f-49de-8c67-ea6fae9d262c
- Captured: 2026-08-06T12:07:35.309142Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
