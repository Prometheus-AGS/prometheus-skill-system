# KBD control-plane startup: one project can disable the whole box

**Status:** Diagnosed, cross-model reviewed, **fix not yet implemented**
**Date:** 2026-08-06
**Reported by:** Codex — "signed waypoint remains unchanged at revision 146 because the
typed mutation endpoint at 127.0.0.1:7892 refused the connection"

## Symptom

5 of 24 registered KBD projects failed to open at startup. **All typed mutation routes
for all 24 projects** were refused with `local authority routes are not ready`. A single
bad project disabled the control plane for every project on the machine.

Observed: `stage: failed`, `failedProjects: 5`, `projectTotal: 24`, `elapsedMs: 10713242`
(~3 h wedged). The 5 failures are instantaneous (`elapsed_ms=0`) — lock contention, not
corruption. No live process holds the locks; they are stale from 07-28 and 08-02.

**This project (`6ac090a4…`) was NOT among the 5 failures and was still refused service.**

## Two independent problems

### 1. Transport — Codex used TCP :7892, which does not exist

sovereign-sync 1.7.0 serves HTTP over a **Unix socket**; TCP `:7892` requires an explicit
`--tcp`, which the managed LaunchAgent does not pass. `shared/scripts/kbd-harness-adapter.sh:65`
still defaults `PROMETHEUS_CONTROL_ENDPOINT` to `http://127.0.0.1:7892`. Same class of bug
as the health/installer probes fixed in `3c31581`, not extended to the adapter.

### 2. Blast radius — an aggregate gate destroys per-project isolation

`substrate/sovereign-sync/src/rest_api.rs` (`try_new_inner`):

```rust
let (total, opened, failed) = kbd_projects.startup_counts();
if failed != 0 {
    anyhow::bail!("{failed} registered local authorit{} failed to open", …);
}
```

The layer below **already isolates correctly**. `kbd_control.rs reload()` opens each project
independently, records per-project failures in an `errors: BTreeMap<String, String>`, and
returns `Ok(())`. `control()` already distinguishes healthy / failed / unknown.

The aggregate gate discards that isolation: any non-zero count aborts construction of the
whole router, `gate.app` stays `None`, and `startup_dispatch` (`rest_api.rs:3182`) then
serves 503 to everything.

## Cross-model adversarial review

Reviewed by **k3** (Moonshot, judge) and **MiniMax-M3** (critic) — neither shares the
producer's model family. Both returned **PARTIALLY CORRECT — right locus, unsafe fix.**

The naive fix (delete the `bail!`) was **rejected**, for reasons that survived verification:

| Finding | Status |
|---|---|
| Deleting the gate alone converts an honest outage into a **silent degraded mode** | Accepted |
| A systemic failure (unreadable registry, permissions) would then boot "successfully" with an **empty control plane** | Accepted |
| Collapsing failed-project into 404 invites a caller to **re-register/re-initialize** → duplicate authority / split-brain | Accepted |
| Errors are stored as `error.to_string()`, destroying classification — a **stale lock (retryable)** and **corruption (permanent)** become indistinguishable strings | Accepted, verified at `kbd_control.rs:208` |
| `std::sync::RwLock` + 18 × `.expect("lock poisoned")` — one panicked writer permanently disables **every** project | **Accepted, independently verified**: imports are `std::sync::{Arc, RwLock}` (poisons), not `tokio::sync` (does not) |
| No timeout on per-project open — one hung open stalls startup before the gate can fire | Accepted |
| `ensure_registered_path(…).await?` is a **second** global abort path in the same function | Accepted |
| `<startup-task>` pseudo-key pollutes a per-project map with an unaddressable ID | Accepted |
| Critic: "the bail is not the only gate" | **Partially right** — `startup_dispatch` is a second gate, but same causal chain (`bail` → `app = None` → 503). Corrected. |

## Required fix (not "delete the gate")

1. **Gate on systemic conditions only** — e.g. `opened == 0 && total > 0`, or registry
   unreadable. Boot degraded otherwise.
2. **Honest HTTP semantics**, never collapsing the three cases:
   | Condition | Status |
   |---|---|
   | Project failed to open | **503** + `Retry-After`, `{retryable}` |
   | Project never registered | **404** |
   | Healthy | normal |
3. **Typed errors**, not `to_string()` — retryable (stale lock) vs permanent (corruption).
4. **Repair without restart** — lazy retry on the error-map arm, or an admin reopen route.
   Otherwise "whole box down until restart" merely becomes "5 projects down until restart."
5. **Steady-state readiness endpoint** exposing per-project `{id, state, error, since}`,
   plus an alertable metric — a degraded boot must not look clean to a supervisor.
6. **Replace `std::sync::RwLock`** with a non-poisoning lock, or handle poisoning, so one
   panic cannot disable the fleet.

## Lesson

Per-project isolation existed and was correct. It was **destroyed one layer up by an
aggregate count**. Isolation is only as good as the narrowest place a failure is
re-aggregated — a `HashMap<ProjectId, _>` is a data structure, not an isolation guarantee.

Producer note: the initial diagnosis located the defect but prescribed an unsafe fix. It
took a non-Claude judge to catch that. Recorded because it is the same failure this
repo's adversarial-review machinery exists to prevent.
