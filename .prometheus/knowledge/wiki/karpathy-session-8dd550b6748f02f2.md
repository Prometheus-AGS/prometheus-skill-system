---
type: SessionRecord
id: karpathy-session-8dd550b6748f02f2
title: Karpathy session 8dd550b6748f
tags:
- karpathy
- session-learning
sources:
- session:cebcfbb5-668f-49de-8c67-ea6fae9d262c
timestamp: 2026-08-06T11:50:07.519554+00:00
created_at: 2026-08-06T11:50:07.519554+00:00
updated_at: 2026-08-06T11:50:07.519554+00:00
revision: 0
---

## Delta

You're right on the architecture, and my first prescription was wrong. Here's what the cross-model review established.

## You called it correctly

`substrate/sovereign-sync/src/rest_api.rs`, in `try_new_inner`:

```rust
let (total, opened, failed) = kbd_projects.startup_counts();
if failed != 0 {
    anyhow::bail!("{failed} registered local authorities failed to open");
}
```

The layer *below* is already correct — `reload()` opens every project independently, records failures in a per-project `errors` map, returns `Ok`. `control()` already distinguishes healthy / failed / unknown.

That isolation is then **thrown away by an aggregate count one layer up.** 5 bad projects → whole router never constructs → `gate.app = None` → `startup_dispatch` serves 503 to all 24. Your project wasn't even among the 5 failures and was still refused.

## Where I was wrong

I was about to prescribe "delete the gate." **k3 (Moonshot) and MiniMax-M3 both rejected that as unsafe**, and they were right:

- It converts an honest outage into a **silent degraded mode** — a systemic failure (unreadable registry, bad permissions) would boot "successfully" with an empty control plane, and a supervisor would see a clean start.
- If a failed project collapses to **404**, a caller concludes it was never provisioned and re-registers it → **duplicate authority / split-brain**.
- Failed projects would stay dead until restart anyway — the availability bug survives in miniature.

## One finding I verified independently

The judge flagged `.expect("lock poisoned")`. I checked: the imports are `std::sync::{Arc, RwLock}` — which **poisons** — with **18** such call sites. `tokio::sync::RwLock` does not poison. So a single panicked writer permanently disables every project's requests. That's the same all-or-nothing fragility one layer further down, and it's real.

The critic also claimed the bail wasn't the only gate. Partially right — `startup_dispatch:3182` is a second gate, but in the same causal chain. I corrected that in the record rather than accepting it wholesale.

## What the fix actually requires

Six things, not one: gate on **systemic** conditions only (`opened == 0 && total > 0`); honest 503-vs-404 semantics; **typed** errors instead of `to_string()` (a stale lock and corruption are currently indistinguishable strings); repair without restart; a steady-state readiness endpoint; and a non-poisoning lock.

Written up in `docs/decisions/kbd-authority-startup-blast-radius.md` with the full review table.

## Two things still outstanding

**I have not written any fix yet** — this is diagnosis only, and it's a change to the control plane that governs 24 projects. I'd want your sign-off on the approach before touching it.

**Codex is still blocked**, and that's a separate, simpler problem: `kbd-harness-adapter.sh:65` defaults to `http://127.0.0.1:7892`, which doesn't exist in 1.7.0 — the same dead-TCP-probe class I fixed in the health and installer scripts earlier today but didn't extend to the adapter. I can fix that transport bug now to unblock the waypoint write, independently of the larger architectural work.

Also still uncommitted: the article outline and session logs from before this interruption.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cebcfbb5-668f-49de-8c67-ea6fae9d262c
- Captured: 2026-08-06T11:49:44.189809Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
