---
type: SessionRecord
id: karpathy-session-987780abbd4fc4e1
title: Karpathy session 987780abbd4f
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-12T14:24:15.689141+00:00
created_at: 2026-08-12T14:24:15.689141+00:00
updated_at: 2026-08-12T14:24:15.689141+00:00
revision: 0
---

## Delta

Two skills with the same `name: kbd-assess` on disk. Step 2 is verified; let me report before touching anything deletable.

## Steps 0–2 done, and one finding that corrects my own plan

| Step | Result |
|---|---|
| **0** — stale lock | Removed; PID 52159 re-verified dead first |
| **1** — checkout on main | `main` @ `e097e4e`, in sync, 4 stale local branches deleted |
| **2** — reinstall skills | **Verified: 25/25 kbd skills identical to repo, 0 drift** |

The decisive check passes: installed `kbd-assess` now carries *"Enter/complete the assessment stage with a typed `prometheus kbd stage` command"* at line 78. `kbd-doctor`, `kbd-audit`, `kbd-cancel`, `kbd-pause`, `kbd-resume` are all present.

## I got the drift measurement wrong

After the install I measured **"6 identical, 11 different, 5 missing — unchanged"** and nearly reported the installer as a no-op printing a false ✅.

It wasn't. **I was measuring the wrong directory.** The installer writes a content-addressed generation under `~/.prometheus/plugins/.../generations/<sha>/` and symlinks flat names into `~/.claude/skills/`. I kept comparing against `~/.claude/skills/prometheus/` — the July tree the new layout doesn't use.

Same defect as the whole session: I sampled one location and generalised. The difference is I caught it before reporting.

## What that exposes — and it needs your call

Both trees now exist, and **both define `name: kbd-assess`**:

| Path | Date | Has the CLI step? |
|---|---|---|
| `~/.claude/skills/kbd-assess` → generation | Aug 12 | **Yes** |
| `~/.claude/skills/prometheus/process/.../kbd-assess` | **Jul 8** | **No** |

Which one a harness loads depends on scan order. That ambiguity is arguably the *real* root cause — not that the install was stale, but that a July copy has been shadowing a correct one, and any agent could resolve to either.

Deleting `~/.claude/skills/prometheus/` would remove it, but that's a 21-directory deletion outside the repo and I won't do it unasked. Two options: remove it, or have step 3's freshness check detect duplicate skill names and refuse.

Steps 3–7 remain — the drift check, the fail-open gate, the runtime projection fix, A0, and the hook scope. Want me to continue, and how do you want the shadowed July tree handled?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-12T14:23:38.458214Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
