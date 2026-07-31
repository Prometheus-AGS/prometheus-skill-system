# Decision: four version invariants the fabric must hold

**Status:** accepted · 2026-07-31 · `change-idt-009-record-fabric-decisions`
**Phase:** ideation-and-decision-tools

## The decision

Four cross-repository version constraints. Each is here because violating it
produces a failure that is **silent or confusing at the point it appears**, far
from the version mismatch that caused it.

## The invariants

### 1. Loro minor aligned

| Repo | Declared | File |
|---|---|---|
| flint-realtime-fabric | `1.13.1` | `Cargo.toml:127` (`loro-ffi` 1.13.1 at :134) |
| prometheus-skill-pack | `1.13` | `substrate/storage-provider/Cargo.toml:7` |

**Aligned** at minor 1.13 as of 2026-07-31.

Why it matters: Loro's document encoding is a wire format between peers. A minor
mismatch does not fail at build time or at connect time — it fails on **merge**,
as a decode error or, worse, a silently divergent document. Both are diagnosed
far from the version difference.

### 2. wasmtime major aligned

| Repo | Declared | File |
|---|---|---|
| universal-agent-runtime | `46` | `Cargo.toml:219–220` |
| know-me-system | `46` | `rust/crates/knowme_plugin_host/Cargo.toml:24` |

**Aligned** at major 46. KnowMe's own comment records the reason: *"wasmtime 46
matches … a single wasmtime major. `.cwasm` caches are pinned to this major."*

A precompiled `.cwasm` produced by one major will not load on another. Two hosts
on different majors means every component is compiled twice, and a cache built by
one is useless to the other.

### 3. iroh ≥ 1.0.2

Floor set by the relay DoS fix — see
[`fabric-transport-iroh.md`](fabric-transport-iroh.md). Verified and enforced in
this repo by `change-idt-008`; `sovereign-sync` previously resolved to the
vulnerable **1.0.0**.

### 4. WIT world version pinned

No world may be depended on without an explicit version. `knowme:plugin` already
exists at **both 0.1.0 and 1.0.0** simultaneously (see
[`wit-world-unification.md`](wit-world-unification.md)) — which is precisely the
state an unpinned dependency resolves arbitrarily against.

## Aligned today is not enforced

Three of the four hold right now. **None is checked by anything.** They drift the
moment someone bumps a dependency in one repo, and the resulting failure surfaces
as a merge error, a cache miss, or a component that will not instantiate — none
of which points at a version table.

Mechanical verification belongs to the `fabric-integration` skill, deferred to
`mobile-skill-portability` along with the WIT authoring it depends on. Until that
exists, these are **documented invariants, not enforced ones**, and this record
should not be read as saying otherwise.

## Alternatives considered

- **A shared workspace pinning all four.** Rejected: the repos have independent
  release cycles and separate consumers; a single workspace couples them harder
  than the problem warrants.
- **Check at runtime and refuse to start.** Rejected as the primary mechanism —
  it converts a build-time-detectable mismatch into a production outage. Useful
  as a secondary guard, not a substitute for a build check.
- **Leave them to drift and fix on failure.** Rejected: the failures are exactly
  the ones that do not name their cause.

## What would change this

Loro guaranteeing wire compatibility across minors, or wasmtime making `.cwasm`
portable across majors, would retire invariants 1 and 2 respectively.
