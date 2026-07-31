# Decision: `frf-transport-iroh` is a `FederationBridge` adapter, additive to FRF

**Status:** accepted · 2026-07-31 · `change-idt-009-record-fabric-decisions`
**Phase:** ideation-and-decision-tools

## The decision

Add iroh to `flint-realtime-fabric` as a **new crate implementing the existing
`FederationBridge` trait**, beside the bridges already there. No change to FRF's
architecture, no modification of existing crates.

## Why additive

FRF already has the extension point. Verified on disk 2026-07-31:

```
flint-realtime-fabric/crates/frf-ports/src/federation.rs:35
    pub trait FederationBridge: Send + Sync + 'static { … }

flint-realtime-fabric/crates/frf-bridge-atproto
flint-realtime-fabric/crates/frf-bridge-matrix
```

Two bridges already implement it. A third is the pattern working as designed,
not an architectural change requiring a case. That is the point of recording it:
the decision is *not* to invent a new transport layer for iroh when the trait
that iroh should implement already exists and already has two implementors.

## Naming

`frf-transport-iroh`, not `frf-bridge-iroh`. The existing two federate with
**foreign networks** (Matrix, ATProto); iroh carries **our own** traffic between
our own peers. Same trait, different role — and a name that says so prevents a
reader from assuming iroh is another federation target.

If FRF maintainers prefer `frf-bridge-iroh` for consistency with the trait, the
name is not worth arguing; the additive shape is what matters.

## Scope boundary — nothing lands in FRF this phase

No code is written into `flint-realtime-fabric` here. The user's constraint at
analyze was explicit: **design and record only, no cross-repo code**. This record
exists so the eventual implementation starts from a settled shape.

## Alternatives considered

- **Fork FRF's transport layer for iroh.** Rejected: duplicates a working
  abstraction and puts us on a fork we would have to maintain against upstream.
- **Put iroh behind FRF's Matrix bridge.** Rejected: tunnels our own peer traffic
  through a foreign-federation abstraction, inheriting Matrix's semantics for
  something that is not Matrix.
- **Skip FRF; have skills talk to iroh directly.** Rejected: every consumer would
  reimplement peer lifecycle and reconnection, and FRF's existing transport
  selection would not see iroh at all.

## What would change this

`FederationBridge` proving too narrow for iroh's connection lifecycle — for
instance if direct/relay path transitions cannot be expressed through it — would
mean the trait needs extending first, and this record would be superseded.
