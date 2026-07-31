---
name: fabric-integration
description: Enforce the cross-repository fabric version invariants (Loro minor, wasmtime major, iroh floor, WIT world pinning) so a drift that would surface as a merge error, a cache miss, or a component that will not instantiate fails at check time instead.
license: MIT
version: '1.0.0'
metadata:
  author: prometheus
  version: '1.0.0'
  category: devops
  tags: [fabric, invariants, versions, loro, wasmtime, iroh, wit, ci]
---

# Fabric Integration

Four version constraints span this pack, `flint-realtime-fabric`,
`universal-agent-runtime`, and `know-me-system`. Each is here because violating
it produces a failure that is **silent or misattributed at the point it
appears** — far from the version difference that caused it.

Before this skill they were prose in
[`docs/decisions/fabric-version-invariants.md`](../../../docs/decisions/fabric-version-invariants.md).
Nothing checked any of them.

## Run it

```bash
bash skills/devops/fabric-integration/scripts/check-invariants.sh
bash skills/devops/fabric-integration/scripts/check-invariants.sh --json
```

| Exit | Meaning |
|---|---|
| 0 | every enforced invariant holds and the allowlist is exact |
| 2 | an invariant is **violated and not allowlisted** |
| 3 | an allowlisted violation has been **fixed** — delete the stale entry |

## The invariants

| Invariant | Failure it prevents |
|---|---|
| **Loro minor aligned** | Loro's document encoding is a wire format between peers. A minor mismatch fails on **merge** — a decode error, or worse a silently divergent document — never at build or connect time. |
| **wasmtime major aligned** | A precompiled `.cwasm` will not load across majors. Two hosts on different majors compile every component twice and share no cache. |
| **iroh floor ≥ 1.0.2** | 1.0.2 fixed a relay DoS: one malformed datagram from any client crashed an entire relay, disconnecting every peer on it. The browser path is relay-only by architecture, so a crashable relay is its only path. |
| **WIT world version pinned** | An unpinned world resolves arbitrarily. `knowme:plugin` is already declared at **two** versions at once. |

## Three are enforced; one is quarantined

Three invariants hold today and are enforced outright — a violation exits 2.

The **WIT** invariant is **already violated** (`knowme:plugin` at 0.1.0 and
1.0.0). Gating on it would block every PR for a pre-existing condition, so it
sits in [`assets/known-violations.json`](assets/known-violations.json).

**The allowlist is scoped per WIT package**, not per invariant. An entry reads
`wit-world-version-pinned:knowme:plugin`, so quarantining that package cannot
also excuse a split introduced in `prometheus:component` or anywhere else.
(A first version keyed entries by invariant name alone; a mutation test showed
it silently permitted a brand-new split — the quarantine leaking to cover a
defect it was never granted for.)

**The allowlist is enforcement, not an escape hatch.** It is checked in both
directions:

- a violation **not** in the allowlist → **exit 2**
- an allowlisted entry that **no longer reproduces** → **exit 3**, demanding
  removal

A quarantine that never shrinks is a suppressed check. The second rule is what
prevents that.

## An absent repository is SKIP, never PASS

Three invariants compare versions **across repositories** that may not be
checked out. When one is missing the invariant is **unverifiable** and reports
`SKIP` — it is never counted as holding. Reporting "aligned" because a file
could not be read is how a check becomes decorative.

**Consequence for CI — measured, not estimated** (`change-uhe-002`):

| Invariant | CI status | Why |
|---|---|---|
| `loro-minor-aligned` | **PASS** | `flint-realtime-fabric` is public and checked out |
| `iroh-floor-1.0.2` | **PASS** | in-repo |
| `wasmtime-major-aligned` | **SKIP** | needs `know-me-system` |
| `wit-world-version-pinned` | **SKIP** | needs `know-me-system` |

**`know-me-system` is deliberately not reachable from CI.** It is private and in
a different org (`Know-Me-Tools/know-me-system`); adding a cross-org PAT to a
public workflow to compare two version strings is a poor trade. **Two of four
verified is the honest ceiling here, and it is not described as full coverage.**

The split is **pinned** by `scripts/assert-ci-coverage.sh`, which fails if
coverage drifts in *either* direction. SKIP never fails a build, so an invariant
that quietly stops being verified would otherwise go unnoticed — that is how
coverage rots. If a SKIP ever becomes a PASS, the assertion fails too, which is
the prompt to update the expectation rather than let it drift upward unrecorded.

Override the search paths with `FRF_ROOT`, `UAR_ROOT`, and `KNOWME_ROOT`.

## Verified behaviour

Every path below was exercised on 2026-07-31, not asserted:

| Mutation | Expected | Got |
|---|---|---|
| lower the iroh floor to 1.0.0 | exit 2 | exit 2, naming `storage-provider=1.0.0` |
| allowlist an invariant that passes | exit 3 | exit 3, naming the stale entry |
| empty the allowlist | exit 2 | exit 2 on the WIT violation |
| point all external roots at a missing dir | SKIP ×3, no false PASS | SKIP ×3, exit 0 |
| omit only `know-me-system` | `wit-world-version-pinned` must be SKIP, **not** PASS | SKIP — fixed in `change-uhe-002`; it previously reported PASS because the repo holding the split was simply absent |
| drop `FRF_ROOT` in CI | coverage assertion fails | exit 2, naming `loro-minor-aligned: want PASS, got SKIP` |
| split `prometheus:component` across two versions | exit 2 — a knowme entry must not cover it | exit 2, naming the new package |
