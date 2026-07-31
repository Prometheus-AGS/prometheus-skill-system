# Decision: iroh is the fabric transport; the browser is relay-only

**Status:** accepted · 2026-07-31 · `change-idt-009-record-fabric-decisions`
**Phase:** ideation-and-decision-tools

## The decision

Use **iroh ≥ 1.0.2** as the P2P transport across desktop, server, and mobile.
**Accept that the browser is relay-only** — not as a temporary gap, but as a
property of the platform. Reject `iroh-webrtc-transport`.

## Why the browser is relay-only by architecture, not by omission

iroh's direct path is QUIC over UDP. **A browser sandbox exposes no UDP socket
API.** No amount of work on our side changes that: WebTransport and WebRTC both
exist precisely because raw UDP is unavailable to page JavaScript.

This matters because "relay-only in the browser" reads like an unfinished item
on a roadmap, and it is not one. A browser peer connects through a relay; that
is the finished state. Planning that assumes browser hole-punching will arrive
later is planning on something that cannot arrive.

Relay-only is not degraded correctness — CRDT merge is identical either way.
What differs is latency and the fact that a relay observes connection metadata.

## Why `iroh-webrtc-transport` is rejected

Checked against crates.io on 2026-07-31:

| Signal | Value |
|---|---|
| Total downloads | **33** |
| Newest version | **0.1.0-alpha.2** (pre-1.0, alpha) |
| iroh dependency | **`^0.98.2`** — pre-1.0 iroh |
| Repository | `github.com/SuddenlyHazel/iroh-webrtc-transport` (HTTP 200) |
| Listed in n0's `TRANSPORTS.md` | no |

The disqualifying signal is the **`^0.98.2` iroh pin**. Our floor is 1.0.2, set
by a relay DoS fix (below). A transport crate pinned to pre-1.0 iroh cannot be
used with a post-1.0 iroh without the crate itself being updated, and 33
downloads against an alpha is not a maintenance signal that justifies depending
on that happening.

> **Correction to the analyze-stage note.** That note recorded the repository as
> 404. It resolves (HTTP 200) as of this check. The rejection does not rest on
> that claim, and the record should not carry a fact that is no longer true.

## Why the floor is 1.0.2, not 1.0

iroh 1.0.2 fixed a relay denial-of-service: a single malformed datagram from any
client crashed an entire relay, disconnecting **every** peer using it. Since the
browser path is relay-only by the argument above, a crashable relay is not a
peripheral concern — it is the browser's only path.

Verified in this repo at the time of writing: `substrate/sovereign-sync` had
`iroh = "1.0"` with a lockfile resolving to **1.0.0** — the vulnerable version.
Raised to `1.0.2` in `change-idt-008`.

## Alternatives considered

- **WebRTC everywhere (drop iroh).** Rejected: gives up QUIC and iroh's relay and
  discovery infrastructure on the platforms that *can* use them, to make the
  browser look the same as native. Optimises for uniformity over capability.
- **Wait for browser QUIC (WebTransport) in iroh.** Rejected as a plan: it may
  arrive, but nothing can be built on the assumption. Relay-only works today.
- **`iroh-webrtc-transport`.** Rejected above.

## What would change this

A first-party n0 browser transport listed in `TRANSPORTS.md`, at a version
compatible with iroh ≥ 1.0.2, would make the browser a direct peer and retire
the relay-only constraint. Nothing else would.
